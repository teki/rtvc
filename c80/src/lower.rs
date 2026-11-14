//! Straight-line register-leaf lowering from typed IR to structured Z80 items.

use super::C80Target;
use super::abi::{assign_params, return_home};
use super::ast::{AsmClobber, AsmGpr, CallConv};
use super::diagnostic::{DiagCode, Diagnostic};
use super::inline_asm::{self, RES_GP, clobber_mask, gpr_mask};
use super::ir::{
    BlockId, FuncId, GlobalId, InlineFrame, InlineSite, IrAsmOutKind, IrAsmOutput, IrBinary, IrOp,
    IrUnary, LocalId, TypedFunction, TypedGlobal, TypedProgram, VReg,
};
use super::shift::{self, ShiftKind};
use super::source::{FileId, IdGen, NodeId, SourceSpan};
use super::source_map::static_timing;
use super::types::{CType, PtrType};
use super::z80::{
    AluSrc, AsmInstructionId, Cc, Contract, DataRef, GeneratedFunction, GeneratedGlobal,
    GeneratedProgram, InlineReport, MappedInstruction, MappedKind, R8, RegHome, Rr, SpillReport,
    StackProvenance, Z80Item, Z80Op, asm_block_label, asm_global_label, asm_label,
    bytes_in_segments, render_items,
};
use rtvc_core::asm::assemble_program;
use rtvc_core::disasm::disassemble_at;
use std::collections::{HashMap, HashSet};

pub const DEFAULT_CODE_ORIGIN: u16 = 0x8000;

/// `bus` is scratch memory shared by all items of one mapping run: the
/// item's bytes are written for the disassembler and cleared again.
fn map_z80_item(
    bus: &mut rtvc_core::bus::FakeBus,
    item: &Z80Item,
    addr: u16,
    bytes: Vec<u8>,
    func: Option<FuncId>,
) -> Option<MappedInstruction> {
    let mapped = map_z80_item_in(bus, item, addr, bytes, func);
    for i in 0..8u16 {
        bus.mem[addr.wrapping_add(i) as usize] = 0;
    }
    mapped
}

fn map_z80_item_in(
    bus: &mut rtvc_core::bus::FakeBus,
    item: &Z80Item,
    addr: u16,
    bytes: Vec<u8>,
    func: Option<FuncId>,
) -> Option<MappedInstruction> {
    for (i, b) in bytes.iter().take(8).enumerate() {
        bus.mem[addr.wrapping_add(i as u16) as usize] = *b;
    }
    let (id, text, span, expr, synthetic, node, kind, t_states) = match item {
        Z80Item::Instruction {
            id,
            op,
            span,
            expr,
            synthetic,
            node,
            ..
        } => {
            let d = disassemble_at(bus, addr);
            (
                *id,
                op.render(),
                *span,
                *expr,
                *synthetic,
                *node,
                MappedKind::Instruction,
                d.t_states,
            )
        }
        Z80Item::Raw {
            id,
            text,
            span,
            expr,
            synthetic,
            node,
            ..
        } => {
            let d = disassemble_at(bus, addr);
            (
                *id,
                text.clone(),
                *span,
                *expr,
                *synthetic,
                *node,
                MappedKind::Instruction,
                d.t_states,
            )
        }
        Z80Item::Data {
            id,
            text,
            span,
            node,
            synthetic,
        } => (
            *id,
            text.clone(),
            *span,
            None,
            *synthetic,
            Some(*node),
            MappedKind::Data,
            None,
        ),
        Z80Item::Label { .. } | Z80Item::Directive { .. } => return None,
    };
    let timing = static_timing(&bytes, t_states);
    Some(MappedInstruction {
        id,
        address: addr,
        bytes,
        text,
        t_states,
        timing,
        kind,
        synthetic,
        expression_span: expr,
        statement_span: Some(span),
        function: func,
        span,
        node,
        inlined: Vec::new(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Loc {
    Byte(R8),
    Word(Rr),
    Imm8(u8),
    Imm16(u16),
    /// `(IX+disp)`: only `@stackcall` parameters (H2a).
    Frame {
        disp: i8,
        width: u8,
    },
    /// A static slot: `width` bytes at `<function>_S+off`, emitted as zero
    /// data right after the function (H2a). Spills, overflow locals, and
    /// address-taken locals live here.
    Slot {
        off: u16,
        width: u8,
    },
}

impl Loc {
    fn from_home(home: RegHome) -> Self {
        match home {
            RegHome::Byte(r) => Self::Byte(r),
            RegHome::Word(rr) => Self::Word(rr),
        }
    }

    fn regs(self) -> &'static [R8] {
        match self {
            Self::Byte(r) => match r {
                R8::A => &[R8::A],
                R8::B => &[R8::B],
                R8::C => &[R8::C],
                R8::D => &[R8::D],
                R8::E => &[R8::E],
                R8::H => &[R8::H],
                R8::L => &[R8::L],
            },
            Self::Word(Rr::Bc) => &[R8::B, R8::C],
            Self::Word(Rr::De) => &[R8::D, R8::E],
            Self::Word(Rr::Hl) => &[R8::H, R8::L],
            Self::Imm8(_) | Self::Imm16(_) | Self::Frame { .. } | Self::Slot { .. } => &[],
        }
    }

    fn is_mem(self) -> bool {
        matches!(self, Self::Frame { .. } | Self::Slot { .. })
    }

    /// Byte range of a slot.
    fn slot_range(self) -> Option<(u16, u16)> {
        match self {
            Self::Slot { off, width } => Some((off, off + u16::from(width.max(1)))),
            _ => None,
        }
    }

    /// The two locations share a register or slot bytes.
    fn overlaps(self, other: Loc) -> bool {
        if self.regs().iter().any(|r| other.regs().contains(r)) {
            return true;
        }
        match (self.slot_range(), other.slot_range()) {
            (Some((a, b)), Some((c, d))) => a < d && c < b,
            _ => false,
        }
    }

    fn as_home(self) -> Option<RegHome> {
        match self {
            Self::Byte(r) => Some(RegHome::Byte(r)),
            Self::Word(rr) => Some(RegHome::Word(rr)),
            _ => None,
        }
    }

    fn width(self) -> u8 {
        match self {
            Self::Byte(_) | Self::Imm8(_) => 1,
            Self::Word(_) | Self::Imm16(_) => 2,
            Self::Frame { width, .. } | Self::Slot { width, .. } => width,
        }
    }
}

fn loc_reg_name(loc: Loc) -> &'static str {
    match loc {
        Loc::Byte(r) => match r {
            R8::A => "a",
            R8::B => "b",
            R8::C => "c",
            R8::D => "d",
            R8::E => "e",
            R8::H => "h",
            R8::L => "l",
        },
        Loc::Word(Rr::Bc) => "bc",
        Loc::Word(Rr::De) => "de",
        Loc::Word(Rr::Hl) => "hl",
        _ => "memory",
    }
}

fn gpr_loc(reg: AsmGpr) -> Loc {
    match reg {
        AsmGpr::A => Loc::Byte(R8::A),
        AsmGpr::B => Loc::Byte(R8::B),
        AsmGpr::C => Loc::Byte(R8::C),
        AsmGpr::D => Loc::Byte(R8::D),
        AsmGpr::E => Loc::Byte(R8::E),
        AsmGpr::H => Loc::Byte(R8::H),
        AsmGpr::L => Loc::Byte(R8::L),
        AsmGpr::Bc => Loc::Word(Rr::Bc),
        AsmGpr::De => Loc::Word(Rr::De),
        AsmGpr::Hl => Loc::Word(Rr::Hl),
    }
}

fn mask_to_r8s(mask: u16) -> Vec<R8> {
    let mut out = Vec::new();
    for (bit, r) in [
        (inline_asm::RES_A, R8::A),
        (inline_asm::RES_B, R8::B),
        (inline_asm::RES_C, R8::C),
        (inline_asm::RES_D, R8::D),
        (inline_asm::RES_E, R8::E),
        (inline_asm::RES_H, R8::H),
        (inline_asm::RES_L, R8::L),
    ] {
        if mask & bit != 0 {
            out.push(r);
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    V(VReg),
    L(LocalId),
}

struct Lowerer<'a> {
    func: &'a TypedFunction,
    ids: &'a mut IdGen,
    items: Vec<Z80Item>,
    vreg_loc: HashMap<VReg, Loc>,
    local_loc: HashMap<LocalId, Loc>,
    vreg_last: HashMap<VReg, usize>,
    local_last: HashMap<LocalId, usize>,
    /// Locals live after each op (CFG fixed point), by function-wide op index.
    local_after: Vec<HashSet<LocalId>>,
    vreg_ty: HashMap<VReg, CType>,
    local_ty: HashMap<LocalId, CType>,
    op_index: usize,
    func_label: String,
    globals: HashMap<GlobalId, (String, CType)>,
    callees: &'a HashMap<FuncId, CalleeInfo>,
    next_aux: u32,
    stable: bool,
    uses_ix: bool,
    home_mode: HomeMode,
    /// Bytes of static slots handed out so far (the area after the function).
    slot_used: u16,
    /// Registers a parallel move or output capture still needs; spill code
    /// must not use them as scratch.
    move_blocked: Vec<R8>,
    /// Listing note for emitted instructions (`spill`).
    note: Option<&'static str>,
    /// Read positions (function-wide op index) of every vreg and local, for
    /// the farthest-next-use spill choice.
    vreg_uses: HashMap<VReg, Vec<usize>>,
    local_uses: HashMap<LocalId, Vec<usize>>,
    total_ops: usize,
    /// Bytes pushed per save site, and T-states of `; spill` code.
    saves: Vec<u16>,
    spill_tstates: u32,
    /// Callee and the bytes pushed below its return address at each call.
    call_sites: Vec<(FuncId, u16)>,
    /// Peak bytes this function pushes itself (saves, cycle breaks, inline
    /// asm), on top of its frame.
    asm_stack: u16,
    /// Bytes pushed by the code emitted so far in the current operation.
    push_depth: u16,
    synthetic: bool,
    /// Keys the current operation is (re)defining; the clobber guard lets
    /// their registers be written.
    defining: Vec<Key>,
    /// Registers that hold an unbound intermediate of the current operation
    /// (an in-place destination or a scratch copy). Allocation skips them.
    reserved: Vec<R8>,
    /// Dead operands whose value was copied into a scratch register of the
    /// current operation; evicting them again would only waste code.
    consumed: Vec<VReg>,
    /// First clobber-guard violation (H1): reported as `cg-internal`.
    guard_error: Option<Diagnostic>,
    /// Planned local homes in stable mode; every block starts from them.
    homes: HashMap<LocalId, Loc>,
    stack_unknown: bool,
    stack_declared: bool,
    target: C80Target,
    /// Explicit `@fastcall` (H2b): any fallback is an error.
    strict: bool,
    /// Pinned parameters and locals (H2b): register and scope.
    pins: HashMap<LocalId, (Loc, (usize, usize))>,
    /// Op indexes of the stores into each pinned local.
    pin_stores: HashMap<LocalId, Vec<usize>>,
    /// What the current operation is, for pin and strict diagnostics.
    cur_op: String,
    /// First strict-mode violation found while emitting.
    strict_error: Option<Diagnostic>,
    /// Registers inline `asm` blocks may change (declared outputs, inputs,
    /// clobbers): part of the inferred clobber set (H2c).
    asm_writes: Vec<R8>,
    /// The callee of the call being lowered, for contract diagnostics.
    cur_callee: Option<FuncId>,
    /// Locals a fallback (`; spill`) moved away from their home; their
    /// block-end move back is marked too.
    spill_moved: HashSet<LocalId>,
    /// The next IR op stores this value into this local (`x <<= 1`): a
    /// shift may then run in the local's own register.
    next_store: Option<(VReg, LocalId)>,
    /// Expanded-call chains of the function (H3i) and the current op's.
    sites: Vec<InlineSite>,
    cur_origin: u32,
    inline_of: HashMap<AsmInstructionId, Vec<InlineFrame>>,
}

struct CalleeInfo {
    name: String,
    /// Registers a call may change (H2c): the declared or inferred contract,
    /// otherwise every register.
    clobbers: Vec<R8>,
    /// Where `clobbers` comes from, for diagnostics.
    contract: Contract,
    label: String,
    conv: CallConv,
    params: Vec<CType>,
    ret: CType,
}

pub fn lower_program(
    program: &TypedProgram,
    origin: u16,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
    optimize: bool,
    target: C80Target,
) -> Option<GeneratedProgram> {
    let mut chunks = lower_to_chunks(program, ids, diagnostics, target)?;
    let mut done = None;
    if optimize {
        done = super::optimize::optimize_chunks(&mut chunks, origin, &[], diagnostics);
        if diagnostics.iter().any(Diagnostic::is_error) {
            return None;
        }
        // Tail calls (H5) push no return address.
        fill_stack_bounds(&mut chunks);
    }
    let (items, assembly) = render_chunks(&chunks);
    let assembled = match done.map_or_else(|| assemble_program(&assembly, origin), Ok) {
        Ok(assembled) => assembled,
        Err(err) => {
            let span = chunks
                .first()
                .map(|c| c.span())
                .unwrap_or_else(|| SourceSpan::point(FileId(0), 0));
            diagnostics.push(Diagnostic::error(
                DiagCode::CgInternal,
                span,
                format!("assembler rejected generated code: {err}"),
            ));
            return None;
        }
    };
    finish_generated(
        chunks,
        items,
        assembly,
        assembled,
        origin,
        Vec::new(),
        diagnostics,
    )
}

/// Trial lowering for automatic inlining (H3i): each emitted function's
/// optimized size, assembled alone (other chunks' labels are placeholders,
/// as in the per-unit layout pass), and its fallback bytes.
pub(crate) fn measure_functions(
    program: &TypedProgram,
    ids: &IdGen,
    target: C80Target,
) -> Option<HashMap<FuncId, super::inline::Measure>> {
    let mut ids = ids.clone();
    let mut diags = Vec::new();
    let mut chunks = lower_to_chunks(program, &mut ids, &mut diags, target)?;
    let labels: Vec<Vec<String>> = chunks
        .iter()
        .map(|c| {
            c.items()
                .iter()
                .filter_map(|item| match item {
                    Z80Item::Label { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect()
        })
        .collect();
    let mut out = HashMap::new();
    for i in 0..chunks.len() {
        let (id, spill) = match &chunks[i] {
            EmitChunk::Func(part) => (
                part.id,
                u32::from(part.spill.slot_bytes)
                    + part.spill.saves.iter().map(|&b| u32::from(b)).sum::<u32>(),
            ),
            EmitChunk::Global(_) => continue,
        };
        let externals: Vec<String> = labels
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .flat_map(|(_, l)| l.iter().cloned())
            .collect();
        let assembled = super::optimize::optimize_chunks(
            &mut chunks[i..i + 1],
            DEFAULT_CODE_ORIGIN,
            &externals,
            &mut diags,
        )?;
        out.insert(
            id,
            super::inline::Measure {
                bytes: assembled.bytes.len() as u32,
                spill,
            },
        );
    }
    Some(out)
}

pub(crate) fn lower_to_chunks(
    program: &TypedProgram,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
    target: C80Target,
) -> Option<Vec<EmitChunk>> {
    let global_map = global_symbols(program);
    let ref_targets: HashMap<GlobalId, (String, String)> = program
        .globals
        .iter()
        .map(|g| (g.id, (g.name.clone(), asm_global_label(g.id, &g.name))))
        .collect();
    let mut callee_map = callee_map(program);
    let mut order: Vec<OrderItem<'_>> = program.globals.iter().map(OrderItem::Global).collect();
    order.extend(program.functions.iter().map(OrderItem::Func));
    order.sort_by_key(|item| item.span_key());

    // Functions are lowered callees first (the call graph is acyclic), so a
    // caller sees each private callee's inferred clobber set (H2c). Output
    // order stays source order.
    let mut lowered: HashMap<FuncId, FnPart> = HashMap::new();
    let called: HashSet<FuncId> = program
        .functions
        .iter()
        .flat_map(|f| f.blocks.iter().flat_map(|b| b.ops.iter()))
        .filter_map(|op| match op {
            IrOp::Call { func, .. } => Some(*func),
            _ => None,
        })
        .collect();
    let mut failed = false;
    for func in callee_first(program) {
        match lower_function(func, &callee_map, &global_map, ids, target) {
            Ok(mut part) => {
                let (preserves, contract) = contract_of(func, &part);
                part.preserves = preserves.clone();
                part.contract = contract;
                // The listing shows a contract where a caller relies on it:
                // every declared one, and inferred ones of called functions.
                if contract == Contract::Declared
                    || (contract == Contract::Inferred && called.contains(&func.id))
                {
                    let regs: Vec<&str> = preserves.iter().map(|r| r.name()).collect();
                    let text = format!("; preserves {} ({})", regs.join(" "), contract.as_str());
                    part.items.push(Z80Item::Directive {
                        id: AsmInstructionId(ids.next().0),
                        text,
                        span: func.span,
                        node: Some(func.id.0),
                    });
                }
                if let Some(info) = callee_map.get_mut(&func.id) {
                    info.clobbers = R8::ALL
                        .into_iter()
                        .filter(|r| !preserves.contains(r))
                        .collect();
                    info.contract = contract;
                }
                lowered.insert(func.id, part);
            }
            Err(diag) => {
                diagnostics.push(diag);
                failed = true;
            }
        }
    }

    let mut chunks: Vec<EmitChunk> = Vec::new();
    for item in order {
        match item {
            OrderItem::Global(global) if global.is_const && global.extra.is_empty() => {}
            OrderItem::Global(global) => match lower_global(global, &ref_targets, ids) {
                Ok(part) => chunks.push(EmitChunk::Global(part)),
                Err(diag) => {
                    diagnostics.push(diag);
                    failed = true;
                }
            },
            OrderItem::Func(func) => {
                if let Some(part) = lowered.remove(&func.id) {
                    // A private `@inline` function, or a private function
                    // whose every call was inlined automatically, has no
                    // out-of-line copy (H3i); it was lowered only to check
                    // its contracts.
                    if !part.inline_only {
                        chunks.push(EmitChunk::Func(part));
                    }
                }
            }
        }
    }

    if failed {
        return None;
    }
    fill_stack_bounds(&mut chunks);
    Some(chunks)
}

/// Expansion sizes per outermost call site, in listing order (H3i).
/// `calls` adds the expansions that left no bytes (a constant result).
fn inline_reports(mapped: &[MappedInstruction], calls: &[InlineFrame]) -> Vec<InlineReport> {
    let mut out: Vec<InlineReport> = Vec::new();
    for m in mapped {
        let Some(frame) = m.inlined.first() else {
            continue;
        };
        match out.iter_mut().find(|r| r.call == frame.call) {
            Some(r) => r.bytes += m.bytes.len() as u16,
            None => out.push(InlineReport {
                callee: frame.name.clone(),
                call: frame.call,
                auto: frame.auto,
                bytes: m.bytes.len() as u16,
            }),
        }
    }
    for frame in calls {
        if !out.iter().any(|r| r.call == frame.call) {
            out.push(InlineReport {
                callee: frame.name.clone(),
                call: frame.call,
                auto: frame.auto,
                bytes: 0,
            });
        }
    }
    out
}

pub(crate) fn chunks_emit_bytes(chunks: &[EmitChunk]) -> bool {
    chunks
        .iter()
        .any(|chunk| chunk.items().iter().any(|item| item.emits_bytes()))
}

pub(crate) fn render_chunks(chunks: &[EmitChunk]) -> (Vec<Z80Item>, String) {
    let mut items = Vec::new();
    for chunk in chunks {
        items.extend(chunk.items().iter().cloned());
    }
    let assembly = render_items(&items);
    (items, assembly)
}

pub(crate) fn finish_generated(
    chunks: Vec<EmitChunk>,
    items: Vec<Z80Item>,
    assembly: String,
    assembled: rtvc_core::asm::AssembledProgram,
    origin: u16,
    extra_functions: Vec<GeneratedFunction>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<GeneratedProgram> {
    let emitting: Vec<&Z80Item> = items.iter().filter(|i| i.emits_bytes()).collect();
    let mapped_all = map_emitting_items(&emitting, &assembled, &chunks, diagnostics)?;

    let mut functions = extra_functions;
    let mut globals = Vec::new();
    let mut map_i = 0usize;
    for chunk in chunks {
        let emit_count = chunk.items().iter().filter(|i| i.emits_bytes()).count();
        let mapped = mapped_all[map_i..map_i + emit_count].to_vec();
        map_i += emit_count;
        match chunk {
            EmitChunk::Func(part) => {
                let code_mapped: Vec<_> = mapped
                    .into_iter()
                    .filter(|m| {
                        part.items
                            .iter()
                            .any(|item| item.emits_bytes() && item.id() == m.id)
                    })
                    .collect();
                let addr = code_mapped
                    .first()
                    .map(|m| m.address)
                    .or_else(|| assembled.symbols.get(&part.label).copied())
                    .unwrap_or(origin);
                let size = code_mapped.iter().map(|m| m.bytes.len() as u16).sum();
                functions.push(GeneratedFunction {
                    name: part.name,
                    label: part.label,
                    id: part.id,
                    span: part.span,
                    addr,
                    size,
                    conv: part.conv,
                    strict_registers: part.strict_registers,
                    preserves: part.preserves.clone(),
                    contract: part.contract,
                    param_homes: part.param_homes,
                    param_types: part.param_types,
                    ret: part.ret,
                    stack_bound: part.stack_bound,
                    stack_provenance: part.stack_provenance,
                    frame_bytes: part.frame_bytes,
                    spill: part.spill.clone(),
                    instruction_ids: code_mapped.iter().map(|m| m.id).collect(),
                    mapped: {
                        let mut mapped = code_mapped;
                        for m in &mut mapped {
                            m.function = Some(part.id);
                            if let Some(frames) = part.inline_of.get(&m.id) {
                                m.inlined = frames.clone();
                            }
                        }
                        mapped
                    },
                    inlined: Vec::new(),
                });
                let f = functions.last_mut().unwrap();
                f.inlined = inline_reports(&f.mapped, &part.inline_calls);
            }
            EmitChunk::Global(part) => {
                let addr = mapped
                    .first()
                    .map(|m| m.address)
                    .or_else(|| assembled.symbols.get(&part.label).copied())
                    .unwrap_or(origin);
                let size = mapped.iter().map(|m| m.bytes.len() as u16).sum();
                globals.push(GeneratedGlobal {
                    name: part.name,
                    label: part.label,
                    addr,
                    size,
                    ty: part.ty,
                    span: part.span,
                    refs: part.refs,
                });
            }
        }
    }

    Some(GeneratedProgram {
        origin,
        items,
        assembly,
        assembled,
        functions,
        globals,
    })
}

fn map_emitting_items(
    emitting: &[&Z80Item],
    assembled: &rtvc_core::asm::AssembledProgram,
    chunks: &[EmitChunk],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Vec<MappedInstruction>> {
    if emitting.len() != assembled.lines.len() {
        diagnostics.push(Diagnostic::error(
            DiagCode::CgInternal,
            chunks
                .first()
                .map(|c| c.span())
                .unwrap_or_else(|| SourceSpan::point(FileId(0), 0)),
            format!(
                "assembled line count {} does not match generated items {}",
                assembled.lines.len(),
                emitting.len()
            ),
        ));
        return None;
    }
    map_item_lines(emitting, assembled)
}

pub(crate) fn map_item_lines(
    emitting: &[&Z80Item],
    assembled: &rtvc_core::asm::AssembledProgram,
) -> Option<Vec<MappedInstruction>> {
    let mut mapped_all = Vec::new();
    let mut bus = Box::new(rtvc_core::bus::FakeBus::new());
    for (item, line) in emitting.iter().zip(assembled.lines.iter()) {
        let bytes = bytes_in_segments(assembled, line.addr, line.len)
            .map(|b| b.to_vec())
            .unwrap_or_else(|| {
                let start = line.addr.wrapping_sub(assembled.origin) as usize;
                assembled
                    .bytes
                    .get(start..start + line.len)
                    .unwrap_or(&[])
                    .to_vec()
            });
        if let Some(mapped) = map_z80_item(&mut bus, item, line.addr, bytes, None) {
            mapped_all.push(mapped);
        }
    }
    Some(mapped_all)
}

pub(crate) fn map_chunks_from_lines(
    chunks: &[EmitChunk],
    assembled: &rtvc_core::asm::AssembledProgram,
    line_lo: usize,
    line_hi: usize,
    origin: u16,
) -> (Vec<GeneratedFunction>, Vec<GeneratedGlobal>) {
    let emitting: Vec<&Z80Item> = chunks
        .iter()
        .flat_map(|c| c.items().iter())
        .filter(|i| i.emits_bytes())
        .collect();
    let unit_lines: Vec<_> = assembled
        .lines
        .iter()
        .filter(|line| line.line >= line_lo && line.line < line_hi)
        .cloned()
        .collect();
    let mut mapped_all = Vec::new();
    let mut bus = Box::new(rtvc_core::bus::FakeBus::new());
    for (item, line) in emitting.iter().zip(unit_lines.iter()) {
        let bytes = bytes_in_segments(assembled, line.addr, line.len)
            .map(|b| b.to_vec())
            .unwrap_or_default();
        if let Some(mapped) = map_z80_item(&mut bus, item, line.addr, bytes, None) {
            mapped_all.push(mapped);
        }
    }
    let mut functions = Vec::new();
    let mut globals = Vec::new();
    let mut map_i = 0usize;
    for chunk in chunks {
        let emit_count = chunk.items().iter().filter(|i| i.emits_bytes()).count();
        let mapped = mapped_all
            .get(map_i..map_i + emit_count)
            .unwrap_or(&[])
            .to_vec();
        map_i += emit_count;
        match chunk {
            EmitChunk::Func(part) => {
                let code_mapped: Vec<_> = mapped
                    .iter()
                    .filter(|m| {
                        part.items
                            .iter()
                            .any(|item| item.emits_bytes() && item.id() == m.id)
                    })
                    .cloned()
                    .collect();
                let addr = code_mapped
                    .first()
                    .map(|m| m.address)
                    .or_else(|| assembled.symbols.get(&part.label).copied())
                    .unwrap_or(origin);
                let size = code_mapped.iter().map(|m| m.bytes.len() as u16).sum();
                functions.push(GeneratedFunction {
                    name: part.name.clone(),
                    label: part.label.clone(),
                    id: part.id,
                    span: part.span,
                    addr,
                    size,
                    conv: part.conv,
                    strict_registers: part.strict_registers,
                    preserves: part.preserves.clone(),
                    contract: part.contract,
                    param_homes: part.param_homes.clone(),
                    param_types: part.param_types.clone(),
                    ret: part.ret,
                    stack_bound: part.stack_bound,
                    stack_provenance: part.stack_provenance,
                    frame_bytes: part.frame_bytes,
                    spill: part.spill.clone(),
                    instruction_ids: code_mapped.iter().map(|m| m.id).collect(),
                    mapped: {
                        let mut mapped = code_mapped;
                        for m in &mut mapped {
                            m.function = Some(part.id);
                            if let Some(frames) = part.inline_of.get(&m.id) {
                                m.inlined = frames.clone();
                            }
                        }
                        mapped
                    },
                    inlined: Vec::new(),
                });
                let f = functions.last_mut().unwrap();
                f.inlined = inline_reports(&f.mapped, &part.inline_calls);
            }
            EmitChunk::Global(part) => {
                let addr = mapped
                    .first()
                    .map(|m| m.address)
                    .or_else(|| assembled.symbols.get(&part.label).copied())
                    .unwrap_or(origin);
                let size = mapped.iter().map(|m| m.bytes.len() as u16).sum();
                globals.push(GeneratedGlobal {
                    name: part.name.clone(),
                    label: part.label.clone(),
                    addr,
                    size,
                    ty: part.ty,
                    span: part.span,
                    refs: part.refs.clone(),
                });
            }
        }
    }
    (functions, globals)
}

pub(crate) fn max_stack_bound(chunks: &[EmitChunk]) -> u16 {
    chunks
        .iter()
        .filter_map(|chunk| match chunk {
            EmitChunk::Func(part) => Some(part.stack_bound),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

enum OrderItem<'a> {
    Global(&'a TypedGlobal),
    Func(&'a TypedFunction),
}

impl OrderItem<'_> {
    fn span_key(&self) -> (u32, u32) {
        let span = match self {
            Self::Global(g) => g.span,
            Self::Func(f) => f.span,
        };
        (span.file.0, span.start)
    }
}

pub(crate) enum EmitChunk {
    Func(FnPart),
    Global(GlobalPart),
}

impl EmitChunk {
    /// Registers a `RET` of this chunk hands back to its caller: the return
    /// value and the registers its contract preserves. Everything else is
    /// dead at `RET` (H5 liveness).
    pub(crate) fn ret_regs(&self) -> Vec<R8> {
        match self {
            Self::Func(p) => {
                let mut regs = p.preserves.clone();
                match p.ret {
                    Some(RegHome::Byte(r)) => regs.push(r),
                    Some(RegHome::Word(rr)) => {
                        let (h, l) = rr.halves();
                        regs.extend([h, l]);
                    }
                    None => {}
                }
                regs
            }
            Self::Global(_) => R8::ALL.to_vec(),
        }
    }

    /// Whether a `CALL x` / `RET` pair of this chunk may become `JP x`:
    /// functions only (H5).
    pub(crate) fn allows_tail_calls(&self) -> bool {
        matches!(self, Self::Func(_))
    }

    /// Records that a `CALL target` became `JP target` (H5).
    pub(crate) fn record_tail_call(&mut self, target: String) {
        if let Self::Func(p) = self {
            p.tail_calls.push(target);
        }
    }

    pub(crate) fn items(&self) -> &[Z80Item] {
        match self {
            Self::Func(p) => &p.items,
            Self::Global(p) => &p.items,
        }
    }

    pub(crate) fn items_mut(&mut self) -> &mut Vec<Z80Item> {
        match self {
            Self::Func(p) => &mut p.items,
            Self::Global(p) => &mut p.items,
        }
    }

    pub(crate) fn span(&self) -> SourceSpan {
        match self {
            Self::Func(p) => p.span,
            Self::Global(p) => p.span,
        }
    }

    pub(crate) fn file_id(&self) -> FileId {
        self.span().file
    }
}

/// Values pushed around one call, `LDIR` or inline-asm block.
#[derive(Debug, Default)]
struct SavePlan {
    /// Pairs pushed, in push order.
    pushed: Vec<Rr>,
    /// A pushed pair popped into another pair (it holds a result).
    pop_to: Vec<(Rr, Rr)>,
    /// `PUSH AF` after the pairs.
    af: bool,
    /// Saved keys and their location after the pops.
    groups: Vec<(Vec<Key>, Loc)>,
    bytes: u16,
}

pub(crate) struct FnPart {
    name: String,
    label: String,
    id: FuncId,
    span: SourceSpan,
    items: Vec<Z80Item>,
    conv: CallConv,
    strict_registers: bool,
    param_homes: Vec<RegHome>,
    param_types: Vec<CType>,
    ret: Option<RegHome>,
    stack_bound: u16,
    stack_provenance: StackProvenance,
    frame_bytes: u16,
    spill: SpillReport,
    call_sites: Vec<(FuncId, u16)>,
    /// Call sites the optimizer turned into `JP callee` (H5 tail calls), by
    /// callee label: they push no return address.
    tail_calls: Vec<String>,
    asm_stack: u16,
    /// Registers inline `asm` may change (H2c inference).
    asm_writes: Vec<R8>,
    /// The function's register contract, set once it is lowered (H2c).
    preserves: Vec<R8>,
    contract: Contract,
    /// Expanded-call chain of each emitted item that came from an inlined
    /// body (H3i), by item id.
    inline_of: HashMap<AsmInstructionId, Vec<InlineFrame>>,
    /// `@inline` and private: lowered for its checks, never emitted.
    inline_only: bool,
    /// Outermost expanded calls, in expansion order (H3i reports).
    inline_calls: Vec<InlineFrame>,
}

pub(crate) struct GlobalPart {
    name: String,
    label: String,
    ty: CType,
    span: SourceSpan,
    items: Vec<Z80Item>,
    /// Words holding another global's address (H3a), for the map.
    refs: Vec<DataRef>,
}

/// Functions in an order where every callee comes before its callers:
/// depth-first post-order from each function in source order, callees in
/// the order their calls appear. Deterministic for a given program.
fn callee_first(program: &TypedProgram) -> Vec<&TypedFunction> {
    let by_id: HashMap<FuncId, &TypedFunction> =
        program.functions.iter().map(|f| (f.id, f)).collect();
    let mut roots: Vec<&TypedFunction> = program.functions.iter().collect();
    roots.sort_by_key(|f| (f.span.file.0, f.span.start));
    let mut seen: HashSet<FuncId> = HashSet::new();
    let mut out = Vec::new();
    fn visit<'a>(
        f: &'a TypedFunction,
        by_id: &HashMap<FuncId, &'a TypedFunction>,
        seen: &mut HashSet<FuncId>,
        out: &mut Vec<&'a TypedFunction>,
    ) {
        if !seen.insert(f.id) {
            return;
        }
        for block in &f.blocks {
            for op in &block.ops {
                if let IrOp::Call { func, .. } = op {
                    if let Some(callee) = by_id.get(func) {
                        visit(callee, by_id, seen, out);
                    }
                }
            }
        }
        out.push(f);
    }
    for f in roots {
        visit(f, &by_id, &mut seen, &mut out);
    }
    out
}

/// The register contract of a lowered function (H2c, F-028): declared
/// `@preserves`, or for a private function the registers its code never
/// writes (calls count their callee's clobbers, `asm` its declared
/// registers). A `pub` function without `@preserves` has none.
fn contract_of(func: &TypedFunction, part: &FnPart) -> (Vec<R8>, Contract) {
    if func.preserves != 0 {
        return (mask_to_r8s(func.preserves), Contract::Declared);
    }
    if func.is_pub {
        return (Vec::new(), Contract::None);
    }
    // A result register is never promised, even when the code leaves it as
    // it came in (`u8 id(u8 x) { return x; }` is just `RET`).
    let mut written: Vec<R8> = part.asm_writes.clone();
    if let Some(home) = part.ret {
        written.extend_from_slice(Loc::from_home(home).regs());
    }
    for item in &part.items {
        if let Z80Item::Instruction { op, .. } = item {
            written.extend(op.writes());
        }
    }
    let kept: Vec<R8> = R8::ALL
        .into_iter()
        .filter(|r| !written.contains(r))
        .collect();
    if kept.is_empty() {
        (kept, Contract::None)
    } else {
        (kept, Contract::Inferred)
    }
}

/// Pseudo-locals for `@preserves` registers: a whole pair when both halves
/// are preserved, otherwise single bytes.
fn preserve_pins(func: &TypedFunction) -> Vec<(LocalId, Loc)> {
    let regs = mask_to_r8s(func.preserves);
    let mut out = Vec::new();
    let mut next = 0u32;
    let mut id = || {
        next += 1;
        LocalId(NodeId(PRESERVE_ID_BASE + next))
    };
    if regs.contains(&R8::A) {
        out.push((id(), Loc::Byte(R8::A)));
    }
    for rr in [Rr::Bc, Rr::De, Rr::Hl] {
        let (h, l) = rr.halves();
        match (regs.contains(&h), regs.contains(&l)) {
            (true, true) => out.push((id(), Loc::Word(rr))),
            (true, false) => out.push((id(), Loc::Byte(h))),
            (false, true) => out.push((id(), Loc::Byte(l))),
            (false, false) => {}
        }
    }
    out
}

/// Ids of `@preserves` pseudo-locals start above every real node id.
const PRESERVE_ID_BASE: u32 = u32::MAX - 64;

fn is_preserve_key(l: LocalId) -> bool {
    l.0.0 > PRESERVE_ID_BASE
}

fn callee_map(program: &TypedProgram) -> HashMap<FuncId, CalleeInfo> {
    program
        .functions
        .iter()
        .map(|f| {
            (
                f.id,
                CalleeInfo {
                    name: f.name.clone(),
                    clobbers: if f.preserves != 0 {
                        R8::ALL
                            .into_iter()
                            .filter(|r| !mask_to_r8s(f.preserves).contains(r))
                            .collect()
                    } else {
                        R8::ALL.to_vec()
                    },
                    contract: if f.preserves != 0 {
                        Contract::Declared
                    } else {
                        Contract::None
                    },
                    label: asm_label(f.id, &f.name),
                    conv: f.conv,
                    params: f.params.iter().map(|p| p.ty).collect(),
                    ret: f.ret,
                },
            )
        })
        .collect()
}

fn global_symbols(program: &TypedProgram) -> HashMap<GlobalId, (String, CType)> {
    program
        .globals
        .iter()
        .map(|g| (g.id, (asm_global_label(g.id, &g.name), g.ty)))
        .collect()
}

pub(crate) fn fill_stack_bounds(chunks: &mut [EmitChunk]) {
    let mut ids: HashMap<FuncId, usize> = HashMap::new();
    let mut by_label: HashMap<String, FuncId> = HashMap::new();
    for (i, chunk) in chunks.iter().enumerate() {
        if let EmitChunk::Func(part) = chunk {
            ids.insert(part.id, i);
            by_label.insert(part.label.clone(), part.id);
        }
    }
    let mut memo: HashMap<FuncId, u16> = HashMap::new();
    fn bound_of(
        chunks: &[EmitChunk],
        ids: &HashMap<FuncId, usize>,
        by_label: &HashMap<String, FuncId>,
        memo: &mut HashMap<FuncId, u16>,
        id: FuncId,
    ) -> u16 {
        if let Some(&b) = memo.get(&id) {
            return b;
        }
        memo.insert(id, 0);
        let Some(&idx) = ids.get(&id) else {
            return 0;
        };
        let EmitChunk::Func(part) = &chunks[idx] else {
            return 0;
        };
        let mut extra = 0u16;
        let mut sites = part.call_sites.clone();
        let frame = part.frame_bytes;
        let asm = part.asm_stack;
        // A tail call (H5) is `JP callee` with nothing of this function on
        // the stack: the callee starts at this function's entry SP. One
        // plain site of that callee is replaced by it.
        let mut tail = 0u16;
        for label in part.tail_calls.clone() {
            let Some(callee) = by_label.get(&label).copied() else {
                continue;
            };
            if let Some(k) = sites
                .iter()
                .position(|&(c, slots)| c == callee && slots == 0)
            {
                sites.remove(k);
                tail = tail.max(bound_of(chunks, ids, by_label, memo, callee));
            }
        }
        for (callee, arg_slots) in sites {
            extra = extra.max(
                arg_slots
                    .saturating_add(2)
                    .saturating_add(bound_of(chunks, ids, by_label, memo, callee)),
            );
        }
        extra = extra.max(asm);
        let bound = frame.saturating_add(extra).max(tail);
        memo.insert(id, bound);
        bound
    }
    let computed: Vec<(usize, u16)> = chunks
        .iter()
        .enumerate()
        .filter_map(|(i, chunk)| match chunk {
            EmitChunk::Func(part) => {
                Some((i, bound_of(chunks, &ids, &by_label, &mut memo, part.id)))
            }
            _ => None,
        })
        .collect();
    for (i, bound) in computed {
        if let EmitChunk::Func(part) = &mut chunks[i] {
            part.stack_bound = bound;
        }
    }
}

fn lower_global(
    global: &TypedGlobal,
    targets: &HashMap<GlobalId, (String, String)>,
    ids: &mut IdGen,
) -> Result<GlobalPart, Diagnostic> {
    let label = asm_global_label(global.id, &global.name);
    let bits = global.init.unwrap_or(0);
    let bytes: Vec<u8> = if !global.extra.is_empty() {
        global.extra.clone()
    } else {
        match global.ty.byte_width() {
            Some(1) => vec![bits as u8],
            Some(2) => bits.to_le_bytes().to_vec(),
            _ => {
                return Err(Diagnostic::error(
                    DiagCode::CgUnsupported,
                    global.span,
                    format!("cannot emit global '{}'", global.name),
                ));
            }
        }
    };
    // Plain data is one `DB`/`DW` line as before; a link-time address (H3a)
    // is its own `DW label+offset` line between `DB` runs.
    let mut refs = Vec::new();
    let mut lines = Vec::new();
    if global.relocs.is_empty() {
        lines.push(if global.extra.is_empty() && bytes.len() == 2 {
            format!("DW {bits}")
        } else {
            format!("DB {}", join_bytes(&bytes))
        });
    } else {
        let mut relocs = global.relocs.clone();
        relocs.sort_by_key(|r| r.offset);
        let mut at = 0usize;
        for reloc in relocs {
            let off = usize::from(reloc.offset);
            let Some((target_name, target_label)) = targets.get(&reloc.target) else {
                return Err(Diagnostic::error(
                    DiagCode::CgInternal,
                    global.span,
                    format!(
                        "static address in '{}' names an unknown global",
                        global.name
                    ),
                ));
            };
            if off < at || off + 2 > bytes.len() {
                return Err(Diagnostic::error(
                    DiagCode::CgInternal,
                    global.span,
                    format!("static address in '{}' is outside its data", global.name),
                ));
            }
            if off > at {
                lines.push(format!("DB {}", join_bytes(&bytes[at..off])));
            }
            lines.push(if reloc.addend == 0 {
                format!("DW {target_label}")
            } else {
                format!("DW {target_label}+{}", reloc.addend)
            });
            refs.push(DataRef {
                offset: reloc.offset,
                target: target_name.clone(),
                label: target_label.clone(),
                addend: reloc.addend,
            });
            at = off + 2;
        }
        if at < bytes.len() {
            lines.push(format!("DB {}", join_bytes(&bytes[at..])));
        }
    }
    let mut items = Vec::new();
    items.push(Z80Item::Label {
        id: AsmInstructionId(ids.next().0),
        name: label.clone(),
        span: global.span,
        node: global.id.0,
    });
    for text in lines {
        items.push(Z80Item::Data {
            id: AsmInstructionId(ids.next().0),
            text,
            span: global.span,
            node: global.id.0,
            synthetic: false,
        });
    }
    Ok(GlobalPart {
        name: global.name.clone(),
        label,
        ty: global.ty,
        span: global.span,
        items,
        refs,
    })
}

fn join_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn lower_function(
    func: &TypedFunction,
    callees: &HashMap<FuncId, CalleeInfo>,
    globals: &HashMap<GlobalId, (String, CType)>,
    ids: &mut IdGen,
    target: C80Target,
) -> Result<FnPart, Diagnostic> {
    let mut ids_a = ids.clone();
    let packed = lower_function_with(func, callees, globals, &mut ids_a, target, HomeMode::Packed);
    let needs_fallback = |part: &FnPart| {
        part.spill.slot_bytes > 0 || !part.spill.saves.is_empty() || part.spill.tstates > 0
    };
    let stable_regs = func.conv != CallConv::Stack
        && (func.blocks.len() > 1
            || func
                .blocks
                .iter()
                .any(|b| b.ops.iter().any(|op| matches!(op, IrOp::AddrLocal { .. }))));
    let try_spread = stable_regs && packed.as_ref().map_or(true, needs_fallback);
    let try_pairs = stable_regs && !pointer_locals(func).is_empty();
    if !try_spread && !try_pairs {
        *ids = ids_a;
        return packed;
    }
    let mut alternatives = Vec::new();
    if try_spread {
        alternatives.extend([HomeMode::Spread, HomeMode::Dense]);
    }
    if try_pairs {
        alternatives.push(HomeMode::PackedPairs);
    }
    // Cheapest estimated T-states wins; ties keep the earlier plan (the H1
    // plan first).
    let mut best: Option<(u64, FnPart, IdGen)> = None;
    let mut first_err = None;
    for (mode, result, idgen) in
        [(HomeMode::Packed, packed, ids_a)]
            .into_iter()
            .chain(alternatives.into_iter().map(|mode| {
                let mut idgen = ids.clone();
                let r = lower_function_with(func, callees, globals, &mut idgen, target, mode);
                (mode, r, idgen)
            }))
    {
        let _ = mode;
        match result {
            Ok(part) => {
                let cost = weighted_cost(func, &part);
                if best.as_ref().is_none_or(|(c, _, _)| cost < *c) {
                    best = Some((cost, part, idgen));
                }
            }
            Err(e) => {
                first_err.get_or_insert(e);
            }
        }
    }
    match best {
        Some((_, part, idgen)) => {
            *ids = idgen;
            Ok(part)
        }
        None => Err(first_err.expect("at least one plan was tried")),
    }
}

fn lower_function_with(
    func: &TypedFunction,
    callees: &HashMap<FuncId, CalleeInfo>,
    globals: &HashMap<GlobalId, (String, CType)>,
    ids: &mut IdGen,
    target: C80Target,
    home_mode: HomeMode,
) -> Result<FnPart, Diagnostic> {
    let param_tys: Vec<CType> = func.params.iter().map(|p| p.ty).collect();
    let param_homes = if func.conv == CallConv::Stack {
        Vec::new()
    } else {
        assign_params(&param_tys).ok_or_else(|| {
            Diagnostic::error(
                DiagCode::CgUnsupported,
                func.span,
                format!(
                    "register signature of '{}' does not fit; use explicit @stackcall",
                    func.name
                ),
            )
        })?
    };

    let (vreg_last, local_last) = liveness_fn(func);
    let local_after = local_live_after(func);
    let (vreg_uses, local_uses, total_ops) = use_positions(func);
    let mut local_ty = HashMap::new();
    for p in &func.params {
        local_ty.insert(p.id, p.ty);
    }
    for l in &func.locals {
        local_ty.insert(l.id, l.ty);
    }
    let label = asm_label(func.id, &func.name);
    // H2a: IX is only the `@stackcall` parameter frame. Values that do not
    // fit registers, or must survive a call, use PUSH/POP or static slots.
    let uses_ix = func.conv == CallConv::Stack;
    let needs_addr = func
        .blocks
        .iter()
        .any(|b| b.ops.iter().any(|op| matches!(op, IrOp::AddrLocal { .. })));
    let mut pins = pin_map(func);
    // Locals keep one home per function when control flow joins, when an
    // address is taken (the slot must not move), and when a pin fixes one.
    let stable = func.blocks.len() > 1 || uses_ix || needs_addr || !pins.is_empty();
    for (id, loc) in preserve_pins(func) {
        pins.insert(id, (loc, (0, usize::MAX)));
    }
    let mut lowerer = Lowerer {
        func,
        ids,
        items: Vec::new(),
        vreg_loc: HashMap::new(),
        local_loc: HashMap::new(),
        vreg_last,
        local_last,
        local_after,
        vreg_ty: HashMap::new(),
        local_ty,
        op_index: 0,
        func_label: label.clone(),
        globals: globals.clone(),
        callees,
        next_aux: 0,
        stable,
        uses_ix,
        home_mode,
        slot_used: 0,
        move_blocked: Vec::new(),
        note: None,
        vreg_uses,
        local_uses,
        total_ops,
        saves: Vec::new(),
        spill_tstates: 0,
        call_sites: Vec::new(),
        push_depth: 0,
        asm_stack: 0,
        synthetic: false,
        defining: Vec::new(),
        reserved: Vec::new(),
        consumed: Vec::new(),
        guard_error: None,
        homes: HashMap::new(),
        stack_unknown: false,
        stack_declared: false,
        target,
        strict: func.strict_registers,
        pin_stores: store_positions(func, &pins),
        pins,
        cur_op: String::new(),
        strict_error: None,
        asm_writes: Vec::new(),
        cur_callee: None,
        spill_moved: HashSet::new(),
        next_store: None,
        sites: func.inline_sites.clone(),
        cur_origin: 0,
        inline_of: HashMap::new(),
    };
    // `@preserves` (H2c): each preserved register holds the caller's value
    // as a pinned pseudo-local for the whole function.
    for (id, loc) in preserve_pins(func) {
        lowerer.local_loc.insert(id, loc);
    }
    lowerer.emit_label(&label, func.span, func.id.0);
    // The `@stackcall` frame comes first: a pinned stack parameter's entry
    // move reads it.
    lowerer.synthetic = true;
    if uses_ix {
        lowerer.emit_prologue(func.span);
    }
    lowerer.synthetic = false;
    lowerer.plan_homes(&param_homes)?;
    lowerer.homes = lowerer.local_loc.clone();

    let mut index = 0usize;
    let mut prev_origin = 0u32;
    for block in &func.blocks {
        lowerer.emit_label(&asm_block_label(&label, block.id), func.span, func.id.0);
        lowerer.vreg_loc.clear();
        if lowerer.stable && lowerer.local_loc != lowerer.homes {
            return Err(Diagnostic::error(
                DiagCode::CgInternal,
                func.span,
                format!(
                    "internal: a local left its home across a block boundary in '{}'",
                    func.name
                ),
            ));
        }
        let ops = &block.ops;
        let mut i = 0;
        while i < ops.len() {
            lowerer.op_index = index;
            lowerer.cur_op = describe_op(&ops[i], callees);
            let origin = block.origin_of(i);
            if origin != prev_origin {
                lowerer.mark_inline_entry(prev_origin, origin, ops[i].span());
                prev_origin = origin;
            }
            lowerer.cur_origin = origin;
            let items_before = lowerer.items.len();
            let strict_before = lowerer.strict_error.is_some();
            let guard_before = lowerer.guard_error.is_some();
            let step = lowerer.lower_one(func, ops, i, callees, index);
            lowerer.tag_inline_items(items_before);
            if !strict_before {
                if let Some(d) = lowerer.strict_error.take() {
                    lowerer.strict_error = Some(lowerer.with_origin(d));
                }
            }
            if !guard_before {
                if let Some(d) = lowerer.guard_error.take() {
                    lowerer.guard_error = Some(lowerer.with_origin(d));
                }
            }
            let n = step.map_err(|d| lowerer.with_origin(d))?;
            index += n;
            i += n;
        }
        lowerer.cur_origin = 0;
        if needs_fallthrough(ops) {
            if let Some(next) = next_block(func, block.id) {
                lowerer.op_index = index.saturating_sub(1);
                lowerer.restore_homes(func.span)?;
                lowerer.finish_op()?;
                lowerer.synthetic = true;
                lowerer.emit(
                    Z80Op::Jp {
                        cc: None,
                        target: asm_block_label(&label, next),
                    },
                    func.span,
                );
                lowerer.synthetic = false;
            } else {
                lowerer.emit_epilogue(func.span);
            }
        }
    }
    if let Some(err) = lowerer.strict_error.take() {
        return Err(err);
    }
    lowerer.emit_slot_area(func.span);

    let frame_bytes = if uses_ix { 2 } else { 0 };
    let stack_provenance = lowerer.stack_provenance();
    let spill = SpillReport {
        slot_bytes: lowerer.slot_used,
        saves: lowerer.saves.clone(),
        tstates: lowerer.spill_tstates,
    };
    Ok(FnPart {
        name: func.name.clone(),
        label,
        id: func.id,
        span: func.span,
        items: lowerer.items,
        conv: func.conv,
        strict_registers: func.strict_registers,
        param_homes,
        param_types: param_tys,
        ret: return_home(func.ret),
        stack_bound: 0,
        stack_provenance,
        frame_bytes,
        spill,
        call_sites: lowerer.call_sites,
        tail_calls: Vec::new(),
        asm_stack: lowerer.asm_stack,
        asm_writes: lowerer.asm_writes,
        preserves: Vec::new(),
        contract: Contract::None,
        inline_of: lowerer.inline_of,
        inline_only: !func.out_of_line,
        inline_calls: {
            let mut calls: Vec<InlineFrame> = Vec::new();
            for site in &func.inline_sites {
                if let Some(first) = site.frames.first() {
                    if !calls.iter().any(|c| c.call == first.call) {
                        calls.push(first.clone());
                    }
                }
            }
            calls
        },
    })
}

/// A short name for an operation in pin and strict diagnostics.
fn describe_op(op: &IrOp, callees: &HashMap<FuncId, CalleeInfo>) -> String {
    let bin = |op: IrBinary| match op {
        IrBinary::Or => "||",
        IrBinary::And => "&&",
        IrBinary::BitOr => "|",
        IrBinary::BitXor => "^",
        IrBinary::BitAnd => "&",
        IrBinary::Eq => "==",
        IrBinary::Ne => "!=",
        IrBinary::Lt => "<",
        IrBinary::Le => "<=",
        IrBinary::Gt => ">",
        IrBinary::Ge => ">=",
        IrBinary::Shl => "<<",
        IrBinary::Shr => ">>",
        IrBinary::Add => "+",
        IrBinary::Sub => "-",
    };
    match op {
        IrOp::Binary { ty, op, .. } => format!("the {} `{}`", ty.as_str(), bin(*op)),
        IrOp::Unary { ty, .. } => format!("the {} unary operator", ty.as_str()),
        IrOp::Cast { to, from, .. } => format!("the cast {} to {}", from.as_str(), to.as_str()),
        IrOp::Call { func, .. } => match callees.get(func) {
            Some(c) => format!("the call to '{}'", c.name),
            None => "the call".to_string(),
        },
        IrOp::Ldir { .. } => "cpu::ldir (BC, DE, HL)".to_string(),
        IrOp::PortIn { .. } => "cpu::in".to_string(),
        IrOp::PortOut { .. } => "cpu::out".to_string(),
        IrOp::InlineAsm { .. } => "the asm block".to_string(),
        IrOp::LoadIndirect { .. } => "the pointer load".to_string(),
        IrOp::StoreIndirect { .. } => "the pointer store".to_string(),
        IrOp::LoadGlobal { .. } | IrOp::StoreGlobal { .. } => "the global access".to_string(),
        IrOp::AddrGlobal { .. } | IrOp::AddrLocal { .. } => "the address".to_string(),
        IrOp::Return { .. } => "the return value".to_string(),
        IrOp::Branch { .. } => "the condition".to_string(),
        _ => "this statement".to_string(),
    }
}

/// Op indexes of the `StoreLocal`s into the pinned locals.
fn store_positions(
    func: &TypedFunction,
    pins: &HashMap<LocalId, (Loc, (usize, usize))>,
) -> HashMap<LocalId, Vec<usize>> {
    let mut out: HashMap<LocalId, Vec<usize>> = HashMap::new();
    let mut i = 0usize;
    for block in &func.blocks {
        for op in &block.ops {
            if let IrOp::StoreLocal { local, .. } = op {
                if pins.contains_key(local) {
                    out.entry(*local).or_default().push(i);
                }
            }
            i += 1;
        }
    }
    out
}

/// Pinned parameters and locals with their register and scope.
fn pin_map(func: &TypedFunction) -> HashMap<LocalId, (Loc, (usize, usize))> {
    func.params
        .iter()
        .map(|p| (p.id, p.pin))
        .chain(func.locals.iter().map(|l| (l.id, l.pin)))
        .filter_map(|(id, pin)| pin.map(|p| (id, (gpr_loc(p.reg), p.scope))))
        .collect()
}

/// Register homes for locals in declaration order. Byte homes come from
/// `C B E D L H`, word homes from `DE BC`, and no register is handed out
/// twice (F-026: byte and word homes used to be counted independently, so a
/// byte local could share `C` with a word local in `BC`).
#[derive(Default)]
struct HomeRegs {
    taken: Vec<R8>,
}

impl HomeRegs {
    const BYTES: [R8; 6] = [R8::C, R8::B, R8::E, R8::D, R8::L, R8::H];
    const WORDS: [Rr; 2] = [Rr::De, Rr::Bc];

    /// Homes for `(local, width)` in declaration order, without overlaps
    /// (F-026). Words take pairs first, because a byte in `C` or `E` would
    /// break a pair; bytes then take `C B E D`, and `L H` only when
    /// `allow_hl` (frame-free functions). When `allow_hl`, the first local in
    /// `hl_first` (a pointer) gets HL, where `LD r,(HL)` and `INC HL` work on
    /// it in place. Entries without a register are left out.
    fn plan(
        items: &[(LocalId, u8)],
        allow_hl: bool,
        hl_first: &HashSet<LocalId>,
        pre: &[R8],
    ) -> HashMap<LocalId, Loc> {
        let mut regs = HomeRegs {
            taken: pre.to_vec(),
        };
        let mut out = HashMap::new();
        let words = items.iter().filter(|(_, w)| *w == 2).count();
        let bytes = items.iter().filter(|(_, w)| *w == 1).count();
        if allow_hl
            && !pre.contains(&R8::H)
            && !pre.contains(&R8::L)
            && words < 3
            && 2 * words + bytes + pre.len() <= 6
        {
            if let Some(&(id, _)) = items
                .iter()
                .find(|(id, w)| *w == 2 && hl_first.contains(id))
            {
                regs.taken.extend([R8::H, R8::L]);
                out.insert(id, Loc::Word(Rr::Hl));
            }
        }
        let in_hl: Vec<LocalId> = out.keys().copied().collect();
        for &(id, width) in items
            .iter()
            .filter(|(id, w)| *w == 2 && !in_hl.contains(id))
        {
            if let Some(loc) = regs.take(width, allow_hl) {
                out.insert(id, loc);
            }
        }
        for &(id, width) in items.iter().filter(|(_, w)| *w == 1) {
            if let Some(loc) = regs.take(width, allow_hl) {
                out.insert(id, loc);
            }
        }
        out
    }

    fn take(&mut self, width: u8, allow_hl: bool) -> Option<Loc> {
        match width {
            1 => {
                let r = Self::BYTES
                    .into_iter()
                    .filter(|r| allow_hl || !matches!(r, R8::H | R8::L))
                    .find(|r| !self.taken.contains(r))?;
                self.taken.push(r);
                Some(Loc::Byte(r))
            }
            2 => {
                let rr = Self::WORDS.into_iter().find(|rr| {
                    let (h, l) = rr.halves();
                    !self.taken.contains(&h) && !self.taken.contains(&l)
                })?;
                let (h, l) = rr.halves();
                self.taken.extend([h, l]);
                Some(Loc::Word(rr))
            }
            _ => None,
        }
    }
}

/// Parameters and locals of pointer type.
fn pointer_locals(func: &TypedFunction) -> HashSet<LocalId> {
    func.params
        .iter()
        .map(|p| (p.id, p.ty))
        .chain(func.locals.iter().map(|l| (l.id, l.ty)))
        .filter(|(_, ty)| matches!(ty, CType::Ptr(_)))
        .map(|(id, _)| id)
        .collect()
}

/// Locals that need a stable home: parameters that are read (register
/// convention only) and every declared local, in declaration order.
fn stable_items(
    func: &TypedFunction,
    conv: CallConv,
    local_last: &HashMap<LocalId, usize>,
) -> Vec<(LocalId, u8)> {
    let params = func
        .params
        .iter()
        .filter(|p| conv != CallConv::Stack && local_last.contains_key(&p.id))
        .map(|p| (p.id, p.ty));
    params
        .chain(func.locals.iter().map(|l| (l.id, l.ty)))
        .filter_map(|(id, ty)| ty.byte_width().map(|w| (id, w)))
        .collect()
}

/// Locals whose address is taken: they live in a static slot for the whole
/// function, so the address stays valid.
fn addressed_locals(func: &TypedFunction) -> HashSet<LocalId> {
    func.blocks
        .iter()
        .flat_map(|b| b.ops.iter())
        .filter_map(|op| match op {
            IrOp::AddrLocal { local, .. } => Some(*local),
            _ => None,
        })
        .collect()
}

/// Successor block positions of every block (first terminator, or the next
/// block when it falls through).
fn block_successors(func: &TypedFunction) -> Vec<Vec<usize>> {
    let pos: HashMap<BlockId, usize> = func
        .blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.id, i))
        .collect();
    let n = func.blocks.len();
    func.blocks
        .iter()
        .enumerate()
        .map(|(bi, block)| {
            let succ = block
                .ops
                .iter()
                .find_map(|op| match op {
                    IrOp::Jump { blk, .. } => Some(vec![*blk]),
                    IrOp::Branch {
                        true_blk,
                        false_blk,
                        ..
                    } => Some(vec![*true_blk, *false_blk]),
                    IrOp::Return { .. } => Some(Vec::new()),
                    _ => None,
                })
                .unwrap_or_else(|| {
                    if bi + 1 < n {
                        vec![func.blocks[bi + 1].id]
                    } else {
                        Vec::new()
                    }
                });
            succ.iter().filter_map(|b| pos.get(b).copied()).collect()
        })
        .collect()
}

/// Blocks that lie on a cycle (a loop body or header).
fn loop_blocks(func: &TypedFunction) -> Vec<bool> {
    let succ = block_successors(func);
    (0..succ.len())
        .map(|start| {
            let mut seen = vec![false; succ.len()];
            let mut stack = succ[start].clone();
            while let Some(b) = stack.pop() {
                if b == start {
                    return true;
                }
                if !seen[b] {
                    seen[b] = true;
                    stack.extend(succ[b].iter().copied());
                }
            }
            false
        })
        .collect()
}

/// How hot each local is: one point per access, eight inside a loop. Used
/// only to choose which locals keep registers when not all of them fit.
fn local_weights(func: &TypedFunction) -> HashMap<LocalId, u32> {
    let hot = loop_blocks(func);
    let mut w: HashMap<LocalId, u32> = HashMap::new();
    for (bi, block) in func.blocks.iter().enumerate() {
        let k = if hot[bi] { 8 } else { 1 };
        for op in &block.ops {
            if let IrOp::LoadLocal { local, .. }
            | IrOp::StoreLocal { local, .. }
            | IrOp::AddrLocal { local, .. } = op
            {
                *w.entry(*local).or_default() += k;
            }
        }
    }
    w
}

/// Register homes for `items`. When everything fits this is exactly the H1
/// plan; otherwise the hottest locals (see [`local_weights`]) take registers
/// first and the rest get static slots from the caller.
fn plan_register_homes(
    func: &TypedFunction,
    items: &[(LocalId, u8)],
    allow_hl: bool,
    pre: &[R8],
) -> HashMap<LocalId, Loc> {
    let homes = HomeRegs::plan(items, allow_hl, &pointer_locals(func), pre);
    if homes.len() == items.len() {
        return homes;
    }
    weighted_homes(func, items, false, pre)
}

/// Register homes by access weight (loop bodies 8x, then declaration
/// order); the rest go to static slots. Without `allow_hl` HL stays free as
/// the working pair, which most word operations need.
fn weighted_homes(
    func: &TypedFunction,
    items: &[(LocalId, u8)],
    allow_hl: bool,
    pre: &[R8],
) -> HashMap<LocalId, Loc> {
    weighted_homes_by(func, items, allow_hl, false, pre)
}

/// [`weighted_homes`], optionally ranking by weight per register (`dense`):
/// a byte local then beats a word local with the same access count.
fn weighted_homes_by(
    func: &TypedFunction,
    items: &[(LocalId, u8)],
    allow_hl: bool,
    dense: bool,
    pre: &[R8],
) -> HashMap<LocalId, Loc> {
    let weights = local_weights(func);
    let mut order: Vec<(usize, LocalId, u8)> = items
        .iter()
        .enumerate()
        .map(|(i, &(id, w))| (i, id, w))
        .collect();
    order.sort_by_key(|&(i, id, width)| {
        let w = weights.get(&id).copied().unwrap_or(0);
        let rank = if dense {
            2 * w / u32::from(width.max(1))
        } else {
            w
        };
        (std::cmp::Reverse(rank), i)
    });
    // Choose who gets a register by weight within capacity (words need one
    // of the two pairs), then hand out pairs before bytes so a byte never
    // splits a pair a chosen word needs.
    let pool: Vec<R8> = HomeRegs::BYTES
        .into_iter()
        .filter(|r| allow_hl || !matches!(r, R8::H | R8::L))
        .filter(|r| !pre.contains(r))
        .collect();
    let capacity = pool.len();
    let pairs = HomeRegs::WORDS
        .iter()
        .filter(|rr| {
            let (h, l) = rr.halves();
            !pre.contains(&h) && !pre.contains(&l)
        })
        .count();
    let (mut words, mut bytes) = (0usize, 0usize);
    let mut chosen: Vec<(usize, LocalId, u8)> = Vec::new();
    for &(i, id, width) in &order {
        let (w, b) = if width == 2 {
            (words + 1, bytes)
        } else {
            (words, bytes + 1)
        };
        if w <= pairs && 2 * w + b <= capacity {
            (words, bytes) = (w, b);
            chosen.push((i, id, width));
        }
    }
    chosen.sort_by_key(|&(i, _, width)| (std::cmp::Reverse(width), i));
    let mut regs = HomeRegs {
        taken: pre.to_vec(),
    };
    let mut out = HashMap::new();
    for (_, id, width) in chosen {
        if let Some(loc) = regs.take(width, allow_hl) {
            out.insert(id, loc);
        }
    }
    out
}

/// Which home plan a stable function is lowered with (H2a tries both when
/// the first one needs fallback code, and keeps the cheaper).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HomeMode {
    /// The H1 plan: every local in a register when they fit (HL included).
    Packed,
    /// Hot locals in `BC DE` registers, HL kept free, the rest in slots.
    Spread,
    /// As `Spread`, ranked by accesses per register used.
    Dense,
    /// As `Packed`, but no pointer gets HL first (H5): with `LD A,(BC)`,
    /// `LD A,(DE)` and `INC rr` a byte pointer can live in BC or DE and
    /// leave HL to a word accumulator.
    PackedPairs,
}

/// Estimated cost of a lowering: approximate T-states, loop blocks 8x.
fn weighted_cost(func: &TypedFunction, part: &FnPart) -> u64 {
    let hot = loop_blocks(func);
    let labels: HashMap<String, bool> = func
        .blocks
        .iter()
        .zip(hot.iter())
        .map(|(b, &h)| (asm_block_label(&part.label, b.id), h))
        .collect();
    let mut weight = 1u64;
    let mut cost = 0u64;
    for item in &part.items {
        match item {
            Z80Item::Label { name, .. } => {
                if let Some(&h) = labels.get(name) {
                    weight = if h { 8 } else { 1 };
                }
            }
            Z80Item::Instruction { op, .. } => cost += weight * u64::from(op.approx_tstates()),
            _ => {}
        }
    }
    cost
}

/// Vregs an op reads.
fn op_reads(op: &IrOp) -> Vec<VReg> {
    match op {
        IrOp::Unary { src, .. } | IrOp::Cast { src, .. } => vec![*src],
        IrOp::Binary { lhs, rhs, .. } => vec![*lhs, *rhs],
        IrOp::StoreLocal { src, .. } | IrOp::StoreGlobal { src, .. } => vec![*src],
        IrOp::LoadIndirect { ptr, .. } => vec![*ptr],
        IrOp::StoreIndirect { ptr, src, .. } => vec![*ptr, *src],
        IrOp::Branch { cond, .. } => vec![*cond],
        IrOp::Return {
            value: Some(src), ..
        } => vec![*src],
        IrOp::Call { args, .. } => args.clone(),
        IrOp::PortIn { port, .. } => vec![*port],
        IrOp::PortOut { port, value, .. } => vec![*port, *value],
        IrOp::Ldir { hl, de, bc, .. } => vec![*hl, *de, *bc],
        IrOp::InlineAsm { inputs, .. } => inputs.iter().map(|(_, v)| *v).collect(),
        _ => Vec::new(),
    }
}

/// Read positions of every vreg and local, by function-wide op index. A
/// `Jump` into an `&&`/`||` join reads that join's right operand.
fn use_positions(
    func: &TypedFunction,
) -> (
    HashMap<VReg, Vec<usize>>,
    HashMap<LocalId, Vec<usize>>,
    usize,
) {
    let mut vregs: HashMap<VReg, Vec<usize>> = HashMap::new();
    let mut locals: HashMap<LocalId, Vec<usize>> = HashMap::new();
    let mut i = 0usize;
    for block in &func.blocks {
        for op in &block.ops {
            for v in op_reads(op) {
                vregs.entry(v).or_default().push(i);
            }
            if let IrOp::LoadLocal { local, .. } | IrOp::AddrLocal { local, .. } = op {
                locals.entry(*local).or_default().push(i);
            }
            i += 1;
        }
    }
    (vregs, locals, i)
}

fn is_compare(op: IrBinary) -> bool {
    matches!(
        op,
        IrBinary::Eq | IrBinary::Ne | IrBinary::Lt | IrBinary::Le | IrBinary::Gt | IrBinary::Ge
    )
}

fn vreg_use_count(func: &TypedFunction, v: VReg) -> usize {
    let mut n = 0;
    for block in &func.blocks {
        for op in &block.ops {
            n += match op {
                IrOp::Unary { src, .. } | IrOp::Cast { src, .. } if *src == v => 1,
                IrOp::Binary { lhs, rhs, .. } if *lhs == v || *rhs == v => {
                    usize::from(*lhs == v) + usize::from(*rhs == v)
                }
                IrOp::StoreLocal { src, .. } | IrOp::StoreGlobal { src, .. } if *src == v => 1,
                IrOp::LoadIndirect { ptr, .. } if *ptr == v => 1,
                IrOp::StoreIndirect { ptr, src, .. } if *ptr == v || *src == v => {
                    usize::from(*ptr == v) + usize::from(*src == v)
                }
                IrOp::Branch { cond, .. } if *cond == v => 1,
                IrOp::Return {
                    value: Some(src), ..
                } if *src == v => 1,
                IrOp::Call { dst, args, .. } => {
                    usize::from(dst.as_ref() == Some(&v)) + args.iter().filter(|a| **a == v).count()
                }
                IrOp::PortIn { dst, port, .. } => usize::from(*dst == v) + usize::from(*port == v),
                IrOp::PortOut { port, value, .. } => {
                    usize::from(*port == v) + usize::from(*value == v)
                }
                IrOp::Ldir { hl, de, bc, .. } => {
                    usize::from(*hl == v) + usize::from(*de == v) + usize::from(*bc == v)
                }
                IrOp::InlineAsm {
                    inputs, outputs, ..
                } => {
                    inputs.iter().filter(|(_, a)| *a == v).count()
                        + outputs.iter().filter(|o| o.dst == v).count()
                }
                _ => 0,
            };
        }
    }
    n
}

fn needs_fallthrough(ops: &[IrOp]) -> bool {
    match ops.last() {
        Some(IrOp::Jump { .. } | IrOp::Branch { .. } | IrOp::Return { .. }) => false,
        _ => true,
    }
}

fn next_block(func: &TypedFunction, id: BlockId) -> Option<BlockId> {
    let pos = func.blocks.iter().position(|b| b.id == id)?;
    func.blocks.get(pos + 1).map(|b| b.id)
}

/// Locals live *after* each op, indexed by the function-wide op index used by
/// `liveness_fn`. Computed to a fixed point over the CFG, so a local read
/// again through a loop back edge stays live across the loop body (D3/F-020).
fn local_live_after(func: &TypedFunction) -> Vec<HashSet<LocalId>> {
    fn successors(op: &IrOp) -> Option<Vec<BlockId>> {
        match op {
            IrOp::Jump { blk, .. } => Some(vec![*blk]),
            IrOp::Branch {
                true_blk,
                false_blk,
                ..
            } => Some(vec![*true_blk, *false_blk]),
            IrOp::Return { .. } => Some(Vec::new()),
            _ => None,
        }
    }
    let n_blocks = func.blocks.len();
    let pos: HashMap<BlockId, usize> = func
        .blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.id, i))
        .collect();
    let mut starts = Vec::with_capacity(n_blocks);
    let mut total = 0usize;
    for block in &func.blocks {
        starts.push(total);
        total += block.ops.len();
    }
    // Successors of a block are those of its first terminator, or the next
    // block when it falls through.
    let block_succ: Vec<Vec<usize>> = func
        .blocks
        .iter()
        .enumerate()
        .map(|(bi, block)| {
            let succ = block.ops.iter().find_map(successors).unwrap_or_else(|| {
                if bi + 1 < n_blocks {
                    vec![func.blocks[bi + 1].id]
                } else {
                    Vec::new()
                }
            });
            succ.iter().filter_map(|b| pos.get(b).copied()).collect()
        })
        .collect();
    let mut live_in: Vec<HashSet<LocalId>> = vec![HashSet::new(); n_blocks];
    let mut after: Vec<HashSet<LocalId>> = vec![HashSet::new(); total];
    let mut changed = true;
    while changed {
        changed = false;
        for bi in (0..n_blocks).rev() {
            let block = &func.blocks[bi];
            let out: HashSet<LocalId> = block_succ[bi]
                .iter()
                .flat_map(|&s| live_in[s].iter().copied())
                .collect();
            let mut live = out.clone();
            for (k, op) in block.ops.iter().enumerate().rev() {
                if let Some(succ) = successors(op) {
                    // A terminator before the end makes the rest unreachable.
                    live = succ
                        .iter()
                        .filter_map(|b| pos.get(b))
                        .flat_map(|&s| live_in[s].iter().copied())
                        .collect();
                }
                after[starts[bi] + k] = live.clone();
                match op {
                    IrOp::StoreLocal { local, .. } => {
                        live.remove(local);
                    }
                    IrOp::LoadLocal { local, .. } | IrOp::AddrLocal { local, .. } => {
                        live.insert(*local);
                    }
                    _ => {}
                }
            }
            if live != live_in[bi] {
                live_in[bi] = live;
                changed = true;
            }
        }
    }
    after
}

fn liveness_fn(func: &TypedFunction) -> (HashMap<VReg, usize>, HashMap<LocalId, usize>) {
    let mut vreg_last = HashMap::new();
    let mut local_last = HashMap::new();
    let mut i = 0usize;
    for block in &func.blocks {
        for op in &block.ops {
            match op {
                IrOp::Const { dst, .. } => {
                    vreg_last.insert(*dst, i);
                }
                IrOp::Unary { dst, src, .. } | IrOp::Cast { dst, src, .. } => {
                    vreg_last.insert(*src, i);
                    vreg_last.insert(*dst, i);
                }
                IrOp::Binary { dst, lhs, rhs, .. } => {
                    vreg_last.insert(*lhs, i);
                    vreg_last.insert(*rhs, i);
                    vreg_last.insert(*dst, i);
                }
                IrOp::LoadLocal { dst, local, .. } => {
                    vreg_last.insert(*dst, i);
                    local_last.insert(*local, i);
                }
                IrOp::StoreLocal { local, src, .. } => {
                    vreg_last.insert(*src, i);
                    local_last.insert(*local, i);
                }
                IrOp::LoadGlobal { dst, .. } | IrOp::AddrGlobal { dst, .. } => {
                    vreg_last.insert(*dst, i);
                }
                IrOp::AddrLocal { dst, local, .. } => {
                    vreg_last.insert(*dst, i);
                    local_last.insert(*local, i);
                }
                IrOp::StoreGlobal { src, .. } => {
                    vreg_last.insert(*src, i);
                }
                IrOp::LoadIndirect { dst, ptr, .. } => {
                    vreg_last.insert(*ptr, i);
                    vreg_last.insert(*dst, i);
                }
                IrOp::StoreIndirect { ptr, src, .. } => {
                    vreg_last.insert(*ptr, i);
                    vreg_last.insert(*src, i);
                }
                IrOp::Branch { cond, .. } => {
                    vreg_last.insert(*cond, i);
                }
                IrOp::Return {
                    value: Some(src), ..
                } => {
                    vreg_last.insert(*src, i);
                }
                IrOp::Call { dst, args, .. } => {
                    for arg in args {
                        vreg_last.insert(*arg, i);
                    }
                    if let Some(dst) = dst {
                        vreg_last.insert(*dst, i);
                    }
                }
                IrOp::PortIn { dst, port, .. } => {
                    vreg_last.insert(*port, i);
                    vreg_last.insert(*dst, i);
                }
                IrOp::PortOut { port, value, .. } => {
                    vreg_last.insert(*port, i);
                    vreg_last.insert(*value, i);
                }
                IrOp::Ldir { hl, de, bc, .. } => {
                    vreg_last.insert(*hl, i);
                    vreg_last.insert(*de, i);
                    vreg_last.insert(*bc, i);
                }
                IrOp::InlineAsm {
                    inputs, outputs, ..
                } => {
                    for (_, src) in inputs {
                        vreg_last.insert(*src, i);
                    }
                    for out in outputs {
                        vreg_last.insert(out.dst, i);
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    (vreg_last, local_last)
}

/// Test-only switches that break the lowering on purpose, to prove that the
/// clobber guard catches what parking normally prevents.
#[cfg(test)]
pub(crate) mod test_hooks {
    use std::cell::Cell;

    thread_local! {
        static NO_PARKING: Cell<bool> = const { Cell::new(false) };
    }

    pub(crate) fn parking_disabled() -> bool {
        NO_PARKING.with(Cell::get)
    }

    /// Run `f` with occupant parking switched off on this thread.
    pub(crate) fn without_parking<T>(f: impl FnOnce() -> T) -> T {
        NO_PARKING.with(|c| c.set(true));
        let out = f();
        NO_PARKING.with(|c| c.set(false));
        out
    }
}

impl Lowerer<'_> {
    /// Lower op `i` of a block (fusing a compare with the branch that
    /// consumes it). Returns how many ops were consumed.
    fn lower_one(
        &mut self,
        func: &TypedFunction,
        ops: &[IrOp],
        i: usize,
        _callees: &HashMap<FuncId, CalleeInfo>,
        index: usize,
    ) -> Result<usize, Diagnostic> {
        if i + 1 < ops.len() {
            if let (
                IrOp::Binary {
                    dst,
                    ty,
                    op,
                    lhs,
                    rhs,
                    span,
                },
                IrOp::Branch {
                    cond,
                    true_blk,
                    false_blk,
                    ..
                },
            ) = (&ops[i], &ops[i + 1])
            {
                if cond == dst && is_compare(*op) && vreg_use_count(func, *dst) == 1 {
                    self.lower_compare_branch(*ty, *op, *lhs, *rhs, *true_blk, *false_blk, *span)?;
                    self.finish_op()?;
                    return Ok(2);
                }
            }
        }
        self.next_store = match ops.get(i + 1) {
            Some(IrOp::StoreLocal { local, src, .. }) => Some((*src, *local)),
            _ => None,
        };
        self.lower_op(&ops[i])?;
        self.next_store = None;
        self.finish_op()?;
        self.drop_dead_after(index);
        Ok(1)
    }

    fn frames_of(&self, origin: u32) -> &[InlineFrame] {
        match origin {
            0 => &[],
            o => self
                .sites
                .get(o as usize - 1)
                .map_or(&[][..], |s| s.frames.as_slice()),
        }
    }

    /// Items emitted since `from` belong to the current op's expansion.
    fn tag_inline_items(&mut self, from: usize) {
        if self.cur_origin == 0 {
            return;
        }
        let frames = self.frames_of(self.cur_origin).to_vec();
        for item in &self.items[from..] {
            if item.emits_bytes() {
                self.inline_of.insert(item.id(), frames.clone());
            }
        }
    }

    /// A listing line where an expansion starts (`; inline put_row`, with
    /// `(auto)` for automatic inlining), once per newly entered call.
    fn mark_inline_entry(&mut self, prev: u32, cur: u32, span: SourceSpan) {
        let old = self.frames_of(prev).to_vec();
        let new = self.frames_of(cur).to_vec();
        let common = old
            .iter()
            .zip(new.iter())
            .take_while(|(a, b)| a == b)
            .count();
        for frame in &new[common..] {
            let text = if frame.auto {
                format!("; inline {} (auto)", frame.name)
            } else {
                format!("; inline {}", frame.name)
            };
            let id = self.next_id();
            self.items.push(Z80Item::Directive {
                id,
                text,
                span,
                node: None,
            });
        }
    }

    /// A diagnostic raised inside an expansion names the inlined callee and
    /// every call site it came through (H3i).
    fn with_origin(&self, mut diag: Diagnostic) -> Diagnostic {
        let frames = self.frames_of(self.cur_origin).to_vec();
        if frames.is_empty() || diag.related.iter().any(|r| r.message.contains("inlined")) {
            return diag;
        }
        let inner = frames.last().unwrap();
        diag.message = format!(
            "{} (in '{}', inlined into '{}')",
            diag.message, inner.name, self.func.name
        );
        for frame in frames.iter().rev() {
            diag.related.push(super::diagnostic::RelatedSpan {
                span: frame.call,
                message: format!(
                    "'{}' is inlined here{}",
                    frame.name,
                    if frame.auto {
                        " (automatic)"
                    } else {
                        " (@inline)"
                    }
                ),
            });
        }
        diag
    }

    fn error(&self, span: SourceSpan, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(DiagCode::CgUnsupported, span, message)
    }

    /// No register can be found or freed for an operation. Since H2a values
    /// spill to static slots instead, this only remains for operations whose
    /// fixed operand registers are all reserved by the operation itself.
    fn pressure(&self, span: SourceSpan) -> Diagnostic {
        self.error(
            span,
            format!(
                "register pressure in '{}': no register can be freed for this operation",
                self.func.name
            ),
        )
    }

    /// No register in `regs` can be freed: when a pin holds one of them, say
    /// so (the operation needs the pinned register), else report pressure.
    fn pressure_or_pin(&self, regs: &[R8], span: SourceSpan) -> Diagnostic {
        let pinned = regs.iter().find_map(|&r| {
            self.occupants(r, None).into_iter().find_map(|k| match k {
                Key::L(l) if self.pin_occupies(l) => Some(l),
                _ => None,
            })
        });
        match pinned {
            Some(l) => self.pin_conflict(l, span),
            None => self.pressure(span),
        }
    }

    fn frame_error(&self, span: SourceSpan) -> Diagnostic {
        self.error(
            span,
            format!(
                "indexed frame of '{}' does not fit a signed displacement",
                self.func.name
            ),
        )
    }

    fn next_id(&mut self) -> AsmInstructionId {
        AsmInstructionId(self.ids.next().0)
    }

    fn emit_label(&mut self, name: &str, span: SourceSpan, node: NodeId) {
        let id = self.next_id();
        self.items.push(Z80Item::Label {
            id,
            name: name.to_string(),
            span,
            node,
        });
    }

    fn emit(&mut self, op: Z80Op, span: SourceSpan) {
        if !self.synthetic {
            let writes = op.writes();
            self.guard_regs(&writes, &op.render(), span);
        }
        let id = self.next_id();
        let synthetic = self.synthetic;
        if self.note.is_some() {
            self.spill_tstates += op.spill_tstates().unwrap_or(0);
        }
        // A register-to-register move (keeping a value in a register the
        // callee or operation leaves alone, H2c) is allowed in strict code.
        let reg_move = matches!(op, Z80Op::Ld8 { .. });
        if self.strict
            && self.strict_error.is_none()
            && ((self.note.is_some() && !reg_move)
                || matches!(
                    op,
                    Z80Op::Push(_)
                        | Z80Op::Pop(_)
                        | Z80Op::PushAf
                        | Z80Op::PopAf
                        | Z80Op::PushIx
                        | Z80Op::PopIx
                        | Z80Op::IncSp
                        | Z80Op::DecSp
                ))
        {
            let what = format!("{} needs `{}`", self.cur_op, op.render());
            self.strict_error = Some(self.strict_violation(&what, span));
        }
        // Every push the function makes counts toward its stack bound.
        match op {
            Z80Op::Push(_) | Z80Op::PushAf | Z80Op::PushIx => {
                self.push_depth = self.push_depth.saturating_add(2);
                self.asm_stack = self.asm_stack.max(self.push_depth);
            }
            Z80Op::Pop(_) | Z80Op::PopAf | Z80Op::PopIx => {
                self.push_depth = self.push_depth.saturating_sub(2);
            }
            Z80Op::IncSp => self.push_depth = self.push_depth.saturating_sub(1),
            Z80Op::DecSp => {
                self.push_depth = self.push_depth.saturating_add(1);
                self.asm_stack = self.asm_stack.max(self.push_depth);
            }
            Z80Op::Call { .. } => {
                // Saved pairs and stack arguments sit below the return address.
                if let Some(site) = self.call_sites.last_mut() {
                    site.1 = self.push_depth;
                }
            }
            _ => {}
        }
        self.items.push(Z80Item::Instruction {
            id,
            op,
            span,
            expr: if synthetic { None } else { Some(span) },
            synthetic,
            node: Some(self.func.id.0),
            note: self.note,
        });
    }

    /// Emit compiler-inserted fallback code: synthetic, marked `; spill`.
    fn emit_spill(&mut self, op: Z80Op, span: SourceSpan) {
        let (was_syn, was_note) = (self.synthetic, self.note);
        self.synthetic = true;
        self.note = Some("spill");
        self.emit(op, span);
        self.synthetic = was_syn;
        self.note = was_note;
    }

    fn slot_symbol(&self, off: i32) -> String {
        match off {
            0 => format!("{}_S", self.func_label),
            n if n > 0 => format!("{}_S+{n}", self.func_label),
            n => format!("{}_S{n}", self.func_label),
        }
    }

    /// A free static slot of `width` bytes: not bound to any key, not a
    /// planned home, and not in `taken`. Slots are reused once their values
    /// die, so the area grows only to the peak need (D6).
    fn alloc_slot(&mut self, width: u8, taken: &[Loc]) -> u16 {
        let busy: Vec<(u16, u16)> = self
            .vreg_loc
            .values()
            .chain(self.local_loc.values())
            .chain(self.homes.values())
            .chain(taken.iter())
            .filter_map(|loc| loc.slot_range())
            .collect();
        let w = u16::from(width.max(1));
        let mut off = 0u16;
        while busy.iter().any(|&(lo, hi)| off < hi && lo < off + w) {
            off += 1;
        }
        self.slot_used = self.slot_used.max(off + w);
        off
    }

    /// `r` may be overwritten by spill code right now: it is not reserved,
    /// not needed by a pending move, and holds nothing the current or a
    /// later operation reads.
    fn reg_scratch_ok(&self, r: R8) -> bool {
        !self.reserved.contains(&r)
            && !self.move_blocked.contains(&r)
            && self.occupants(r, None).iter().all(|&k| match k {
                Key::V(v) => {
                    self.consumed.contains(&v)
                        || self
                            .vreg_last
                            .get(&v)
                            .is_none_or(|&last| last < self.op_index)
                }
                Key::L(l) => !self.local_live(l),
            })
    }

    /// Run `f` with `A` usable as scratch: free, or saved with PUSH AF.
    fn with_a_scratch(&mut self, span: SourceSpan, f: impl FnOnce(&mut Self)) {
        if self.reg_scratch_ok(R8::A) {
            f(self);
        } else {
            self.emit_spill(Z80Op::PushAf, span);
            f(self);
            self.emit_spill(Z80Op::PopAf, span);
        }
    }

    fn slot_load8(&mut self, off: u16, dst: R8, span: SourceSpan) {
        let symbol = self.slot_symbol(i32::from(off));
        if dst == R8::A {
            self.emit_spill(Z80Op::LdAbs8 { dst, symbol }, span);
            return;
        }
        if self.reg_scratch_ok(R8::A) {
            self.emit_spill(Z80Op::LdAbs8 { dst: R8::A, symbol }, span);
            self.emit_spill(Z80Op::Ld8 { dst, src: R8::A }, span);
            return;
        }
        // Load through the pair when its other half is free: the slot byte
        // lands in `dst`, a neighbouring byte in the other half.
        let rr = [Rr::Bc, Rr::De, Rr::Hl]
            .into_iter()
            .find(|rr| rr.contains(dst))
            .unwrap_or(Rr::Hl);
        let (h, l) = rr.halves();
        let other = if dst == h { l } else { h };
        // The high half reads the byte before the slot: not for slot 0,
        // which has no slot below it (F-033).
        if self.reg_scratch_ok(other) && (dst == l || off > 0) {
            let at = if dst == l {
                i32::from(off)
            } else {
                i32::from(off) - 1
            };
            let symbol = self.slot_symbol(at);
            self.emit_spill(Z80Op::LdAbs16 { dst: rr, symbol }, span);
            return;
        }
        self.with_a_scratch(span, |this| {
            this.emit_spill(Z80Op::LdAbs8 { dst: R8::A, symbol }, span);
            this.emit_spill(Z80Op::Ld8 { dst, src: R8::A }, span);
        });
    }

    fn slot_store8(&mut self, src: R8, off: u16, span: SourceSpan) {
        let symbol = self.slot_symbol(i32::from(off));
        if src == R8::A {
            self.emit_spill(Z80Op::StAbs8 { src, symbol }, span);
            return;
        }
        self.with_a_scratch(span, |this| {
            this.emit_spill(Z80Op::Ld8 { dst: R8::A, src }, span);
            this.emit_spill(Z80Op::StAbs8 { src: R8::A, symbol }, span);
        });
    }

    fn slot_store_imm8(&mut self, imm: u8, off: u16, span: SourceSpan) {
        let symbol = self.slot_symbol(i32::from(off));
        self.with_a_scratch(span, |this| {
            this.emit_spill(Z80Op::Ld8Imm { dst: R8::A, imm }, span);
            this.emit_spill(Z80Op::StAbs8 { src: R8::A, symbol }, span);
        });
    }

    /// A word constant into a slot: through a free pair (`LD HL,n` +
    /// `LD (nn),HL` is 26 T, DE/BC 30 T), else byte by byte through `A`.
    fn slot_store_imm16(&mut self, imm: u16, off: u16, span: SourceSpan) {
        let pair = [Rr::Hl, Rr::De, Rr::Bc].into_iter().find(|rr| {
            let (h, l) = rr.halves();
            self.reg_scratch_ok(h) && self.reg_scratch_ok(l)
        });
        let symbol = self.slot_symbol(i32::from(off));
        if let Some(rr) = pair {
            self.emit_spill(Z80Op::Ld16Imm { dst: rr, imm }, span);
            self.emit_spill(Z80Op::StAbs16 { src: rr, symbol }, span);
            return;
        }
        let (lo, hi) = (imm as u8, (imm >> 8) as u8);
        let hi_symbol = self.slot_symbol(i32::from(off) + 1);
        self.with_a_scratch(span, |this| {
            this.emit_spill(
                Z80Op::Ld8Imm {
                    dst: R8::A,
                    imm: lo,
                },
                span,
            );
            this.emit_spill(Z80Op::StAbs8 { src: R8::A, symbol }, span);
            if hi != lo {
                this.emit_spill(
                    Z80Op::Ld8Imm {
                        dst: R8::A,
                        imm: hi,
                    },
                    span,
                );
            }
            this.emit_spill(
                Z80Op::StAbs8 {
                    src: R8::A,
                    symbol: hi_symbol,
                },
                span,
            );
        });
    }

    /// Byte `k` of a memory location into `A` (A must be scratch).
    fn mem_byte_to_a(&mut self, loc: Loc, k: u8, span: SourceSpan) -> Result<(), Diagnostic> {
        match loc {
            Loc::Slot { off, .. } => {
                let symbol = self.slot_symbol(i32::from(off) + i32::from(k));
                self.emit_spill(Z80Op::LdAbs8 { dst: R8::A, symbol }, span);
            }
            Loc::Frame { disp, .. } => {
                let disp = disp
                    .checked_add(k as i8)
                    .ok_or_else(|| self.frame_error(span))?;
                self.emit(Z80Op::Ld8Ix { dst: R8::A, disp }, span);
            }
            _ => return Err(self.error(span, "internal: not a memory location")),
        }
        Ok(())
    }

    fn a_to_mem_byte(&mut self, loc: Loc, k: u8, span: SourceSpan) -> Result<(), Diagnostic> {
        match loc {
            Loc::Slot { off, .. } => {
                let symbol = self.slot_symbol(i32::from(off) + i32::from(k));
                self.emit_spill(Z80Op::StAbs8 { src: R8::A, symbol }, span);
            }
            Loc::Frame { disp, .. } => {
                let disp = disp
                    .checked_add(k as i8)
                    .ok_or_else(|| self.frame_error(span))?;
                self.emit(Z80Op::St8Ix { src: R8::A, disp }, span);
            }
            _ => return Err(self.error(span, "internal: not a memory location")),
        }
        Ok(())
    }

    /// Memory to memory, byte by byte through `A` (zero-extending).
    fn mem_to_mem(&mut self, src: Loc, dst: Loc, span: SourceSpan) -> Result<(), Diagnostic> {
        let mut result = Ok(());
        self.with_a_scratch(span, |this| {
            for k in 0..dst.width() {
                let step = if k < src.width() {
                    this.mem_byte_to_a(src, k, span)
                } else {
                    this.emit_spill(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }, span);
                    Ok(())
                };
                if let Err(e) = step.and_then(|()| this.a_to_mem_byte(dst, k, span)) {
                    result = Err(e);
                    return;
                }
            }
        });
        result
    }

    /// Next read of `key` at or after the current op. A local read only
    /// through a loop back edge counts from the end of the function.
    fn next_use(&self, key: Key) -> usize {
        let at = self.op_index;
        match key {
            Key::V(v) => self
                .vreg_uses
                .get(&v)
                .and_then(|u| u.iter().copied().find(|&i| i >= at))
                .unwrap_or(usize::MAX),
            Key::L(l) => match self.local_uses.get(&l) {
                Some(u) => u
                    .iter()
                    .copied()
                    .find(|&i| i >= at)
                    .or_else(|| u.first().map(|&i| self.total_ops + i))
                    .unwrap_or(usize::MAX),
                None => usize::MAX,
            },
        }
    }

    fn group_next_use(&self, keys: &[Key]) -> usize {
        keys.iter()
            .filter(|&&k| self.key_live(k))
            .map(|&k| self.next_use(k))
            .min()
            .unwrap_or(usize::MAX)
    }

    /// Move the value at register location `loc` (bound to `group`) into a
    /// fresh static slot. Byte views of a spilled word follow it.
    fn spill_loc(
        &mut self,
        loc: Loc,
        group: &[Key],
        keep: Option<Key>,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        if let Some(l) = self.pinned_in(group) {
            return Err(self.pin_conflict(l, span));
        }
        if self.strict {
            let what = format!(
                "{} has no free register at {}",
                self.key_name(group),
                self.cur_op
            );
            return Err(self.strict_violation(&what, span));
        }
        let width = loc.width().max(1);
        let off = self.alloc_slot(width, &[]);
        let slot = Loc::Slot { off, width };
        self.emit_move(loc, slot, span)?;
        for &k in group {
            self.bind_key(k, slot);
        }
        if let Loc::Word(rr) = loc {
            let (h, l) = rr.halves();
            for (r, o) in [(l, off), (h, off + 1)] {
                for k in self.keys_at(Loc::Byte(r)) {
                    if Some(k) != keep {
                        self.bind_key(k, Loc::Slot { off: o, width: 1 });
                    }
                }
            }
        }
        Ok(())
    }

    /// Live register groups (location and keys) in `r`, excluding `keep`.
    fn live_groups_at(&self, r: R8, keep: Option<Key>) -> Vec<(Loc, Vec<Key>)> {
        let mut out: Vec<(Loc, Vec<Key>)> = Vec::new();
        for k in self.occupants(r, None) {
            if !self.key_live(k) || Some(k) == keep {
                continue;
            }
            let Some(loc) = self.key_loc(k) else { continue };
            if out.iter().any(|(l, _)| *l == loc) {
                continue;
            }
            let keys = self
                .keys_at(loc)
                .into_iter()
                .filter(|&k| Some(k) != keep)
                .collect();
            out.push((loc, keys));
        }
        out
    }

    /// `relocate` found no register for `key`: spill it, or spill a value
    /// whose next use is farther away and take its register (design order:
    /// farthest next use, fixed register order on ties, the evicted value
    /// first).
    fn spill_or_swap(
        &mut self,
        key: Key,
        loc: Loc,
        group: &[Key],
        avoid: &[R8],
        keep: Option<Key>,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let own = self.group_next_use(group);
        let mut best: Option<(usize, Loc, Vec<Key>)> = None;
        let candidates: Vec<Loc> = match loc {
            Loc::Byte(_) => [R8::C, R8::B, R8::E, R8::D, R8::L, R8::H, R8::A]
                .into_iter()
                .map(Loc::Byte)
                .collect(),
            _ => [Rr::De, Rr::Bc, Rr::Hl]
                .into_iter()
                .map(Loc::Word)
                .collect(),
        };
        for cand in candidates {
            let regs = cand.regs();
            if regs.iter().any(|&r| {
                avoid.contains(&r)
                    || self.reserved.contains(&r)
                    || self.move_blocked.contains(&r)
                    || loc.regs().contains(&r)
                    || self.reg_holds_operand(r)
                    || self.reg_pinned(r)
            }) {
                continue;
            }
            let groups: Vec<(Loc, Vec<Key>)> = regs
                .iter()
                .flat_map(|&r| self.live_groups_at(r, keep))
                .collect();
            // Only whole values can be displaced: a byte slot for a byte,
            // a pair holding one word (or byte values) for a word.
            if groups.is_empty()
                || groups.iter().any(|(l, keys)| {
                    keys.contains(&key) || l.regs().iter().any(|r| !regs.contains(r))
                })
                || self.occupants_include(regs, keep)
            {
                continue;
            }
            let nu = groups
                .iter()
                .map(|(_, keys)| self.group_next_use(keys))
                .min()
                .unwrap_or(usize::MAX);
            if nu <= self.op_index || nu <= own {
                continue;
            }
            if best.as_ref().is_none_or(|(b, _, _)| nu > *b) {
                let keys = groups.into_iter().flat_map(|(_, k)| k).collect();
                best = Some((nu, cand, keys));
            }
        }
        match best {
            None => self.spill_loc(loc, group, keep, span),
            Some((_, cand, _)) => {
                for &r in cand.regs() {
                    for (l, keys) in self.live_groups_at(r, keep) {
                        if self.key_loc(keys[0]) == Some(l) {
                            self.spill_loc(l, &keys, keep, span)?;
                        }
                    }
                }
                self.relocate(key, avoid, keep, span)
            }
        }
    }

    /// `r` holds an operand the current operation has not consumed yet: it
    /// can neither be spilled nor handed out as scratch.
    fn reg_holds_operand(&self, r: R8) -> bool {
        self.occupants(r, None).iter().any(|&k| match k {
            Key::V(v) => {
                !self.consumed.contains(&v)
                    && self
                        .vreg_last
                        .get(&v)
                        .is_some_and(|&last| last == self.op_index)
            }
            Key::L(_) => false,
        })
    }

    fn occupants_include(&self, regs: &[R8], keep: Option<Key>) -> bool {
        keep.is_some_and(|k| regs.iter().any(|&r| self.occupants(r, None).contains(&k)))
    }

    /// A free register (preferring `prefer`), spilling the value with the
    /// farthest next use when none is free.
    fn free_r8_or_spill(
        &mut self,
        prefer: Option<R8>,
        ignore: Option<Key>,
        span: SourceSpan,
    ) -> Result<R8, Diagnostic> {
        if let Some(r) = self.alloc_r8(prefer, ignore) {
            return Ok(r);
        }
        let mut best: Option<(usize, R8)> = None;
        for r in [R8::C, R8::B, R8::E, R8::D, R8::L, R8::H, R8::A] {
            if self.reserved.contains(&r)
                || self.move_blocked.contains(&r)
                || self.reg_holds_operand(r)
                || self.reg_pinned(r)
            {
                continue;
            }
            let groups = self.live_groups_at(r, None);
            if groups
                .iter()
                .any(|(_, keys)| ignore.is_some_and(|i| keys.contains(&i)))
            {
                continue;
            }
            let nu = groups
                .iter()
                .map(|(_, keys)| self.group_next_use(keys))
                .min()
                .unwrap_or(usize::MAX);
            if nu <= self.op_index {
                continue;
            }
            if best.is_none_or(|(b, _)| nu > b) {
                best = Some((nu, r));
            }
        }
        let (_, r) = best.ok_or_else(|| self.pressure_or_pin(&R8::ALL, span))?;
        for (loc, keys) in self.live_groups_at(r, None) {
            self.spill_loc(loc, &keys, None, span)?;
        }
        Ok(r)
    }

    /// A free pair among `choices`, spilling the values in the one whose
    /// next use is farthest when none is free.
    fn free_rr_or_spill(
        &mut self,
        choices: &[Rr],
        ignore: Option<Key>,
        span: SourceSpan,
    ) -> Result<Rr, Diagnostic> {
        if let Some(rr) = choices
            .iter()
            .copied()
            .find(|&rr| self.pair_free(rr, ignore))
        {
            return Ok(rr);
        }
        let mut best: Option<(usize, Rr)> = None;
        for &rr in choices {
            let (h, l) = rr.halves();
            if [h, l].iter().any(|&r| {
                self.reserved.contains(&r)
                    || self.move_blocked.contains(&r)
                    || self.reg_holds_operand(r)
                    || self.reg_pinned(r)
            }) {
                continue;
            }
            let groups: Vec<(Loc, Vec<Key>)> = [h, l]
                .iter()
                .flat_map(|&r| self.live_groups_at(r, None))
                .collect();
            if groups
                .iter()
                .any(|(_, keys)| ignore.is_some_and(|i| keys.contains(&i)))
            {
                continue;
            }
            let nu = groups
                .iter()
                .map(|(_, keys)| self.group_next_use(keys))
                .min()
                .unwrap_or(usize::MAX);
            if nu <= self.op_index {
                continue;
            }
            if best.is_none_or(|(b, _)| nu > b) {
                best = Some((nu, rr));
            }
        }
        let regs: Vec<R8> = choices
            .iter()
            .flat_map(|rr| {
                let (h, l) = rr.halves();
                [h, l]
            })
            .collect();
        let (_, rr) = best.ok_or_else(|| self.pressure_or_pin(&regs, span))?;
        let (h, l) = rr.halves();
        for r in [h, l] {
            for (loc, keys) in self.live_groups_at(r, None) {
                if self.key_loc(keys[0]) == Some(loc) {
                    self.spill_loc(loc, &keys, None, span)?;
                }
            }
        }
        Ok(rr)
    }

    /// Before a store into memory home `home` of `owner`: values still
    /// holding the old bytes there (a loaded copy, a byte view) move to a
    /// free register, or to another slot when none is free.
    fn evacuate_mem(
        &mut self,
        home: Loc,
        owner: Key,
        src: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        loop {
            let blocker = self
                .vreg_loc
                .iter()
                .map(|(&v, &l)| (Key::V(v), l))
                .chain(self.local_loc.iter().map(|(&l, &loc)| (Key::L(l), loc)))
                .filter(|&(k, l)| {
                    k != owner && k != Key::V(src) && self.key_live(k) && l.overlaps(home)
                })
                .map(|(k, l)| (l, k))
                .min_by_key(|&(_, k)| match k {
                    Key::V(v) => (0, v.0),
                    Key::L(l) => (1, l.0.0),
                });
            let Some((loc, _)) = blocker else { break };
            let group: Vec<Key> = self
                .keys_at(loc)
                .into_iter()
                .filter(|&k| k != owner)
                .collect();
            let new_loc = match loc.width() {
                1 => self.alloc_r8(None, None).map(Loc::Byte),
                _ => self.alloc_rr(None, None).map(Loc::Word),
            };
            let new_loc = match new_loc {
                Some(l) => l,
                None => {
                    let width = loc.width();
                    let off = self.alloc_slot(width, &[home]);
                    Loc::Slot { off, width }
                }
            };
            self.emit_move(loc, new_loc, span)?;
            for k in group {
                self.bind_key(k, new_loc);
            }
        }
        Ok(())
    }

    /// Static slots after the function, with a one-line cost report.
    fn emit_slot_area(&mut self, span: SourceSpan) {
        // Register shuffles count too: every function with `; spill` lines
        // gets the report (H2c made shuffle-only functions common).
        if self.slot_used == 0 && self.saves.is_empty() && self.spill_tstates == 0 {
            return;
        }
        let saves = if self.saves.is_empty() {
            "none".to_string()
        } else {
            self.saves
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        };
        let report = format!(
            "; spill report: {} slot bytes, saves (bytes per site) {saves}, {} T-states of spill code",
            self.slot_used, self.spill_tstates
        );
        self.emit_directive(report, span);
        if self.slot_used == 0 {
            return;
        }
        let label = self.slot_symbol(0);
        self.emit_label(&label, span, self.func.id.0);
        let id = self.next_id();
        let zeros = vec!["0"; usize::from(self.slot_used)].join(",");
        self.items.push(Z80Item::Data {
            id,
            text: format!("DB {zeros} ; static slots"),
            span,
            node: self.func.id.0,
            synthetic: true,
        });
    }

    /// Structural clobber guard: writing `regs` must not destroy a value that
    /// is still live, unless the current operation is redefining it.
    fn guard_regs(&mut self, regs: &[R8], what: &str, span: SourceSpan) {
        if self.guard_error.is_some() {
            return;
        }
        for &r in regs {
            for key in self.occupants(r, None) {
                if self.defining.contains(&key) || !self.key_live(key) {
                    continue;
                }
                self.guard_error = Some(Diagnostic::error(
                    DiagCode::CgInternal,
                    span,
                    format!(
                        "internal: live value in {} would be overwritten by `{what}` in '{}'",
                        r.name(),
                        self.func.name
                    ),
                ));
                return;
            }
        }
    }

    /// End of one IR operation: report a guard violation and drop per-op state.
    fn finish_op(&mut self) -> Result<(), Diagnostic> {
        self.defining.clear();
        self.reserved.clear();
        self.consumed.clear();
        match self.guard_error.take() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn emit_raw(&mut self, text: String, span: SourceSpan) {
        let id = self.next_id();
        self.items.push(Z80Item::Raw {
            id,
            text,
            span,
            expr: None,
            synthetic: self.synthetic,
            node: Some(self.func.id.0),
        });
    }

    fn emit_directive(&mut self, text: String, span: SourceSpan) {
        let id = self.next_id();
        self.items.push(Z80Item::Directive {
            id,
            text,
            span,
            node: Some(self.func.id.0),
        });
    }

    fn vreg_live(&self, v: VReg) -> bool {
        self.vreg_last
            .get(&v)
            .is_some_and(|&last| last > self.op_index)
    }

    fn local_live(&self, l: LocalId) -> bool {
        self.pin_occupies(l)
            || self
                .local_after
                .get(self.op_index)
                .is_some_and(|live| live.contains(&l))
    }

    /// A pinned local owns its register for its whole scope, read or not.
    fn pin_in_scope(&self, l: LocalId) -> bool {
        self.pins
            .get(&l)
            .is_some_and(|&(_, (lo, hi))| self.op_index >= lo && self.op_index < hi)
    }

    /// A pinned local owns its register while in scope, except while its
    /// value is dead because a store in scope is about to redefine it: then
    /// the new value may be computed in place.
    fn pin_occupies(&self, l: LocalId) -> bool {
        if !self.pin_in_scope(l) {
            return false;
        }
        let live = self
            .local_after
            .get(self.op_index)
            .is_some_and(|live| live.contains(&l));
        let hi = self.pins.get(&l).map_or(0, |&(_, (_, hi))| hi);
        live || !self
            .pin_stores
            .get(&l)
            .is_some_and(|st| st.iter().any(|&i| i > self.op_index && i < hi))
    }

    /// The pinned local among `keys` that owns its register now, if any (it
    /// must never move).
    fn pinned_in(&self, keys: &[Key]) -> Option<LocalId> {
        keys.iter().find_map(|k| match k {
            Key::L(l) if self.pin_occupies(*l) => Some(*l),
            _ => None,
        })
    }

    /// `r` holds a pinned local that owns it now.
    fn reg_pinned(&self, r: R8) -> bool {
        self.occupants(r, None)
            .iter()
            .any(|k| matches!(k, Key::L(l) if self.pin_occupies(*l)))
    }

    fn local_name(&self, l: LocalId) -> String {
        if is_preserve_key(l) {
            let reg = self
                .pins
                .get(&l)
                .map(|(loc, _)| loc_reg_name(*loc))
                .unwrap_or("?");
            return format!("the caller's {reg} (kept by @preserves)");
        }
        self.func
            .params
            .iter()
            .map(|p| (p.id, &p.name))
            .chain(self.func.locals.iter().map(|x| (x.id, &x.name)))
            .find(|(id, _)| *id == l)
            .map_or_else(|| "a local".to_string(), |(_, n)| format!("'{n}'"))
    }

    fn key_name(&self, keys: &[Key]) -> String {
        keys.iter()
            .find_map(|k| match k {
                Key::L(l) => Some(self.local_name(*l)),
                Key::V(_) => None,
            })
            .unwrap_or_else(|| "an intermediate value".to_string())
    }

    /// An operation needs a pinned register for something else (H2b): an
    /// error naming the pin and the operation, never a hidden move.
    fn pin_conflict(&self, l: LocalId, span: SourceSpan) -> Diagnostic {
        let reg = self
            .pins
            .get(&l)
            .map(|(loc, _)| loc_reg_name(*loc))
            .unwrap_or("?");
        if is_preserve_key(l) {
            return self.error(
                span,
                format!(
                    "'{}' declares @preserves({reg}), but {} needs {reg}; remove {reg} from @preserves",
                    self.func.name, self.cur_op
                ),
            );
        }
        self.error(
            span,
            format!(
                "{} is pinned to {reg}, but {} needs {reg} for something else; pin {} to another register, or remove the pin",
                self.local_name(l),
                self.cur_op,
                self.local_name(l)
            ),
        )
    }

    /// A strict `@fastcall` function would need fallback code (H2b).
    fn strict_violation(&self, what: &str, span: SourceSpan) -> Diagnostic {
        self.error(
            span,
            format!(
                "@fastcall '{}' must fit in registers, but {what}; pin values to free registers, declare a local 'static', or remove @fastcall",
                self.func.name
            ),
        )
    }

    fn key_live(&self, key: Key) -> bool {
        match key {
            Key::V(v) => self.vreg_live(v),
            Key::L(l) => self.local_live(l),
        }
    }

    fn drop_dead_after(&mut self, index: usize) {
        self.vreg_loc
            .retain(|v, _| self.vreg_last.get(v).is_some_and(|&last| last > index));
        if !self.stable {
            let mut live = self.local_after.get(index).cloned().unwrap_or_default();
            // `@preserves` pseudo-locals hold the caller's values to the end.
            live.extend(self.pins.keys().copied().filter(|&l| is_preserve_key(l)));
            self.local_loc.retain(|l, _| live.contains(l));
        }
    }

    fn occupants(&self, r: R8, ignore: Option<Key>) -> Vec<Key> {
        let mut out = Vec::new();
        for (&v, loc) in &self.vreg_loc {
            if loc.regs().contains(&r) && ignore != Some(Key::V(v)) && !out.contains(&Key::V(v)) {
                out.push(Key::V(v));
            }
        }
        for (&l, loc) in &self.local_loc {
            if loc.regs().contains(&r) && ignore != Some(Key::L(l)) && !out.contains(&Key::L(l)) {
                out.push(Key::L(l));
            }
        }
        out
    }

    /// `r` holds nothing worth keeping: a local that is dead here (its next
    /// access is a store) does not occupy its register.
    fn reg_free(&self, r: R8, ignore: Option<Key>) -> bool {
        !self.reserved.contains(&r)
            && !self.move_blocked.contains(&r)
            && self
                .occupants(r, ignore)
                .into_iter()
                .all(|k| matches!(k, Key::L(l) if !self.local_live(l)))
    }

    fn pair_free(&self, rr: Rr, ignore: Option<Key>) -> bool {
        let (h, l) = rr.halves();
        self.reg_free(h, ignore) && self.reg_free(l, ignore)
    }

    fn alloc_r8(&self, prefer: Option<R8>, ignore: Option<Key>) -> Option<R8> {
        if let Some(r) = prefer {
            if self.reg_free(r, ignore) {
                return Some(r);
            }
        }
        const ORDER: [R8; 7] = [R8::C, R8::B, R8::E, R8::D, R8::L, R8::H, R8::A];
        ORDER.into_iter().find(|&r| self.reg_free(r, ignore))
    }

    fn alloc_rr(&self, prefer: Option<Rr>, ignore: Option<Key>) -> Option<Rr> {
        if let Some(rr) = prefer {
            if self.pair_free(rr, ignore) {
                return Some(rr);
            }
        }
        [Rr::De, Rr::Bc, Rr::Hl]
            .into_iter()
            .find(|&rr| self.pair_free(rr, ignore))
    }

    fn evict_regs(
        &mut self,
        regs: &[R8],
        ignore: Option<Key>,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let mut seen = Vec::new();
        for &r in regs {
            for key in self.occupants(r, ignore) {
                // A local that is dead here holds nothing worth keeping; moving
                // it would only waste code and drift a stable home.
                if matches!(key, Key::L(l) if !self.local_live(l)) {
                    continue;
                }
                // A dead operand already copied into a scratch register of
                // this operation has been consumed.
                if matches!(key, Key::V(v) if self.consumed.contains(&v)) {
                    continue;
                }
                if !seen.contains(&key) {
                    seen.push(key);
                }
            }
        }
        for key in seen {
            // An earlier group move may already have taken this key along.
            let still_here = self
                .key_loc(key)
                .is_some_and(|loc| loc.regs().iter().any(|r| regs.contains(r)));
            if still_here {
                self.relocate(key, regs, ignore, span)?;
            }
        }
        Ok(())
    }

    fn key_loc(&self, key: Key) -> Option<Loc> {
        match key {
            Key::V(v) => self.vreg_loc.get(&v).copied(),
            Key::L(l) => self.local_loc.get(&l).copied(),
        }
    }

    fn bind_key(&mut self, key: Key, loc: Loc) {
        match key {
            Key::V(v) => {
                self.vreg_loc.insert(v, loc);
            }
            Key::L(l) => {
                if self.note.is_some() {
                    self.spill_moved.insert(l);
                }
                self.local_loc.insert(l, loc);
            }
        }
    }

    /// Keys bound to exactly `loc`. They name one value, so they move together.
    fn keys_at(&self, loc: Loc) -> Vec<Key> {
        let mut keys: Vec<Key> = self
            .vreg_loc
            .iter()
            .filter(|(_, l)| **l == loc)
            .map(|(&v, _)| Key::V(v))
            .chain(
                self.local_loc
                    .iter()
                    .filter(|(_, l)| **l == loc)
                    .map(|(&l, _)| Key::L(l)),
            )
            .collect();
        keys.sort_by_key(|k| match k {
            Key::V(v) => (0, v.0),
            Key::L(l) => (1, l.0.0),
        });
        keys
    }

    /// Move `key`'s value to a free register outside `avoid`, rebinding every
    /// other key bound to the same location (except `keep`) with it. Aliases
    /// of one value therefore never end up as two copies, and none is left
    /// behind in a register that is about to be overwritten.
    fn relocate(
        &mut self,
        key: Key,
        avoid: &[R8],
        keep: Option<Key>,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let loc = self.key_loc(key).ok_or_else(|| self.pressure(span))?;
        // A pinned local out of its scope stays at its home; one in scope
        // never moves.
        let group: Vec<Key> = self
            .keys_at(loc)
            .into_iter()
            .filter(|k| Some(*k) != keep)
            .filter(
                |k| !matches!(k, Key::L(l) if self.pins.contains_key(l) && !self.pin_occupies(*l)),
            )
            .collect();
        if let Some(l) = self.pinned_in(&group) {
            return Err(self.pin_conflict(l, span));
        }
        if group.is_empty() {
            return Ok(());
        }
        let new_loc = match loc {
            Loc::Byte(_) => {
                const ORDER: [R8; 7] = [R8::C, R8::B, R8::E, R8::D, R8::L, R8::H, R8::A];
                let found = ORDER
                    .into_iter()
                    .find(|&r| !avoid.contains(&r) && self.reg_free_for(r, &group));
                match found {
                    Some(r) => Loc::Byte(r),
                    None => return self.spill_or_swap(key, loc, &group, avoid, keep, span),
                }
            }
            Loc::Word(_) => {
                let free = |this: &Self, rr: Rr| {
                    let (h, l) = rr.halves();
                    !avoid.contains(&h)
                        && !avoid.contains(&l)
                        && this.reg_free_for(h, &group)
                        && this.reg_free_for(l, &group)
                };
                let rr = match [Rr::De, Rr::Bc, Rr::Hl]
                    .into_iter()
                    .find(|&rr| free(self, rr))
                {
                    Some(rr) => rr,
                    None => match self.make_pair(loc, avoid, &group, keep, span)? {
                        Some(rr) => rr,
                        None => return self.spill_or_swap(key, loc, &group, avoid, keep, span),
                    },
                };
                Loc::Word(rr)
            }
            Loc::Imm8(_) | Loc::Imm16(_) | Loc::Frame { .. } | Loc::Slot { .. } => return Ok(()),
        };
        self.emit_move(loc, new_loc, span)?;
        for k in group {
            self.bind_key(k, new_loc);
        }
        // Byte views of a moved word hold the same bytes in the new pair.
        if let (Loc::Word(old), Loc::Word(new)) = (loc, new_loc) {
            let (oh, ol) = old.halves();
            let (nh, nl) = new.halves();
            for (o, n) in [(oh, nh), (ol, nl)] {
                for k in self.keys_at(Loc::Byte(o)) {
                    if Some(k) != keep {
                        self.bind_key(k, Loc::Byte(n));
                    }
                }
            }
        }
        Ok(())
    }

    /// No pair is free for a word: free one by moving the byte values that
    /// sit in it to free single registers (a register shuffle, not a store).
    fn make_pair(
        &mut self,
        loc: Loc,
        avoid: &[R8],
        group: &[Key],
        keep: Option<Key>,
        span: SourceSpan,
    ) -> Result<Option<Rr>, Diagnostic> {
        let moved = loc.regs();
        for rr in [Rr::De, Rr::Bc, Rr::Hl] {
            let (h, l) = rr.halves();
            if avoid.contains(&h) || avoid.contains(&l) {
                continue;
            }
            if self.reserved.contains(&h) || self.reserved.contains(&l) {
                continue;
            }
            if self.reg_pinned(h) || self.reg_pinned(l) {
                continue;
            }
            // Byte groups in the pair that would have to move.
            let mut blockers: Vec<Loc> = Vec::new();
            let mut ok = true;
            for r in [h, l] {
                if moved.contains(&r) {
                    continue;
                }
                for k in self.occupants(r, None) {
                    if group.contains(&k) {
                        continue;
                    }
                    if Some(k) == keep {
                        ok = false;
                    }
                    match self.key_loc(k) {
                        Some(b @ Loc::Byte(_)) => {
                            if !blockers.contains(&b) {
                                blockers.push(b);
                            }
                        }
                        _ => ok = false,
                    }
                }
            }
            if !ok || blockers.is_empty() {
                continue;
            }
            let spare = R8::ALL
                .into_iter()
                .filter(|r| {
                    *r != h
                        && *r != l
                        && !avoid.contains(r)
                        && !moved.contains(r)
                        && self.reg_free_for(*r, group)
                })
                .count();
            if spare < blockers.len() {
                continue;
            }
            let mut avoid2 = avoid.to_vec();
            avoid2.extend([h, l]);
            avoid2.extend(moved.iter().copied());
            for b in blockers {
                if let Some(k) = self.keys_at(b).into_iter().find(|k| !group.contains(k)) {
                    self.relocate(k, &avoid2, keep, span)?;
                }
            }
            return Ok(Some(rr));
        }
        Ok(None)
    }

    /// `r` is free apart from the keys in `group` (which are being moved).
    fn reg_free_for(&self, r: R8, group: &[Key]) -> bool {
        !self.reserved.contains(&r)
            && !self.move_blocked.contains(&r)
            && self
                .occupants(r, None)
                .iter()
                .all(|k| group.contains(k) || matches!(k, Key::L(l) if !self.local_live(*l)))
    }

    /// Before an in-place write of `regs`: move every value bound there that
    /// is still live after this operation to a free register (D1/F-018,
    /// F-025: locals and byte views of a pair included), then reserve `regs`
    /// for the rest of the operation so no later step allocates into them.
    fn park_live(&mut self, regs: &[R8], span: SourceSpan) -> Result<(), Diagnostic> {
        #[cfg(test)]
        if test_hooks::parking_disabled() {
            self.reserve(regs);
            return Ok(());
        }
        loop {
            let live = regs.iter().find_map(|&r| {
                self.occupants(r, None)
                    .into_iter()
                    .find(|&k| self.key_live(k))
            });
            let Some(key) = live else { break };
            self.relocate(key, regs, None, span)?;
        }
        for &r in regs {
            if !self.reserved.contains(&r) {
                self.reserved.push(r);
            }
        }
        Ok(())
    }

    fn emit_move(&mut self, src: Loc, dst: Loc, span: SourceSpan) -> Result<(), Diagnostic> {
        if src == dst {
            return Ok(());
        }
        match (src, dst) {
            (Loc::Byte(s), Loc::Byte(d)) => {
                if s != d {
                    self.emit(Z80Op::Ld8 { dst: d, src: s }, span);
                }
            }
            (Loc::Imm8(n), Loc::Byte(d)) => self.emit(Z80Op::Ld8Imm { dst: d, imm: n }, span),
            (Loc::Imm16(n), Loc::Word(d)) => self.emit(Z80Op::Ld16Imm { dst: d, imm: n }, span),
            (Loc::Word(s), Loc::Word(d)) => {
                if s == d {
                    return Ok(());
                }
                if (s == Rr::De && d == Rr::Hl) || (s == Rr::Hl && d == Rr::De) {
                    // Copy, not swap: move halves. EX would swap an occupant we may not own.
                    let (sh, sl) = s.halves();
                    let (dh, dl) = d.halves();
                    self.emit(Z80Op::Ld8 { dst: dh, src: sh }, span);
                    self.emit(Z80Op::Ld8 { dst: dl, src: sl }, span);
                } else {
                    let (sh, sl) = s.halves();
                    let (dh, dl) = d.halves();
                    self.emit(Z80Op::Ld8 { dst: dh, src: sh }, span);
                    self.emit(Z80Op::Ld8 { dst: dl, src: sl }, span);
                }
            }
            (Loc::Byte(s), Loc::Word(d)) => {
                let (h, l) = d.halves();
                // Move H's occupants out first, and never into L, which is
                // about to receive the low byte.
                let had_l = self.reserved.contains(&l);
                self.reserve(&[l]);
                self.evict_regs(&[h], None, span)?;
                if !had_l {
                    self.reserved.retain(|&r| r != l);
                }
                if s != l {
                    self.emit(Z80Op::Ld8 { dst: l, src: s }, span);
                }
                self.emit(Z80Op::Ld8Imm { dst: h, imm: 0 }, span);
            }
            (Loc::Imm8(n), Loc::Word(d)) => {
                self.emit(
                    Z80Op::Ld16Imm {
                        dst: d,
                        imm: u16::from(n),
                    },
                    span,
                );
            }
            (src, dst) if src.is_mem() && dst.is_mem() => self.mem_to_mem(src, dst, span)?,
            (Loc::Slot { off, .. }, Loc::Byte(d)) => self.slot_load8(off, d, span),
            (Loc::Slot { off, width }, Loc::Word(d)) => {
                let symbol = self.slot_symbol(i32::from(off));
                self.emit_spill(Z80Op::LdAbs16 { dst: d, symbol }, span);
                if width == 1 {
                    self.emit_spill(
                        Z80Op::Ld8Imm {
                            dst: d.halves().0,
                            imm: 0,
                        },
                        span,
                    );
                }
            }
            (Loc::Byte(s), Loc::Slot { off, width }) => {
                self.slot_store8(s, off, span);
                if width == 2 {
                    self.slot_store_imm8(0, off + 1, span);
                }
            }
            (Loc::Word(s), Loc::Slot { off, width }) => {
                if width == 2 {
                    let symbol = self.slot_symbol(i32::from(off));
                    self.emit_spill(Z80Op::StAbs16 { src: s, symbol }, span);
                } else {
                    self.slot_store8(s.halves().1, off, span);
                }
            }
            (Loc::Imm8(n), Loc::Slot { off, width: 2 }) => {
                self.slot_store_imm16(u16::from(n), off, span)
            }
            (Loc::Imm8(n), Loc::Slot { off, .. }) => self.slot_store_imm8(n, off, span),
            (Loc::Imm16(n), Loc::Slot { off, width: 2 }) => self.slot_store_imm16(n, off, span),
            (Loc::Imm16(n), Loc::Slot { off, .. }) => self.slot_store_imm8(n as u8, off, span),
            (Loc::Frame { disp, width: 1 }, Loc::Byte(d)) => {
                self.emit(Z80Op::Ld8Ix { dst: d, disp }, span);
            }
            (Loc::Frame { disp, width: 2 }, Loc::Byte(d)) => {
                self.emit(Z80Op::Ld8Ix { dst: d, disp }, span);
            }
            (Loc::Frame { disp, width: 1 }, Loc::Word(d)) => {
                let (h, l) = d.halves();
                self.emit(Z80Op::Ld8Ix { dst: l, disp }, span);
                self.emit(Z80Op::Ld8Imm { dst: h, imm: 0 }, span);
            }
            (Loc::Frame { disp, width: 2 }, Loc::Word(d)) => {
                let hi = disp.checked_add(1).ok_or_else(|| self.frame_error(span))?;
                let (h, l) = d.halves();
                self.emit(Z80Op::Ld8Ix { dst: l, disp }, span);
                self.emit(Z80Op::Ld8Ix { dst: h, disp: hi }, span);
            }
            (Loc::Byte(s), Loc::Frame { disp, width: 1 }) => {
                self.emit(Z80Op::St8Ix { src: s, disp }, span);
            }
            (Loc::Byte(s), Loc::Frame { disp, width: 2 }) => {
                self.emit(Z80Op::St8Ix { src: s, disp }, span);
                let hi = disp.checked_add(1).ok_or_else(|| self.frame_error(span))?;
                self.emit(Z80Op::Ld8IxImm { disp: hi, imm: 0 }, span);
            }
            (Loc::Word(s), Loc::Frame { disp, width: 2 }) => {
                let hi = disp.checked_add(1).ok_or_else(|| self.frame_error(span))?;
                let (h, l) = s.halves();
                self.emit(Z80Op::St8Ix { src: l, disp }, span);
                self.emit(Z80Op::St8Ix { src: h, disp: hi }, span);
            }
            (Loc::Imm8(n), Loc::Frame { disp, width: 1 }) => {
                self.emit(Z80Op::Ld8IxImm { disp, imm: n }, span);
            }
            (Loc::Imm8(n), Loc::Frame { disp, width: 2 }) => {
                let hi = disp.checked_add(1).ok_or_else(|| self.frame_error(span))?;
                self.emit(Z80Op::Ld8IxImm { disp, imm: n }, span);
                self.emit(Z80Op::Ld8IxImm { disp: hi, imm: 0 }, span);
            }
            (Loc::Imm16(n), Loc::Frame { disp, width: 2 }) => {
                let hi = disp.checked_add(1).ok_or_else(|| self.frame_error(span))?;
                self.emit(Z80Op::Ld8IxImm { disp, imm: n as u8 }, span);
                self.emit(
                    Z80Op::Ld8IxImm {
                        disp: hi,
                        imm: (n >> 8) as u8,
                    },
                    span,
                );
            }
            (src, dst @ Loc::Frame { .. }) => match src.width() {
                1 => {
                    self.claim_r8(R8::A, None, span)?;
                    self.emit_move(src, Loc::Byte(R8::A), span)?;
                    self.emit_move(Loc::Byte(R8::A), dst, span)?;
                }
                _ => {
                    self.claim_rr(Rr::Hl, None, span)?;
                    self.emit_move(src, Loc::Word(Rr::Hl), span)?;
                    self.emit_move(Loc::Word(Rr::Hl), dst, span)?;
                }
            },
            (src @ Loc::Frame { .. }, dst) => match dst.width() {
                1 => {
                    self.claim_r8(R8::A, None, span)?;
                    self.emit_move(src, Loc::Byte(R8::A), span)?;
                    self.emit_move(Loc::Byte(R8::A), dst, span)?;
                }
                _ => {
                    self.claim_rr(Rr::Hl, None, span)?;
                    self.emit_move(src, Loc::Word(Rr::Hl), span)?;
                    self.emit_move(Loc::Word(Rr::Hl), dst, span)?;
                }
            },
            _ => {
                return Err(self.error(span, "cannot move between these locations"));
            }
        }
        Ok(())
    }

    fn bind_vreg(&mut self, v: VReg, loc: Loc, ty: CType) {
        self.vreg_loc.insert(v, loc);
        self.vreg_ty.insert(v, ty);
    }

    fn loc_of(&self, v: VReg, span: SourceSpan) -> Result<Loc, Diagnostic> {
        self.vreg_loc
            .get(&v)
            .copied()
            .ok_or_else(|| self.error(span, "internal: value has no location"))
    }

    fn claim_r8(&mut self, r: R8, keep: Option<Key>, span: SourceSpan) -> Result<(), Diagnostic> {
        self.evict_regs(&[r], keep, span)
    }

    fn claim_rr(&mut self, rr: Rr, keep: Option<Key>, span: SourceSpan) -> Result<(), Diagnostic> {
        let (h, l) = rr.halves();
        self.evict_regs(&[h, l], keep, span)
    }

    fn ensure_in_r8(&mut self, v: VReg, dst: R8, span: SourceSpan) -> Result<(), Diagnostic> {
        let src = self.loc_of(v, span)?;
        if src == Loc::Byte(dst) {
            return Ok(());
        }
        self.claim_r8(dst, Some(Key::V(v)), span)?;
        let src = self.loc_of(v, span)?;
        self.emit_move(src, Loc::Byte(dst), span)?;
        self.vreg_loc.insert(v, Loc::Byte(dst));
        Ok(())
    }

    fn ensure_in_rr(&mut self, v: VReg, dst: Rr, span: SourceSpan) -> Result<(), Diagnostic> {
        let src = self.loc_of(v, span)?;
        if src == Loc::Word(dst) {
            return Ok(());
        }
        self.claim_rr(dst, Some(Key::V(v)), span)?;
        let src = self.loc_of(v, span)?;
        match src {
            Loc::Byte(r) => {
                let (h, l) = dst.halves();
                if r != l {
                    self.emit(Z80Op::Ld8 { dst: l, src: r }, span);
                }
                self.emit(Z80Op::Ld8Imm { dst: h, imm: 0 }, span);
            }
            Loc::Imm8(n) => self.emit(
                Z80Op::Ld16Imm {
                    dst,
                    imm: u16::from(n),
                },
                span,
            ),
            Loc::Imm16(_) | Loc::Word(_) | Loc::Frame { .. } | Loc::Slot { .. } => {
                self.emit_move(src, Loc::Word(dst), span)?
            }
        }
        self.vreg_loc.insert(v, Loc::Word(dst));
        Ok(())
    }

    fn ensure_a(&mut self, v: VReg, span: SourceSpan) -> Result<(), Diagnostic> {
        self.ensure_in_r8(v, R8::A, span)
    }

    fn ensure_hl(&mut self, v: VReg, span: SourceSpan) -> Result<(), Diagnostic> {
        self.ensure_in_rr(v, Rr::Hl, span)
    }

    fn reserve(&mut self, regs: &[R8]) {
        for &r in regs {
            if !self.reserved.contains(&r) {
                self.reserved.push(r);
            }
        }
    }

    /// Put a copy of `v` in `r` that the current operation may overwrite.
    /// Values still live after the operation stay where they are (or are
    /// moved out of `r`), so an in-place lowering never destroys them.
    fn scratch_r8(&mut self, v: VReg, r: R8, span: SourceSpan) -> Result<(), Diagnostic> {
        let before = self.loc_of(v, span)?;
        self.park_live(&[r], span)?;
        if before == Loc::Byte(r) {
            return Ok(());
        }
        self.claim_r8(r, Some(Key::V(v)), span)?;
        let src = self.loc_of(v, span)?;
        self.emit_move(src, Loc::Byte(r), span)?;
        if !self.vreg_live(v) {
            // The old register is free from here on.
            self.vreg_loc.insert(v, Loc::Byte(r));
            self.consumed.push(v);
        }
        Ok(())
    }

    fn scratch_a(&mut self, v: VReg, span: SourceSpan) -> Result<(), Diagnostic> {
        self.scratch_r8(v, R8::A, span)
    }

    /// Word counterpart of [`Self::scratch_r8`].
    fn scratch_rr(&mut self, v: VReg, rr: Rr, span: SourceSpan) -> Result<(), Diagnostic> {
        let (h, l) = rr.halves();
        let before = self.loc_of(v, span)?;
        self.park_live(&[h, l], span)?;
        if before == Loc::Word(rr) {
            return Ok(());
        }
        match self.loc_of(v, span)? {
            Loc::Byte(_) => self.ensure_in_rr(v, rr, span),
            _ => {
                self.claim_rr(rr, Some(Key::V(v)), span)?;
                let src = self.loc_of(v, span)?;
                self.emit_move(src, Loc::Word(rr), span)?;
                if !self.vreg_live(v) {
                    self.vreg_loc.insert(v, Loc::Word(rr));
                    self.consumed.push(v);
                }
                Ok(())
            }
        }
    }

    fn scratch_hl(&mut self, v: VReg, span: SourceSpan) -> Result<(), Diagnostic> {
        self.scratch_rr(v, Rr::Hl, span)
    }

    /// Whether HL can take a scratch copy of `lhs` without running out of
    /// pairs: everything that must leave HL (and `lhs` itself, when it stays
    /// live) needs a free pair to go to.
    fn hl_scratch_fits(&self, lhs: VReg) -> bool {
        let must_move = |r: R8| {
            self.occupants(r, None)
                .into_iter()
                .any(|k| !matches!(k, Key::L(l) if !self.local_live(l)))
        };
        let free_pairs = [Rr::De, Rr::Bc]
            .into_iter()
            .filter(|&rr| {
                let (h, l) = rr.halves();
                !must_move(h)
                    && !must_move(l)
                    && !self.reserved.contains(&h)
                    && !self.reserved.contains(&l)
            })
            .count();
        let in_hl = self.vreg_loc.get(&lhs) == Some(&Loc::Word(Rr::Hl));
        let hl_busy = must_move(R8::H) || must_move(R8::L);
        let need = if in_hl {
            usize::from(self.vreg_live(lhs))
        } else {
            usize::from(hl_busy)
        };
        free_pairs >= need
    }

    /// A pair other than HL that holds `v` and nothing live after this
    /// operation, so a result may be written over it.
    fn dead_pair_of(&self, v: VReg) -> Option<Rr> {
        match self.vreg_loc.get(&v)? {
            Loc::Word(rr @ (Rr::De | Rr::Bc)) => {
                let (h, l) = rr.halves();
                let dead = [h, l]
                    .into_iter()
                    .all(|r| self.occupants(r, None).iter().all(|&k| !self.key_live(k)));
                (dead && !self.reserved.contains(&h) && !self.reserved.contains(&l)).then_some(*rr)
            }
            _ => None,
        }
    }

    fn word_alu_halves(&self, v: VReg) -> Option<(AluSrc, AluSrc)> {
        match self.vreg_loc.get(&v)? {
            Loc::Word(rr) => {
                let (h, l) = rr.halves();
                Some((AluSrc::Reg(h), AluSrc::Reg(l)))
            }
            Loc::Imm16(n) => Some((AluSrc::Imm((n >> 8) as u8), AluSrc::Imm(*n as u8))),
            _ => None,
        }
    }

    /// Word ALU through A into the pair of an operand that dies here, used
    /// when HL is not available: `LD A,lo; op; LD lo,A; LD A,hi; op'; LD hi,A`.
    /// Returns false (emitting nothing) when no operand pair can take it.
    fn word_alu_in_pair(
        &mut self,
        dst: VReg,
        ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<bool, Diagnostic> {
        if !matches!(
            op,
            IrBinary::Add | IrBinary::Sub | IrBinary::BitAnd | IrBinary::BitOr | IrBinary::BitXor
        ) {
            return Ok(false);
        }
        let Some(w) = self.dead_pair_of(lhs).or_else(|| self.dead_pair_of(rhs)) else {
            return Ok(false);
        };
        if self.word_alu_halves(lhs).is_none() || self.word_alu_halves(rhs).is_none() {
            return Ok(false);
        }
        let (wh, wl) = w.halves();
        self.reserve(&[wh, wl]);
        self.claim_r8(R8::A, None, span)?;
        // The claim may not move operands into W (reserved), but it may move
        // them elsewhere: re-read both before emitting.
        let (Some((lh2, ll2)), Some((rh2, rl2))) =
            (self.word_alu_halves(lhs), self.word_alu_halves(rhs))
        else {
            return Err(self.pressure(span));
        };
        let load = |s: AluSrc| match s {
            AluSrc::Reg(r) => Z80Op::Ld8 { dst: R8::A, src: r },
            AluSrc::Imm(n) => Z80Op::Ld8Imm { dst: R8::A, imm: n },
        };
        let (lo_op, hi_op): (fn(AluSrc) -> Z80Op, fn(AluSrc) -> Z80Op) = match op {
            IrBinary::Add => (Z80Op::AddA, Z80Op::AdcA),
            IrBinary::Sub => (Z80Op::Sub, Z80Op::SbcA),
            IrBinary::BitAnd => (Z80Op::And, Z80Op::And),
            IrBinary::BitOr => (Z80Op::Or, Z80Op::Or),
            _ => (Z80Op::Xor, Z80Op::Xor),
        };
        self.emit(load(ll2), span);
        self.emit(lo_op(rl2), span);
        self.emit(
            Z80Op::Ld8 {
                dst: wl,
                src: R8::A,
            },
            span,
        );
        self.emit(load(lh2), span);
        self.emit(hi_op(rh2), span);
        self.emit(
            Z80Op::Ld8 {
                dst: wh,
                src: R8::A,
            },
            span,
        );
        self.bind_vreg(dst, Loc::Word(w), ty);
        Ok(true)
    }

    fn alu_src_of(&mut self, v: VReg, span: SourceSpan) -> Result<AluSrc, Diagnostic> {
        match self.loc_of(v, span)? {
            Loc::Imm8(n) => Ok(AluSrc::Imm(n)),
            Loc::Byte(r) => {
                if r == R8::A {
                    let tmp = self.free_r8_or_spill(Some(R8::C), Some(Key::V(v)), span)?;
                    self.claim_r8(tmp, Some(Key::V(v)), span)?;
                    self.emit(
                        Z80Op::Ld8 {
                            dst: tmp,
                            src: R8::A,
                        },
                        span,
                    );
                    self.vreg_loc.insert(v, Loc::Byte(tmp));
                    Ok(AluSrc::Reg(tmp))
                } else {
                    Ok(AluSrc::Reg(r))
                }
            }
            Loc::Word(rr) => Ok(AluSrc::Reg(rr.halves().1)),
            Loc::Imm16(n) => Ok(AluSrc::Imm(n as u8)),
            Loc::Frame { .. } | Loc::Slot { .. } => {
                let tmp = self.free_r8_or_spill(Some(R8::C), Some(Key::V(v)), span)?;
                self.claim_r8(tmp, Some(Key::V(v)), span)?;
                self.emit_move(self.loc_of(v, span)?, Loc::Byte(tmp), span)?;
                self.vreg_loc.insert(v, Loc::Byte(tmp));
                Ok(AluSrc::Reg(tmp))
            }
        }
    }

    fn word_src_rr(&mut self, v: VReg, span: SourceSpan) -> Result<Rr, Diagnostic> {
        match self.loc_of(v, span)? {
            Loc::Word(rr) if rr != Rr::Hl => Ok(rr),
            _ => {
                let dst = self.free_rr_or_spill(&[Rr::De, Rr::Bc], Some(Key::V(v)), span)?;
                self.ensure_in_rr(v, dst, span)?;
                Ok(dst)
            }
        }
    }

    fn lower_op(&mut self, op: &IrOp) -> Result<(), Diagnostic> {
        match op {
            IrOp::Const {
                dst,
                ty,
                bits,
                span,
            } => {
                let loc = match ty.byte_width() {
                    Some(1) => Loc::Imm8(*bits as u8),
                    Some(2) => Loc::Imm16(*bits),
                    _ => return Err(self.error(*span, "void constant")),
                };
                self.bind_vreg(*dst, loc, *ty);
            }
            IrOp::LoadLocal { dst, local, span } => {
                let loc = *self
                    .local_loc
                    .get(local)
                    .ok_or_else(|| self.error(*span, "local has no register home"))?;
                let ty = *self
                    .local_ty
                    .get(local)
                    .ok_or_else(|| self.error(*span, "local has no type"))?;
                self.bind_vreg(*dst, loc, ty);
            }
            IrOp::StoreLocal { local, src, span } => {
                let ty = *self
                    .local_ty
                    .get(local)
                    .ok_or_else(|| self.error(*span, "local has no type"))?;
                self.defining.push(Key::L(*local));
                if let Some(&home) = self.local_loc.get(local) {
                    // Values that still hold the local's old contents (a
                    // loaded copy, a byte view) leave the home first.
                    let home_regs = home.regs();
                    loop {
                        let blocker = home_regs.iter().find_map(|&r| {
                            self.occupants(r, Some(Key::L(*local)))
                                .into_iter()
                                .find(|&k| k != Key::V(*src) && self.key_live(k))
                        });
                        let Some(k) = blocker else { break };
                        self.relocate(k, &home_regs, Some(Key::L(*local)), *span)?;
                    }
                    if home.is_mem() {
                        self.evacuate_mem(home, Key::L(*local), *src, *span)?;
                    }
                    let src_loc = self.loc_of(*src, *span)?;
                    self.emit_move(src_loc, home, *span)?;
                    return Ok(());
                }
                let src_loc = self.loc_of(*src, *span)?;
                let src_dies = !self.vreg_live(*src);
                if src_dies {
                    if let Some(home) = src_loc.as_home() {
                        self.local_loc.insert(*local, Loc::from_home(home));
                        return Ok(());
                    }
                }
                let prefer = match ty.byte_width() {
                    Some(1) => Some(RegHome::Byte(R8::A)),
                    Some(2) => Some(RegHome::Word(Rr::Hl)),
                    _ => None,
                };
                let home = match prefer {
                    Some(RegHome::Byte(r)) => {
                        let r = self.free_r8_or_spill(Some(r), Some(Key::V(*src)), *span)?;
                        self.claim_r8(r, Some(Key::V(*src)), *span)?;
                        Loc::Byte(r)
                    }
                    Some(RegHome::Word(rr)) => {
                        let order = [rr, Rr::De, Rr::Bc, Rr::Hl];
                        let rr = self.free_rr_or_spill(&order, Some(Key::V(*src)), *span)?;
                        self.claim_rr(rr, Some(Key::V(*src)), *span)?;
                        Loc::Word(rr)
                    }
                    None => return Err(self.error(*span, "void local")),
                };
                self.emit_move(src_loc, home, *span)?;
                self.local_loc.insert(*local, home);
            }
            IrOp::Unary {
                dst,
                ty,
                op,
                src,
                span,
            } => self.lower_unary(*dst, *ty, *op, *src, *span)?,
            IrOp::Binary {
                dst,
                ty,
                op,
                lhs,
                rhs,
                span,
            } => self.lower_binary(*dst, *ty, *op, *lhs, *rhs, *span)?,
            IrOp::Cast {
                dst,
                to,
                from,
                src,
                span,
            } => self.lower_cast(*dst, *to, *from, *src, *span)?,
            IrOp::Return { value, span } => self.lower_return(*value, *span)?,
            IrOp::Jump { blk, span } => self.lower_jump(*blk, *span)?,
            IrOp::Branch {
                cond,
                true_blk,
                false_blk,
                span,
            } => self.lower_branch(*cond, *true_blk, *false_blk, *span)?,
            IrOp::LoadGlobal { dst, global, span } => {
                self.lower_load_global(*dst, *global, *span)?
            }
            IrOp::AddrGlobal { dst, global, span } => {
                self.lower_addr_global(*dst, *global, *span)?
            }
            IrOp::AddrLocal { dst, local, span } => self.lower_addr_local(*dst, *local, *span)?,
            IrOp::StoreGlobal { global, src, span } => {
                self.lower_store_global(*global, *src, *span)?
            }
            IrOp::Call {
                dst,
                func,
                args,
                span,
            } => self.lower_call(*dst, *func, args, *span)?,
            IrOp::PortIn { dst, port, span } => self.lower_port_in(*dst, *port, *span)?,
            IrOp::PortOut { port, value, span } => self.lower_port_out(*port, *value, *span)?,
            IrOp::Di { span } => self.emit(Z80Op::Di, *span),
            IrOp::Ei { span } => self.emit(Z80Op::Ei, *span),
            IrOp::Ldir { hl, de, bc, span } => self.lower_ldir(*hl, *de, *bc, *span)?,
            IrOp::InlineAsm {
                inputs,
                outputs,
                clobbers,
                lines,
                stack,
                unknown_stack,
                plain,
                span,
                node,
            } => self.lower_inline_asm(
                inputs,
                outputs,
                clobbers,
                lines,
                *stack,
                *unknown_stack,
                *plain,
                *span,
                *node,
            )?,
            IrOp::LoadIndirect { dst, ptr, ty, span } => {
                self.lower_load_indirect(*dst, *ptr, *ty, *span)?
            }
            IrOp::StoreIndirect { ptr, src, ty, span } => {
                self.lower_store_indirect(*ptr, *src, *ty, *span)?
            }
        }
        Ok(())
    }

    fn lower_port_in(&mut self, dst: VReg, port: VReg, span: SourceSpan) -> Result<(), Diagnostic> {
        let imm = match self.loc_of(port, span)? {
            Loc::Imm8(n) if self.target.uses_immediate_port() => Some(n),
            _ => None,
        };
        if let Some(n) = imm {
            self.claim_r8(R8::A, None, span)?;
            self.emit(Z80Op::InImm { port: n }, span);
            self.bind_vreg(dst, Loc::Byte(R8::A), CType::U8);
            return Ok(());
        }
        self.prepare_port_in_c(port, span)?;
        self.claim_r8(R8::A, Some(Key::V(port)), span)?;
        self.emit(Z80Op::InC, span);
        self.bind_vreg(dst, Loc::Byte(R8::A), CType::U8);
        Ok(())
    }

    fn lower_port_out(
        &mut self,
        port: VReg,
        value: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let imm = match self.loc_of(port, span)? {
            Loc::Imm8(n) if self.target.uses_immediate_port() => Some(n),
            _ => None,
        };
        if let Some(n) = imm {
            self.ensure_a(value, span)?;
            self.emit(Z80Op::OutImm { port: n }, span);
            return Ok(());
        }
        if port == value {
            self.prepare_port_in_c(port, span)?;
            self.claim_r8(R8::A, Some(Key::V(port)), span)?;
            self.emit(
                Z80Op::Ld8 {
                    dst: R8::A,
                    src: R8::C,
                },
                span,
            );
            self.emit(Z80Op::OutC, span);
            return Ok(());
        }
        self.prepare_port_in_c(port, span)?;
        self.ensure_a(value, span)?;
        self.emit(Z80Op::OutC, span);
        Ok(())
    }

    fn prepare_port_in_c(&mut self, port: VReg, span: SourceSpan) -> Result<(), Diagnostic> {
        if self.target.uses_immediate_port() {
            self.ensure_in_r8(port, R8::C, span)
        } else {
            self.ensure_in_rr(port, Rr::Bc, span)
        }
    }

    fn lower_ldir(
        &mut self,
        hl: VReg,
        de: VReg,
        bc: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let clobbered = [R8::B, R8::C, R8::D, R8::E, R8::H, R8::L];
        let saved = self.save_around(&clobbered, &[], span)?;
        let moves = vec![
            (self.loc_of(hl, span)?, Loc::Word(Rr::Hl)),
            (self.loc_of(de, span)?, Loc::Word(Rr::De)),
            (self.loc_of(bc, span)?, Loc::Word(Rr::Bc)),
        ];
        self.unbind_saved(&saved);
        self.emit_parallel_moves(moves, span)?;
        self.emit(Z80Op::Ldir, span);
        self.restore_saved(saved, span);
        Ok(())
    }

    fn lower_unary(
        &mut self,
        dst: VReg,
        ty: CType,
        op: IrUnary,
        src: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        match op {
            IrUnary::Plus => {
                let loc = self.loc_of(src, span)?;
                self.bind_vreg(dst, loc, ty);
            }
            IrUnary::Not => {
                self.scratch_a(src, span)?;
                self.emit(Z80Op::Xor(AluSrc::Imm(1)), span);
                self.bind_vreg(dst, Loc::Byte(R8::A), ty);
            }
            IrUnary::BitNot => match ty.byte_width() {
                Some(1) => {
                    self.scratch_a(src, span)?;
                    self.emit(Z80Op::Cpl, span);
                    self.bind_vreg(dst, Loc::Byte(R8::A), ty);
                }
                Some(2) => {
                    self.scratch_hl(src, span)?;
                    self.claim_r8(R8::A, Some(Key::V(src)), span)?;
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::A,
                            src: R8::H,
                        },
                        span,
                    );
                    self.emit(Z80Op::Cpl, span);
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::H,
                            src: R8::A,
                        },
                        span,
                    );
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::A,
                            src: R8::L,
                        },
                        span,
                    );
                    self.emit(Z80Op::Cpl, span);
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::L,
                            src: R8::A,
                        },
                        span,
                    );
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
                }
                _ => return Err(self.error(span, "void bitwise not")),
            },
            IrUnary::Neg => match ty.byte_width() {
                Some(1) => {
                    self.scratch_a(src, span)?;
                    self.emit(Z80Op::Neg, span);
                    self.bind_vreg(dst, Loc::Byte(R8::A), ty);
                }
                Some(2) => {
                    self.scratch_hl(src, span)?;
                    self.claim_r8(R8::A, Some(Key::V(src)), span)?;
                    self.emit(Z80Op::Xor(AluSrc::Reg(R8::A)), span);
                    self.emit(Z80Op::Sub(AluSrc::Reg(R8::L)), span);
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::L,
                            src: R8::A,
                        },
                        span,
                    );
                    self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }, span);
                    self.emit(Z80Op::SbcA(AluSrc::Reg(R8::H)), span);
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::H,
                            src: R8::A,
                        },
                        span,
                    );
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
                }
                _ => return Err(self.error(span, "void negate")),
            },
        }
        Ok(())
    }

    fn lower_binary(
        &mut self,
        dst: VReg,
        ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        if matches!(op, IrBinary::And | IrBinary::Or) {
            return self.lower_bool_phi(dst, op, lhs, rhs, span);
        }
        if matches!(op, IrBinary::Shl | IrBinary::Shr) {
            return self.lower_shift(dst, ty, op, lhs, rhs, span);
        }
        match ty.byte_width() {
            Some(1) => {
                if is_compare(op) {
                    return self.lower_compare_value(dst, ty, op, lhs, rhs, span);
                }
                self.lower_binary8(dst, ty, op, lhs, rhs, span)
            }
            Some(2) => {
                if is_compare(op) {
                    return self.lower_compare_value(dst, ty, op, lhs, rhs, span);
                }
                self.lower_binary16(dst, ty, op, lhs, rhs, span)
            }
            _ => Err(self.error(span, "void binary")),
        }
    }

    fn lower_binary8(
        &mut self,
        dst: VReg,
        ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        // `x = x + 1` / `x = x - 1` on a byte in a register (H3i: an inlined
        // `inc(x)` is `x + 1` stored back): `INC r` / `DEC r` in place, 1
        // byte and 4 T instead of a trip through A. No IR op reads the carry
        // of a byte add.
        if matches!(op, IrBinary::Add | IrBinary::Sub) {
            if let Loc::Imm8(k @ (1 | 255)) = self.loc_of(rhs, span)? {
                let up = (op == IrBinary::Add) == (k == 1);
                // Only when the result goes straight back into that
                // register (`x = x + 1`); leaving `x + 1` in the operand's
                // register otherwise would keep it busy.
                if let Some(Loc::Byte(r)) = self.shift_store_back(dst, lhs) {
                    self.emit(if up { Z80Op::Inc8(r) } else { Z80Op::Dec8(r) }, span);
                    self.bind_vreg(dst, Loc::Byte(r), ty);
                    return Ok(());
                }
            }
        }
        if matches!(self.loc_of(rhs, span)?, Loc::Byte(R8::A)) && lhs != rhs {
            if !matches!(self.loc_of(lhs, span)?, Loc::Byte(R8::A)) {
                let tmp = self.free_r8_or_spill(Some(R8::C), Some(Key::V(rhs)), span)?;
                self.claim_r8(tmp, Some(Key::V(rhs)), span)?;
                self.emit(
                    Z80Op::Ld8 {
                        dst: tmp,
                        src: R8::A,
                    },
                    span,
                );
                self.vreg_loc.insert(rhs, Loc::Byte(tmp));
            }
        }
        self.scratch_a(lhs, span)?;
        let src = self.alu_src_of(rhs, span)?;
        let zop = match op {
            IrBinary::Add => Z80Op::AddA(src),
            IrBinary::Sub => Z80Op::Sub(src),
            IrBinary::BitAnd => Z80Op::And(src),
            IrBinary::BitXor => Z80Op::Xor(src),
            IrBinary::BitOr => Z80Op::Or(src),
            _ => return Err(self.error(span, "unsupported byte operator in this increment")),
        };
        self.emit(zop, span);
        self.bind_vreg(dst, Loc::Byte(R8::A), ty);
        Ok(())
    }

    fn lower_binary16(
        &mut self,
        dst: VReg,
        ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        // H5: `p = p + 1` (or + 2) on a word in BC or DE: `INC rr` in place,
        // so a pointer walked there never needs HL. Only when the result goes
        // straight back into that register; no IR op reads a word add's flags.
        if op == IrBinary::Add
            && let Loc::Imm16(n @ (1 | 2)) = self.loc_of(rhs, span)?
            && let Some(Loc::Word(rr)) = self.shift_store_back(dst, lhs)
        {
            for _ in 0..n {
                self.emit(Z80Op::Inc16(rr), span);
            }
            self.bind_vreg(dst, Loc::Word(rr), ty);
            return Ok(());
        }
        let rhs_in_hl = self.vreg_loc.get(&rhs) == Some(&Loc::Word(Rr::Hl));
        let commuted_add = op == IrBinary::Add
            && rhs_in_hl
            && matches!(self.vreg_loc.get(&lhs), Some(Loc::Word(Rr::De | Rr::Bc)));
        if !commuted_add
            && !self.hl_scratch_fits(lhs)
            && self.word_alu_in_pair(dst, ty, op, lhs, rhs, span)?
        {
            return Ok(());
        }
        match op {
            IrBinary::Add => {
                if let Loc::Imm16(n) = self.loc_of(rhs, span)? {
                    self.scratch_hl(lhs, span)?;
                    self.add_hl_imm(n, span)?;
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
                    return Ok(());
                }
                if let (Loc::Word(rr @ (Rr::De | Rr::Bc)), Loc::Word(Rr::Hl)) =
                    (self.loc_of(lhs, span)?, self.loc_of(rhs, span)?)
                {
                    // Addition commutes: add the left operand into HL.
                    self.park_live(&[R8::H, R8::L], span)?;
                    self.emit(Z80Op::AddHl(rr), span);
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
                    return Ok(());
                }
                self.scratch_hl(lhs, span)?;
                let src = self.word_src_rr(rhs, span)?;
                self.emit(Z80Op::AddHl(src), span);
                self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            }
            IrBinary::Sub => {
                self.scratch_hl(lhs, span)?;
                let src = self.word_src_rr(rhs, span)?;
                self.claim_r8(R8::A, Some(Key::V(lhs)), span)?;
                self.emit(Z80Op::Or(AluSrc::Reg(R8::A)), span);
                self.emit(Z80Op::SbcHl(src), span);
                self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            }
            IrBinary::BitAnd | IrBinary::BitXor | IrBinary::BitOr => {
                self.scratch_hl(lhs, span)?;
                // A constant mask goes straight into the ALU ops; it needs
                // no register pair.
                let (sh, sl) = match self.loc_of(rhs, span)? {
                    Loc::Imm16(n) => (AluSrc::Imm((n >> 8) as u8), AluSrc::Imm(n as u8)),
                    _ => {
                        let src = self.word_src_rr(rhs, span)?;
                        let (sh, sl) = src.halves();
                        (AluSrc::Reg(sh), AluSrc::Reg(sl))
                    }
                };
                self.claim_r8(R8::A, Some(Key::V(lhs)), span)?;
                let alu = |s: AluSrc| match op {
                    IrBinary::BitAnd => Z80Op::And(s),
                    IrBinary::BitXor => Z80Op::Xor(s),
                    _ => Z80Op::Or(s),
                };
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::A,
                        src: R8::H,
                    },
                    span,
                );
                self.emit(alu(sh), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::H,
                        src: R8::A,
                    },
                    span,
                );
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::A,
                        src: R8::L,
                    },
                    span,
                );
                self.emit(alu(sl), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::L,
                        src: R8::A,
                    },
                    span,
                );
                self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            }
            _ => return Err(self.error(span, "unsupported word operator in this increment")),
        }
        Ok(())
    }

    /// `r` holds nothing live after this operation and is not reserved.
    fn reg_dead(&self, r: R8) -> bool {
        !self.reserved.contains(&r)
            && !self.move_blocked.contains(&r)
            && self.occupants(r, None).iter().all(|&k| !self.key_live(k))
    }

    /// The next op stores `dst` into a local whose register `lhs` occupies
    /// (`x <<= 1`), and nothing there is live afterwards: the shift may run
    /// in that register. `HL` and `A` are the default places anyway.
    fn shift_store_back(&self, dst: VReg, lhs: VReg) -> Option<Loc> {
        let (src, local) = self.next_store?;
        if src != dst {
            return None;
        }
        let home = *self.local_loc.get(&local)?;
        if self.vreg_loc.get(&lhs) != Some(&home) {
            return None;
        }
        match home {
            Loc::Byte(r) if r != R8::A && self.reg_dead(r) => Some(home),
            Loc::Word(rr) if rr != Rr::Hl => {
                let (h, l) = rr.halves();
                (self.reg_dead(h) && self.reg_dead(l)).then_some(home)
            }
            _ => None,
        }
    }

    /// The local the next op stores into keeps its register (stable homes
    /// or a pin), so a result computed elsewhere must move back.
    fn store_home_fixed(&self) -> bool {
        self.stable
            || self
                .next_store
                .is_some_and(|(_, l)| self.pins.contains_key(&l))
    }

    /// `lhs` in a register other than `A`/`HL` that nothing needs after
    /// this operation.
    fn dead_home_of(&self, lhs: VReg) -> Option<Loc> {
        match self.vreg_loc.get(&lhs)? {
            Loc::Byte(r) if *r != R8::A && self.reg_dead(*r) => Some(Loc::Byte(*r)),
            Loc::Word(rr) if *rr != Rr::Hl => {
                let (h, l) = rr.halves();
                (self.reg_dead(h) && self.reg_dead(l)).then_some(Loc::Word(*rr))
            }
            _ => None,
        }
    }

    /// `<<` and `>>` (H3, semantics F001): the count is a `u8`; a count at
    /// or above the width gives 0, or all sign bits for a signed `>>`.
    fn lower_shift(
        &mut self,
        dst: VReg,
        ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let kind = match (op, ty.is_signed()) {
            (IrBinary::Shl, _) => ShiftKind::Shl,
            (_, false) => ShiftKind::Shr,
            (_, true) => ShiftKind::Sar,
        };
        let bits = ty
            .bit_width()
            .ok_or_else(|| self.error(span, "void shift"))?;
        let word = bits == 16;
        let count = match self.loc_of(rhs, span)? {
            Loc::Imm8(k) => Some(u32::from(k)),
            Loc::Imm16(k) => Some(u32::from(k & 0xFF)),
            _ => None,
        };
        let Some(k) = count else {
            return self.lower_shift_var(dst, ty, kind, word, lhs, rhs, span);
        };
        let imm = match self.loc_of(lhs, span)? {
            Loc::Imm8(n) => Some(u16::from(n)),
            Loc::Imm16(n) => Some(n),
            _ => None,
        };
        let constant = |x: u16| -> u16 {
            let la = ty.interpret_bits(x);
            let out = if k == 0 {
                la
            } else if k >= bits {
                match kind {
                    ShiftKind::Sar if la < 0 => -1,
                    _ => 0,
                }
            } else {
                match kind {
                    ShiftKind::Shl => la.wrapping_shl(k),
                    ShiftKind::Shr => ((ty.wrap_bits(la) as u32) >> k) as i32,
                    ShiftKind::Sar => la >> k,
                }
            };
            ty.wrap_bits(out)
        };
        let folded = imm
            .map(constant)
            .or_else(|| (k >= bits && kind != ShiftKind::Sar).then_some(0));
        if let Some(v) = folded {
            let loc = if word {
                Loc::Imm16(v)
            } else {
                Loc::Imm8(v as u8)
            };
            self.bind_vreg(dst, loc, ty);
            return Ok(());
        }
        if k == 0 {
            let loc = self.loc_of(lhs, span)?;
            self.bind_vreg(dst, loc, ty);
            return Ok(());
        }
        let k = k.min(bits - 1) as u8;
        let a_free = self.reg_dead(R8::A);
        let back = self.shift_store_back(dst, lhs);
        let fixed = self.store_home_fixed();
        let cost = |ops: &[Z80Op]| shift::seq_cost(ops);
        if !word {
            // In place in the operand's register, or through A.
            if let Some(Loc::Byte(r)) = back.or_else(|| self.dead_home_of(lhs)) {
                let here = shift::byte_const(kind, k, r, a_free);
                let (mut t_here, b_here) = cost(&here);
                if back.is_none() {
                    // The result most likely moves to A afterwards.
                    t_here += 4;
                }
                let (mut t_a, b_a) = cost(&shift::a_const(kind, k));
                t_a += 4;
                if back.is_some() && fixed {
                    t_a += 4;
                }
                if !a_free {
                    t_a += 8;
                }
                if (t_here, b_here) <= (t_a, b_a + 1) {
                    for op in here {
                        self.emit(op, span);
                    }
                    self.bind_vreg(dst, Loc::Byte(r), ty);
                    return Ok(());
                }
            }
            self.scratch_a(lhs, span)?;
            for op in shift::a_const(kind, k) {
                self.emit(op, span);
            }
            self.bind_vreg(dst, Loc::Byte(R8::A), ty);
            return Ok(());
        }
        let hl_busy = !matches!(self.vreg_loc.get(&lhs), Some(Loc::Word(Rr::Hl)))
            && !(self.reg_dead(R8::H) && self.reg_dead(R8::L));
        if let Some(Loc::Word(rr)) = back.or_else(|| self.dead_home_of(lhs)) {
            let here = shift::word_const(kind, k, rr, a_free);
            let (mut t_here, b_here) = cost(&here);
            if back.is_none() {
                t_here += 8;
            }
            let (mut t_hl, b_hl) = cost(&shift::word_const(kind, k, Rr::Hl, a_free));
            t_hl += 8;
            if back.is_some() && fixed {
                t_hl += 8;
            }
            if hl_busy {
                t_hl += 16;
            }
            if (t_here, b_here) <= (t_hl, b_hl + 2) {
                for op in here {
                    self.emit(op, span);
                }
                self.bind_vreg(dst, Loc::Word(rr), ty);
                return Ok(());
            }
        }
        self.scratch_hl(lhs, span)?;
        let a_free = self.reg_dead(R8::A);
        for op in shift::word_const(kind, k, Rr::Hl, a_free) {
            self.emit(op, span);
        }
        self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
        Ok(())
    }

    /// A shift by a run-time count: one step per count behind a zero guard
    /// (a zero count runs no step, so `DJNZ` never sees `B == 0`). Counts at
    /// or above the width run every step; the value stops changing after
    /// `width` steps, which gives F001's result.
    #[allow(clippy::too_many_arguments)]
    fn lower_shift_var(
        &mut self,
        dst: VReg,
        ty: CType,
        kind: ShiftKind,
        word: bool,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let back = self.shift_store_back(dst, lhs);
        // A step on A or HL is no slower than on another register, except
        // the 4 T `ADD A,A` / 11 T `ADD HL,HL`: shift left there unless the
        // accumulator is busy.
        let acc_busy = if word {
            !matches!(self.vreg_loc.get(&lhs), Some(Loc::Word(Rr::Hl)))
                && !(self.reg_dead(R8::H) && self.reg_dead(R8::L))
        } else {
            !matches!(self.vreg_loc.get(&lhs), Some(Loc::Byte(R8::A))) && !self.reg_dead(R8::A)
        };
        let here = match back {
            Some(loc) if kind != ShiftKind::Shl || acc_busy => Some(loc),
            Some(_) => None,
            None if acc_busy => self.dead_home_of(lhs),
            None => None,
        };
        let value_loc = here.unwrap_or(if word {
            Loc::Word(Rr::Hl)
        } else {
            Loc::Byte(R8::A)
        });
        let value_regs: Vec<R8> = value_loc.regs().to_vec();
        let mut avoid = value_regs.clone();
        if let Some(loc) = self.vreg_loc.get(&lhs).filter(|_| lhs != rhs) {
            avoid.extend_from_slice(loc.regs());
        }
        let ok = |this: &Self, r: R8| {
            !avoid.contains(&r)
                && !this.reserved.contains(&r)
                && !this.move_blocked.contains(&r)
                && this.occupants(r, None).iter().all(|&k| !this.key_live(k))
        };
        let counter = if ok(self, R8::B) {
            R8::B
        } else if let Some(Loc::Byte(r)) = self
            .vreg_loc
            .get(&rhs)
            .copied()
            .filter(|l| matches!(l, Loc::Byte(r) if ok(self, *r)))
        {
            r
        } else if let Some(r) = [R8::C, R8::D, R8::E, R8::H, R8::L, R8::A]
            .into_iter()
            .find(|&r| ok(self, r))
        {
            r
        } else {
            [R8::B, R8::C, R8::D, R8::E]
                .into_iter()
                .find(|r| !avoid.contains(r) && !self.reg_pinned(*r))
                .ok_or_else(|| self.pressure(span))?
        };
        self.reserve(&value_regs);
        self.scratch_r8(rhs, counter, span)?;
        if here.is_none() {
            // Release the value registers for the scratch copy below.
            self.reserved.retain(|r| !value_regs.contains(r));
            if word {
                self.scratch_hl(lhs, span)?;
            } else {
                self.scratch_a(lhs, span)?;
            }
        }
        self.reserve(&[counter]);
        let step = match value_loc {
            Loc::Word(rr) => shift::word_step(kind, rr),
            Loc::Byte(r) => shift::byte_step(kind, r),
            _ => return Err(self.error(span, "internal: shift operand has no register")),
        };
        let node = self.func.id.0;
        let top = self.aux_label();
        let done = self.aux_label();
        self.emit(Z80Op::Inc8(counter), span);
        self.emit(Z80Op::Dec8(counter), span);
        self.emit_jp(Some(Cc::Z), &done, span);
        self.emit_label(&top, span, node);
        for op in step {
            self.emit(op, span);
        }
        if counter == R8::B {
            self.emit(Z80Op::Djnz { target: top }, span);
        } else {
            self.emit(Z80Op::Dec8(counter), span);
            self.emit(
                Z80Op::Jr {
                    cc: Some(Cc::Nz),
                    target: top,
                },
                span,
            );
        }
        self.emit_label(&done, span, node);
        // The count register ends at zero: whatever named it is spent.
        if self.vreg_loc.get(&rhs) == Some(&Loc::Byte(counter)) && !self.consumed.contains(&rhs) {
            self.consumed.push(rhs);
        }
        self.bind_vreg(dst, value_loc, ty);
        Ok(())
    }

    fn add_hl_imm(&mut self, n: u16, span: SourceSpan) -> Result<(), Diagnostic> {
        match n {
            0 => Ok(()),
            1 => {
                self.emit(Z80Op::IncHl, span);
                Ok(())
            }
            2 => {
                self.emit(Z80Op::IncHl, span);
                self.emit(Z80Op::IncHl, span);
                Ok(())
            }
            _ => {
                self.claim_r8(R8::A, None, span)?;
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::A,
                        src: R8::L,
                    },
                    span,
                );
                self.emit(Z80Op::AddA(AluSrc::Imm(n as u8)), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::L,
                        src: R8::A,
                    },
                    span,
                );
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::A,
                        src: R8::H,
                    },
                    span,
                );
                self.emit(Z80Op::AdcA(AluSrc::Imm((n >> 8) as u8)), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::H,
                        src: R8::A,
                    },
                    span,
                );
                Ok(())
            }
        }
    }

    fn lower_cast(
        &mut self,
        dst: VReg,
        to: CType,
        from: CType,
        src: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let from_w = from
            .byte_width()
            .ok_or_else(|| self.error(span, "void cast source"))?;
        let to_w = to
            .byte_width()
            .ok_or_else(|| self.error(span, "void cast dest"))?;
        if to == CType::Bool {
            match from_w {
                1 => {
                    self.scratch_a(src, span)?;
                    self.emit_boolize_a(span);
                    self.bind_vreg(dst, Loc::Byte(R8::A), to);
                }
                2 => {
                    self.ensure_hl(src, span)?;
                    self.claim_r8(R8::A, Some(Key::V(src)), span)?;
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::A,
                            src: R8::H,
                        },
                        span,
                    );
                    self.emit(Z80Op::Or(AluSrc::Reg(R8::L)), span);
                    self.emit_boolize_a(span);
                    self.bind_vreg(dst, Loc::Byte(R8::A), to);
                }
                _ => return Err(self.error(span, "void to bool")),
            }
            return Ok(());
        }
        if from_w == to_w {
            let loc = self.loc_of(src, span)?;
            self.bind_vreg(dst, loc, to);
            return Ok(());
        }
        if from_w == 2 && to_w == 1 {
            match self.loc_of(src, span)? {
                Loc::Word(rr) => self.bind_vreg(dst, Loc::Byte(rr.halves().1), to),
                Loc::Imm16(n) => self.bind_vreg(dst, Loc::Imm8(n as u8), to),
                Loc::Byte(r) => self.bind_vreg(dst, Loc::Byte(r), to),
                Loc::Imm8(n) => self.bind_vreg(dst, Loc::Imm8(n), to),
                // The low byte of a word slot is a byte view of it.
                Loc::Slot { off, .. } => self.bind_vreg(dst, Loc::Slot { off, width: 1 }, to),
                Loc::Frame { .. } => {
                    self.ensure_hl(src, span)?;
                    self.bind_vreg(dst, Loc::Byte(R8::L), to);
                }
            }
            return Ok(());
        }
        if from_w == 1 && to_w == 2 {
            if from.is_signed() {
                if let Loc::Imm8(n) = self.loc_of(src, span)? {
                    self.bind_vreg(dst, Loc::Imm16(n as i8 as i16 as u16), to);
                    return Ok(());
                }
                self.scratch_a(src, span)?;
                self.park_live(&[R8::H, R8::L], span)?;
                self.claim_rr(Rr::Hl, Some(Key::V(src)), span)?;
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::L,
                        src: R8::A,
                    },
                    span,
                );
                self.emit(Z80Op::AddA(AluSrc::Reg(R8::A)), span);
                self.emit(Z80Op::SbcA(AluSrc::Reg(R8::A)), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::H,
                        src: R8::A,
                    },
                    span,
                );
                self.bind_vreg(dst, Loc::Word(Rr::Hl), to);
                return Ok(());
            }
            match self.loc_of(src, span)? {
                Loc::Imm8(n) => self.bind_vreg(dst, Loc::Imm16(u16::from(n)), to),
                Loc::Byte(r) if r != R8::H || !self.vreg_live(src) => {
                    self.claim_rr(Rr::Hl, Some(Key::V(src)), span)?;
                    // The claim may have shuffled registers: re-read.
                    let Loc::Byte(r) = self.loc_of(src, span)? else {
                        return Err(self.pressure(span));
                    };
                    if r != R8::L {
                        self.emit(Z80Op::Ld8 { dst: R8::L, src: r }, span);
                    }
                    self.emit(Z80Op::Ld8Imm { dst: R8::H, imm: 0 }, span);
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), to);
                }
                Loc::Byte(_) => {
                    // A live source in H: move it out before `LD H,0`.
                    self.park_live(&[R8::H, R8::L], span)?;
                    let r = match self.loc_of(src, span)? {
                        Loc::Byte(r) => r,
                        // No register was free: it went to a slot.
                        Loc::Slot { off, .. } => {
                            self.claim_rr(Rr::Hl, Some(Key::V(src)), span)?;
                            self.emit_move(Loc::Slot { off, width: 1 }, Loc::Word(Rr::Hl), span)?;
                            self.bind_vreg(dst, Loc::Word(Rr::Hl), to);
                            return Ok(());
                        }
                        _ => return Err(self.pressure(span)),
                    };
                    self.claim_rr(Rr::Hl, Some(Key::V(src)), span)?;
                    self.emit(Z80Op::Ld8 { dst: R8::L, src: r }, span);
                    self.emit(Z80Op::Ld8Imm { dst: R8::H, imm: 0 }, span);
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), to);
                }
                Loc::Word(rr) => self.bind_vreg(dst, Loc::Word(rr), to),
                Loc::Imm16(n) => self.bind_vreg(dst, Loc::Imm16(n), to),
                Loc::Slot { off, .. } => {
                    self.claim_rr(Rr::Hl, Some(Key::V(src)), span)?;
                    self.emit_move(Loc::Slot { off, width: 1 }, Loc::Word(Rr::Hl), span)?;
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), to);
                }
                Loc::Frame { .. } => {
                    self.ensure_a(src, span)?;
                    self.claim_rr(Rr::Hl, Some(Key::V(src)), span)?;
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::L,
                            src: R8::A,
                        },
                        span,
                    );
                    self.emit(Z80Op::Ld8Imm { dst: R8::H, imm: 0 }, span);
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), to);
                }
            }
            return Ok(());
        }
        Err(self.error(span, "unsupported cast"))
    }

    fn emit_boolize_a(&mut self, span: SourceSpan) {
        // NEG; SBC A,A; NEG maps 0 -> 0 and nonzero -> 1.
        self.emit(Z80Op::Neg, span);
        self.emit(Z80Op::SbcA(AluSrc::Reg(R8::A)), span);
        self.emit(Z80Op::Neg, span);
    }

    fn lower_return(&mut self, value: Option<VReg>, span: SourceSpan) -> Result<(), Diagnostic> {
        if let Some(v) = value {
            match return_home(self.func.ret) {
                Some(RegHome::Byte(R8::A)) => self.ensure_a(v, span)?,
                Some(RegHome::Word(Rr::Hl)) => self.ensure_hl(v, span)?,
                Some(_) => return Err(self.error(span, "unexpected return home")),
                None => return Err(self.error(span, "void function returned a value")),
            }
        }
        self.emit_epilogue(span);
        Ok(())
    }

    fn aux_label(&mut self) -> String {
        let n = self.next_aux;
        self.next_aux += 1;
        format!("{}_C{n}", self.func_label)
    }

    fn block_target(&self, blk: BlockId) -> String {
        asm_block_label(&self.func_label, blk)
    }

    /// Homes for parameters and locals. Without joins (and no address
    /// taken) parameters simply stay in their ABI registers. Otherwise each
    /// gets one home for the whole function: a register when the H1 plan
    /// fits, else registers by access weight (loop bodies count 8x) and a
    /// static slot for the rest. Address-taken locals always get a slot.
    fn plan_homes(&mut self, param_homes: &[RegHome]) -> Result<(), Diagnostic> {
        if !self.stable {
            for (param, home) in self.func.params.iter().zip(param_homes.iter()) {
                if self.local_last.contains_key(&param.id) {
                    self.local_loc.insert(param.id, Loc::from_home(*home));
                }
            }
            return Ok(());
        }
        let span = self.func.span;
        let addressed = addressed_locals(self.func);
        // Pins (H2b) are fixed homes; no other local is planned into them.
        let pinned_regs: Vec<R8> = self
            .pins
            .values()
            .flat_map(|(loc, _)| loc.regs().iter().copied())
            .collect();
        let mut stack_pins = Vec::new();
        if self.func.conv == CallConv::Stack {
            for (i, param) in self.func.params.iter().enumerate() {
                let disp = 4i16 + 2 * i as i16;
                let hi = disp + 1;
                let disp = i8::try_from(disp).map_err(|_| self.frame_error(param.span))?;
                let _ = i8::try_from(hi).map_err(|_| self.frame_error(param.span))?;
                let frame = Loc::Frame { disp, width: 2 };
                if let Some(&(pin, _)) = self.pins.get(&param.id) {
                    // The 2-byte slot holds a byte in its low half.
                    let frame = Loc::Frame {
                        disp,
                        width: pin.width(),
                    };
                    stack_pins.push((frame, pin, param.id));
                } else if self.local_last.contains_key(&param.id) {
                    self.local_loc.insert(param.id, frame);
                }
            }
        }
        let items: Vec<(LocalId, u8)> = stable_items(self.func, self.func.conv, &self.local_last)
            .into_iter()
            .filter(|(id, _)| !addressed.contains(id) && !self.pins.contains_key(id))
            .collect();
        let mut homes = if self.func.conv == CallConv::Stack {
            // Same register plan as H1 for `@stackcall` locals.
            let homes = HomeRegs::plan(&items, false, &HashSet::new(), &pinned_regs);
            if homes.len() == items.len() {
                homes
            } else {
                plan_register_homes(self.func, &items, false, &pinned_regs)
            }
        } else {
            match self.home_mode {
                HomeMode::Packed => plan_register_homes(self.func, &items, true, &pinned_regs),
                HomeMode::PackedPairs => {
                    let homes = HomeRegs::plan(&items, true, &HashSet::new(), &pinned_regs);
                    if homes.len() == items.len() {
                        homes
                    } else {
                        weighted_homes(self.func, &items, false, &pinned_regs)
                    }
                }
                HomeMode::Spread => weighted_homes(self.func, &items, false, &pinned_regs),
                HomeMode::Dense => weighted_homes_by(self.func, &items, false, true, &pinned_regs),
            }
        };
        for (&id, &(loc, _)) in &self.pins {
            homes.insert(id, loc);
        }
        if self.strict {
            let homeless = self
                .func
                .params
                .iter()
                .filter(|p| self.local_last.contains_key(&p.id))
                .map(|p| (p.id, p.span))
                .chain(self.func.locals.iter().map(|l| (l.id, l.span)))
                .find(|(id, _)| !homes.contains_key(id));
            if let Some((id, at)) = homeless {
                let what = if addressed.contains(&id) {
                    format!(
                        "{} has its address taken, so it needs memory",
                        self.local_name(id)
                    )
                } else {
                    format!(
                        "{} gets no register home (more locals are live than registers)",
                        self.local_name(id)
                    )
                };
                return Err(self.strict_violation(&what, at));
            }
        }
        // Planned homes, slots included, before any entry move runs.
        let mut planned: Vec<(LocalId, Loc)> = Vec::new();
        let mut taken: Vec<Loc> = homes.values().copied().collect();
        let width_of = |ty: CType| ty.byte_width().unwrap_or(1).max(1);
        let mut pending = stack_pins;
        for &(_, pin, id) in &pending {
            planned.push((id, pin));
        }
        if self.func.conv != CallConv::Stack {
            for (param, abi) in self.func.params.iter().zip(param_homes.iter()) {
                if !self.local_last.contains_key(&param.id) {
                    // Never read: no entry copy (a pin still owns its home).
                    if let Some(&(pin, _)) = self.pins.get(&param.id) {
                        planned.push((param.id, pin));
                    }
                    continue;
                }
                let home = match homes.get(&param.id) {
                    Some(&home) => home,
                    None => {
                        let width = width_of(param.ty);
                        let off = self.alloc_slot(width, &taken);
                        let slot = Loc::Slot { off, width };
                        taken.push(slot);
                        slot
                    }
                };
                planned.push((param.id, home));
                pending.push((Loc::from_home(*abi), home, param.id));
            }
        }
        for local in &self.func.locals {
            if self.local_loc.contains_key(&local.id)
                || planned.iter().any(|(id, _)| *id == local.id)
            {
                continue;
            }
            let home = match homes.get(&local.id) {
                Some(&home) => home,
                None => {
                    let width = width_of(local.ty);
                    let off = self.alloc_slot(width, &taken);
                    let slot = Loc::Slot { off, width };
                    taken.push(slot);
                    slot
                }
            };
            planned.push((local.id, home));
        }
        // Keep slot homes from being handed out while the entry moves run.
        for &(id, home) in &planned {
            if home.is_mem() {
                self.homes.insert(id, home);
            }
        }
        let saved_blocked = std::mem::take(&mut self.move_blocked);
        let mut done: Vec<R8> = Vec::new();
        while !pending.is_empty() {
            let is_ready = |dst: &Loc, pending: &[(Loc, Loc, LocalId)]| {
                !pending.iter().any(|(src, _, _)| dst.overlaps(*src))
            };
            let ready: Vec<usize> = (0..pending.len())
                .filter(|&i| is_ready(&pending[i].1, &pending))
                .collect();
            // Register moves first (the H1 order), then stores to slots.
            let pick = ready
                .iter()
                .copied()
                .find(|&i| !pending[i].1.is_mem())
                .or_else(|| ready.first().copied());
            let Some(i) = pick else {
                // A cycle: let the general parallel move break it.
                let moves = pending.iter().map(|&(src, dst, _)| (src, dst)).collect();
                self.move_blocked = done.clone();
                self.emit_parallel_moves(moves, span)?;
                for (_, dst, id) in pending.drain(..) {
                    self.local_loc.insert(id, dst);
                }
                break;
            };
            self.move_blocked = done.clone();
            for (src, _, _) in &pending {
                self.move_blocked.extend_from_slice(src.regs());
            }
            let (src, dst, id) = pending.remove(i);
            self.emit_move(src, dst, span)?;
            done.extend_from_slice(dst.regs());
            if self.local_last.contains_key(&id) {
                self.local_loc.insert(id, dst);
            }
        }
        self.move_blocked = saved_blocked;
        for (id, home) in planned {
            self.local_loc.entry(id).or_insert(home);
        }
        Ok(())
    }

    /// `@stackcall` only: IX addresses the parameters (`IX+4` onward). There
    /// are no IX locals since H2a.
    fn emit_prologue(&mut self, span: SourceSpan) {
        self.emit(Z80Op::PushIx, span);
        self.emit(Z80Op::LdIxImm(0), span);
        self.emit(Z80Op::AddIxSp, span);
        // The saved IX is the frame (`frame_bytes`), not a local push.
        self.push_depth = 0;
        self.asm_stack = 0;
    }

    fn emit_epilogue(&mut self, span: SourceSpan) {
        let was = self.synthetic;
        if self.uses_ix {
            self.synthetic = true;
            self.emit(Z80Op::LdSpIx, span);
            self.emit(Z80Op::PopIx, span);
            self.synthetic = was;
        }
        self.emit(Z80Op::Ret, span);
    }

    fn stack_provenance(&self) -> StackProvenance {
        if self.stack_unknown {
            StackProvenance::Unknown
        } else if self.stack_declared {
            StackProvenance::Declared
        } else {
            StackProvenance::Proven
        }
    }

    /// Emit moves that happen "at once". While a move runs, `move_blocked`
    /// holds the registers still read by pending moves and those already
    /// written, so slot code never takes them as scratch.
    fn emit_parallel_moves(
        &mut self,
        mut pending: Vec<(Loc, Loc)>,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let outer = self.move_blocked.clone();
        let mut done: Vec<R8> = Vec::new();
        let result = self.parallel_moves_inner(&mut pending, &outer, &mut done, span);
        self.move_blocked = outer;
        result
    }

    fn block_moves(&mut self, outer: &[R8], done: &[R8], pending: &[(Loc, Loc)], cur: Loc) {
        self.move_blocked = outer.to_vec();
        self.move_blocked.extend_from_slice(done);
        self.move_blocked.extend_from_slice(cur.regs());
        for (s, _) in pending {
            self.move_blocked.extend_from_slice(s.regs());
        }
    }

    fn parallel_moves_inner(
        &mut self,
        pending: &mut Vec<(Loc, Loc)>,
        outer: &[R8],
        done: &mut Vec<R8>,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        pending.retain(|(s, d)| s != d);
        while !pending.is_empty() {
            if let Some(i) = pending
                .iter()
                .position(|(_, dst)| !pending.iter().any(|(src, _)| dst.overlaps(*src)))
            {
                let (src, dst) = pending.remove(i);
                self.block_moves(outer, done, pending, src);
                self.emit_move(src, dst, span)?;
                done.extend_from_slice(dst.regs());
                continue;
            }
            if pending.len() == 2 {
                let (a, b) = (pending[0], pending[1]);
                if matches!(
                    (a, b),
                    (
                        (Loc::Word(Rr::Hl), Loc::Word(Rr::De)),
                        (Loc::Word(Rr::De), Loc::Word(Rr::Hl))
                    ) | (
                        (Loc::Word(Rr::De), Loc::Word(Rr::Hl)),
                        (Loc::Word(Rr::Hl), Loc::Word(Rr::De))
                    )
                ) {
                    self.emit(Z80Op::ExDeHl, span);
                    pending.clear();
                    continue;
                }
            }
            let (src, dst) = pending.remove(0);
            match (src, dst) {
                (Loc::Word(s), Loc::Word(d)) => {
                    self.emit(Z80Op::Push(s), span);
                    self.parallel_moves_inner(pending, outer, done, span)?;
                    self.emit(Z80Op::Pop(d), span);
                    done.extend_from_slice(dst.regs());
                    return Ok(());
                }
                (src, dst) => {
                    // Saving through A used to `LD A,src` first, which dropped a
                    // live byte already in A (`fill_rows(by, oy - by)` became
                    // `fill_rows(by, by)`). Park `src` in a scratch that the
                    // remaining moves do not touch, then restore into `dst`.
                    let mut blocked = Vec::from(dst.regs());
                    blocked.extend_from_slice(outer);
                    blocked.extend_from_slice(done);
                    for (s, d) in pending.iter() {
                        blocked.extend_from_slice(s.regs());
                        blocked.extend_from_slice(d.regs());
                    }
                    // F-036: nor a register holding a live value that is
                    // not part of this move set (a carried local at home).
                    let tmp = [R8::B, R8::D, R8::E, R8::H, R8::L]
                        .into_iter()
                        .find(|&r| {
                            !blocked.contains(&r)
                                && self
                                    .occupants(r, None)
                                    .iter()
                                    .all(|&k| self.defining.contains(&k) || !self.key_live(k))
                        })
                        .filter(|_| src.width() <= 1);
                    let tmp = match tmp {
                        Some(r) => Loc::Byte(r),
                        None => {
                            // No register left: break the cycle through a slot.
                            let width = src.width().max(dst.width()).max(1);
                            let mut taken: Vec<Loc> = pending.iter().map(|&(_, d)| d).collect();
                            taken.push(dst);
                            let off = self.alloc_slot(width, &taken);
                            Loc::Slot { off, width }
                        }
                    };
                    self.block_moves(outer, done, pending, src);
                    self.emit_move(src, tmp, span)?;
                    self.parallel_moves_inner(pending, outer, done, span)?;
                    self.block_moves(outer, done, &[], tmp);
                    self.emit_move(tmp, dst, span)?;
                    done.extend_from_slice(dst.regs());
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Run `f` as compiler-inserted fallback code (synthetic, `; spill`).
    fn spilling<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let (was_syn, was_note) = (self.synthetic, self.note);
        self.synthetic = true;
        self.note = Some("spill");
        let out = f(self);
        self.synthetic = was_syn;
        self.note = was_note;
        out
    }

    /// Live register groups with a register in `clobbered`.
    fn clobbered_groups(&self, clobbered: &[R8]) -> Vec<(Loc, Vec<Key>)> {
        let mut out: Vec<(Loc, Vec<Key>)> = Vec::new();
        for &r in R8::ALL.iter().filter(|r| clobbered.contains(r)) {
            for g in self.live_groups_at(r, None) {
                if !out.iter().any(|(l, _)| *l == g.0) {
                    out.push(g);
                }
            }
        }
        out
    }

    /// Save every value that is live after an operation clobbering
    /// `clobbered` (results land in `results`). Cheapest first: a move into
    /// a free register the operation does not touch (8 T), PUSH/POP of each
    /// affected pair (21 T, BC DE HL order), and for `A` a free half of a
    /// pushed pair (4 T), PUSH AF (21 T), a free half of another pair plus
    /// its push (25 T), or a static slot (26 T). A pair that holds a result
    /// afterwards is popped into a free pair, or its values go to slots.
    /// Saved keys are unbound until [`Self::restore_saved`].
    fn save_around(
        &mut self,
        clobbered: &[R8],
        results: &[R8],
        span: SourceSpan,
    ) -> Result<SavePlan, Diagnostic> {
        self.spilling(|this| this.save_around_inner(clobbered, results, span))
    }

    fn save_around_inner(
        &mut self,
        clobbered: &[R8],
        results: &[R8],
        span: SourceSpan,
    ) -> Result<SavePlan, Diagnostic> {
        let mut plan = SavePlan::default();
        let mut avoid: Vec<R8> = clobbered.to_vec();
        avoid.extend_from_slice(results);
        // Register shuffles into registers the operation leaves alone.
        for (loc, keys) in self.clobbered_groups(clobbered) {
            if self.key_loc(keys[0]) != Some(loc) || self.pinned_in(&keys).is_some() {
                continue;
            }
            let fits = match loc {
                Loc::Byte(_) => R8::ALL
                    .into_iter()
                    .any(|r| !avoid.contains(&r) && self.reg_scratch_ok(r)),
                Loc::Word(_) => [Rr::De, Rr::Bc, Rr::Hl].into_iter().any(|rr| {
                    let (h, l) = rr.halves();
                    !avoid.contains(&h)
                        && !avoid.contains(&l)
                        && self.reg_scratch_ok(h)
                        && self.reg_scratch_ok(l)
                }),
                _ => false,
            };
            if fits {
                self.relocate(keys[0], &avoid, None, span)?;
            }
        }
        let groups = self.clobbered_groups(clobbered);
        if self.strict {
            if let Some((loc, keys)) = groups.first() {
                let what = if let Some(callee) =
                    self.cur_callee.and_then(|id| self.callees.get(&id))
                {
                    let kept: Vec<&str> = R8::ALL
                        .into_iter()
                        .filter(|r| !callee.clobbers.contains(r))
                        .map(|r| loc_reg_name(Loc::Byte(r)))
                        .collect();
                    let reg = loc_reg_name(*loc);
                    if kept.is_empty() {
                        format!(
                            "{} is live in {reg} across {}, which has no register contract (it may change every register); declare @preserves({reg}) on '{}'",
                            self.key_name(keys),
                            self.cur_op,
                            callee.name
                        )
                    } else {
                        format!(
                            "{} is live in {reg} across {}, which preserves only {} ({}); keep the value in one of those, or add {reg} to the @preserves of '{}'",
                            self.key_name(keys),
                            self.cur_op,
                            kept.join(" "),
                            callee.contract.as_str(),
                            callee.name
                        )
                    }
                } else {
                    format!(
                        "{} is live in a register that {} changes",
                        self.key_name(keys),
                        self.cur_op
                    )
                };
                return Err(self.strict_violation(&what, span));
            }
        }
        // A pinned value is saved where it is: a pushed pair, or PUSH AF.
        for (loc, keys) in &groups {
            if let Some(l) = self.pinned_in(keys) {
                if loc.regs().iter().any(|r| results.contains(r)) {
                    return Err(self.pin_conflict(l, span));
                }
            }
        }
        let a_group = groups.iter().find(|(l, _)| *l == Loc::Byte(R8::A)).cloned();
        let mut pushed: Vec<Rr> = [Rr::Bc, Rr::De, Rr::Hl]
            .into_iter()
            .filter(|rr| {
                groups.iter().any(|(l, _)| {
                    *l != Loc::Byte(R8::A) && l.regs().iter().any(|&r| rr.contains(r))
                })
            })
            .collect();
        let after_free = |this: &Self, r: R8| {
            !this.reserved.contains(&r)
                && !this.move_blocked.contains(&r)
                && this.occupants(r, None).iter().all(|&k| !this.key_live(k))
        };
        let meets = |rr: Rr| {
            let (h, l) = rr.halves();
            results.contains(&h) || results.contains(&l)
        };
        for p in pushed.clone() {
            if !meets(p) {
                continue;
            }
            let q = [Rr::De, Rr::Bc, Rr::Hl].into_iter().find(|&q| {
                let (h, l) = q.halves();
                !pushed.contains(&q)
                    && !plan.pop_to.iter().any(|&(_, t)| t == q)
                    && !meets(q)
                    && after_free(self, h)
                    && after_free(self, l)
            });
            match q {
                Some(q) => plan.pop_to.push((p, q)),
                None => {
                    let (h, l) = p.halves();
                    for r in [h, l] {
                        for (loc, keys) in self.live_groups_at(r, None) {
                            if self.key_loc(keys[0]) == Some(loc) {
                                self.spill_loc(loc, &keys, None, span)?;
                            }
                        }
                    }
                    pushed.retain(|&x| x != p);
                }
            }
        }
        let mut a_keys: Option<(Vec<Key>, Loc)> = None;
        if let Some((_, keys)) = a_group.filter(|_| clobbered.contains(&R8::A)) {
            let a_pinned = self.pinned_in(&keys).is_some();
            let half = pushed
                .iter()
                .flat_map(|rr| {
                    let (h, l) = rr.halves();
                    [l, h]
                })
                .filter(|_| !a_pinned)
                .find(|&h| self.reg_scratch_ok(h) && after_free(self, h));
            if let Some(h) = half {
                self.emit(Z80Op::Ld8 { dst: h, src: R8::A }, span);
                a_keys = Some((keys, Loc::Byte(h)));
            } else if !results.contains(&R8::A) {
                plan.af = true;
                a_keys = Some((keys, Loc::Byte(R8::A)));
            } else {
                let other = [Rr::Bc, Rr::De, Rr::Hl]
                    .into_iter()
                    .filter(|&rr| !pushed.contains(&rr) && !meets(rr))
                    .find_map(|rr| {
                        let (h, l) = rr.halves();
                        [(l, h), (h, l)].into_iter().find_map(|(x, y)| {
                            (self.reg_scratch_ok(x) && after_free(self, x) && after_free(self, y))
                                .then_some((rr, x))
                        })
                    });
                match other {
                    Some((rr, x)) => {
                        self.emit(Z80Op::Ld8 { dst: x, src: R8::A }, span);
                        pushed.push(rr);
                        pushed.sort_by_key(|rr| match rr {
                            Rr::Bc => 0,
                            Rr::De => 1,
                            Rr::Hl => 2,
                        });
                        a_keys = Some((keys, Loc::Byte(x)));
                    }
                    None => self.spill_loc(Loc::Byte(R8::A), &keys, None, span)?,
                }
            }
        }
        let map = |plan: &SavePlan, loc: Loc| -> Loc {
            let remap = |r: R8| {
                for &(p, q) in &plan.pop_to {
                    let ((ph, pl), (qh, ql)) = (p.halves(), q.halves());
                    if r == ph {
                        return qh;
                    }
                    if r == pl {
                        return ql;
                    }
                }
                r
            };
            match loc {
                Loc::Byte(r) => Loc::Byte(remap(r)),
                Loc::Word(rr) => {
                    let target = plan
                        .pop_to
                        .iter()
                        .find(|&&(p, _)| p == rr)
                        .map_or(rr, |&(_, q)| q);
                    Loc::Word(target)
                }
                other => other,
            }
        };
        for (loc, keys) in self.clobbered_groups(clobbered) {
            if loc == Loc::Byte(R8::A) {
                continue;
            }
            if loc
                .regs()
                .iter()
                .any(|&r| pushed.iter().any(|rr| rr.contains(r)))
            {
                let after = map(&plan, loc);
                plan.groups.push((keys, after));
            }
        }
        if let Some((keys, loc)) = a_keys {
            let after = map(&plan, loc);
            plan.groups.push((keys, after));
        }
        for &rr in &pushed {
            self.emit(Z80Op::Push(rr), span);
        }
        if plan.af {
            self.emit(Z80Op::PushAf, span);
        }
        plan.pushed = pushed;
        plan.bytes = 2 * (plan.pushed.len() as u16 + u16::from(plan.af));
        if plan.bytes > 0 {
            self.saves.push(plan.bytes);
        }
        Ok(plan)
    }

    /// Unbind the keys a save plan holds on the stack for the operation.
    fn unbind_saved(&mut self, plan: &SavePlan) {
        for (keys, _) in &plan.groups {
            for k in keys {
                match k {
                    Key::V(v) => {
                        self.vreg_loc.remove(v);
                    }
                    Key::L(l) => {
                        self.local_loc.remove(l);
                    }
                }
            }
        }
    }

    fn restore_saved(&mut self, plan: SavePlan, span: SourceSpan) {
        self.spilling(|this| {
            if plan.af {
                this.emit(Z80Op::PopAf, span);
            }
            for &rr in plan.pushed.iter().rev() {
                let to = plan
                    .pop_to
                    .iter()
                    .find(|&&(p, _)| p == rr)
                    .map_or(rr, |&(_, q)| q);
                this.emit(Z80Op::Pop(to), span);
            }
        });
        for (keys, loc) in plan.groups {
            for k in keys {
                if let Key::L(l) = k {
                    self.spill_moved.insert(l);
                }
                self.bind_key(k, loc);
            }
        }
    }

    fn lower_call(
        &mut self,
        dst: Option<VReg>,
        func: FuncId,
        args: &[VReg],
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let callee = self
            .callees
            .get(&func)
            .ok_or_else(|| self.error(span, "call to unknown function"))?;
        let label = callee.label.clone();
        let clobbers = callee.clobbers.clone();
        let conv = callee.conv;
        let params = callee.params.clone();
        let ret = callee.ret;
        if args.len() != params.len() {
            return Err(self.error(span, "call argument count mismatch"));
        }

        let results: Vec<R8> = match return_home(ret) {
            Some(home) => Loc::from_home(home).regs().to_vec(),
            None => Vec::new(),
        };
        // The call changes the callee's clobbers, the argument registers the
        // caller loads, and the result. A `@stackcall` call may use any
        // register while pushing its arguments.
        let mut changed = clobbers.clone();
        if conv == CallConv::Stack {
            changed = R8::ALL.to_vec();
        } else if let Some(homes) = assign_params(&params) {
            for home in homes {
                changed.extend_from_slice(Loc::from_home(home).regs());
            }
        } else {
            changed = R8::ALL.to_vec();
        }
        changed.extend_from_slice(&results);
        let changed: Vec<R8> = R8::ALL
            .into_iter()
            .filter(|r| changed.contains(r))
            .collect();
        self.cur_callee = Some(func);
        let saved = self.save_around(&changed, &results, span);
        self.cur_callee = None;
        let saved = saved?;

        let arg_slots = if conv == CallConv::Stack {
            2u16.saturating_mul(args.len() as u16)
        } else {
            0
        };
        // The depth (saved pairs, stack arguments) is set when CALL is emitted.
        self.call_sites.push((func, 0));

        if conv == CallConv::Stack {
            self.unbind_saved(&saved);
            for i in (0..args.len()).rev() {
                self.push_stack_arg(args[i], params[i], &args[..i], span)?;
            }
        } else {
            let homes = assign_params(&params)
                .ok_or_else(|| self.error(span, "callee register signature does not fit"))?;
            let mut moves = Vec::new();
            for (arg, home) in args.iter().zip(homes.iter()) {
                let src = self.loc_of(*arg, span)?;
                moves.push((src, Loc::from_home(*home)));
            }
            self.unbind_saved(&saved);
            self.emit_parallel_moves(moves, span)?;
        }

        self.emit(
            Z80Op::Call {
                target: label,
                clobbers,
            },
            span,
        );

        if conv == CallConv::Stack {
            for _ in 0..arg_slots {
                self.emit(Z80Op::IncSp, span);
            }
        }

        if let Some(dst) = dst {
            match return_home(ret) {
                Some(RegHome::Byte(R8::A)) => self.bind_vreg(dst, Loc::Byte(R8::A), ret),
                Some(RegHome::Word(Rr::Hl)) => self.bind_vreg(dst, Loc::Word(Rr::Hl), ret),
                None => {}
                Some(_) => return Err(self.error(span, "unexpected return home")),
            }
        }
        self.restore_saved(saved, span);
        Ok(())
    }

    fn lower_inline_asm(
        &mut self,
        inputs: &[(AsmGpr, VReg)],
        outputs: &[IrAsmOutput],
        clobbers: &[AsmClobber],
        lines: &[String],
        stack: Option<u16>,
        unknown_stack: bool,
        plain: bool,
        span: SourceSpan,
        node: NodeId,
    ) -> Result<(), Diagnostic> {
        if unknown_stack {
            self.stack_unknown = true;
        }
        if stack.is_some() {
            self.stack_declared = true;
        }
        let flag_out = outputs
            .iter()
            .any(|o| matches!(o.kind, IrAsmOutKind::Carry | IrAsmOutKind::Zero));
        let mut dirty = if plain {
            RES_GP
        } else {
            let mut m = 0u16;
            for (reg, _) in inputs {
                m |= gpr_mask(*reg);
            }
            for out in outputs {
                if let IrAsmOutKind::Gpr(r) = out.kind {
                    m |= gpr_mask(r);
                }
            }
            for c in clobbers {
                m |= clobber_mask(*c);
            }
            m & RES_GP
        };
        if flag_out {
            dirty |= gpr_mask(AsmGpr::A) | gpr_mask(AsmGpr::Hl);
        }
        let dirty_regs = mask_to_r8s(dirty);

        let mut capture_taken: Vec<Loc> = Vec::new();
        let gpr_count = outputs
            .iter()
            .filter(|o| matches!(o.kind, IrAsmOutKind::Gpr(_)))
            .count();
        // Captured outputs go to slots; a single output stays in its register.
        let mut results: Vec<R8> = Vec::new();
        if !flag_out && gpr_count == 1 {
            for out in outputs {
                if let IrAsmOutKind::Gpr(r) = out.kind {
                    results.extend_from_slice(gpr_loc(r).regs());
                    if out.ty == CType::Bool {
                        results.push(R8::A);
                    }
                }
            }
        }
        self.asm_writes.extend_from_slice(&dirty_regs);
        let saved = self.save_around(&dirty_regs, &results, span)?;

        let mut moves = Vec::new();
        for (reg, v) in inputs {
            let src = self.loc_of(*v, span)?;
            moves.push((src, gpr_loc(*reg)));
        }
        self.unbind_saved(&saved);
        self.emit_parallel_moves(moves, span)?;
        self.guard_regs(&dirty_regs, "inline asm", span);

        for line in lines {
            if let Some(name) = line.strip_suffix(':') {
                if !name.contains(char::is_whitespace) && !name.is_empty() {
                    self.emit_label(name, span, node);
                    continue;
                }
            }
            if line.to_ascii_uppercase().contains(" EQU ") {
                self.emit_directive(line.clone(), span);
            } else {
                self.emit_raw(line.clone(), span);
            }
        }

        let gpr_outs: Vec<(AsmGpr, &IrAsmOutput)> = outputs
            .iter()
            .filter_map(|o| match o.kind {
                IrAsmOutKind::Gpr(r) => Some((r, o)),
                _ => None,
            })
            .collect();
        let capture = flag_out || gpr_outs.len() > 1;
        let mut capture_push = 0u16;
        if capture {
            let mut slots: Vec<(VReg, CType, Loc)> = Vec::new();
            if flag_out {
                self.emit(Z80Op::PushAf, span);
                capture_push = 2;
            }
            // Every output register is still unread here.
            let blocked = std::mem::take(&mut self.move_blocked);
            for (reg, _) in &gpr_outs {
                self.move_blocked.extend_from_slice(gpr_loc(*reg).regs());
            }
            for (reg, out) in &gpr_outs {
                let width = out.ty.byte_width().unwrap_or(1);
                let off = self.alloc_slot(width, &capture_taken);
                let slot = Loc::Slot { off, width };
                capture_taken.push(slot);
                self.emit_move(gpr_loc(*reg), slot, span)?;
                slots.push((out.dst, out.ty, slot));
            }
            self.move_blocked = blocked;
            if flag_out {
                self.emit(Z80Op::Pop(Rr::Hl), span);
                for out in outputs {
                    match out.kind {
                        IrAsmOutKind::Carry => {
                            self.emit(
                                Z80Op::Ld8 {
                                    dst: R8::A,
                                    src: R8::L,
                                },
                                span,
                            );
                            self.emit(Z80Op::And(AluSrc::Imm(1)), span);
                            let off = self.alloc_slot(1, &capture_taken);
                            let slot = Loc::Slot { off, width: 1 };
                            capture_taken.push(slot);
                            self.emit_move(Loc::Byte(R8::A), slot, span)?;
                            slots.push((out.dst, out.ty, slot));
                        }
                        IrAsmOutKind::Zero => {
                            self.emit(
                                Z80Op::Ld8 {
                                    dst: R8::A,
                                    src: R8::L,
                                },
                                span,
                            );
                            self.emit(Z80Op::And(AluSrc::Imm(0x40)), span);
                            let z1 = self.aux_label();
                            let z2 = self.aux_label();
                            self.emit(
                                Z80Op::Jp {
                                    cc: Some(Cc::Z),
                                    target: z1.clone(),
                                },
                                span,
                            );
                            self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 1 }, span);
                            self.emit(
                                Z80Op::Jp {
                                    cc: None,
                                    target: z2.clone(),
                                },
                                span,
                            );
                            self.emit_label(&z1, span, node);
                            self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }, span);
                            self.emit_label(&z2, span, node);
                            let off = self.alloc_slot(1, &capture_taken);
                            let slot = Loc::Slot { off, width: 1 };
                            capture_taken.push(slot);
                            self.emit_move(Loc::Byte(R8::A), slot, span)?;
                            slots.push((out.dst, out.ty, slot));
                        }
                        IrAsmOutKind::Gpr(_) => {}
                    }
                }
            }
            for (dst, ty, slot) in slots {
                if ty == CType::Bool {
                    self.emit_move(slot, Loc::Byte(R8::A), span)?;
                    self.normalize_bool_a(span, node)?;
                    self.bind_vreg(dst, Loc::Byte(R8::A), ty);
                } else {
                    self.bind_vreg(dst, slot, ty);
                }
            }
        } else if let Some((reg, out)) = gpr_outs.first() {
            if out.ty == CType::Bool {
                self.emit_move(gpr_loc(*reg), Loc::Byte(R8::A), span)?;
                self.normalize_bool_a(span, node)?;
                self.bind_vreg(out.dst, Loc::Byte(R8::A), out.ty);
            } else {
                self.bind_vreg(out.dst, gpr_loc(*reg), out.ty);
            }
        }

        let save_bytes = saved.bytes;
        self.restore_saved(saved, span);
        // The block's own stack use sits on top of the saved pairs; the
        // capture PUSH AF is counted where it is emitted.
        let _ = capture_push;
        let extra = stack.unwrap_or(0).saturating_add(save_bytes);
        self.asm_stack = self.asm_stack.max(extra);
        Ok(())
    }

    fn normalize_bool_a(&mut self, span: SourceSpan, node: NodeId) -> Result<(), Diagnostic> {
        let done = self.aux_label();
        self.emit(Z80Op::Or(AluSrc::Reg(R8::A)), span);
        self.emit(
            Z80Op::Jp {
                cc: Some(Cc::Z),
                target: done.clone(),
            },
            span,
        );
        self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 1 }, span);
        self.emit_label(&done, span, node);
        Ok(())
    }

    fn remaining_regs(&self, remaining: &[VReg]) -> Vec<R8> {
        let mut regs = Vec::new();
        for &v in remaining {
            if let Some(loc) = self.vreg_loc.get(&v) {
                for &r in loc.regs() {
                    if !regs.contains(&r) {
                        regs.push(r);
                    }
                }
            }
        }
        regs
    }

    fn push_pair_free(&self, remaining: &[VReg]) -> Option<Rr> {
        let busy = self.remaining_regs(remaining);
        [Rr::Hl, Rr::De, Rr::Bc].into_iter().find(|&rr| {
            let (h, l) = rr.halves();
            !busy.contains(&h) && !busy.contains(&l)
        })
    }

    /// A pair not holding a later argument; when every pair does, one of
    /// those arguments moves to a static slot until it is pushed.
    fn push_pair_or_spill(
        &mut self,
        remaining: &[VReg],
        span: SourceSpan,
    ) -> Result<Rr, Diagnostic> {
        loop {
            if let Some(rr) = self.push_pair_free(remaining) {
                return Ok(rr);
            }
            let victim = remaining
                .iter()
                .rev()
                .find_map(|&v| match self.vreg_loc.get(&v) {
                    Some(&loc @ (Loc::Byte(_) | Loc::Word(_))) => Some(loc),
                    _ => None,
                });
            let Some(loc) = victim else {
                return Err(self.pressure(span));
            };
            let keys = self.keys_at(loc);
            self.spilling(|this| this.spill_loc(loc, &keys, None, span))?;
        }
    }

    fn push_stack_arg(
        &mut self,
        arg: VReg,
        ty: CType,
        remaining: &[VReg],
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let loc = self.loc_of(arg, span)?;
        match (ty.byte_width(), loc) {
            (Some(2), Loc::Word(rr)) => self.emit(Z80Op::Push(rr), span),
            (Some(2), _) => {
                let rr = self.push_pair_or_spill(remaining, span)?;
                self.ensure_in_rr(arg, rr, span)?;
                self.emit(Z80Op::Push(rr), span);
            }
            (Some(1), _) => {
                let rr = self.push_pair_or_spill(remaining, span)?;
                // Loading A must not relocate anything into the chosen pair.
                let (ph, pl) = rr.halves();
                self.reserve(&[ph, pl]);
                self.ensure_a(arg, span)?;
                let (h, l) = rr.halves();
                if l != R8::A {
                    self.emit(Z80Op::Ld8 { dst: l, src: R8::A }, span);
                }
                if ty.is_signed() {
                    self.emit(Z80Op::Rla, span);
                    self.emit(Z80Op::SbcA(AluSrc::Reg(R8::A)), span);
                    self.emit(Z80Op::Ld8 { dst: h, src: R8::A }, span);
                    if l == R8::A {
                        // A was overwritten by sign extend; restore low from L after if needed.
                    }
                } else {
                    self.emit(Z80Op::Ld8Imm { dst: h, imm: 0 }, span);
                }
                if l == R8::A {
                    self.emit(
                        Z80Op::Ld8 {
                            dst: R8::A,
                            src: R8::L,
                        },
                        span,
                    );
                }
                self.emit(Z80Op::Push(rr), span);
            }
            _ => return Err(self.error(span, "void argument")),
        }
        // Pushed: an argument that dies here no longer needs its register.
        if !self.vreg_live(arg) && !self.consumed.contains(&arg) {
            self.consumed.push(arg);
        }
        Ok(())
    }

    /// Stable mode: put every live local that an operation moved off its home
    /// back before control leaves the block, so all predecessors of a block
    /// agree on where locals live.
    fn restore_homes(&mut self, span: SourceSpan) -> Result<(), Diagnostic> {
        if !self.stable {
            return Ok(());
        }
        let mut moves = Vec::new();
        let mut after_spill = false;
        let mut drifted: Vec<(LocalId, Loc)> = self
            .homes
            .iter()
            .filter(|(l, home)| self.local_loc.get(l) != Some(home))
            .map(|(&l, &home)| (l, home))
            .collect();
        drifted.sort_by_key(|(l, _)| l.0.0);
        for (l, home) in drifted {
            if let Some(&cur) = self.local_loc.get(&l) {
                if self.local_live(l) {
                    moves.push((cur, home));
                    self.defining.push(Key::L(l));
                    after_spill |= self.spill_moved.contains(&l);
                }
            }
            self.spill_moved.remove(&l);
            self.local_loc.insert(l, home);
        }
        // Moving a value back after a fallback moved it away is part of the
        // fallback: mark it `; spill` so the report counts it (H3).
        if after_spill {
            self.spilling(|this| this.emit_parallel_moves(moves, span))
        } else {
            self.emit_parallel_moves(moves, span)
        }
    }

    fn lower_jump(&mut self, blk: BlockId, span: SourceSpan) -> Result<(), Diagnostic> {
        if let Some(block) = self.func.blocks.iter().find(|b| b.id == blk) {
            if let Some(IrOp::Binary { op, rhs, .. }) = block.ops.first() {
                if matches!(op, IrBinary::And | IrBinary::Or) {
                    if self.vreg_loc.contains_key(rhs) {
                        self.ensure_a(*rhs, span)?;
                    } else {
                        let imm = u8::from(*op == IrBinary::Or);
                        self.claim_r8(R8::A, None, span)?;
                        self.emit(Z80Op::Ld8Imm { dst: R8::A, imm }, span);
                    }
                }
            }
        }
        self.restore_homes(span)?;
        self.synthetic = true;
        self.emit(
            Z80Op::Jp {
                cc: None,
                target: self.block_target(blk),
            },
            span,
        );
        self.synthetic = false;
        Ok(())
    }

    fn lower_branch(
        &mut self,
        cond: VReg,
        true_blk: BlockId,
        false_blk: BlockId,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        self.restore_homes(span)?;
        self.ensure_a(cond, span)?;
        self.emit(Z80Op::Or(AluSrc::Reg(R8::A)), span);
        self.emit(
            Z80Op::Jp {
                cc: Some(Cc::Nz),
                target: self.block_target(true_blk),
            },
            span,
        );
        self.emit(
            Z80Op::Jp {
                cc: None,
                target: self.block_target(false_blk),
            },
            span,
        );
        Ok(())
    }

    fn lower_bool_phi(
        &mut self,
        dst: VReg,
        _op: IrBinary,
        _lhs: VReg,
        _rhs: VReg,
        _span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        self.bind_vreg(dst, Loc::Byte(R8::A), CType::Bool);
        Ok(())
    }

    fn lower_load_indirect(
        &mut self,
        dst: VReg,
        ptr: VReg,
        ty: CType,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        // H5: a byte load through a pointer already in BC or DE reads it in
        // place (`LD A,(BC)` / `LD A,(DE)`), leaving HL alone.
        if ty.byte_width() == Some(1)
            && let Some(Loc::Word(rr @ (Rr::Bc | Rr::De))) = self.vreg_loc.get(&ptr).copied()
        {
            let (h, l) = rr.halves();
            self.reserve(&[h, l]);
            self.claim_r8(R8::A, Some(Key::V(ptr)), span)?;
            if self.vreg_loc.get(&ptr) == Some(&Loc::Word(rr)) {
                self.emit(Z80Op::LdAInd(rr), span);
                self.bind_vreg(dst, Loc::Byte(R8::A), ty);
                return Ok(());
            }
        }
        self.ensure_hl(ptr, span)?;
        match ty.byte_width() {
            Some(1) => {
                self.claim_r8(R8::A, Some(Key::V(ptr)), span)?;
                self.emit(Z80Op::LdHl(R8::A), span);
                self.bind_vreg(dst, Loc::Byte(R8::A), ty);
            }
            Some(2) => {
                // `INC HL` walks the pointer: HL must be a scratch copy.
                self.scratch_hl(ptr, span)?;
                self.claim_r8(R8::A, Some(Key::V(ptr)), span)?;
                self.emit(Z80Op::LdHl(R8::A), span);
                self.emit(Z80Op::IncHl, span);
                self.emit(Z80Op::LdHl(R8::H), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::L,
                        src: R8::A,
                    },
                    span,
                );
                self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            }
            _ => return Err(self.error(span, "cannot load this type from a pointer")),
        }
        Ok(())
    }

    fn lower_store_indirect(
        &mut self,
        ptr: VReg,
        src: VReg,
        ty: CType,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        match ty.byte_width() {
            Some(1) => {
                self.ensure_a(src, span)?;
                self.reserve(&[R8::A]);
                // H5: through BC or DE in place when the pointer is there.
                if let Some(Loc::Word(rr @ (Rr::Bc | Rr::De))) = self.vreg_loc.get(&ptr).copied() {
                    self.emit(Z80Op::StIndA(rr), span);
                    return Ok(());
                }
                self.ensure_hl(ptr, span)?;
                self.emit(Z80Op::StHl(R8::A), span);
            }
            Some(2) => {
                // The value is stored straight from its pair (`LD (HL),r`);
                // HL is a scratch copy of the pointer because of `INC HL`.
                let rr = match self.loc_of(src, span)? {
                    Loc::Word(rr @ (Rr::De | Rr::Bc)) => rr,
                    _ => {
                        self.ensure_in_rr(src, Rr::De, span)?;
                        Rr::De
                    }
                };
                let (h, l) = rr.halves();
                self.reserve(&[h, l]);
                self.scratch_hl(ptr, span)?;
                let Loc::Word(rr) = self.loc_of(src, span)? else {
                    return Err(self.pressure(span));
                };
                let (h, l) = rr.halves();
                self.emit(Z80Op::StHl(l), span);
                self.emit(Z80Op::IncHl, span);
                self.emit(Z80Op::StHl(h), span);
            }
            _ => return Err(self.error(span, "cannot store this type through a pointer")),
        }
        Ok(())
    }

    fn lower_addr_global(
        &mut self,
        dst: VReg,
        global: GlobalId,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let (symbol, ty) = self
            .globals
            .get(&global)
            .cloned()
            .ok_or_else(|| self.error(span, "unknown global"))?;
        self.claim_rr(Rr::Hl, None, span)?;
        self.emit(
            Z80Op::Ld16Sym {
                dst: Rr::Hl,
                symbol,
            },
            span,
        );
        let result_ty = match ty {
            CType::Array(a) => PtrType::of(a.elem.to_ctype()).map(CType::Ptr).unwrap_or(ty),
            CType::Str => CType::Str,
            other => PtrType::of(other).map(CType::Ptr).unwrap_or(CType::U16),
        };
        self.bind_vreg(dst, Loc::Word(Rr::Hl), result_ty);
        Ok(())
    }

    fn lower_addr_local(
        &mut self,
        dst: VReg,
        local: LocalId,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let loc = *self
            .local_loc
            .get(&local)
            .ok_or_else(|| self.error(span, "addressable local has no home"))?;
        if let Loc::Slot { off, .. } = loc {
            // A static slot has a link-time address.
            self.claim_rr(Rr::Hl, None, span)?;
            let symbol = self.slot_symbol(i32::from(off));
            self.emit(
                Z80Op::Ld16Sym {
                    dst: Rr::Hl,
                    symbol,
                },
                span,
            );
            let elem = *self
                .local_ty
                .get(&local)
                .ok_or_else(|| self.error(span, "addressable local has no type"))?;
            let ty = PtrType::of(elem).map(CType::Ptr).unwrap_or(CType::U16);
            self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            return Ok(());
        }
        let Loc::Frame { disp, .. } = loc else {
            return Err(self.error(span, "addressable local has no memory home"));
        };
        self.claim_rr(Rr::Hl, None, span)?;
        self.reserve(&[R8::H, R8::L]);
        self.emit(Z80Op::PushIx, span);
        self.emit(Z80Op::Pop(Rr::Hl), span);
        if disp != 0 {
            self.claim_r8(R8::A, None, span)?;
            self.emit(
                Z80Op::Ld8 {
                    dst: R8::A,
                    src: R8::L,
                },
                span,
            );
            self.emit(Z80Op::AddA(AluSrc::Imm(disp as u8)), span);
            self.emit(
                Z80Op::Ld8 {
                    dst: R8::L,
                    src: R8::A,
                },
                span,
            );
            self.emit(
                Z80Op::Ld8 {
                    dst: R8::A,
                    src: R8::H,
                },
                span,
            );
            let high = if disp < 0 { 0xFF } else { 0 };
            self.emit(Z80Op::AdcA(AluSrc::Imm(high)), span);
            self.emit(
                Z80Op::Ld8 {
                    dst: R8::H,
                    src: R8::A,
                },
                span,
            );
        }
        let elem = *self
            .local_ty
            .get(&local)
            .ok_or_else(|| self.error(span, "addressable local has no type"))?;
        let ty = PtrType::of(elem).map(CType::Ptr).unwrap_or(CType::U16);
        self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
        Ok(())
    }

    fn lower_load_global(
        &mut self,
        dst: VReg,
        global: GlobalId,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let (symbol, ty) = self
            .globals
            .get(&global)
            .cloned()
            .ok_or_else(|| self.error(span, "unknown global"))?;
        match ty.byte_width() {
            Some(1) => {
                self.claim_r8(R8::A, None, span)?;
                self.emit(Z80Op::LdAbs8 { dst: R8::A, symbol }, span);
                self.bind_vreg(dst, Loc::Byte(R8::A), ty);
            }
            Some(2) => {
                self.claim_rr(Rr::Hl, None, span)?;
                self.emit(
                    Z80Op::LdAbs16 {
                        dst: Rr::Hl,
                        symbol,
                    },
                    span,
                );
                self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            }
            _ => return Err(self.error(span, "void global load")),
        }
        Ok(())
    }

    fn lower_store_global(
        &mut self,
        global: GlobalId,
        src: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let (symbol, ty) = self
            .globals
            .get(&global)
            .cloned()
            .ok_or_else(|| self.error(span, "unknown global"))?;
        match ty.byte_width() {
            Some(1) => {
                self.ensure_a(src, span)?;
                self.emit(Z80Op::StAbs8 { src: R8::A, symbol }, span);
            }
            Some(2) => {
                self.ensure_hl(src, span)?;
                self.emit(
                    Z80Op::StAbs16 {
                        src: Rr::Hl,
                        symbol,
                    },
                    span,
                );
            }
            _ => return Err(self.error(span, "void global store")),
        }
        Ok(())
    }

    fn lower_compare_branch(
        &mut self,
        _ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        true_blk: BlockId,
        false_blk: BlockId,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        self.emit_compare_flags(op, lhs, rhs, span)?;
        // Restoring homes uses only flag-preserving moves.
        self.restore_homes(span)?;
        self.emit_jp_condition(op, true_blk, false_blk, span);
        Ok(())
    }

    fn lower_compare_value(
        &mut self,
        dst: VReg,
        _ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        self.emit_compare_flags(op, lhs, rhs, span)?;
        // The flags hold the comparison: operands that die here are spent.
        for v in [lhs, rhs] {
            if !self.vreg_live(v) && !self.consumed.contains(&v) {
                self.consumed.push(v);
            }
        }
        let end = self.aux_label();
        self.claim_r8(R8::A, None, span)?;
        match op {
            IrBinary::Gt => {
                self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }, span);
                self.emit_jp(Some(Cc::Z), &end, span);
                self.emit_jp(Some(Cc::C), &end, span);
                self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 1 }, span);
            }
            _ => {
                self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 1 }, span);
                match op {
                    IrBinary::Eq => self.emit_jp(Some(Cc::Z), &end, span),
                    IrBinary::Ne => self.emit_jp(Some(Cc::Nz), &end, span),
                    IrBinary::Lt => self.emit_jp(Some(Cc::C), &end, span),
                    IrBinary::Ge => self.emit_jp(Some(Cc::Nc), &end, span),
                    IrBinary::Le => {
                        self.emit_jp(Some(Cc::C), &end, span);
                        self.emit_jp(Some(Cc::Z), &end, span);
                    }
                    _ => return Err(self.error(span, "not a comparison")),
                }
                self.emit(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }, span);
            }
        }
        self.emit_label(&end, span, self.func.id.0);
        self.bind_vreg(dst, Loc::Byte(R8::A), CType::Bool);
        Ok(())
    }

    fn emit_jp(&mut self, cc: Option<Cc>, target: &str, span: SourceSpan) {
        self.emit(
            Z80Op::Jp {
                cc,
                target: target.to_string(),
            },
            span,
        );
    }

    fn emit_jp_condition(
        &mut self,
        op: IrBinary,
        true_blk: BlockId,
        false_blk: BlockId,
        span: SourceSpan,
    ) {
        let t = self.block_target(true_blk);
        let f = self.block_target(false_blk);
        match op {
            IrBinary::Eq => {
                self.emit_jp(Some(Cc::Z), &t, span);
                self.emit_jp(None, &f, span);
            }
            IrBinary::Ne => {
                self.emit_jp(Some(Cc::Nz), &t, span);
                self.emit_jp(None, &f, span);
            }
            IrBinary::Lt => {
                self.emit_jp(Some(Cc::C), &t, span);
                self.emit_jp(None, &f, span);
            }
            IrBinary::Ge => {
                self.emit_jp(Some(Cc::Nc), &t, span);
                self.emit_jp(None, &f, span);
            }
            IrBinary::Le => {
                self.emit_jp(Some(Cc::C), &t, span);
                self.emit_jp(Some(Cc::Z), &t, span);
                self.emit_jp(None, &f, span);
            }
            IrBinary::Gt => {
                self.emit_jp(Some(Cc::Z), &f, span);
                self.emit_jp(Some(Cc::C), &f, span);
                self.emit_jp(None, &t, span);
            }
            _ => {
                self.emit_jp(None, &t, span);
            }
        }
    }

    /// The right operand of a signed word compare with its sign bit flipped,
    /// in a pair `SBC HL` can use. The flip happens in place only when no
    /// live value shares the pair (F-018 for `XOR 80H` on the high byte).
    fn signed_word_rhs(
        &mut self,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(Rr, Option<R8>), Diagnostic> {
        if let Loc::Imm16(n) = self.loc_of(rhs, span)? {
            let rr = self.free_rr_or_spill(&[Rr::De, Rr::Bc, Rr::Hl], None, span)?;
            self.reserve(&{
                let (h, l) = rr.halves();
                [h, l]
            });
            self.emit(
                Z80Op::Ld16Imm {
                    dst: rr,
                    imm: n ^ 0x8000,
                },
                span,
            );
            return Ok((rr, None));
        }
        let src = self.word_src_rr(rhs, span)?;
        let (sh, sl) = src.halves();
        let shared = [sh, sl]
            .into_iter()
            .any(|r| self.occupants(r, None).iter().any(|&k| self.key_live(k)));
        let mut restore = None;
        let rr = match (shared, self.alloc_rr(Some(Rr::De), None)) {
            (true, Some(rr)) => {
                let (h, l) = rr.halves();
                self.emit(Z80Op::Ld8 { dst: l, src: sl }, span);
                self.emit(Z80Op::Ld8 { dst: h, src: sh }, span);
                rr
            }
            (true, None) => {
                // No pair for a copy: flip the live high byte in place and
                // flip it back after the compare (see `emit_compare_flags`).
                // The values in it are only borrowed for this operation.
                let keys = self.occupants(sh, None);
                self.defining.extend(keys);
                restore = Some(sh);
                src
            }
            (false, _) => src,
        };
        let (hh, hl) = rr.halves();
        self.reserve(&[hh, hl]);
        self.emit(
            Z80Op::Ld8 {
                dst: R8::A,
                src: hh,
            },
            span,
        );
        self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
        self.emit(
            Z80Op::Ld8 {
                dst: hh,
                src: R8::A,
            },
            span,
        );
        Ok((rr, restore))
    }

    fn emit_compare_flags(
        &mut self,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let equality = matches!(op, IrBinary::Eq | IrBinary::Ne);
        let ty = self
            .vreg_ty
            .get(&lhs)
            .copied()
            .ok_or_else(|| self.error(span, "compare operand has no type"))?;
        let width = ty.byte_width().unwrap_or(1);
        // Equality does not depend on signedness, and it is symmetric: the
        // operand already in A or HL becomes the scratch side.
        let signed = ty.is_signed() && !equality;
        let (lhs, rhs) = if equality {
            let hot = if width == 1 {
                Loc::Byte(R8::A)
            } else {
                Loc::Word(Rr::Hl)
            };
            if self.vreg_loc.get(&rhs) == Some(&hot) && self.vreg_loc.get(&lhs) != Some(&hot) {
                (rhs, lhs)
            } else {
                (lhs, rhs)
            }
        } else {
            (lhs, rhs)
        };
        if width == 1 {
            if signed {
                // F-024: bias copies in A, never the operands' own registers.
                if let Loc::Imm8(n) = self.loc_of(rhs, span)? {
                    self.scratch_a(lhs, span)?;
                    self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                    self.emit(Z80Op::Cp(AluSrc::Imm(n ^ 0x80)), span);
                } else {
                    self.scratch_a(rhs, span)?;
                    self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                    let tmp = self.free_r8_or_spill(Some(R8::B), None, span)?;
                    self.reserve(&[tmp]);
                    self.emit(
                        Z80Op::Ld8 {
                            dst: tmp,
                            src: R8::A,
                        },
                        span,
                    );
                    // The biased copy in `tmp` is all that is left to read.
                    if !self.vreg_live(rhs) && !self.consumed.contains(&rhs) {
                        self.consumed.push(rhs);
                    }
                    self.scratch_a(lhs, span)?;
                    self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                    self.emit(Z80Op::Cp(AluSrc::Reg(tmp)), span);
                }
            } else {
                self.ensure_a(lhs, span)?;
                let src = self.alu_src_of(rhs, span)?;
                self.emit(Z80Op::Cp(src), span);
            }
        } else {
            self.scratch_hl(lhs, span)?;
            let mut restore = None;
            let src = if signed {
                self.claim_r8(R8::A, None, span)?;
                self.reserve(&[R8::A]);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::A,
                        src: R8::H,
                    },
                    span,
                );
                self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::H,
                        src: R8::A,
                    },
                    span,
                );
                let (rr, back) = self.signed_word_rhs(rhs, span)?;
                restore = back;
                rr
            } else {
                self.word_src_rr(rhs, span)?
            };
            self.emit(Z80Op::Or(AluSrc::Reg(R8::A)), span);
            self.emit(Z80Op::SbcHl(src), span);
            if let Some(hh) = restore {
                self.emit(Z80Op::PushAf, span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: R8::A,
                        src: hh,
                    },
                    span,
                );
                self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                self.emit(
                    Z80Op::Ld8 {
                        dst: hh,
                        src: R8::A,
                    },
                    span,
                );
                self.emit(Z80Op::PopAf, span);
            }
        }
        Ok(())
    }
}
