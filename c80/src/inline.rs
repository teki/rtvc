//! Inline expansion (H3i, F-031): every C80 call to an `@inline` function is
//! replaced by a copy of the callee's IR, before lowering. Optimized builds
//! may also expand calls automatically (see [`auto_inline`]).
//!
//! An expansion is IR the caller could have written itself:
//! - each parameter becomes a fresh caller local stored from the argument
//!   (arguments are already evaluated once, left to right, before the call
//!   op); a parameter the body never writes reads the argument directly, as
//!   an immediate when the argument is a constant;
//! - a single-block callee is spliced into the calling block; otherwise the
//!   calling block is split, the callee's blocks are cloned with fresh ids,
//!   each `return` stores a fresh result local and jumps to the
//!   continuation, and values the caller computed before the call and uses
//!   after it cross the new block boundary through fresh locals (vregs never
//!   cross blocks);
//! - cloned ops keep their source spans; `IrBlock::origin` records the call
//!   chain they came through, for maps, listings, and diagnostics;
//! - `static` locals are globals, so every expansion and the out-of-line
//!   copy share one storage location;
//! - each cloned inline `asm` block gets a fresh node id, so its local
//!   labels get a fresh namespace.

use super::ast::AsmGpr;
use super::diagnostic::{DiagCode, Diagnostic, RelatedSpan};
use super::inline_asm::gpr_mask;
use super::ir::*;
use super::source::{IdGen, SourceSpan};
use super::types::CType;
use std::collections::{HashMap, HashSet};

/// Expand every call to an `@inline` function, callees first. Errors are
/// located at the call site.
pub(crate) fn expand_inline_calls(
    program: &mut TypedProgram,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let mut ok = true;
    for fid in callee_first_ids(program) {
        loop {
            let Some(fi) = program.functions.iter().position(|f| f.id == fid) else {
                break;
            };
            let site = find_call(&program.functions[fi], |callee| {
                program
                    .functions
                    .iter()
                    .any(|f| f.id == callee && f.inline.is_some())
            });
            let Some((bi, k)) = site else {
                break;
            };
            match expand_at(program, fi, bi, k, false, ids) {
                Ok(func) => program.functions[fi] = func,
                Err(diag) => {
                    diagnostics.push(diag);
                    ok = false;
                    break;
                }
            }
        }
    }
    ok
}

/// Size and fallback cost of one lowered function, as measured by a trial
/// lowering (optimized, assembled alone).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Measure {
    pub bytes: u32,
    /// Static slot bytes plus bytes pushed by saves: any new spill rejects
    /// an automatic expansion.
    pub spill: u32,
}

/// Automatic inlining (optimized builds only). Candidates, in callee-first
/// caller order and op order within a caller:
/// - a private, non-`@stackcall` callee whose body has no inline `asm`
///   containing `CALL`/`RST` and is not `@inline` (those are already gone);
/// - rule A: the callee is private and has exactly one C80 call site;
/// - rule B: any such callee, when the caller's lowered size does not grow.
///
/// Each candidate is expanded on a copy and both versions are lowered with
/// `measure`; it is kept only when the caller gets no new spill and either
/// does not grow (rule B) or the callee is rule-A (one site, so the call's
/// `CALL`/`RET` and argument moves are saved on every execution) and the
/// caller grows by no more than the callee's own body. Deterministic: the
/// order is fixed and the measure is a pure function of the IR.
pub(crate) fn auto_inline(
    program: &mut TypedProgram,
    ids: &mut IdGen,
    measure: &mut dyn FnMut(&TypedProgram) -> Option<HashMap<FuncId, Measure>>,
) {
    // Measuring lowers and optimizes the whole program; skip it when no call
    // could be inlined (the loop below would find no site either).
    let candidates: HashSet<FuncId> = program
        .functions
        .iter()
        .filter(|f| auto_candidate(f))
        .map(|f| f.id)
        .collect();
    let any_site = program.functions.iter().any(|f| {
        f.blocks
            .iter()
            .flat_map(|b| b.ops.iter())
            .any(|op| matches!(op, IrOp::Call { func, .. } if candidates.contains(func)))
    });
    if !any_site {
        return;
    }
    let Some(mut base) = measure(program) else {
        return;
    };
    let mut rejected: HashSet<(FuncId, u32, u32)> = HashSet::new();
    for fid in callee_first_ids(program) {
        loop {
            let Some(fi) = program.functions.iter().position(|f| f.id == fid) else {
                break;
            };
            let counts = call_counts(program);
            let caller = &program.functions[fi];
            let site = all_calls(caller).into_iter().find(|&(bi, k)| {
                let IrOp::Call { func, span, .. } = &caller.blocks[bi].ops[k] else {
                    return false;
                };
                program
                    .functions
                    .iter()
                    .any(|f| f.id == *func && auto_candidate(f))
                    && !rejected.contains(&(fid, span.start, span.end))
            });
            let Some((bi, k)) = site else {
                break;
            };
            let IrOp::Call {
                func: callee_id,
                span,
                ..
            } = caller.blocks[bi].ops[k].clone()
            else {
                break;
            };
            let callee = program
                .functions
                .iter()
                .find(|f| f.id == callee_id)
                .expect("candidate exists");
            let single = !callee.is_pub && counts.get(&callee_id) == Some(&1);
            let mut trial_ids = ids.clone();
            let Ok(expanded) = expand_at(program, fi, bi, k, true, &mut trial_ids) else {
                rejected.insert((fid, span.start, span.end));
                continue;
            };
            let mut trial = program.clone();
            trial.functions[fi] = expanded;
            let accepted = match measure(&trial) {
                Some(after) => {
                    let (Some(old), Some(new)) = (base.get(&fid), after.get(&fid)) else {
                        rejected.insert((fid, span.start, span.end));
                        continue;
                    };
                    let callee_bytes = base.get(&callee_id).map_or(0, |m| m.bytes);
                    let no_new_spill = new.spill <= old.spill;
                    let fits =
                        new.bytes <= old.bytes || (single && new.bytes <= old.bytes + callee_bytes);
                    if no_new_spill && fits {
                        base = after;
                        true
                    } else {
                        false
                    }
                }
                None => false,
            };
            if accepted {
                *ids = trial_ids;
                *program = trial;
            } else {
                rejected.insert((fid, span.start, span.end));
            }
        }
    }
}

fn auto_candidate(f: &TypedFunction) -> bool {
    !f.is_pub
        && f.inline.is_none()
        && !f.noinline
        && f.conv != super::ast::CallConv::Stack
        && !f.blocks.iter().flat_map(|b| b.ops.iter()).any(
            |op| matches!(op, IrOp::InlineAsm { lines, .. } if lines.iter().any(|l| asm_calls(l))),
        )
}

fn asm_calls(line: &str) -> bool {
    let code = line.split(';').next().unwrap_or("").trim();
    let code = match code.rsplit_once(':') {
        Some((_, rest)) if !rest.trim_start().starts_with(['(', '\'']) => rest.trim(),
        _ => code,
    };
    let head = code
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_uppercase();
    head == "CALL" || head == "RST"
}

pub(crate) fn call_counts(program: &TypedProgram) -> HashMap<FuncId, usize> {
    let mut out = HashMap::new();
    for f in &program.functions {
        for op in f.blocks.iter().flat_map(|b| b.ops.iter()) {
            if let IrOp::Call { func, .. } = op {
                *out.entry(*func).or_insert(0) += 1;
            }
        }
    }
    out
}

fn all_calls(func: &TypedFunction) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (bi, block) in func.blocks.iter().enumerate() {
        for (k, op) in block.ops.iter().enumerate() {
            if matches!(op, IrOp::Call { .. }) {
                out.push((bi, k));
            }
        }
    }
    out
}

fn find_call(func: &TypedFunction, wanted: impl Fn(FuncId) -> bool) -> Option<(usize, usize)> {
    all_calls(func).into_iter().find(
        |&(bi, k)| matches!(&func.blocks[bi].ops[k], IrOp::Call { func, .. } if wanted(*func)),
    )
}

/// Function ids with every callee before its callers (the graph is acyclic).
pub(crate) fn callee_first_ids(program: &TypedProgram) -> Vec<FuncId> {
    let by_id: HashMap<FuncId, &TypedFunction> =
        program.functions.iter().map(|f| (f.id, f)).collect();
    let mut roots: Vec<&TypedFunction> = program.functions.iter().collect();
    roots.sort_by_key(|f| (f.span.file.0, f.span.start));
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    fn visit(
        f: &TypedFunction,
        by_id: &HashMap<FuncId, &TypedFunction>,
        seen: &mut HashSet<FuncId>,
        out: &mut Vec<FuncId>,
    ) {
        if !seen.insert(f.id) {
            return;
        }
        for op in f.blocks.iter().flat_map(|b| b.ops.iter()) {
            if let IrOp::Call { func, .. } = op {
                if let Some(callee) = by_id.get(func) {
                    visit(callee, by_id, seen, out);
                }
            }
        }
        out.push(f.id);
    }
    for f in roots {
        visit(f, &by_id, &mut seen, &mut out);
    }
    out
}

impl IrOp {
    pub(crate) fn span(&self) -> SourceSpan {
        match self {
            IrOp::Const { span, .. }
            | IrOp::Unary { span, .. }
            | IrOp::Binary { span, .. }
            | IrOp::Cast { span, .. }
            | IrOp::LoadLocal { span, .. }
            | IrOp::StoreLocal { span, .. }
            | IrOp::LoadGlobal { span, .. }
            | IrOp::AddrGlobal { span, .. }
            | IrOp::AddrLocal { span, .. }
            | IrOp::StoreGlobal { span, .. }
            | IrOp::LoadIndirect { span, .. }
            | IrOp::StoreIndirect { span, .. }
            | IrOp::Call { span, .. }
            | IrOp::PortIn { span, .. }
            | IrOp::PortOut { span, .. }
            | IrOp::Di { span }
            | IrOp::Ei { span }
            | IrOp::Ldir { span, .. }
            | IrOp::InlineAsm { span, .. }
            | IrOp::Branch { span, .. }
            | IrOp::Jump { span, .. }
            | IrOp::Return { span, .. } => *span,
        }
    }
}

/// Vregs an op reads.
fn uses(op: &IrOp) -> Vec<VReg> {
    match op {
        IrOp::Const { .. }
        | IrOp::LoadLocal { .. }
        | IrOp::LoadGlobal { .. }
        | IrOp::AddrGlobal { .. }
        | IrOp::AddrLocal { .. }
        | IrOp::Di { .. }
        | IrOp::Ei { .. }
        | IrOp::Jump { .. } => Vec::new(),
        IrOp::Unary { src, .. } | IrOp::Cast { src, .. } => vec![*src],
        IrOp::Binary { lhs, rhs, .. } => vec![*lhs, *rhs],
        IrOp::StoreLocal { src, .. } | IrOp::StoreGlobal { src, .. } => vec![*src],
        IrOp::LoadIndirect { ptr, .. } => vec![*ptr],
        IrOp::StoreIndirect { ptr, src, .. } => vec![*ptr, *src],
        IrOp::Call { args, .. } => args.clone(),
        IrOp::PortIn { port, .. } => vec![*port],
        IrOp::PortOut { port, value, .. } => vec![*port, *value],
        IrOp::Ldir { hl, de, bc, .. } => vec![*hl, *de, *bc],
        IrOp::InlineAsm { inputs, .. } => inputs.iter().map(|(_, v)| *v).collect(),
        IrOp::Branch { cond, .. } => vec![*cond],
        IrOp::Return { value, .. } => value.iter().copied().collect(),
    }
}

/// Vregs an op defines.
fn defs(op: &IrOp) -> Vec<VReg> {
    match op {
        IrOp::Const { dst, .. }
        | IrOp::Unary { dst, .. }
        | IrOp::Binary { dst, .. }
        | IrOp::Cast { dst, .. }
        | IrOp::LoadLocal { dst, .. }
        | IrOp::LoadGlobal { dst, .. }
        | IrOp::AddrGlobal { dst, .. }
        | IrOp::AddrLocal { dst, .. }
        | IrOp::LoadIndirect { dst, .. }
        | IrOp::PortIn { dst, .. } => vec![*dst],
        IrOp::Call { dst, .. } => dst.iter().copied().collect(),
        IrOp::InlineAsm { outputs, .. } => outputs.iter().map(|o| o.dst).collect(),
        _ => Vec::new(),
    }
}

/// Rename every vreg, block, and local of an op.
fn rewrite(
    op: &IrOp,
    v: &mut dyn FnMut(VReg) -> VReg,
    b: &dyn Fn(BlockId) -> BlockId,
    l: &dyn Fn(LocalId) -> LocalId,
) -> IrOp {
    let mut op = op.clone();
    match &mut op {
        IrOp::Const { dst, .. } => *dst = v(*dst),
        IrOp::Unary { dst, src, .. } | IrOp::Cast { dst, src, .. } => {
            *src = v(*src);
            *dst = v(*dst);
        }
        IrOp::Binary { dst, lhs, rhs, .. } => {
            *lhs = v(*lhs);
            *rhs = v(*rhs);
            *dst = v(*dst);
        }
        IrOp::LoadLocal { dst, local, .. } | IrOp::AddrLocal { dst, local, .. } => {
            *dst = v(*dst);
            *local = l(*local);
        }
        IrOp::StoreLocal { local, src, .. } => {
            *src = v(*src);
            *local = l(*local);
        }
        IrOp::LoadGlobal { dst, .. } | IrOp::AddrGlobal { dst, .. } => *dst = v(*dst),
        IrOp::StoreGlobal { src, .. } => *src = v(*src),
        IrOp::LoadIndirect { dst, ptr, .. } => {
            *ptr = v(*ptr);
            *dst = v(*dst);
        }
        IrOp::StoreIndirect { ptr, src, .. } => {
            *ptr = v(*ptr);
            *src = v(*src);
        }
        IrOp::Call { dst, args, .. } => {
            for a in args.iter_mut() {
                *a = v(*a);
            }
            if let Some(d) = dst {
                *d = v(*d);
            }
        }
        IrOp::PortIn { dst, port, .. } => {
            *port = v(*port);
            *dst = v(*dst);
        }
        IrOp::PortOut { port, value, .. } => {
            *port = v(*port);
            *value = v(*value);
        }
        IrOp::Ldir { hl, de, bc, .. } => {
            *hl = v(*hl);
            *de = v(*de);
            *bc = v(*bc);
        }
        IrOp::InlineAsm {
            inputs, outputs, ..
        } => {
            for (_, r) in inputs.iter_mut() {
                *r = v(*r);
            }
            for o in outputs.iter_mut() {
                o.dst = v(o.dst);
            }
        }
        IrOp::Branch {
            cond,
            true_blk,
            false_blk,
            ..
        } => {
            *cond = v(*cond);
            *true_blk = b(*true_blk);
            *false_blk = b(*false_blk);
        }
        IrOp::Jump { blk, .. } => *blk = b(*blk),
        IrOp::Return { value, .. } => {
            if let Some(r) = value {
                *r = v(*r);
            }
        }
        IrOp::Di { .. } | IrOp::Ei { .. } => {}
    }
    op
}

fn is_terminator(op: &IrOp) -> bool {
    matches!(
        op,
        IrOp::Branch { .. } | IrOp::Jump { .. } | IrOp::Return { .. }
    )
}

/// The type of every vreg a function defines, for the locals that carry
/// values across a split block. Pointers are stored as `u16` (same width).
fn vreg_types(func: &TypedFunction, program: &TypedProgram) -> HashMap<VReg, CType> {
    let local_ty: HashMap<LocalId, CType> = func
        .params
        .iter()
        .map(|p| (p.id, p.ty))
        .chain(func.locals.iter().map(|l| (l.id, l.ty)))
        .collect();
    let mut out = HashMap::new();
    for op in func.blocks.iter().flat_map(|b| b.ops.iter()) {
        let entries: Vec<(VReg, CType)> = match op {
            IrOp::Const { dst, ty, .. } | IrOp::Unary { dst, ty, .. } => vec![(*dst, *ty)],
            IrOp::Binary { dst, ty, op, .. } => {
                let t = if matches!(
                    op,
                    IrBinary::Eq
                        | IrBinary::Ne
                        | IrBinary::Lt
                        | IrBinary::Le
                        | IrBinary::Gt
                        | IrBinary::Ge
                        | IrBinary::Or
                        | IrBinary::And
                ) {
                    CType::Bool
                } else {
                    *ty
                };
                vec![(*dst, t)]
            }
            IrOp::Cast { dst, to, .. } => vec![(*dst, *to)],
            IrOp::LoadLocal { dst, local, .. } => {
                vec![(*dst, local_ty.get(local).copied().unwrap_or(CType::U16))]
            }
            IrOp::LoadGlobal { dst, global, .. } => vec![(
                *dst,
                program
                    .globals
                    .iter()
                    .find(|g| g.id == *global)
                    .map_or(CType::U16, |g| g.ty),
            )],
            IrOp::AddrGlobal { dst, .. } | IrOp::AddrLocal { dst, .. } => {
                vec![(*dst, CType::U16)]
            }
            IrOp::LoadIndirect { dst, ty, .. } => vec![(*dst, *ty)],
            IrOp::PortIn { dst, .. } => vec![(*dst, CType::U8)],
            IrOp::Call {
                dst: Some(d), func, ..
            } => vec![(
                *d,
                program
                    .functions
                    .iter()
                    .find(|f| f.id == *func)
                    .map_or(CType::U16, |f| f.ret),
            )],
            IrOp::InlineAsm { outputs, .. } => outputs.iter().map(|o| (o.dst, o.ty)).collect(),
            _ => Vec::new(),
        };
        for (v, t) in entries {
            let t = if matches!(t, CType::Ptr(_) | CType::Str) {
                CType::U16
            } else {
                t
            };
            out.insert(v, t);
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tag {
    /// The caller's own op, by its old function-wide index.
    Caller(usize),
    /// Part of the expansion: a cloned callee op (callee-wide index), or
    /// glue (argument and result moves, `None`).
    Site(Option<usize>),
}

#[derive(Clone, Copy)]
enum ParamMode {
    Const(u16),
    Vreg(VReg),
    /// The argument is a caller local the expansion cannot change (not
    /// address-taken, not stored between the read and the call): every read
    /// of the parameter reads that local.
    Alias(LocalId),
    Local(LocalId),
}

/// Expand the call at `caller.blocks[bi].ops[k]`.
fn expand_at(
    program: &TypedProgram,
    fi: usize,
    bi: usize,
    k: usize,
    auto: bool,
    ids: &mut IdGen,
) -> Result<TypedFunction, Diagnostic> {
    let caller = &program.functions[fi];
    let IrOp::Call {
        dst,
        func: callee_id,
        args,
        span: call_span,
    } = caller.blocks[bi].ops[k].clone()
    else {
        unreachable!("expand_at on a call");
    };
    let callee = program
        .functions
        .iter()
        .find(|f| f.id == callee_id)
        .expect("callee exists");
    if callee.conv == super::ast::CallConv::Stack {
        return Err(Diagnostic::error(
            DiagCode::CgUnsupported,
            call_span,
            format!("cannot inline '{}': it is @stackcall", callee.name),
        ));
    }
    let frame = InlineFrame {
        callee: callee.id,
        name: callee.name.clone(),
        call: call_span,
        auto,
    };

    // Origins: the caller's table plus the chains through this call.
    let mut sites = caller.inline_sites.clone();
    let call_origin = caller.blocks[bi].origin_of(k);
    let base_frames: Vec<InlineFrame> = if call_origin == 0 {
        Vec::new()
    } else {
        sites[call_origin as usize - 1].frames.clone()
    };
    let mut intern = |frames: Vec<InlineFrame>| -> u32 {
        if let Some(i) = sites.iter().position(|s| s.frames == frames) {
            return i as u32 + 1;
        }
        sites.push(InlineSite { frames });
        sites.len() as u32
    };
    let mut callee_origin: HashMap<u32, u32> = HashMap::new();
    let mut origin_for = |o: u32, intern: &mut dyn FnMut(Vec<InlineFrame>) -> u32| -> u32 {
        if let Some(&n) = callee_origin.get(&o) {
            return n;
        }
        let mut frames = base_frames.clone();
        frames.push(frame.clone());
        if o != 0 {
            frames.extend(callee.inline_sites[o as usize - 1].frames.iter().cloned());
        }
        let n = intern(frames);
        callee_origin.insert(o, n);
        n
    };

    // Fresh vregs and blocks above everything the caller uses.
    let mut next_v = caller
        .blocks
        .iter()
        .flat_map(|b| b.ops.iter())
        .flat_map(|op| defs(op).into_iter().chain(uses(op)))
        .map(|v| v.0 + 1)
        .max()
        .unwrap_or(0);
    let mut next_b = caller.blocks.iter().map(|b| b.id.0 + 1).max().unwrap_or(0);
    let mut fresh_v = || {
        let v = VReg(next_v);
        next_v += 1;
        v
    };

    let callee_ops = || callee.blocks.iter().flat_map(|b| b.ops.iter());
    let written: HashSet<LocalId> = callee_ops()
        .filter_map(|op| match op {
            IrOp::StoreLocal { local, .. } | IrOp::AddrLocal { local, .. } => Some(*local),
            _ => None,
        })
        .collect();
    let splice = callee.blocks.len() == 1
        && callee.blocks[0]
            .ops
            .last()
            .is_some_and(|op| matches!(op, IrOp::Return { .. }))
        && callee.blocks[0].ops[..callee.blocks[0].ops.len() - 1]
            .iter()
            .all(|op| !is_terminator(op));

    // Constant arguments: a `Const` defined earlier in the calling block.
    let consts: HashMap<VReg, u16> = caller.blocks[bi].ops[..k]
        .iter()
        .filter_map(|op| match op {
            IrOp::Const { dst, bits, .. } => Some((*dst, *bits)),
            _ => None,
        })
        .collect();

    // Arguments read from a caller local that nothing can change before
    // the expansion reads them.
    let addr_taken: HashSet<LocalId> = caller
        .blocks
        .iter()
        .flat_map(|b| b.ops.iter())
        .filter_map(|op| match op {
            IrOp::AddrLocal { local, .. } => Some(*local),
            _ => None,
        })
        .collect();
    let mut aliases: HashMap<VReg, LocalId> = HashMap::new();
    for op in &caller.blocks[bi].ops[..k] {
        match op {
            IrOp::LoadLocal { dst, local, .. } if !addr_taken.contains(local) => {
                aliases.insert(*dst, *local);
            }
            IrOp::StoreLocal { local, .. } => aliases.retain(|_, l| l != local),
            _ => {}
        }
    }
    let mut new_locals: Vec<TypedLocal> = Vec::new();
    let mut lmap: HashMap<LocalId, LocalId> = HashMap::new();
    let mut modes: Vec<ParamMode> = Vec::new();
    for (p, &arg) in callee.params.iter().zip(args.iter()) {
        let mode = if written.contains(&p.id) || p.pin.is_some() {
            None
        } else if let Some(&bits) = consts.get(&arg) {
            Some(ParamMode::Const(bits))
        } else if splice {
            Some(ParamMode::Vreg(arg))
        } else {
            aliases.get(&arg).map(|&l| ParamMode::Alias(l))
        };
        let mode = mode.unwrap_or_else(|| {
            let id = LocalId(ids.next());
            new_locals.push(TypedLocal {
                id,
                name: format!("{}.{}", callee.name, p.name),
                ty: p.ty,
                span: p.span,
                pin: p.pin,
            });
            lmap.insert(p.id, id);
            ParamMode::Local(id)
        });
        modes.push(mode);
    }
    for l in &callee.locals {
        let id = LocalId(ids.next());
        new_locals.push(TypedLocal {
            id,
            name: format!("{}.{}", callee.name, l.name),
            ty: l.ty,
            span: l.span,
            pin: l.pin,
        });
        lmap.insert(l.id, id);
    }
    let param_mode: HashMap<LocalId, (ParamMode, CType)> = callee
        .params
        .iter()
        .zip(modes.iter())
        .map(|(p, m)| (p.id, (*m, p.ty)))
        .collect();

    let result_local = if callee.ret != CType::Void && !splice {
        let id = LocalId(ids.next());
        new_locals.push(TypedLocal {
            id,
            name: format!("{}.result", callee.name),
            ty: callee.ret,
            span: callee.span,
            pin: None,
        });
        Some(id)
    } else {
        None
    };

    let bmap: HashMap<BlockId, BlockId> = callee
        .blocks
        .iter()
        .map(|b| {
            let id = BlockId(next_b);
            next_b += 1;
            (b.id, id)
        })
        .collect();
    let cont = BlockId(next_b);

    // Clone the callee's blocks.
    let mut vmap: HashMap<VReg, VReg> = HashMap::new();
    let mut ret_vreg: Option<VReg> = None;
    let mut cloned: Vec<(BlockId, Vec<IrOp>, Vec<u32>, Vec<Tag>)> = Vec::new();
    let mut j = 0usize;
    let nblocks = callee.blocks.len();
    for (cbi, cb) in callee.blocks.iter().enumerate() {
        let mut ops = Vec::new();
        let mut origin = Vec::new();
        let mut tags = Vec::new();
        for (ci, op) in cb.ops.iter().enumerate() {
            let o = origin_for(cb.origin_of(ci), &mut intern);
            let mut push = |op: IrOp, ops: &mut Vec<IrOp>| {
                ops.push(op);
                origin.push(o);
                tags.push(Tag::Site(Some(j)));
            };
            match op {
                IrOp::LoadLocal { dst, local, span } if param_mode.contains_key(local) => {
                    match param_mode[local] {
                        (ParamMode::Const(bits), ty) => {
                            let d = *vmap.entry(*dst).or_insert_with(&mut fresh_v);
                            push(
                                IrOp::Const {
                                    dst: d,
                                    ty,
                                    bits,
                                    span: *span,
                                },
                                &mut ops,
                            );
                        }
                        (ParamMode::Vreg(arg), _) => {
                            vmap.insert(*dst, arg);
                        }
                        (ParamMode::Local(id), _) | (ParamMode::Alias(id), _) => {
                            let d = *vmap.entry(*dst).or_insert_with(&mut fresh_v);
                            push(
                                IrOp::LoadLocal {
                                    dst: d,
                                    local: id,
                                    span: *span,
                                },
                                &mut ops,
                            );
                        }
                    }
                }
                IrOp::Return { value, span } => {
                    let value = value.map(|r| *vmap.entry(r).or_insert_with(&mut fresh_v));
                    if splice {
                        ret_vreg = value;
                    } else {
                        if let (Some(res), Some(v)) = (result_local, value) {
                            push(
                                IrOp::StoreLocal {
                                    local: res,
                                    src: v,
                                    span: *span,
                                },
                                &mut ops,
                            );
                        }
                        push(
                            IrOp::Jump {
                                blk: cont,
                                span: *span,
                            },
                            &mut ops,
                        );
                    }
                }
                _ => {
                    let mut vf = |r: VReg| *vmap.entry(r).or_insert_with(&mut fresh_v);
                    let bf = |b: BlockId| bmap.get(&b).copied().unwrap_or(b);
                    let lf = |l: LocalId| lmap.get(&l).copied().unwrap_or(l);
                    let mut new = rewrite(op, &mut vf, &bf, &lf);
                    if let IrOp::InlineAsm { node, lines, .. } = &mut new {
                        // Local labels were renamed `IA<node>_NAME` by
                        // semantics; each copy gets its own namespace.
                        let fresh = ids.next();
                        let (old, new) = (format!("IA{}_", node.0), format!("IA{}_", fresh.0));
                        for line in lines.iter_mut() {
                            *line = rename_prefix(line, &old, &new);
                        }
                        *node = fresh;
                    }
                    push(new, &mut ops);
                }
            }
            j += 1;
        }
        if !splice && cbi + 1 == nblocks && !ops.last().is_some_and(is_terminator) {
            let o = origin_for(0, &mut intern);
            ops.push(IrOp::Jump {
                blk: cont,
                span: callee.span,
            });
            origin.push(o);
            tags.push(Tag::Site(None));
        }
        cloned.push((bmap[&cb.id], ops, origin, tags));
    }

    // Fold constants inside the expansion; decided branches drop a dead arm.
    for (_, ops, _, _) in cloned.iter_mut() {
        fold_constants(ops, &consts);
    }

    // Assemble the new block list.
    let flat_base: Vec<usize> = caller
        .blocks
        .iter()
        .scan(0usize, |acc, b| {
            let base = *acc;
            *acc += b.ops.len();
            Some(base)
        })
        .collect();
    let caller_tag = |b: usize, i: usize| Tag::Caller(flat_base[b] + i);
    let call_flat = flat_base[bi] + k;
    let block_a = &caller.blocks[bi];

    let mut blocks: Vec<(BlockId, Vec<IrOp>, Vec<u32>, Vec<Tag>)> = Vec::new();
    for (b, block) in caller.blocks.iter().enumerate().take(bi) {
        blocks.push(caller_block(block, b, &caller_tag));
    }
    let mut a_ops: Vec<IrOp> = block_a.ops[..k].to_vec();
    let mut a_origin: Vec<u32> = (0..k).map(|i| block_a.origin_of(i)).collect();
    let mut a_tags: Vec<Tag> = (0..k).map(|i| caller_tag(bi, i)).collect();
    let mut param_store_pos: HashMap<LocalId, usize> = HashMap::new();
    for (mode, &arg) in modes.iter().zip(args.iter()) {
        if let ParamMode::Local(id) = mode {
            param_store_pos.insert(*id, a_ops.len());
            a_ops.push(IrOp::StoreLocal {
                local: *id,
                src: arg,
                span: call_span,
            });
            a_origin.push(call_origin);
            a_tags.push(Tag::Site(None));
        }
    }
    let rest: Vec<IrOp> = block_a.ops[k + 1..].to_vec();
    let rest_origin: Vec<u32> = (k + 1..block_a.ops.len())
        .map(|i| block_a.origin_of(i))
        .collect();
    let rest_tags: Vec<Tag> = (k + 1..block_a.ops.len())
        .map(|i| caller_tag(bi, i))
        .collect();

    if splice {
        let (_, c_ops, c_origin, c_tags) = cloned.pop().expect("one block");
        a_ops.extend(c_ops);
        a_origin.extend(c_origin);
        a_tags.extend(c_tags);
        let rename: HashMap<VReg, VReg> = match (dst, ret_vreg) {
            (Some(d), Some(r)) => [(d, r)].into_iter().collect(),
            _ => HashMap::new(),
        };
        for op in &rest {
            let mut vf = |r: VReg| rename.get(&r).copied().unwrap_or(r);
            a_ops.push(rewrite(op, &mut vf, &|b| b, &|l| l));
        }
        a_origin.extend(rest_origin);
        a_tags.extend(rest_tags);
        blocks.push((block_a.id, a_ops, a_origin, a_tags));
    } else {
        // Values computed before the call and used after it cross the new
        // block boundary through fresh locals.
        let before: HashSet<VReg> = block_a.ops[..k].iter().flat_map(defs).collect();
        let mut crossing: Vec<VReg> = rest
            .iter()
            .flat_map(uses)
            .filter(|v| before.contains(v))
            .collect();
        crossing.sort();
        crossing.dedup();
        let tys = vreg_types(caller, program);
        let mut b_ops = Vec::new();
        let mut b_origin = Vec::new();
        let mut b_tags = Vec::new();
        let mut rename: HashMap<VReg, VReg> = HashMap::new();
        for v in crossing {
            let id = LocalId(ids.next());
            let ty = tys.get(&v).copied().unwrap_or(CType::U16);
            new_locals.push(TypedLocal {
                id,
                name: "inline.carry".to_string(),
                ty,
                span: call_span,
                pin: None,
            });
            a_ops.push(IrOp::StoreLocal {
                local: id,
                src: v,
                span: call_span,
            });
            a_origin.push(call_origin);
            a_tags.push(Tag::Site(None));
            let nv = fresh_v();
            rename.insert(v, nv);
            b_ops.push(IrOp::LoadLocal {
                dst: nv,
                local: id,
                span: call_span,
            });
            b_origin.push(call_origin);
            b_tags.push(Tag::Site(None));
        }
        let entry = cloned.first().map(|c| c.0).unwrap_or(cont);
        a_ops.push(IrOp::Jump {
            blk: entry,
            span: call_span,
        });
        a_origin.push(call_origin);
        a_tags.push(Tag::Site(None));
        blocks.push((block_a.id, a_ops, a_origin, a_tags));
        if let (Some(d), Some(res)) = (dst, result_local) {
            b_ops.push(IrOp::LoadLocal {
                dst: d,
                local: res,
                span: call_span,
            });
            b_origin.push(call_origin);
            b_tags.push(Tag::Site(None));
        }
        for op in &rest {
            let mut vf = |r: VReg| rename.get(&r).copied().unwrap_or(r);
            b_ops.push(rewrite(op, &mut vf, &|b| b, &|l| l));
        }
        b_origin.extend(rest_origin);
        b_tags.extend(rest_tags);
        // Drop cloned blocks a folded branch made unreachable.
        let reachable = reachable_clones(&cloned, entry);
        blocks.extend(cloned.into_iter().filter(|c| reachable.contains(&c.0)));
        blocks.push((cont, b_ops, b_origin, b_tags));
    }
    for (b, block) in caller.blocks.iter().enumerate().skip(bi + 1) {
        blocks.push(caller_block(block, b, &caller_tag));
    }
    if !splice {
        // A callee whose branches folded away (or that had none) becomes
        // straight-line again: merge the chain into the calling block, then
        // pass values through the expansion's locals directly.
        let fresh_blocks: HashSet<BlockId> = bmap.values().copied().chain([cont]).collect();
        merge_chain(&mut blocks, &fresh_blocks);
    }
    let fresh_locals: HashSet<LocalId> = new_locals.iter().map(|l| l.id).collect();
    let forwarded = forward_locals(&mut blocks, &fresh_locals, &new_locals);
    new_locals.retain(|l| !forwarded.contains(&l.id));
    param_store_pos.retain(|id, _| !forwarded.contains(id));

    // Flat positions, for pin scopes.
    let tags: Vec<Tag> = blocks.iter().flat_map(|b| b.3.iter().copied()).collect();
    let total = tags.len();
    let caller_pos = |s: usize| -> usize {
        if s == usize::MAX {
            return usize::MAX;
        }
        tags.iter()
            .position(|t| match *t {
                Tag::Caller(i) => i >= s,
                Tag::Site(_) => call_flat >= s,
            })
            .unwrap_or(total)
    };
    let site_end = tags
        .iter()
        .rposition(|t| matches!(t, Tag::Site(Some(_))))
        .map(|p| p + 1)
        .unwrap_or_else(|| {
            tags.iter()
                .position(|t| matches!(t, Tag::Site(_)))
                .unwrap_or(total)
        });
    let site_pos = |s: usize| -> usize {
        tags.iter()
            .position(|t| matches!(t, Tag::Site(Some(j)) if *j >= s))
            .unwrap_or(site_end)
            .min(site_end)
    };
    // Remap the caller's pins.
    let mut new_func = caller.clone();
    for p in new_func.params.iter_mut() {
        if let Some(pin) = p.pin.as_mut() {
            pin.scope = (caller_pos(pin.scope.0), caller_pos(pin.scope.1));
        }
    }
    for l in new_func.locals.iter_mut() {
        if let Some(pin) = l.pin.as_mut() {
            pin.scope = (caller_pos(pin.scope.0), caller_pos(pin.scope.1));
        }
    }
    // Pins of the expansion: callee locals keep their scope inside the
    // expansion; pinned parameters span from their argument store.
    let a_start = blocks[..bi].iter().map(|b| b.1.len()).sum::<usize>();
    let store_pos = |id: LocalId| -> Option<usize> {
        blocks_flat_ops(&blocks)
            .position(|op| matches!(op, IrOp::StoreLocal { local, .. } if *local == id))
    };
    let _ = a_start;
    for l in new_locals.iter_mut() {
        if let Some(pin) = l.pin.as_mut() {
            pin.scope = match param_store_pos.get(&l.id).and_then(|_| store_pos(l.id)) {
                Some(pos) => (pos, site_end),
                None => (site_pos(pin.scope.0), site_pos(pin.scope.1)),
            };
        }
    }
    // A pin of the expansion must not overlap a caller pin live at the call.
    for l in new_locals.iter().filter(|l| l.pin.is_some()) {
        let pin = l.pin.unwrap();
        if gpr_mask(pin.reg) & caller.preserves != 0 {
            return Err(pin_conflict(
                call_span,
                callee,
                l,
                pin,
                &format!(
                    "the caller '{}' preserves {} (@preserves)",
                    caller.name,
                    pin.reg.as_str()
                ),
                caller.preserves_span,
            ));
        }
        let others = new_func
            .params
            .iter()
            .filter_map(|p| p.pin.map(|q| (p.name.clone(), q)))
            .chain(
                new_func
                    .locals
                    .iter()
                    .filter_map(|x| x.pin.map(|q| (x.name.clone(), q))),
            );
        for (name, q) in others {
            let overlap = q.scope.0 < pin.scope.1.max(pin.scope.0 + 1)
                && pin.scope.0 < q.scope.1.max(q.scope.0 + 1);
            if overlap && regs_overlap(pin.reg, q.reg) {
                return Err(pin_conflict(
                    call_span,
                    callee,
                    l,
                    pin,
                    &format!(
                        "'{name} @ {}' of '{}' is live there",
                        q.reg.as_str(),
                        caller.name
                    ),
                    Some(q.span),
                ));
            }
        }
    }

    new_func.blocks = blocks
        .into_iter()
        .map(|(id, ops, origin, _)| {
            let origin = if origin.iter().all(|&o| o == 0) {
                Vec::new()
            } else {
                origin
            };
            IrBlock { id, ops, origin }
        })
        .collect();
    new_func.locals.extend(new_locals);
    new_func.inline_sites = sites;
    Ok(new_func)
}

type Block4 = (BlockId, Vec<IrOp>, Vec<u32>, Vec<Tag>);

fn blocks_flat_ops(blocks: &[Block4]) -> impl Iterator<Item = &IrOp> {
    blocks.iter().flat_map(|b| b.1.iter())
}

/// Merge `X; jump Y` with the block `Y` right after it when `Y` is part of
/// the expansion and has no other predecessor.
fn merge_chain(blocks: &mut Vec<Block4>, fresh: &HashSet<BlockId>) {
    loop {
        let mut preds: HashMap<BlockId, usize> = HashMap::new();
        for (i, b) in blocks.iter().enumerate() {
            match b.1.last() {
                Some(IrOp::Jump { blk, .. }) => *preds.entry(*blk).or_insert(0) += 1,
                Some(IrOp::Branch {
                    true_blk,
                    false_blk,
                    ..
                }) => {
                    *preds.entry(*true_blk).or_insert(0) += 1;
                    *preds.entry(*false_blk).or_insert(0) += 1;
                }
                Some(IrOp::Return { .. }) => {}
                _ => {
                    if let Some(next) = blocks.get(i + 1) {
                        *preds.entry(next.0).or_insert(0) += 1;
                    }
                }
            }
        }
        let found = (0..blocks.len().saturating_sub(1)).find(|&i| {
            let y = blocks[i + 1].0;
            fresh.contains(&y)
                && preds.get(&y) == Some(&1)
                && matches!(blocks[i].1.last(), Some(IrOp::Jump { blk, .. }) if *blk == y)
        });
        let Some(i) = found else {
            return;
        };
        let y = blocks.remove(i + 1);
        let x = &mut blocks[i];
        x.1.pop();
        x.2.pop();
        x.3.pop();
        x.1.extend(y.1);
        x.2.extend(y.2);
        x.3.extend(y.3);
    }
}

/// Fresh locals of the expansion stored once and only read later in the
/// same block are replaced by the stored vreg. Returns the removed locals.
fn forward_locals(
    blocks: &mut [Block4],
    fresh: &HashSet<LocalId>,
    locals: &[TypedLocal],
) -> HashSet<LocalId> {
    let pinned: HashSet<LocalId> = locals
        .iter()
        .filter(|l| l.pin.is_some())
        .map(|l| l.id)
        .collect();
    let mut removed = HashSet::new();
    for &id in fresh {
        if pinned.contains(&id) {
            continue;
        }
        let mut stores = Vec::new();
        let mut loads = Vec::new();
        let mut other = false;
        for (bi, b) in blocks.iter().enumerate() {
            for (oi, op) in b.1.iter().enumerate() {
                match op {
                    IrOp::StoreLocal { local, src, .. } if *local == id => {
                        stores.push((bi, oi, *src))
                    }
                    IrOp::LoadLocal { local, dst, .. } if *local == id => {
                        loads.push((bi, oi, *dst))
                    }
                    IrOp::AddrLocal { local, .. } if *local == id => other = true,
                    _ => {}
                }
            }
        }
        if other || stores.len() != 1 {
            continue;
        }
        let (sb, so, src) = stores[0];
        if !loads.iter().all(|&(lb, lo, _)| lb == sb && lo > so) {
            continue;
        }
        let rename: HashMap<VReg, VReg> = loads.iter().map(|&(_, _, d)| (d, src)).collect();
        let b = &mut blocks[sb];
        let drop: HashSet<usize> = loads.iter().map(|&(_, lo, _)| lo).chain([so]).collect();
        let mut ops = Vec::new();
        let mut origin = Vec::new();
        let mut tags = Vec::new();
        for (oi, op) in b.1.iter().enumerate() {
            if drop.contains(&oi) {
                continue;
            }
            let mut vf = |r: VReg| rename.get(&r).copied().unwrap_or(r);
            ops.push(rewrite(op, &mut vf, &|x| x, &|x| x));
            origin.push(b.2[oi]);
            tags.push(b.3[oi]);
        }
        b.1 = ops;
        b.2 = origin;
        b.3 = tags;
        removed.insert(id);
    }
    removed
}

fn caller_block(
    block: &IrBlock,
    b: usize,
    tag: &dyn Fn(usize, usize) -> Tag,
) -> (BlockId, Vec<IrOp>, Vec<u32>, Vec<Tag>) {
    (
        block.id,
        block.ops.clone(),
        (0..block.ops.len()).map(|i| block.origin_of(i)).collect(),
        (0..block.ops.len()).map(|i| tag(b, i)).collect(),
    )
}

fn regs_overlap(a: AsmGpr, b: AsmGpr) -> bool {
    gpr_mask(a) & gpr_mask(b) != 0
}

fn pin_conflict(
    call: SourceSpan,
    callee: &TypedFunction,
    local: &TypedLocal,
    pin: RegPin,
    what: &str,
    other: Option<SourceSpan>,
) -> Diagnostic {
    let short = local
        .name
        .strip_prefix(&format!("{}.", callee.name))
        .unwrap_or(&local.name);
    let mut diag = Diagnostic::error(
        DiagCode::TyDuplicateName,
        call,
        format!(
            "cannot inline '{}' here: its pin '{short} @ {}' overlaps a register in use at this call ({what})",
            callee.name,
            pin.reg.as_str()
        ),
    );
    diag.related.push(RelatedSpan {
        span: pin.span,
        message: format!("pin in '{}'", callee.name),
    });
    if let Some(span) = other {
        diag.related.push(RelatedSpan {
            span,
            message: "conflicting register here".to_string(),
        });
    }
    diag
}

fn reachable_clones(
    cloned: &[(BlockId, Vec<IrOp>, Vec<u32>, Vec<Tag>)],
    entry: BlockId,
) -> HashSet<BlockId> {
    let ids: Vec<BlockId> = cloned.iter().map(|c| c.0).collect();
    let mut seen = HashSet::new();
    let mut work = vec![entry];
    while let Some(b) = work.pop() {
        if !seen.insert(b) {
            continue;
        }
        let Some(pos) = ids.iter().position(|&x| x == b) else {
            continue;
        };
        let ops = &cloned[pos].1;
        match ops.last() {
            Some(IrOp::Jump { blk, .. }) => work.push(*blk),
            Some(IrOp::Branch {
                true_blk,
                false_blk,
                ..
            }) => {
                work.push(*true_blk);
                work.push(*false_blk);
            }
            Some(IrOp::Return { .. }) => {}
            _ => {
                if let Some(next) = ids.get(pos + 1) {
                    work.push(*next);
                }
            }
        }
    }
    seen
}

/// Fold operations whose operands are all constants (H3i item 4): the
/// minimum that makes a helper called with constant arguments collapse.
/// A branch on a constant becomes a jump.
fn fold_constants(ops: &mut [IrOp], outer: &HashMap<VReg, u16>) {
    let mut known: HashMap<VReg, u16> = outer.clone();
    for op in ops.iter_mut() {
        let folded = match op {
            IrOp::Const { dst, bits, .. } => {
                known.insert(*dst, *bits);
                None
            }
            IrOp::Unary {
                dst,
                ty,
                op: u,
                src,
                span,
            } => known.get(src).map(|&a| {
                let x = ty.interpret_bits(a);
                let bits = match u {
                    IrUnary::Plus => ty.wrap_bits(x),
                    IrUnary::Neg => ty.wrap_bits(x.wrapping_neg()),
                    IrUnary::Not => u16::from(a == 0),
                    IrUnary::BitNot => ty.wrap_bits(!x),
                };
                (*dst, *ty, bits, *span)
            }),
            IrOp::Cast {
                dst,
                to,
                from,
                src,
                span,
            } => known.get(src).map(|&a| {
                (
                    *dst,
                    *to,
                    super::semantics::convert_bits(*from, *to, a),
                    *span,
                )
            }),
            IrOp::Binary {
                dst,
                ty,
                op: b,
                lhs,
                rhs,
                span,
            } => match (known.get(lhs), known.get(rhs)) {
                (Some(&x), Some(&y)) => {
                    fold_binary(*b, *ty, x, y).map(|(t, bits)| (*dst, t, bits, *span))
                }
                _ => None,
            },
            IrOp::Branch {
                cond,
                true_blk,
                false_blk,
                span,
            } => {
                if let Some(&c) = known.get(cond) {
                    let blk = if c != 0 { *true_blk } else { *false_blk };
                    *op = IrOp::Jump { blk, span: *span };
                }
                None
            }
            _ => None,
        };
        if let Some((dst, ty, bits, span)) = folded {
            known.insert(dst, bits);
            *op = IrOp::Const {
                dst,
                ty,
                bits,
                span,
            };
        }
    }
}

fn fold_binary(op: IrBinary, ty: CType, a: u16, b: u16) -> Option<(CType, u16)> {
    let x = ty.interpret_bits(a);
    let y = ty.interpret_bits(b);
    let cmp = |r: bool| Some((CType::Bool, u16::from(r)));
    match op {
        IrBinary::Eq => cmp(x == y),
        IrBinary::Ne => cmp(x != y),
        IrBinary::Lt => cmp(x < y),
        IrBinary::Le => cmp(x <= y),
        IrBinary::Gt => cmp(x > y),
        IrBinary::Ge => cmp(x >= y),
        IrBinary::Add => Some((ty, ty.wrap_bits(x.wrapping_add(y)))),
        IrBinary::Sub => Some((ty, ty.wrap_bits(x.wrapping_sub(y)))),
        IrBinary::BitAnd => Some((ty, ty.wrap_bits(x & y))),
        IrBinary::BitOr => Some((ty, ty.wrap_bits(x | y))),
        IrBinary::BitXor => Some((ty, ty.wrap_bits(x ^ y))),
        // Shifts, `&&`, `||`: left to the lowering.
        _ => None,
    }
}

/// Replace `old` with `new` where it starts an identifier.
fn rename_prefix(line: &str, old: &str, new: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find(old) {
        let before = if at > 0 {
            rest[..at].chars().last()
        } else {
            out.chars().last()
        };
        let starts_ident = before.is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'));
        out.push_str(&rest[..at]);
        out.push_str(if starts_ident { new } else { old });
        rest = &rest[at + old.len()..];
    }
    out.push_str(rest);
    out
}

/// F-032: a vreg is used only in the block that defines it, except the
/// operands of a short-circuit join (`&&`/`||` as the first op of a block,
/// which the lowerer passes in `A`). `&&`/`||` split an expression into
/// blocks, so a value evaluated before one (an earlier call argument, the
/// left side of an operator) can be used after its join. Such a value is
/// stored to a fresh local right after its definition and loaded at the
/// start of each later block that uses it, the same way an inline expansion
/// carries values across its split.
pub(crate) fn carry_block_values(program: &mut TypedProgram, ids: &mut IdGen) {
    for fi in 0..program.functions.len() {
        let tys = vreg_types(&program.functions[fi], program);
        carry_function(&mut program.functions[fi], &tys, ids);
    }
}

fn is_join(op: &IrOp) -> bool {
    matches!(
        op,
        IrOp::Binary {
            op: IrBinary::And | IrBinary::Or,
            ..
        }
    )
}

fn carry_function(func: &mut TypedFunction, tys: &HashMap<VReg, CType>, ids: &mut IdGen) {
    let mut def_block: HashMap<VReg, usize> = HashMap::new();
    for (bi, b) in func.blocks.iter().enumerate() {
        for op in &b.ops {
            for d in defs(op) {
                def_block.insert(d, bi);
            }
        }
    }
    // (vreg, using block) pairs that cross a boundary.
    let mut crossing: Vec<(VReg, usize)> = Vec::new();
    for (bi, b) in func.blocks.iter().enumerate() {
        for (oi, op) in b.ops.iter().enumerate() {
            if oi == 0 && is_join(op) {
                continue;
            }
            for u in uses(op) {
                if def_block.get(&u).is_some_and(|&d| d != bi) {
                    crossing.push((u, bi));
                }
            }
        }
    }
    if crossing.is_empty() {
        return;
    }
    crossing.sort();
    crossing.dedup();
    let mut next_v = func
        .blocks
        .iter()
        .flat_map(|b| b.ops.iter())
        .flat_map(|op| defs(op).into_iter().chain(uses(op)))
        .map(|v| v.0 + 1)
        .max()
        .unwrap_or(0);
    let mut carried: HashMap<VReg, LocalId> = HashMap::new();
    for &(v, _) in &crossing {
        carried.entry(v).or_insert_with(|| {
            let id = LocalId(ids.next());
            func.locals.push(TypedLocal {
                id,
                name: "expr.carry".to_string(),
                ty: tys.get(&v).copied().unwrap_or(CType::U16),
                span: func.span,
                pin: None,
            });
            id
        });
    }
    // Flat insertion points (index of the original op the new op goes
    // before), for remapping pin scopes.
    let mut points: Vec<usize> = Vec::new();
    let mut flat = 0usize;
    for (bi, b) in func.blocks.iter_mut().enumerate() {
        let has_origin = !b.origin.is_empty();
        let mut loads: Vec<(VReg, VReg, LocalId)> = Vec::new();
        for &(v, ub) in &crossing {
            if ub == bi {
                let nv = VReg(next_v);
                next_v += 1;
                loads.push((v, nv, carried[&v]));
            }
        }
        let rename: HashMap<VReg, VReg> = loads.iter().map(|&(v, nv, _)| (v, nv)).collect();
        let skip = usize::from(b.ops.first().is_some_and(is_join));
        let mut ops = Vec::new();
        let mut origin = Vec::new();
        for (oi, op) in b.ops.iter().enumerate() {
            let o = b.origin_of(oi);
            if oi == skip {
                for &(_, nv, local) in &loads {
                    let span = op_span(op);
                    ops.push(IrOp::LoadLocal {
                        dst: nv,
                        local,
                        span,
                    });
                    origin.push(o);
                    points.push(flat + oi);
                }
            }
            if oi < skip {
                ops.push(op.clone());
            } else {
                let mut vf = |r: VReg| rename.get(&r).copied().unwrap_or(r);
                ops.push(rewrite(op, &mut vf, &|x| x, &|x| x));
            }
            origin.push(o);
            for d in defs(op) {
                if let Some(&local) = carried.get(&d) {
                    ops.push(IrOp::StoreLocal {
                        local,
                        src: d,
                        span: op_span(op),
                    });
                    origin.push(o);
                    points.push(flat + oi + 1);
                }
            }
        }
        flat += b.ops.len();
        b.ops = ops;
        b.origin = if has_origin { origin } else { Vec::new() };
    }
    let shift = |p: usize, end: bool| -> usize {
        if p == usize::MAX {
            return p;
        }
        p + points
            .iter()
            .filter(|&&q| if end { q < p } else { q <= p })
            .count()
    };
    for pin in func
        .params
        .iter_mut()
        .filter_map(|p| p.pin.as_mut())
        .chain(func.locals.iter_mut().filter_map(|l| l.pin.as_mut()))
    {
        pin.scope = (shift(pin.scope.0, false), shift(pin.scope.1, true));
    }
}

fn op_span(op: &IrOp) -> SourceSpan {
    match op {
        IrOp::Const { span, .. }
        | IrOp::Unary { span, .. }
        | IrOp::Binary { span, .. }
        | IrOp::Cast { span, .. }
        | IrOp::LoadLocal { span, .. }
        | IrOp::StoreLocal { span, .. }
        | IrOp::LoadGlobal { span, .. }
        | IrOp::AddrGlobal { span, .. }
        | IrOp::AddrLocal { span, .. }
        | IrOp::StoreGlobal { span, .. }
        | IrOp::LoadIndirect { span, .. }
        | IrOp::StoreIndirect { span, .. }
        | IrOp::Call { span, .. }
        | IrOp::PortIn { span, .. }
        | IrOp::PortOut { span, .. }
        | IrOp::Di { span }
        | IrOp::Ei { span }
        | IrOp::Ldir { span, .. }
        | IrOp::InlineAsm { span, .. }
        | IrOp::Branch { span, .. }
        | IrOp::Jump { span, .. }
        | IrOp::Return { span, .. } => *span,
    }
}
