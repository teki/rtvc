//! Straight-line register-leaf lowering from typed IR to structured Z80 items.

use super::abi::{assign_params, return_home};
use super::ast::CallConv;
use super::diagnostic::{DiagCode, Diagnostic};
use super::ir::{
    BlockId, FuncId, GlobalId, IrBinary, IrOp, IrUnary, LocalId, TypedFunction, TypedGlobal,
    TypedProgram, VReg,
};
use super::source::{FileId, IdGen, NodeId, SourceSpan};
use super::types::CType;
use super::z80::{
    AluSrc, AsmInstructionId, Cc, GeneratedFunction, GeneratedGlobal, GeneratedProgram,
    MappedInstruction, R8, RegHome, Rr, Z80Item, Z80Op, asm_block_label, asm_global_label,
    asm_label, render_items,
};
use crate::asm::assemble_program;
use crate::disasm::disassemble_at;
use std::collections::{HashMap, HashSet};

pub const DEFAULT_CODE_ORIGIN: u16 = 0x8000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Loc {
    Byte(R8),
    Word(Rr),
    Imm8(u8),
    Imm16(u16),
    Frame { disp: i8, width: u8 },
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
            Self::Imm8(_) | Self::Imm16(_) | Self::Frame { .. } => &[],
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
            Self::Frame { width, .. } => width,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    vreg_ty: HashMap<VReg, CType>,
    local_ty: HashMap<LocalId, CType>,
    op_index: usize,
    func_label: String,
    globals: HashMap<GlobalId, (String, CType)>,
    callees: &'a HashMap<FuncId, CalleeInfo>,
    next_aux: u32,
    stable: bool,
    uses_ix: bool,
    frame_used: u8,
    spill_bytes: u8,
    call_sites: Vec<(FuncId, u16)>,
    param_moves: Vec<(Loc, Loc)>,
}

struct CalleeInfo {
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
) -> Option<GeneratedProgram> {
    let global_map = global_symbols(program);
    let callee_map = callee_map(program);
    let mut order: Vec<OrderItem<'_>> = program.globals.iter().map(OrderItem::Global).collect();
    order.extend(program.functions.iter().map(OrderItem::Func));
    order.sort_by_key(|item| item.span_key());

    let mut chunks: Vec<EmitChunk> = Vec::new();
    let mut failed = false;
    for item in order {
        match item {
            OrderItem::Global(global) => match lower_global(global, ids) {
                Ok(part) => chunks.push(EmitChunk::Global(part)),
                Err(diag) => {
                    diagnostics.push(diag);
                    failed = true;
                }
            },
            OrderItem::Func(func) => match lower_function(func, &callee_map, &global_map, ids) {
                Ok(part) => chunks.push(EmitChunk::Func(part)),
                Err(diag) => {
                    diagnostics.push(diag);
                    failed = true;
                }
            },
        }
    }

    if failed {
        return None;
    }
    fill_stack_bounds(&mut chunks);
    if chunks.is_empty() {
        return None;
    }

    let mut items = Vec::new();
    for chunk in &chunks {
        items.extend(chunk.items().iter().cloned());
    }
    let assembly = render_items(&items);
    let assembled = match assemble_program(&assembly, origin) {
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

    let emitting: Vec<&Z80Item> = items.iter().filter(|i| i.emits_bytes()).collect();
    if emitting.len() != assembled.lines.len() {
        diagnostics.push(Diagnostic::error(
            DiagCode::CgInternal,
            chunks[0].span(),
            format!(
                "assembled line count {} does not match generated items {}",
                assembled.lines.len(),
                emitting.len()
            ),
        ));
        return None;
    }

    let mut mapped_all = Vec::new();
    for (item, line) in emitting.into_iter().zip(assembled.lines.iter()) {
        let start = line.addr.wrapping_sub(assembled.origin) as usize;
        let bytes = assembled.bytes[start..start + line.len].to_vec();
        let mut bus = crate::bus::FakeBus::new();
        for (i, b) in bytes.iter().enumerate() {
            bus.mem[line.addr.wrapping_add(i as u16) as usize] = *b;
        }
        let (id, text, span, node, t_states) = match item {
            Z80Item::Instruction { id, op, span, node } => {
                let d = disassemble_at(&mut bus, line.addr);
                (*id, op.render(), *span, *node, d.t_states)
            }
            Z80Item::Data {
                id,
                text,
                span,
                node,
                ..
            } => (*id, text.clone(), *span, Some(*node), None),
            Z80Item::Label { .. } => continue,
        };
        mapped_all.push(MappedInstruction {
            id,
            address: line.addr,
            bytes,
            text,
            t_states,
            span,
            node,
        });
    }

    let mut functions = Vec::new();
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
                            .any(|item| item.is_instruction() && item.id() == m.id)
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
                    param_homes: part.param_homes,
                    param_types: part.param_types,
                    ret: part.ret,
                    stack_bound: part.stack_bound,
                    frame_bytes: part.frame_bytes,
                    instruction_ids: code_mapped.iter().map(|m| m.id).collect(),
                    mapped: code_mapped,
                });
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

enum EmitChunk {
    Func(FnPart),
    Global(GlobalPart),
}

impl EmitChunk {
    fn items(&self) -> &[Z80Item] {
        match self {
            Self::Func(p) => &p.items,
            Self::Global(p) => &p.items,
        }
    }

    fn span(&self) -> SourceSpan {
        match self {
            Self::Func(p) => p.span,
            Self::Global(p) => p.span,
        }
    }
}

struct FnPart {
    name: String,
    label: String,
    id: FuncId,
    span: SourceSpan,
    items: Vec<Z80Item>,
    conv: CallConv,
    param_homes: Vec<RegHome>,
    param_types: Vec<CType>,
    ret: Option<RegHome>,
    stack_bound: u16,
    frame_bytes: u16,
    call_sites: Vec<(FuncId, u16)>,
}

struct GlobalPart {
    name: String,
    label: String,
    ty: CType,
    span: SourceSpan,
    items: Vec<Z80Item>,
}

fn callee_map(program: &TypedProgram) -> HashMap<FuncId, CalleeInfo> {
    program
        .functions
        .iter()
        .map(|f| {
            (
                f.id,
                CalleeInfo {
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

fn fill_stack_bounds(chunks: &mut [EmitChunk]) {
    let mut ids: HashMap<FuncId, usize> = HashMap::new();
    for (i, chunk) in chunks.iter().enumerate() {
        if let EmitChunk::Func(part) = chunk {
            ids.insert(part.id, i);
        }
    }
    let mut memo: HashMap<FuncId, u16> = HashMap::new();
    fn bound_of(
        chunks: &[EmitChunk],
        ids: &HashMap<FuncId, usize>,
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
        let sites = part.call_sites.clone();
        let frame = part.frame_bytes;
        for (callee, arg_slots) in sites {
            extra = extra.max(
                arg_slots
                    .saturating_add(2)
                    .saturating_add(bound_of(chunks, ids, memo, callee)),
            );
        }
        let bound = frame.saturating_add(extra);
        memo.insert(id, bound);
        bound
    }
    let computed: Vec<(usize, u16)> = chunks
        .iter()
        .enumerate()
        .filter_map(|(i, chunk)| match chunk {
            EmitChunk::Func(part) => Some((i, bound_of(chunks, &ids, &mut memo, part.id))),
            _ => None,
        })
        .collect();
    for (i, bound) in computed {
        if let EmitChunk::Func(part) = &mut chunks[i] {
            part.stack_bound = bound;
        }
    }
}

fn lower_global(global: &TypedGlobal, ids: &mut IdGen) -> Result<GlobalPart, Diagnostic> {
    let label = asm_global_label(global.id, &global.name);
    let bits = global.init.unwrap_or(0);
    let text = match global.ty.byte_width() {
        Some(1) => format!("DB {}", bits as u8),
        Some(2) => format!("DW {bits}"),
        _ => {
            return Err(Diagnostic::error(
                DiagCode::CgUnsupported,
                global.span,
                format!("cannot emit global '{}'", global.name),
            ));
        }
    };
    let mut items = Vec::new();
    items.push(Z80Item::Label {
        id: AsmInstructionId(ids.next().0),
        name: label.clone(),
        span: global.span,
        node: global.id.0,
    });
    items.push(Z80Item::Data {
        id: AsmInstructionId(ids.next().0),
        text,
        span: global.span,
        node: global.id.0,
    });
    Ok(GlobalPart {
        name: global.name.clone(),
        label,
        ty: global.ty,
        span: global.span,
        items,
    })
}

fn lower_function(
    func: &TypedFunction,
    callees: &HashMap<FuncId, CalleeInfo>,
    globals: &HashMap<GlobalId, (String, CType)>,
    ids: &mut IdGen,
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
    let mut local_ty = HashMap::new();
    for p in &func.params {
        local_ty.insert(p.id, p.ty);
    }
    for l in &func.locals {
        local_ty.insert(l.id, l.ty);
    }
    let label = asm_label(func.id, &func.name);
    let live_across = live_across_any_call(func, &vreg_last, &local_last);
    let spill_bytes = max_call_spill_bytes(func, &vreg_last, &local_last);
    let pressure = !fits_stable_registers(func, func.conv);
    let uses_ix = func.conv == CallConv::Stack || live_across || spill_bytes > 0 || pressure;
    let stable = func.blocks.len() > 1 || uses_ix;
    let mut lowerer = Lowerer {
        func,
        ids,
        items: Vec::new(),
        vreg_loc: HashMap::new(),
        local_loc: HashMap::new(),
        vreg_last,
        local_last,
        vreg_ty: HashMap::new(),
        local_ty,
        op_index: 0,
        func_label: label.clone(),
        globals: globals.clone(),
        callees,
        next_aux: 0,
        stable,
        uses_ix,
        frame_used: 0,
        spill_bytes,
        call_sites: Vec::new(),
        param_moves: Vec::new(),
    };
    lowerer.emit_label(&label, func.span, func.id.0);
    lowerer.plan_homes(&param_homes)?;
    if uses_ix {
        lowerer.emit_prologue(func.span)?;
    }
    lowerer.emit_param_setup(&param_homes, func.span)?;

    let mut index = 0usize;
    for block in &func.blocks {
        lowerer.emit_label(&asm_block_label(&label, block.id), func.span, func.id.0);
        lowerer.vreg_loc.clear();
        let ops = &block.ops;
        let mut i = 0;
        while i < ops.len() {
            lowerer.op_index = index;
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
                        lowerer.lower_compare_branch(
                            *ty, *op, *lhs, *rhs, *true_blk, *false_blk, *span,
                        )?;
                        index += 2;
                        i += 2;
                        continue;
                    }
                }
            }
            lowerer.lower_op(&ops[i])?;
            lowerer.drop_dead_after(index);
            index += 1;
            i += 1;
        }
        if needs_fallthrough(ops) {
            if let Some(next) = next_block(func, block.id) {
                lowerer.emit(
                    Z80Op::Jp {
                        cc: None,
                        target: asm_block_label(&label, next),
                    },
                    func.span,
                );
            } else {
                lowerer.emit_epilogue(func.span);
            }
        }
    }

    let frame_bytes = if uses_ix {
        2u16.saturating_add(u16::from(lowerer.frame_used))
    } else {
        0
    };
    Ok(FnPart {
        name: func.name.clone(),
        label,
        id: func.id,
        span: func.span,
        items: lowerer.items,
        conv: func.conv,
        param_homes,
        param_types: param_tys,
        ret: return_home(func.ret),
        stack_bound: 0,
        frame_bytes,
        call_sites: lowerer.call_sites,
    })
}

fn fits_stable_registers(func: &TypedFunction, conv: CallConv) -> bool {
    let mut byte_i = 0usize;
    let mut word_i = 0usize;
    const BYTES: usize = 6;
    const WORDS: usize = 2;
    let params: Box<dyn Iterator<Item = CType>> = if conv == CallConv::Stack {
        Box::new(std::iter::empty())
    } else {
        Box::new(func.params.iter().map(|p| p.ty))
    };
    for ty in params.chain(func.locals.iter().map(|l| l.ty)) {
        match ty.byte_width() {
            Some(1) => {
                if byte_i >= BYTES {
                    return false;
                }
                byte_i += 1;
            }
            Some(2) => {
                if word_i >= WORDS {
                    return false;
                }
                word_i += 1;
            }
            _ => {}
        }
    }
    true
}

fn live_across_any_call(
    func: &TypedFunction,
    vreg_last: &HashMap<VReg, usize>,
    local_last: &HashMap<LocalId, usize>,
) -> bool {
    let mut assigned: HashSet<LocalId> = func.params.iter().map(|p| p.id).collect();
    let mut i = 0usize;
    for block in &func.blocks {
        for op in &block.ops {
            if let IrOp::Call { args, dst, .. } = op {
                for id in &assigned {
                    if local_last.get(id).is_some_and(|&last| last > i) {
                        return true;
                    }
                }
                for (&v, &last) in vreg_last {
                    if last > i && Some(v) != *dst {
                        let dying = args.contains(&v) && last == i;
                        if !dying {
                            return true;
                        }
                    }
                }
            }
            if let IrOp::StoreLocal { local, .. } = op {
                assigned.insert(*local);
            }
            i += 1;
        }
    }
    false
}

fn locals_needing_frame(
    func: &TypedFunction,
    local_last: &HashMap<LocalId, usize>,
) -> HashSet<LocalId> {
    let mut assigned: HashSet<LocalId> = func.params.iter().map(|p| p.id).collect();
    let mut need = HashSet::new();
    let mut i = 0usize;
    for block in &func.blocks {
        for op in &block.ops {
            if matches!(op, IrOp::Call { .. }) {
                for id in &assigned {
                    if local_last.get(id).is_some_and(|&last| last > i) {
                        need.insert(*id);
                    }
                }
            }
            if let IrOp::StoreLocal { local, .. } = op {
                assigned.insert(*local);
            }
            i += 1;
        }
    }
    need
}

fn max_call_spill_bytes(
    func: &TypedFunction,
    vreg_last: &HashMap<VReg, usize>,
    _local_last: &HashMap<LocalId, usize>,
) -> u8 {
    let mut i = 0usize;
    let mut max = 0u8;
    for block in &func.blocks {
        for op in &block.ops {
            if let IrOp::Call { args, dst, .. } = op {
                let mut bytes = 0u8;
                for (&v, &last) in vreg_last {
                    if last > i && Some(v) != *dst {
                        let dies_here = args.contains(&v) && last == i;
                        if !dies_here {
                            bytes = bytes.saturating_add(2);
                        }
                    }
                }
                max = max.max(bytes);
            }
            i += 1;
        }
    }
    max
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
                IrOp::Branch { cond, .. } if *cond == v => 1,
                IrOp::Return {
                    value: Some(src), ..
                } if *src == v => 1,
                IrOp::Call { dst, args, .. } => {
                    usize::from(dst.as_ref() == Some(&v)) + args.iter().filter(|a| **a == v).count()
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
                IrOp::LoadGlobal { dst, .. } => {
                    vreg_last.insert(*dst, i);
                }
                IrOp::StoreGlobal { src, .. } => {
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
                _ => {}
            }
            i += 1;
        }
    }
    (vreg_last, local_last)
}

impl Lowerer<'_> {
    fn error(&self, span: SourceSpan, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(DiagCode::CgUnsupported, span, message)
    }

    fn pressure(&self, span: SourceSpan) -> Diagnostic {
        self.error(
            span,
            format!(
                "register pressure in '{}' needs a larger IX frame or @stackcall",
                self.func.name
            ),
        )
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
        let id = self.next_id();
        self.items.push(Z80Item::Instruction {
            id,
            op,
            span,
            node: Some(self.func.id.0),
        });
    }

    fn vreg_live(&self, v: VReg) -> bool {
        self.vreg_last
            .get(&v)
            .is_some_and(|&last| last > self.op_index)
    }

    fn drop_dead_after(&mut self, index: usize) {
        self.vreg_loc
            .retain(|v, _| self.vreg_last.get(v).is_some_and(|&last| last > index));
        if !self.stable {
            self.local_loc
                .retain(|l, _| self.local_last.get(l).is_some_and(|&last| last > index));
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

    fn reg_free(&self, r: R8, ignore: Option<Key>) -> bool {
        self.occupants(r, ignore).is_empty()
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
                if !seen.contains(&key) {
                    seen.push(key);
                }
            }
        }
        for key in seen {
            self.relocate(key, regs, span)?;
        }
        Ok(())
    }

    fn relocate(&mut self, key: Key, avoid: &[R8], span: SourceSpan) -> Result<(), Diagnostic> {
        let loc = match key {
            Key::V(v) => *self.vreg_loc.get(&v).ok_or_else(|| self.pressure(span))?,
            Key::L(l) => *self.local_loc.get(&l).ok_or_else(|| self.pressure(span))?,
        };
        let ignore = Some(key);
        let new_loc = match loc {
            Loc::Byte(_) => {
                let r = self
                    .alloc_r8(None, ignore)
                    .filter(|r| !avoid.contains(r))
                    .or_else(|| {
                        R8::ALL
                            .into_iter()
                            .find(|&r| !avoid.contains(&r) && self.reg_free(r, ignore))
                    })
                    .ok_or_else(|| self.pressure(span))?;
                Loc::Byte(r)
            }
            Loc::Word(_) => {
                let rr = self
                    .alloc_rr(None, ignore)
                    .filter(|rr| {
                        let (h, l) = rr.halves();
                        !avoid.contains(&h) && !avoid.contains(&l)
                    })
                    .ok_or_else(|| self.pressure(span))?;
                Loc::Word(rr)
            }
            Loc::Imm8(_) | Loc::Imm16(_) | Loc::Frame { .. } => return Ok(()),
        };
        self.emit_move(loc, new_loc, span)?;
        match key {
            Key::V(v) => {
                self.vreg_loc.insert(v, new_loc);
            }
            Key::L(l) => {
                self.local_loc.insert(l, new_loc);
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
                let (_h, l) = d.halves();
                self.emit(Z80Op::Ld8 { dst: l, src: s }, span);
                self.evict_regs(&[d.halves().0], None, span)?;
                self.emit(
                    Z80Op::Ld8Imm {
                        dst: d.halves().0,
                        imm: 0,
                    },
                    span,
                );
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
            Loc::Imm16(_) | Loc::Word(_) | Loc::Frame { .. } => {
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

    fn alu_src_of(&mut self, v: VReg, span: SourceSpan) -> Result<AluSrc, Diagnostic> {
        match self.loc_of(v, span)? {
            Loc::Imm8(n) => Ok(AluSrc::Imm(n)),
            Loc::Byte(r) => {
                if r == R8::A {
                    let tmp = self
                        .alloc_r8(Some(R8::C), Some(Key::V(v)))
                        .ok_or_else(|| self.pressure(span))?;
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
            Loc::Frame { .. } => {
                let tmp = self
                    .alloc_r8(Some(R8::C), Some(Key::V(v)))
                    .ok_or_else(|| self.pressure(span))?;
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
                let dst = if self.pair_free(Rr::De, Some(Key::V(v))) {
                    Rr::De
                } else if self.pair_free(Rr::Bc, Some(Key::V(v))) {
                    Rr::Bc
                } else {
                    return Err(self.pressure(span));
                };
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
                let src_loc = self.loc_of(*src, *span)?;
                if let Some(&home) = self.local_loc.get(local) {
                    self.emit_move(src_loc, home, *span)?;
                    return Ok(());
                }
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
                        let r = self
                            .alloc_r8(Some(r), Some(Key::V(*src)))
                            .ok_or_else(|| self.pressure(*span))?;
                        self.claim_r8(r, Some(Key::V(*src)), *span)?;
                        Loc::Byte(r)
                    }
                    Some(RegHome::Word(rr)) => {
                        let rr = self
                            .alloc_rr(Some(rr), Some(Key::V(*src)))
                            .ok_or_else(|| self.pressure(*span))?;
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
            IrOp::StoreGlobal { global, src, span } => {
                self.lower_store_global(*global, *src, *span)?
            }
            IrOp::Call {
                dst,
                func,
                args,
                span,
            } => self.lower_call(*dst, *func, args, *span)?,
        }
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
                self.ensure_a(src, span)?;
                self.emit(Z80Op::Xor(AluSrc::Imm(1)), span);
                self.bind_vreg(dst, Loc::Byte(R8::A), ty);
            }
            IrUnary::BitNot => match ty.byte_width() {
                Some(1) => {
                    self.ensure_a(src, span)?;
                    self.emit(Z80Op::Cpl, span);
                    self.bind_vreg(dst, Loc::Byte(R8::A), ty);
                }
                Some(2) => {
                    self.ensure_hl(src, span)?;
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
                    self.ensure_a(src, span)?;
                    self.emit(Z80Op::Neg, span);
                    self.bind_vreg(dst, Loc::Byte(R8::A), ty);
                }
                Some(2) => {
                    self.ensure_hl(src, span)?;
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
        if matches!(self.loc_of(rhs, span)?, Loc::Byte(R8::A)) && lhs != rhs {
            if !matches!(self.loc_of(lhs, span)?, Loc::Byte(R8::A)) {
                let tmp = self
                    .alloc_r8(Some(R8::C), Some(Key::V(rhs)))
                    .ok_or_else(|| self.pressure(span))?;
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
        self.ensure_a(lhs, span)?;
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
        match op {
            IrBinary::Add => {
                if matches!(self.loc_of(lhs, span)?, Loc::Word(Rr::De))
                    && matches!(self.loc_of(rhs, span)?, Loc::Word(Rr::Hl))
                {
                    self.ensure_hl(rhs, span)?;
                    self.ensure_in_rr(lhs, Rr::De, span)?;
                    self.emit(Z80Op::AddHl(Rr::De), span);
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
                    return Ok(());
                }
                self.ensure_hl(lhs, span)?;
                let src = self.word_src_rr(rhs, span)?;
                self.emit(Z80Op::AddHl(src), span);
                self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            }
            IrBinary::Sub => {
                self.ensure_hl(lhs, span)?;
                let src = self.word_src_rr(rhs, span)?;
                self.claim_r8(R8::A, Some(Key::V(lhs)), span)?;
                self.emit(Z80Op::Or(AluSrc::Reg(R8::A)), span);
                self.emit(Z80Op::SbcHl(src), span);
                self.bind_vreg(dst, Loc::Word(Rr::Hl), ty);
            }
            IrBinary::BitAnd | IrBinary::BitXor | IrBinary::BitOr => {
                self.ensure_hl(lhs, span)?;
                let src = self.word_src_rr(rhs, span)?;
                let (sh, sl) = src.halves();
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
                self.emit(alu(AluSrc::Reg(sh)), span);
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
                self.emit(alu(AluSrc::Reg(sl)), span);
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
                    self.ensure_a(src, span)?;
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
                self.ensure_a(src, span)?;
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
                Loc::Byte(r) => {
                    self.claim_rr(Rr::Hl, Some(Key::V(src)), span)?;
                    if r != R8::L {
                        self.emit(Z80Op::Ld8 { dst: R8::L, src: r }, span);
                    }
                    self.emit(Z80Op::Ld8Imm { dst: R8::H, imm: 0 }, span);
                    self.bind_vreg(dst, Loc::Word(Rr::Hl), to);
                }
                Loc::Word(rr) => self.bind_vreg(dst, Loc::Word(rr), to),
                Loc::Imm16(n) => self.bind_vreg(dst, Loc::Imm16(n), to),
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

    fn assign_stable_homes(&mut self, param_homes: &[RegHome]) -> Result<(), Diagnostic> {
        let mut byte_i = 0usize;
        let mut word_i = 0usize;
        const BYTES: [R8; 6] = [R8::C, R8::B, R8::E, R8::D, R8::L, R8::H];
        const WORDS: [Rr; 2] = [Rr::De, Rr::Bc];
        let span = self.func.span;
        let mut pending = Vec::new();
        for (param, abi) in self.func.params.iter().zip(param_homes.iter()) {
            let home = match param.ty.byte_width() {
                Some(1) => {
                    let r = *BYTES.get(byte_i).ok_or_else(|| self.pressure(span))?;
                    byte_i += 1;
                    Loc::Byte(r)
                }
                Some(2) => {
                    let rr = *WORDS.get(word_i).ok_or_else(|| self.pressure(span))?;
                    word_i += 1;
                    Loc::Word(rr)
                }
                _ => return Err(self.error(span, "void parameter")),
            };
            pending.push((Loc::from_home(*abi), home, param.id));
        }
        while !pending.is_empty() {
            let i = pending
                .iter()
                .position(|(_, dst, _)| {
                    !pending
                        .iter()
                        .any(|(src, _, _)| dst.regs().iter().any(|r| src.regs().contains(r)))
                })
                .unwrap_or(0);
            let (src, dst, id) = pending.remove(i);
            self.emit_move(src, dst, span)?;
            if self.local_last.contains_key(&id) {
                self.local_loc.insert(id, dst);
            }
        }
        for local in &self.func.locals {
            if self.local_loc.contains_key(&local.id) {
                continue;
            }
            let home = match local.ty.byte_width() {
                Some(1) => {
                    let r = *BYTES.get(byte_i).ok_or_else(|| self.pressure(local.span))?;
                    byte_i += 1;
                    Loc::Byte(r)
                }
                Some(2) => {
                    let rr = *WORDS.get(word_i).ok_or_else(|| self.pressure(local.span))?;
                    word_i += 1;
                    Loc::Word(rr)
                }
                _ => return Err(self.error(local.span, "void local")),
            };
            self.local_loc.insert(local.id, home);
        }
        Ok(())
    }

    fn plan_homes(&mut self, param_homes: &[RegHome]) -> Result<(), Diagnostic> {
        if !self.uses_ix {
            if self.stable {
                self.assign_stable_homes(param_homes)?;
            } else {
                for (param, home) in self.func.params.iter().zip(param_homes.iter()) {
                    if self.local_last.contains_key(&param.id) {
                        self.local_loc.insert(param.id, Loc::from_home(*home));
                    }
                }
            }
            return Ok(());
        }
        let need_frame = locals_needing_frame(self.func, &self.local_last);
        const BYTES: [R8; 6] = [R8::C, R8::B, R8::E, R8::D, R8::L, R8::H];
        const WORDS: [Rr; 2] = [Rr::De, Rr::Bc];
        let mut byte_i = 0usize;
        let mut word_i = 0usize;
        let span = self.func.span;
        if self.func.conv == CallConv::Stack {
            for (i, param) in self.func.params.iter().enumerate() {
                let disp = 4i16 + 2 * i as i16;
                let hi = disp + 1;
                let disp = i8::try_from(disp).map_err(|_| self.frame_error(param.span))?;
                let _ = i8::try_from(hi).map_err(|_| self.frame_error(param.span))?;
                if self.local_last.contains_key(&param.id) {
                    self.local_loc
                        .insert(param.id, Loc::Frame { disp, width: 2 });
                }
            }
        } else {
            for (param, abi) in self.func.params.iter().zip(param_homes.iter()) {
                let width = param.ty.byte_width().unwrap_or(1);
                let home = if need_frame.contains(&param.id) {
                    Loc::Frame {
                        disp: self.alloc_neg(width, param.span)?,
                        width,
                    }
                } else {
                    self.take_reg_home(width, &mut byte_i, &mut word_i, &BYTES, &WORDS, param.span)?
                };
                self.param_moves.push((Loc::from_home(*abi), home));
                if self.local_last.contains_key(&param.id) {
                    self.local_loc.insert(param.id, home);
                }
            }
        }
        for local in &self.func.locals {
            if self.local_loc.contains_key(&local.id) {
                continue;
            }
            let width = local.ty.byte_width().unwrap_or(1);
            let home = if need_frame.contains(&local.id) {
                Loc::Frame {
                    disp: self.alloc_neg(width, local.span)?,
                    width,
                }
            } else {
                match self.try_reg_home(width, byte_i, word_i, &BYTES, &WORDS) {
                    Some((home, b, w)) => {
                        byte_i = b;
                        word_i = w;
                        home
                    }
                    None => Loc::Frame {
                        disp: self.alloc_neg(width, local.span)?,
                        width,
                    },
                }
            };
            self.local_loc.insert(local.id, home);
        }
        let spill = self.spill_bytes;
        if spill > 0 {
            let _ = self.alloc_neg(spill, span)?;
        }
        Ok(())
    }

    fn try_reg_home(
        &self,
        width: u8,
        byte_i: usize,
        word_i: usize,
        bytes: &[R8; 6],
        words: &[Rr; 2],
    ) -> Option<(Loc, usize, usize)> {
        match width {
            1 if byte_i < bytes.len() => Some((Loc::Byte(bytes[byte_i]), byte_i + 1, word_i)),
            2 if word_i < words.len() => Some((Loc::Word(words[word_i]), byte_i, word_i + 1)),
            _ => None,
        }
    }

    fn take_reg_home(
        &mut self,
        width: u8,
        byte_i: &mut usize,
        word_i: &mut usize,
        bytes: &[R8; 6],
        words: &[Rr; 2],
        span: SourceSpan,
    ) -> Result<Loc, Diagnostic> {
        match self.try_reg_home(width, *byte_i, *word_i, bytes, words) {
            Some((home, b, w)) => {
                *byte_i = b;
                *word_i = w;
                Ok(home)
            }
            None => Ok(Loc::Frame {
                disp: self.alloc_neg(width, span)?,
                width,
            }),
        }
    }

    fn alloc_neg(&mut self, width: u8, span: SourceSpan) -> Result<i8, Diagnostic> {
        let next = u16::from(self.frame_used).saturating_add(u16::from(width));
        if next > 128 {
            return Err(self.frame_error(span));
        }
        self.frame_used = next as u8;
        i8::try_from(-(next as i16)).map_err(|_| self.frame_error(span))
    }

    fn emit_param_setup(
        &mut self,
        _param_homes: &[RegHome],
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let moves = self.param_moves.clone();
        self.emit_parallel_moves(moves, span)
    }

    fn emit_prologue(&mut self, span: SourceSpan) -> Result<(), Diagnostic> {
        self.emit(Z80Op::PushIx, span);
        self.emit(Z80Op::LdIxImm(0), span);
        self.emit(Z80Op::AddIxSp, span);
        if self.frame_used > 0 {
            let n = u16::from(self.frame_used);
            self.emit(
                Z80Op::Ld16Imm {
                    dst: Rr::Hl,
                    imm: n.wrapping_neg(),
                },
                span,
            );
            self.emit(Z80Op::AddHlSp, span);
            self.emit(Z80Op::LdSpHl, span);
        }
        Ok(())
    }

    fn emit_epilogue(&mut self, span: SourceSpan) {
        if self.uses_ix {
            self.emit(Z80Op::LdSpIx, span);
            self.emit(Z80Op::PopIx, span);
        }
        self.emit(Z80Op::Ret, span);
    }

    fn emit_parallel_moves(
        &mut self,
        mut pending: Vec<(Loc, Loc)>,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        pending.retain(|(s, d)| s != d);
        while !pending.is_empty() {
            if let Some(i) = pending.iter().position(|(_, dst)| {
                !pending
                    .iter()
                    .any(|(src, _)| dst.regs().iter().any(|r| src.regs().contains(r)))
            }) {
                let (src, dst) = pending.remove(i);
                self.emit_move(src, dst, span)?;
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
                    self.emit_parallel_moves(pending, span)?;
                    self.emit(Z80Op::Pop(d), span);
                    return Ok(());
                }
                (src, dst) => {
                    self.emit_move(src, Loc::Byte(R8::A), span)?;
                    self.emit(Z80Op::PushAf, span);
                    self.emit_parallel_moves(pending, span)?;
                    self.emit(Z80Op::PopAf, span);
                    self.emit_move(Loc::Byte(R8::A), dst, span)?;
                    return Ok(());
                }
            }
        }
        Ok(())
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
        let conv = callee.conv;
        let params = callee.params.clone();
        let ret = callee.ret;
        if args.len() != params.len() {
            return Err(self.error(span, "call argument count mismatch"));
        }

        let mut next_spill = if self.spill_bytes > 0 {
            Some(-(self.frame_used as i8))
        } else {
            None
        };
        let mut live_keys = Vec::new();
        for (&v, loc) in &self.vreg_loc.clone() {
            if Some(v) == dst {
                continue;
            }
            let last = self.vreg_last.get(&v).copied().unwrap_or(0);
            let dying = args.contains(&v) && last == self.op_index;
            if last > self.op_index
                && !dying
                && !matches!(loc, Loc::Imm8(_) | Loc::Imm16(_) | Loc::Frame { .. })
            {
                live_keys.push(Key::V(v));
            }
        }
        for (&l, loc) in &self.local_loc.clone() {
            let last = self.local_last.get(&l).copied().unwrap_or(0);
            if last > self.op_index
                && !matches!(loc, Loc::Frame { .. } | Loc::Imm8(_) | Loc::Imm16(_))
            {
                live_keys.push(Key::L(l));
            }
        }
        for key in live_keys {
            let loc = match key {
                Key::V(v) => self.vreg_loc[&v],
                Key::L(l) => self.local_loc[&l],
            };
            let width = loc.width().max(1);
            let disp = next_spill.ok_or_else(|| self.pressure(span))?;
            let hi = (disp as i16) + i16::from(width) - 1;
            if i8::try_from(hi).is_err() {
                return Err(self.frame_error(span));
            }
            let slot = Loc::Frame { disp, width };
            self.emit_move(loc, slot, span)?;
            match key {
                Key::V(v) => {
                    self.vreg_loc.insert(v, slot);
                }
                Key::L(l) => {
                    self.local_loc.insert(l, slot);
                }
            }
            next_spill = Some(disp.saturating_add(width as i8));
        }

        let arg_slots = if conv == CallConv::Stack {
            2u16.saturating_mul(args.len() as u16)
        } else {
            0
        };
        self.call_sites.push((func, arg_slots));

        if conv == CallConv::Stack {
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
            self.emit_parallel_moves(moves, span)?;
        }

        self.emit(Z80Op::Call { target: label }, span);

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
                let rr = self
                    .push_pair_free(remaining)
                    .ok_or_else(|| self.pressure(span))?;
                self.ensure_in_rr(arg, rr, span)?;
                self.emit(Z80Op::Push(rr), span);
            }
            (Some(1), _) => {
                let rr = self
                    .push_pair_free(remaining)
                    .ok_or_else(|| self.pressure(span))?;
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
        Ok(())
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
        self.emit(
            Z80Op::Jp {
                cc: None,
                target: self.block_target(blk),
            },
            span,
        );
        Ok(())
    }

    fn lower_branch(
        &mut self,
        cond: VReg,
        true_blk: BlockId,
        false_blk: BlockId,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
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
        self.emit_compare_flags(lhs, rhs, span)?;
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
        self.emit_compare_flags(lhs, rhs, span)?;
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

    fn emit_compare_flags(
        &mut self,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let ty = self
            .vreg_ty
            .get(&lhs)
            .copied()
            .ok_or_else(|| self.error(span, "compare operand has no type"))?;
        let width = ty.byte_width().unwrap_or(1);
        let signed = ty.is_signed();
        if width == 1 {
            if signed {
                match self.loc_of(rhs, span)? {
                    Loc::Imm8(n) => {
                        self.ensure_a(lhs, span)?;
                        self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                        self.emit(Z80Op::Cp(AluSrc::Imm(n ^ 0x80)), span);
                    }
                    _ => {
                        self.ensure_a(lhs, span)?;
                        self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                        let tmp = self
                            .alloc_r8(Some(R8::B), Some(Key::V(lhs)))
                            .ok_or_else(|| self.pressure(span))?;
                        self.claim_r8(tmp, Some(Key::V(lhs)), span)?;
                        self.emit(
                            Z80Op::Ld8 {
                                dst: tmp,
                                src: R8::A,
                            },
                            span,
                        );
                        self.ensure_a(rhs, span)?;
                        self.emit(Z80Op::Xor(AluSrc::Imm(0x80)), span);
                        let rhs_r = self
                            .alloc_r8(Some(R8::C), Some(Key::V(rhs)))
                            .ok_or_else(|| self.pressure(span))?;
                        self.claim_r8(rhs_r, Some(Key::V(rhs)), span)?;
                        self.emit(
                            Z80Op::Ld8 {
                                dst: rhs_r,
                                src: R8::A,
                            },
                            span,
                        );
                        self.emit(
                            Z80Op::Ld8 {
                                dst: R8::A,
                                src: tmp,
                            },
                            span,
                        );
                        self.emit(Z80Op::Cp(AluSrc::Reg(rhs_r)), span);
                    }
                }
            } else {
                self.ensure_a(lhs, span)?;
                let src = self.alu_src_of(rhs, span)?;
                self.emit(Z80Op::Cp(src), span);
            }
        } else {
            self.ensure_hl(lhs, span)?;
            let src = self.word_src_rr(rhs, span)?;
            if signed {
                self.claim_r8(R8::A, Some(Key::V(lhs)), span)?;
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
                let (hh, _) = src.halves();
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
            }
            self.claim_r8(R8::A, Some(Key::V(lhs)), span)?;
            self.emit(Z80Op::Or(AluSrc::Reg(R8::A)), span);
            self.emit(Z80Op::SbcHl(src), span);
        }
        Ok(())
    }
}
