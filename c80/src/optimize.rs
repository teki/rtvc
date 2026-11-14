//! Provenance-preserving peephole and layout-sensitive branch shortening.

use super::diagnostic::{DiagCode, Diagnostic};
use super::liveness::{ALL_FLAGS, FC, FHN, FPV, Res, labels, live_after, r, rr};
use super::lower::EmitChunk;
use super::source::{FileId, SourceSpan};
use super::z80::{AluSrc, Cc, R8, Rr, Z80Item, Z80Op, render_items};
use rtvc_core::asm::assemble_program;
use std::collections::HashSet;

const MAX_JR_PASSES: usize = 256;

/// Optimize in place. Returns the assembly of the final chunks when the
/// last layout pass already produced it (callers can skip reassembling).
///
/// `externals` are labels defined by other units of a project (F-030). The
/// unit is assembled alone, so they are given placeholder values: their
/// operands have the same size whatever the value, and a jump to one is
/// never shortened (its real distance is unknown here).
pub(crate) fn optimize_chunks(
    chunks: &mut [EmitChunk],
    origin: u16,
    externals: &[String],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<rtvc_core::asm::AssembledProgram> {
    // Names each chunk mentions, so a label another chunk (or any inline
    // `asm` or data text) can reach is never treated as straight-line code.
    let mentions: Vec<HashSet<String>> = chunks
        .iter_mut()
        .map(|c| mentioned_names(c.items_mut()))
        .collect();
    for (k, chunk) in chunks.iter_mut().enumerate() {
        let ret = chunk
            .ret_regs()
            .into_iter()
            .fold(0, |m, x| m | super::liveness::r(x));
        let outside: HashSet<String> = mentions
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != k)
            .flat_map(|(_, m)| m.iter().cloned())
            .collect();
        strip_redundant(chunk.items_mut(), ret, &outside);
        if rotate_loops(chunk.items_mut()) {
            strip_redundant(chunk.items_mut(), ret, &outside);
        }
        if use_djnz(chunk.items_mut(), ret, &outside) {
            strip_redundant(chunk.items_mut(), ret, &outside);
        }
        if chunk.allows_tail_calls() {
            for target in tail_calls(chunk.items_mut()) {
                chunk.record_tail_call(target);
            }
        }
    }
    shorten_jumps(chunks, origin, externals, diagnostics)
}

/// Every identifier-like token in a chunk's code, data and inline `asm`.
fn mentioned_names(items: &[Z80Item]) -> HashSet<String> {
    let mut out = HashSet::new();
    for item in items {
        let text = match item {
            Z80Item::Instruction { op, .. } => op.render(),
            Z80Item::Raw { text, .. } | Z80Item::Data { text, .. } => text.clone(),
            Z80Item::Directive { .. } => format!("{item:?}"),
            Z80Item::Label { .. } => continue,
        };
        for tok in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.')) {
            if !tok.is_empty() {
                out.insert(tok.to_string());
            }
        }
    }
    out
}

fn strip_redundant(items: &mut Vec<Z80Item>, ret: Res, outside: &HashSet<String>) {
    for _ in 0..64 {
        let before = items.len();
        items.retain(|item| !is_identity_move(item));
        peephole(items, ret);
        track_registers(items, outside);
        invert_cond_skip(items);
        thread_jumps(items);
        remove_unreachable(items, outside);
        remove_fallthrough_jumps(items);
        if items.len() == before && !peephole(items, ret) && !track_registers(items, outside) {
            break;
        }
    }
}

fn op_at(items: &[Z80Item], i: usize) -> Option<&Z80Op> {
    match items.get(i) {
        Some(Z80Item::Instruction { op, .. }) => Some(op),
        _ => None,
    }
}

fn set_op(items: &mut [Z80Item], i: usize, new: Z80Op) {
    if let Some(Z80Item::Instruction { op, .. }) = items.get_mut(i) {
        *op = new;
    }
}

/// Flag- and register-aware rewrites (H5 item 2). Each one is checked
/// against [`live_after`]: a rewrite that changes a flag or leaves a
/// register different happens only when nothing reads it afterwards.
/// Returns whether anything changed. Memory and port accesses are never
/// touched.
fn peephole(items: &mut Vec<Z80Item>, ret: Res) -> bool {
    let mut changed = false;
    let mut i = 0;
    let mut label_pos = labels(items);
    let mut stale = false;
    while i < items.len() {
        if stale {
            label_pos = labels(items);
            stale = false;
        }
        let labels = &label_pos;
        let dead =
            |items: &[Z80Item], k: usize, res: Res| live_after(items, labels, ret, k, res) == 0;
        let Some(op) = op_at(items, i).cloned() else {
            i += 1;
            continue;
        };
        let next = |k: usize| op_at(items, i + k).cloned();
        // Word add of an immediate with a zero low byte, as emitted by the
        // byte-wise word add: `LD A,lo / ADD A,0 / LD lo,A / LD A,hi /
        // ADC A,x / LD hi,A` is `LD A,hi / ADD A,x / LD hi,A`. The low half
        // is unchanged and no carry comes out of it, so A, the pair and
        // every flag end the same.
        if let (
            Z80Op::Ld8 {
                dst: R8::A,
                src: lo,
            },
            Some(Z80Op::AddA(AluSrc::Imm(0)) | Z80Op::Sub(AluSrc::Imm(0))),
            Some(Z80Op::Ld8 {
                dst: lo2,
                src: R8::A,
            }),
            Some(Z80Op::Ld8 {
                dst: R8::A,
                src: hi,
            }),
            Some(Z80Op::AdcA(x) | Z80Op::SbcA(x)),
            Some(Z80Op::Ld8 {
                dst: hi2,
                src: R8::A,
            }),
        ) = (&op, next(1), next(2), next(3), next(4), next(5))
        {
            if *lo == lo2 && hi == hi2 && *lo != R8::A && hi != R8::A {
                let add = matches!(next(1), Some(Z80Op::AddA(_)));
                set_op(
                    items,
                    i + 4,
                    if add { Z80Op::AddA(x) } else { Z80Op::Sub(x) },
                );
                items.drain(i..i + 3);
                changed = true;
                stale = true;
                continue;
            }
        }
        // `LD A,r / ADD A,1 (INC A) / LD r,A` is `INC r` when A and carry
        // are dead after (and DEC likewise).
        if let (
            Z80Op::Ld8 {
                dst: R8::A,
                src: reg,
            },
            Some(step),
            Some(Z80Op::Ld8 { dst, src: R8::A }),
        ) = (&op, next(1), next(2))
        {
            let inc = match step {
                Z80Op::AddA(AluSrc::Imm(1)) | Z80Op::Inc8(R8::A) => Some(true),
                Z80Op::Sub(AluSrc::Imm(1)) | Z80Op::Dec8(R8::A) => Some(false),
                _ => None,
            };
            let carry = matches!(step, Z80Op::AddA(_) | Z80Op::Sub(_));
            if let Some(inc) = inc {
                let need = r(R8::A) | if carry { FC } else { 0 };
                if dst == *reg && *reg != R8::A && dead(items, i + 2, need) {
                    set_op(
                        items,
                        i,
                        if inc {
                            Z80Op::Inc8(*reg)
                        } else {
                            Z80Op::Dec8(*reg)
                        },
                    );
                    items.drain(i + 1..i + 3);
                    changed = true;
                    stale = true;
                    continue;
                }
            }
        }
        // `LD D,H / LD E,L / ADD HL,DE` doubles HL: `ADD HL,HL`, same flags,
        // when the copy in DE is not needed afterwards.
        if let (
            Z80Op::Ld8 { dst: h, src: R8::H },
            Some(Z80Op::Ld8 { dst: l, src: R8::L }),
            Some(Z80Op::AddHl(pair)),
        ) = (&op, next(1), next(2))
        {
            if pair != Rr::Hl && pair.halves() == (*h, l) && dead(items, i + 2, rr(pair)) {
                set_op(items, i + 2, Z80Op::AddHl(Rr::Hl));
                items.drain(i..i + 2);
                changed = true;
                stale = true;
                continue;
            }
        }
        let rewrite = match &op {
            // Same Z, S and carry (0); P/V and N differ.
            Z80Op::Cp(AluSrc::Imm(0)) if dead(items, i, FPV | FHN) => {
                Some(Z80Op::Or(AluSrc::Reg(R8::A)))
            }
            // INC/DEC leave carry alone.
            Z80Op::AddA(AluSrc::Imm(1)) if dead(items, i, FC) => Some(Z80Op::Inc8(R8::A)),
            Z80Op::Sub(AluSrc::Imm(1)) if dead(items, i, FC) => Some(Z80Op::Dec8(R8::A)),
            Z80Op::Ld8Imm { dst: R8::A, imm: 0 } if dead(items, i, ALL_FLAGS) => {
                Some(Z80Op::Xor(AluSrc::Reg(R8::A)))
            }
            _ => None,
        };
        // A register load whose destination nobody reads (loads change no
        // flag; memory loads are never removed here).
        let dead_dst = match &op {
            Z80Op::Ld8 { dst, .. } | Z80Op::Ld8Imm { dst, .. } => Some(r(*dst)),
            Z80Op::Ld16Imm { dst, .. } | Z80Op::Ld16Sym { dst, .. } => Some(rr(*dst)),
            _ => None,
        };
        if let Some(mask) = dead_dst {
            if dead(items, i, mask) {
                items.remove(i);
                changed = true;
                stale = true;
                continue;
            }
        }
        if let Some(new) = rewrite {
            set_op(items, i, new);
            changed = true;
        } else if matches!(op, Z80Op::AddA(AluSrc::Imm(0)) | Z80Op::Sub(AluSrc::Imm(0)))
            && dead(items, i, ALL_FLAGS)
        {
            items.remove(i);
            changed = true;
            stale = true;
            continue;
        }
        i += 1;
    }
    changed
}

fn is_identity_move(item: &Z80Item) -> bool {
    matches!(
        item,
        Z80Item::Instruction {
            op: Z80Op::Ld8 { dst, src },
            ..
        } if dst == src
    )
}

fn invert_cond_skip(items: &mut Vec<Z80Item>) {
    let mut i = 0;
    while i + 1 < items.len() {
        let pair = match (&items[i], &items[i + 1]) {
            (
                Z80Item::Instruction {
                    op:
                        Z80Op::Jp {
                            cc: Some(cc),
                            target: taken,
                        },
                    ..
                },
                Z80Item::Instruction {
                    op:
                        Z80Op::Jp {
                            cc: None,
                            target: skipped,
                        },
                    ..
                },
            ) => Some((*cc, taken.clone(), skipped.clone())),
            _ => None,
        };
        if let Some((cc, taken, skipped)) = pair {
            if let Some(inv) = cc.invert() {
                if is_fallthrough_target(items, i + 2, &taken) {
                    if let Z80Item::Instruction { op, .. } = &mut items[i] {
                        *op = Z80Op::Jp {
                            cc: Some(inv),
                            target: skipped,
                        };
                    }
                    items.remove(i + 1);
                    continue;
                }
            }
        }
        i += 1;
    }
}

/// The label's first instruction, skipping labels and listing directives.
fn first_op_after<'a>(items: &'a [Z80Item], label: &str) -> Option<&'a Z80Op> {
    let at = items
        .iter()
        .position(|it| matches!(it, Z80Item::Label { name, .. } if name == label))?;
    items[at + 1..].iter().find_map(|it| match it {
        Z80Item::Label { .. } | Z80Item::Directive { .. } => None,
        Z80Item::Instruction { op, .. } => Some(Some(op)),
        _ => Some(None),
    })?
}

/// A `JP` (conditional or not) to a label whose code is an unconditional
/// `JP y` goes to `y` directly (H5). Only `JP`: a `JR`/`DJNZ` keeps its
/// target, which stays in range.
fn thread_jumps(items: &mut [Z80Item]) {
    for i in 0..items.len() {
        let Z80Item::Instruction {
            op: Z80Op::Jp { target, .. },
            ..
        } = &items[i]
        else {
            continue;
        };
        let mut to = target.clone();
        let mut seen = vec![to.clone()];
        while let Some(
            Z80Op::Jp {
                cc: None,
                target: next,
            }
            | Z80Op::Jr {
                cc: None,
                target: next,
            },
        ) = first_op_after(items, &to)
        {
            if seen.contains(next) {
                break;
            }
            to = next.clone();
            seen.push(to.clone());
        }
        if let Z80Item::Instruction {
            op: Z80Op::Jp { target, .. },
            ..
        } = &mut items[i]
        {
            if *target != to && seen.len() > 1 {
                *target = to;
            }
        }
    }
}

/// Code after an unconditional `JP`/`JR`/`RET` up to the next label that
/// anything names (here, in another chunk, or in `asm`/data text) never
/// runs (H5: blocks left empty by jump threading). Stops at inline `asm`
/// and data.
fn remove_unreachable(items: &mut Vec<Z80Item>, outside: &HashSet<String>) {
    let here = mentioned_names(items);
    let mut dead = false;
    let mut i = 0;
    while i < items.len() {
        match &items[i] {
            Z80Item::Label { name, .. } => {
                if here.contains(name) || outside.contains(name) {
                    dead = false;
                }
            }
            Z80Item::Raw { .. } | Z80Item::Data { .. } => dead = false,
            Z80Item::Directive { .. } => {}
            Z80Item::Instruction { op, .. } => {
                if dead {
                    items.remove(i);
                    continue;
                }
                dead = matches!(
                    op,
                    Z80Op::Ret | Z80Op::Jp { cc: None, .. } | Z80Op::Jr { cc: None, .. }
                );
            }
        }
        i += 1;
    }
}

/// `CALL x` directly followed by `RET` is `JP x` (H5): the callee returns
/// to this function's caller. Only when nothing sits between them (no
/// cleanup, no label). Returns the callees, for stack-bound accounting.
fn tail_calls(items: &mut Vec<Z80Item>) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < items.len() {
        let target = match (&items[i], &items[i + 1]) {
            (
                Z80Item::Instruction {
                    op: Z80Op::Call { target, .. },
                    ..
                },
                Z80Item::Instruction { op: Z80Op::Ret, .. },
            ) => target.clone(),
            _ => {
                i += 1;
                continue;
            }
        };
        if let Z80Item::Instruction { op, .. } = &mut items[i] {
            *op = Z80Op::Jp {
                cc: None,
                target: target.clone(),
            };
        }
        items.remove(i + 1);
        out.push(target);
        i += 1;
    }
    out
}

/// Loop rotation (H5): a loop laid out as
///
/// ```text
/// top:  T            ; straight-line test
///       JP cc,exit
/// body: B
///       JP top
/// exit:
/// ```
///
/// becomes `JR top / body: B / top: T / JP !cc,body / exit:`, one
/// conditional branch per pass and no back `JP`. The back jump itself moves
/// to the entry (dropped when nothing falls into the loop). Returns whether
/// anything changed.
fn rotate_loops(items: &mut Vec<Z80Item>) -> bool {
    let mut changed = false;
    let mut t = 0;
    while t < items.len() {
        if let Some(new) = rotate_at(items, t) {
            *items = new;
            changed = true;
        }
        t += 1;
    }
    changed
}

fn rotate_at(items: &[Z80Item], t: usize) -> Option<Vec<Z80Item>> {
    let Z80Item::Label { name: top, .. } = &items[t] else {
        return None;
    };
    // T: instructions only, no jumps, ending in `JP cc,exit`.
    let mut c = t + 1;
    loop {
        match items.get(c)? {
            Z80Item::Instruction {
                op: Z80Op::Jp { cc: Some(_), .. },
                ..
            } => break,
            Z80Item::Instruction { op, .. } if !is_jump(op) && !matches!(op, Z80Op::Ret) => {}
            Z80Item::Directive { .. } => {}
            _ => return None,
        }
        c += 1;
    }
    let Z80Item::Instruction {
        op: Z80Op::Jp {
            cc: Some(cc),
            target: exit,
        },
        ..
    } = &items[c]
    else {
        return None;
    };
    let inv = cc.invert()?;
    // The body starts with its own label.
    let Z80Item::Label { name: body, .. } = items.get(c + 1)? else {
        return None;
    };
    // The back jump: the first `JP top` after the body label that is
    // directly followed by the exit label (labels in between are fine).
    let j = (c + 2..items.len()).find(|&j| {
        matches!(&items[j], Z80Item::Instruction { op: Z80Op::Jp { cc: None, target }, .. } if target == top)
            && items[j + 1..]
                .iter()
                .take_while(|it| matches!(it, Z80Item::Label { .. } | Z80Item::Directive { .. }))
                .any(|it| matches!(it, Z80Item::Label { name, .. } if name == exit))
    })?;
    // Nothing in the body may be inline `asm` or data (sizes unknown).
    let mut body_bytes = 0u32;
    for it in &items[c + 1..j] {
        match it {
            Z80Item::Instruction { op, .. } => body_bytes += op.size_bytes(),
            Z80Item::Raw { .. } | Z80Item::Data { .. } => return None,
            _ => {}
        }
    }
    // Does code fall into `top`?
    let falls_in = items[..t]
        .iter()
        .rev()
        .find(|it| it.emits_bytes())
        .is_some_and(|it| {
            !matches!(
                it,
                Z80Item::Instruction {
                    op: Z80Op::Ret | Z80Op::Jp { cc: None, .. } | Z80Op::Jr { cc: None, .. },
                    ..
                }
            )
        });
    let mut entry = items[j].clone();
    if body_bytes <= 120 {
        if let Z80Item::Instruction { op, .. } = &mut entry {
            *op = Z80Op::Jr {
                cc: None,
                target: top.clone(),
            };
        }
    }
    let mut test_jump = items[c].clone();
    if let Z80Item::Instruction { op, .. } = &mut test_jump {
        *op = Z80Op::Jp {
            cc: Some(inv),
            target: body.clone(),
        };
    }
    let mut out = Vec::with_capacity(items.len());
    out.extend_from_slice(&items[..t]);
    if falls_in {
        out.push(entry);
    }
    out.extend_from_slice(&items[c + 1..j]); // body (with its label)
    out.extend_from_slice(&items[t..c]); // top label and T
    out.push(test_jump);
    out.extend_from_slice(&items[j + 1..]);
    Some(out)
}

/// `DJNZ` (H5 item 7) for a counter loop on B whose test is the source's
/// own `B != 0` guard:
///
/// - do-while: `DEC B / LD A,B / OR A / JP NZ,body` -> `DJNZ body`;
/// - rotated while: `JR top / body: X / DEC B / top: LD A,B / OR A /
///   JP NZ,body / exit:` -> `LD A,B / OR A / JP Z,exit / body: X /
///   DJNZ body / exit:` (the guard now runs once, at the entry, so a zero
///   count still skips the loop).
///
/// Only when A and every flag are dead after the `JP NZ` (on both paths:
/// `DJNZ` leaves them as the body left them), `top` has no other entrant,
/// and the body (no `asm` or data) fits a relative jump. Returns whether
/// anything changed.
fn use_djnz(items: &mut Vec<Z80Item>, ret: Res, outside: &HashSet<String>) -> bool {
    let mut changed = false;
    let mut k = 0;
    while k + 3 < items.len() {
        if let Some(new) = djnz_at(items, ret, outside, k) {
            *items = new;
            changed = true;
        }
        k += 1;
    }
    changed
}

fn djnz_at(
    items: &[Z80Item],
    ret: Res,
    outside: &HashSet<String>,
    d: usize,
) -> Option<Vec<Z80Item>> {
    let op = |i: usize| match items.get(i) {
        Some(Z80Item::Instruction { op, .. }) => Some(op),
        _ => None,
    };
    if op(d)? != &Z80Op::Dec8(R8::B) {
        return None;
    }
    // An optional `top:` label between `DEC B` and the test.
    let (top, t) = match items.get(d + 1)? {
        Z80Item::Label { name, .. } => (Some(name.clone()), d + 2),
        _ => (None, d + 1),
    };
    if op(t)?
        != &(Z80Op::Ld8 {
            dst: R8::A,
            src: R8::B,
        })
        || op(t + 1)? != &Z80Op::Or(AluSrc::Reg(R8::A))
    {
        return None;
    }
    let Z80Op::Jp {
        cc: Some(Cc::Nz),
        target: body,
    } = op(t + 2)?
    else {
        return None;
    };
    let label_pos = labels(items);
    if live_after(items, &label_pos, ret, t + 2, r(R8::A) | ALL_FLAGS) != 0 {
        return None;
    }
    // Backward, in range, plain code.
    let b = *label_pos.get(body)?;
    if b > d {
        return None;
    }
    let mut bytes = 0u32;
    for it in &items[b..d] {
        match it {
            Z80Item::Instruction { op, .. } => bytes += op.size_bytes(),
            Z80Item::Raw { .. } | Z80Item::Data { .. } => return None,
            _ => {}
        }
    }
    if bytes > 120 {
        return None;
    }
    let mut djnz = items[d].clone();
    if let Z80Item::Instruction { op, .. } = &mut djnz {
        *op = Z80Op::Djnz {
            target: body.clone(),
        };
    }
    let Some(top) = top else {
        // do-while: the test follows the decrement directly.
        let mut out = items[..d].to_vec();
        out.push(djnz);
        out.extend_from_slice(&items[t + 3..]);
        return Some(out);
    };
    // Rotated while: the only other way to `top` is the entry `JR top`
    // right before the body label.
    if outside.contains(&top) {
        return None;
    }
    let entrants: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, it)| match it {
            Z80Item::Instruction { op, .. } => op
                .render()
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
                .any(|w| w == top),
            Z80Item::Raw { text, .. } | Z80Item::Data { text, .. } => text.contains(top.as_str()),
            _ => false,
        })
        .map(|(i, _)| i)
        .collect();
    let [entry] = entrants[..] else {
        return None;
    };
    if entry + 1 != b
        || !matches!(
            op(entry)?,
            Z80Op::Jr { cc: None, .. } | Z80Op::Jp { cc: None, .. }
        )
    {
        return None;
    }
    let Z80Item::Label { name: exit, .. } = items.get(t + 3)? else {
        return None;
    };
    let mut guard = items[t + 2].clone();
    if let Z80Item::Instruction { op, .. } = &mut guard {
        *op = Z80Op::Jp {
            cc: Some(Cc::Z),
            target: exit.clone(),
        };
    }
    let mut out = items[..entry].to_vec();
    out.push(items[t].clone());
    out.push(items[t + 1].clone());
    out.push(guard);
    out.extend_from_slice(&items[b..d]);
    out.push(djnz);
    out.push(items[d + 1].clone()); // `top:` (now unreferenced)
    out.extend_from_slice(&items[t + 3..]);
    let _ = entry;
    Some(out)
}

fn is_jump(op: &Z80Op) -> bool {
    matches!(
        op,
        Z80Op::Jp { .. } | Z80Op::Jr { .. } | Z80Op::Djnz { .. } | Z80Op::Call { .. }
    )
}

fn remove_fallthrough_jumps(items: &mut Vec<Z80Item>) {
    let mut i = 0;
    while i < items.len() {
        let target = match &items[i] {
            Z80Item::Instruction {
                op: Z80Op::Jp { cc: None, target } | Z80Op::Jr { cc: None, target },
                ..
            } => Some(target.clone()),
            _ => None,
        };
        if let Some(target) = target {
            if is_fallthrough_target(items, i + 1, &target) {
                items.remove(i);
                continue;
            }
        }
        i += 1;
    }
}

fn is_fallthrough_target(items: &[Z80Item], from: usize, target: &str) -> bool {
    for item in &items[from..] {
        match item {
            Z80Item::Label { name, .. } if name == target => return true,
            Z80Item::Label { .. } | Z80Item::Directive { .. } => {}
            other if other.emits_bytes() => return false,
            _ => {}
        }
    }
    false
}

/// Forward conditional `JP` that fit become `JR`. Converting a jump only
/// shortens the distances of the others, so every jump that fits under the
/// current layout is converted at once and the layout is recomputed until no
/// more fit (the same fixed point as converting them one by one, with a few
/// assembler runs instead of two per jump).
fn shorten_jumps(
    chunks: &mut [EmitChunk],
    origin: u16,
    externals: &[String],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<rtvc_core::asm::AssembledProgram> {
    // The jumps converted by the previous pass; the next assembler run
    // checks them.
    let mut last: Vec<((usize, usize), Z80Op)> = Vec::new();
    for pass in 0..=MAX_JR_PASSES {
        let assembled = match assemble_current(chunks, origin, externals) {
            Ok(assembled) => assembled,
            Err(_) => {
                if !last.is_empty() {
                    for ((ci, ii), snapshot) in last {
                        if let Z80Item::Instruction { op, .. } = &mut chunks[ci].items_mut()[ii] {
                            *op = snapshot;
                        }
                    }
                    diagnostics.push(Diagnostic::error(
                        DiagCode::CgInternal,
                        chunk_span(chunks),
                        "JR conversion was rejected by the assembler",
                    ));
                }
                return None;
            }
        };
        if pass == MAX_JR_PASSES {
            return Some(assembled);
        }
        let addrs = emitting_addrs(chunks, &assembled)?;
        last.clear();
        for ((ci, ii), addr) in addrs {
            let (cc, target) = match chunks.get(ci).and_then(|c| c.items().get(ii)) {
                Some(Z80Item::Instruction {
                    op:
                        Z80Op::Jp {
                            cc: Some(cc),
                            target,
                        },
                    ..
                }) => (*cc, target.clone()),
                _ => continue,
            };
            if !cc.jr_ok() || externals.contains(&target) {
                continue;
            }
            let Some(&target_addr) = assembled.symbols.get(&target) else {
                continue;
            };
            let adj_target = adjust_target_for_jp_to_jr(addr, target_addr);
            if !is_forward(addr, adj_target) || !jr_fits(addr, adj_target) {
                continue;
            }
            if let Z80Item::Instruction { op, .. } = &mut chunks[ci].items_mut()[ii] {
                let snapshot = op.clone();
                *op = Z80Op::Jr {
                    cc: Some(cc),
                    target: target.clone(),
                };
                last.push(((ci, ii), snapshot));
            }
        }
        if last.is_empty() {
            return Some(assembled);
        }
    }
    None
}

fn assemble_current(
    chunks: &[EmitChunk],
    origin: u16,
    externals: &[String],
) -> Result<rtvc_core::asm::AssembledProgram, rtvc_core::asm::AsmError> {
    let mut items = Vec::new();
    for chunk in chunks {
        items.extend(chunk.items().iter().cloned());
    }
    let mut text = render_items(&items);
    // `EQU` lines emit no bytes, so the line/address pairing is unchanged.
    for label in externals {
        text.push_str(label);
        text.push_str(" EQU 0\n");
    }
    assemble_program(&text, origin)
}

fn emitting_addrs(
    chunks: &[EmitChunk],
    assembled: &rtvc_core::asm::AssembledProgram,
) -> Option<Vec<((usize, usize), u16)>> {
    let mut out = Vec::new();
    let mut line_i = 0usize;
    for (ci, chunk) in chunks.iter().enumerate() {
        for (ii, item) in chunk.items().iter().enumerate() {
            if !item.emits_bytes() {
                continue;
            }
            let line = assembled.lines.get(line_i)?;
            out.push(((ci, ii), line.addr));
            line_i += 1;
        }
    }
    if line_i != assembled.lines.len() {
        return None;
    }
    Some(out)
}

fn is_forward(from: u16, target: u16) -> bool {
    let distance = target.wrapping_sub(from);
    (3..0x8000).contains(&distance)
}

fn adjust_target_for_jp_to_jr(from: u16, target: u16) -> u16 {
    let distance = target.wrapping_sub(from);
    if (3..0x8000).contains(&distance) {
        target.wrapping_sub(1)
    } else {
        target
    }
}

fn jr_fits(pc: u16, target: u16) -> bool {
    let next = pc.wrapping_add(2);
    let displacement = target.wrapping_sub(next) as i16;
    (-128..=127).contains(&displacement)
}

fn chunk_span(chunks: &[EmitChunk]) -> SourceSpan {
    chunks
        .first()
        .map(|c| c.span())
        .unwrap_or_else(|| SourceSpan::point(FileId(0), 0))
}

/// A compiler-private static slot (`FN.._NAME_S`, `FN.._NAME_S+n`): never
/// source-visible, so a load from it may be replaced by a register that
/// already holds the stored value. Any other symbol is left alone.
fn slot_key(symbol: &str) -> Option<(String, u16)> {
    let (base, off) = match symbol.split_once('+') {
        Some((b, o)) => (b, o.parse::<u16>().ok()?),
        None => (symbol, 0),
    };
    let ok = base.starts_with("FN")
        && base.ends_with("_S")
        && base[2..]
            .split('_')
            .next()
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()));
    ok.then(|| (base.to_string(), off))
}

/// Register-content tracking within straight-line code (H5 item 3). Each
/// register carries a value number; a load that would not change its
/// destination is dropped, and a reload of a compiler slot whose value is
/// still in a register becomes a register move. Loads never change flags,
/// so dropping one is flag-safe. Loads and stores of globals, of memory
/// through pointers, and port I/O are never removed. Knowledge is reset at
/// every label and after every unconditional jump; slot knowledge is also
/// dropped at any store through a pointer, call, `LDIR` or inline `asm`
/// (a slot whose address was taken can change there).
fn track_registers(items: &mut Vec<Z80Item>, outside: &HashSet<String>) -> bool {
    use std::collections::HashMap;
    let mut changed = false;
    let mut next_id = 0u32;
    let mut fresh = || {
        next_id += 1;
        next_id
    };
    let mut consts: HashMap<(u8, bool), u32> = HashMap::new();
    let mut syms: HashMap<(String, bool), u32> = HashMap::new();
    let mut val = [0u32; 7];
    let mut slots: HashMap<(String, u16), u32> = HashMap::new();
    let idx = |reg: R8| R8::ALL.iter().position(|&x| x == reg).unwrap();
    let reset = |val: &mut [u32; 7],
                 slots: &mut HashMap<(String, u16), u32>,
                 fresh: &mut dyn FnMut() -> u32| {
        for v in val.iter_mut() {
            *v = fresh();
        }
        slots.clear();
    };
    reset(&mut val, &mut slots, &mut fresh);
    // Only a label something can reach merges paths; a block label nothing
    // names (here, in another chunk, or in any inline `asm`/data text) is
    // straight-line code. The first label is the entry and always resets.
    let here = mentioned_names(items);
    let mut seen_label = false;
    let mut i = 0;
    while i < items.len() {
        let op = match &items[i] {
            Z80Item::Instruction { op, .. } => op.clone(),
            Z80Item::Label { name, .. }
                if std::mem::replace(&mut seen_label, true)
                    && !here.contains(name)
                    && !outside.contains(name) =>
            {
                i += 1;
                continue;
            }
            Z80Item::Label { .. } | Z80Item::Raw { .. } | Z80Item::Data { .. } => {
                reset(&mut val, &mut slots, &mut fresh);
                i += 1;
                continue;
            }
            Z80Item::Directive { .. } => {
                i += 1;
                continue;
            }
        };
        let holder = |val: &[u32; 7], id: u32| R8::ALL.into_iter().find(|&r| val[idx(r)] == id);
        let mut drop_it = false;
        let mut replace: Option<Vec<Z80Op>> = None;
        match &op {
            Z80Op::Ld8 { dst, src } => {
                if val[idx(*dst)] == val[idx(*src)] {
                    drop_it = true;
                } else {
                    val[idx(*dst)] = val[idx(*src)];
                }
            }
            Z80Op::Ld8Imm { dst, imm } => {
                let id = *consts.entry((*imm, false)).or_insert_with(&mut fresh);
                if val[idx(*dst)] == id {
                    drop_it = true;
                } else {
                    val[idx(*dst)] = id;
                }
            }
            Z80Op::Ld16Imm { dst, imm } => {
                let (h, l) = dst.halves();
                let hid = *consts
                    .entry(((imm >> 8) as u8, false))
                    .or_insert_with(&mut fresh);
                let lid = *consts.entry((*imm as u8, false)).or_insert_with(&mut fresh);
                if val[idx(h)] == hid && val[idx(l)] == lid {
                    drop_it = true;
                } else {
                    val[idx(h)] = hid;
                    val[idx(l)] = lid;
                }
            }
            Z80Op::Ld16Sym { dst, symbol } => {
                let (h, l) = dst.halves();
                let hid = *syms
                    .entry((symbol.clone(), true))
                    .or_insert_with(&mut fresh);
                let lid = *syms
                    .entry((symbol.clone(), false))
                    .or_insert_with(&mut fresh);
                if val[idx(h)] == hid && val[idx(l)] == lid {
                    drop_it = true;
                } else {
                    val[idx(h)] = hid;
                    val[idx(l)] = lid;
                }
            }
            Z80Op::StAbs8 { src, symbol } => {
                if let Some(k) = slot_key(symbol) {
                    slots.insert(k, val[idx(*src)]);
                }
            }
            Z80Op::StAbs16 { src, symbol } => {
                if let Some((b, o)) = slot_key(symbol) {
                    let (h, l) = src.halves();
                    slots.insert((b.clone(), o), val[idx(l)]);
                    slots.insert((b, o + 1), val[idx(h)]);
                }
            }
            Z80Op::LdAbs8 { dst, symbol } => {
                match slot_key(symbol).and_then(|k| slots.get(&k).copied()) {
                    Some(id) if val[idx(*dst)] == id => drop_it = true,
                    Some(id) => match holder(&val, id) {
                        Some(r) => {
                            replace = Some(vec![Z80Op::Ld8 { dst: *dst, src: r }]);
                            val[idx(*dst)] = id;
                        }
                        None => val[idx(*dst)] = id,
                    },
                    None => {
                        let id = fresh();
                        val[idx(*dst)] = id;
                        if let Some(k) = slot_key(symbol) {
                            slots.insert(k, id);
                        }
                    }
                }
            }
            Z80Op::LdAbs16 { dst, symbol } => {
                let (h, l) = dst.halves();
                let key = slot_key(symbol);
                let known = key.as_ref().and_then(|(b, o)| {
                    Some((
                        *slots.get(&(b.clone(), *o))?,
                        *slots.get(&(b.clone(), o + 1))?,
                    ))
                });
                match known {
                    Some((lid, hid)) if val[idx(l)] == lid && val[idx(h)] == hid => drop_it = true,
                    Some((lid, hid)) => {
                        let rl = holder(&val, lid);
                        let rh = holder(&val, hid);
                        if let (Some(rl), Some(rh)) = (rl, rh) {
                            // Order the two moves so neither reads a half the
                            // other has just written.
                            let mut moves = Vec::new();
                            if rh == l && rl == h {
                                moves.clear();
                            } else if rh == l {
                                if val[idx(h)] != hid {
                                    moves.push(Z80Op::Ld8 { dst: h, src: rh });
                                }
                                if val[idx(l)] != lid {
                                    moves.push(Z80Op::Ld8 { dst: l, src: rl });
                                }
                            } else {
                                if val[idx(l)] != lid {
                                    moves.push(Z80Op::Ld8 { dst: l, src: rl });
                                }
                                if val[idx(h)] != hid {
                                    moves.push(Z80Op::Ld8 { dst: h, src: rh });
                                }
                            }
                            if !moves.is_empty() && !(rh == l && rl == h) {
                                replace = Some(moves);
                            }
                        }
                        val[idx(l)] = lid;
                        val[idx(h)] = hid;
                    }
                    None => {
                        let (lid, hid) = (fresh(), fresh());
                        val[idx(l)] = lid;
                        val[idx(h)] = hid;
                        if let Some((b, o)) = key {
                            slots.insert((b.clone(), o), lid);
                            slots.insert((b, o + 1), hid);
                        }
                    }
                }
            }
            Z80Op::ExDeHl => {
                val.swap(idx(R8::D), idx(R8::H));
                val.swap(idx(R8::E), idx(R8::L));
            }
            // `OR A` / `AND A` test A without changing it.
            Z80Op::Or(AluSrc::Reg(R8::A)) | Z80Op::And(AluSrc::Reg(R8::A)) | Z80Op::Cp(_) => {}
            other => {
                let (_, writes) = super::liveness::effects(other);
                for reg in R8::ALL {
                    if writes & r(reg) != 0 {
                        val[idx(reg)] = fresh();
                    }
                }
                if matches!(
                    other,
                    Z80Op::StHl(_)
                        | Z80Op::StIndA(_)
                        | Z80Op::St8Ix { .. }
                        | Z80Op::Ld8IxImm { .. }
                        | Z80Op::Ldir
                        | Z80Op::Call { .. }
                ) {
                    slots.clear();
                }
                if matches!(
                    other,
                    Z80Op::Ret | Z80Op::Jp { cc: None, .. } | Z80Op::Jr { cc: None, .. }
                ) {
                    reset(&mut val, &mut slots, &mut fresh);
                }
            }
        }
        if drop_it {
            items.remove(i);
            changed = true;
            continue;
        }
        if let Some(moves) = replace {
            // The state above already accounts for every inserted move:
            // step past all of them.
            let n = moves.len();
            let template = items[i].clone();
            items.remove(i);
            for (k, m) in moves.into_iter().enumerate() {
                let mut item = template.clone();
                if let Z80Item::Instruction { op, .. } = &mut item {
                    *op = m;
                }
                items.insert(i + k, item);
            }
            changed = true;
            i += n;
            continue;
        }
        i += 1;
    }
    changed
}

#[cfg(test)]
mod tests {
    //! One case per peephole with the flag (or register) it changes still
    //! read afterwards (must not fire) and dead (fires).
    use super::*;
    use crate::liveness::ALL_REGS;
    use crate::source::NodeId;
    use crate::z80::{AsmInstructionId, Cc};

    fn ins(op: Z80Op) -> Z80Item {
        Z80Item::Instruction {
            id: AsmInstructionId(0),
            op,
            span: SourceSpan::point(FileId(0), 0),
            expr: None,
            synthetic: false,
            node: None,
            note: None,
        }
    }

    fn label(name: &str) -> Z80Item {
        Z80Item::Label {
            id: AsmInstructionId(0),
            name: name.to_string(),
            span: SourceSpan::point(FileId(0), 0),
            node: NodeId(0),
        }
    }

    fn ops(items: &[Z80Item]) -> Vec<Z80Op> {
        items
            .iter()
            .filter_map(|i| match i {
                Z80Item::Instruction { op, .. } => Some(op.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_jump_to_a_jump_is_threaded() {
        let mut items = vec![
            label("f"),
            ins(jp(Some(Cc::Z), "a")),
            ins(Z80Op::Ret),
            label("a"),
            ins(jp(None, "b")),
            label("b"),
            ins(Z80Op::Cpl),
            ins(Z80Op::Ret),
        ];
        thread_jumps(&mut items);
        assert_eq!(ops(&items)[0], jp(Some(Cc::Z), "b"));
        // A jump cycle is left alone.
        let mut cycle = vec![label("f"), label("a"), ins(jp(None, "a"))];
        thread_jumps(&mut cycle);
        assert_eq!(ops(&cycle), [jp(None, "a")]);
    }

    #[test]
    fn code_after_a_jump_dies_up_to_a_named_label() {
        let mut items = vec![
            label("f"),
            ins(jp(Some(Cc::Z), "b")),
            ins(Z80Op::Ret),
            label("a"), // nothing names it: dead
            ins(jp(None, "b")),
            label("b"),
            ins(Z80Op::Cpl),
            ins(Z80Op::Ret),
            label("c"), // named in another chunk: kept
            ins(Z80Op::Neg),
            ins(Z80Op::Ret),
        ];
        let outside: HashSet<String> = ["c".to_string()].into_iter().collect();
        remove_unreachable(&mut items, &outside);
        assert_eq!(
            ops(&items),
            [
                jp(Some(Cc::Z), "b"),
                Z80Op::Ret,
                Z80Op::Cpl,
                Z80Op::Ret,
                Z80Op::Neg,
                Z80Op::Ret
            ]
        );
    }

    #[test]
    fn call_then_ret_is_a_tail_jump_but_not_with_cleanup_between() {
        let call = |t: &str| Z80Op::Call {
            target: t.into(),
            clobbers: Vec::new(),
        };
        let mut items = vec![label("f"), ins(call("g")), ins(Z80Op::Ret)];
        assert_eq!(tail_calls(&mut items), ["g"]);
        assert_eq!(ops(&items), [jp(None, "g")]);
        let mut kept = vec![
            label("f"),
            ins(call("g")),
            ins(Z80Op::Pop(Rr::Bc)),
            ins(Z80Op::Ret),
        ];
        assert!(tail_calls(&mut kept).is_empty());
        assert_eq!(ops(&kept)[0], call("g"));
    }

    /// `DEC B / LD A,B / OR A / JP NZ,body`, then `tail` after the loop.
    fn do_while(tail: Vec<Z80Item>) -> Vec<Z80Item> {
        let mut items = vec![
            label("f"),
            ins(Z80Op::Ld8Imm { dst: R8::B, imm: 3 }),
            label("body"),
            ins(Z80Op::Inc8(R8::C)),
            ins(Z80Op::Dec8(R8::B)),
            ins(Z80Op::Ld8 {
                dst: R8::A,
                src: R8::B,
            }),
            ins(Z80Op::Or(AluSrc::Reg(R8::A))),
            ins(jp(Some(Cc::Nz), "body")),
        ];
        items.extend(tail);
        items
    }

    #[test]
    fn djnz_replaces_the_counter_test_only_when_a_and_flags_are_dead() {
        let ret_c = r(R8::C);
        // Dead: C is returned, A and flags are not read again.
        let mut dead = do_while(vec![ins(Z80Op::Ret)]);
        assert!(use_djnz(&mut dead, ret_c, &HashSet::new()));
        assert_eq!(
            ops(&dead),
            [
                Z80Op::Ld8Imm { dst: R8::B, imm: 3 },
                Z80Op::Inc8(R8::C),
                Z80Op::Djnz {
                    target: "body".into()
                },
                Z80Op::Ret
            ]
        );
        // A is read after the loop (it would not hold B): must not fire.
        let mut a_live = do_while(vec![ins(Z80Op::Ret)]);
        assert!(!use_djnz(&mut a_live, r(R8::A), &HashSet::new()));
        // The zero flag is read after the loop: must not fire.
        let mut z_live = do_while(vec![ins(jp(Some(Cc::Z), "f")), ins(Z80Op::Ret)]);
        assert!(!use_djnz(&mut z_live, ret_c, &HashSet::new()));
    }

    #[test]
    fn a_rotated_while_gets_its_guard_at_the_entry_and_djnz() {
        let mut items = vec![
            label("f"),
            ins(Z80Op::Jr {
                cc: None,
                target: "top".into(),
            }),
            label("body"),
            ins(Z80Op::Inc8(R8::C)),
            ins(Z80Op::Dec8(R8::B)),
            label("top"),
            ins(Z80Op::Ld8 {
                dst: R8::A,
                src: R8::B,
            }),
            ins(Z80Op::Or(AluSrc::Reg(R8::A))),
            ins(jp(Some(Cc::Nz), "body")),
            label("exit"),
            ins(Z80Op::Ret),
        ];
        // `top` named elsewhere (another entrant): must not fire.
        let named: HashSet<String> = ["top".to_string()].into_iter().collect();
        assert!(!use_djnz(&mut items.clone(), r(R8::C), &named));
        assert!(use_djnz(&mut items, r(R8::C), &HashSet::new()));
        assert_eq!(
            ops(&items),
            [
                Z80Op::Ld8 {
                    dst: R8::A,
                    src: R8::B
                },
                Z80Op::Or(AluSrc::Reg(R8::A)),
                jp(Some(Cc::Z), "exit"),
                Z80Op::Inc8(R8::C),
                Z80Op::Djnz {
                    target: "body".into()
                },
                Z80Op::Ret
            ]
        );
    }

    #[test]
    fn a_while_loop_is_rotated_with_the_test_at_the_bottom() {
        let mut items = vec![
            label("f"),
            ins(Z80Op::Ld8Imm { dst: R8::B, imm: 3 }),
            label("top"),
            ins(Z80Op::Ld8 {
                dst: R8::A,
                src: R8::B,
            }),
            ins(Z80Op::Or(AluSrc::Reg(R8::A))),
            ins(jp(Some(Cc::Z), "exit")),
            label("body"),
            ins(Z80Op::Dec8(R8::B)),
            ins(jp(None, "top")),
            label("exit"),
            ins(Z80Op::Ret),
        ];
        assert!(rotate_loops(&mut items));
        assert_eq!(
            ops(&items),
            [
                Z80Op::Ld8Imm { dst: R8::B, imm: 3 },
                Z80Op::Jr {
                    cc: None,
                    target: "top".into()
                },
                Z80Op::Dec8(R8::B),
                Z80Op::Ld8 {
                    dst: R8::A,
                    src: R8::B
                },
                Z80Op::Or(AluSrc::Reg(R8::A)),
                jp(Some(Cc::Nz), "body"),
                Z80Op::Ret,
            ]
        );
        // Already rotated: nothing more to do.
        assert!(!rotate_loops(&mut items));
    }

    fn run(items: Vec<Z80Item>) -> Vec<Z80Op> {
        let mut items = items;
        while peephole(&mut items, ALL_REGS) {}
        ops(&items)
    }

    fn jp(cc: Option<Cc>, t: &str) -> Z80Op {
        Z80Op::Jp {
            cc,
            target: t.to_string(),
        }
    }

    const A: AluSrc = AluSrc::Reg(R8::A);

    #[test]
    fn cp_zero_becomes_or_a_only_when_pv_and_n_are_dead() {
        let dead = run(vec![
            ins(Z80Op::Cp(AluSrc::Imm(0))),
            ins(jp(Some(Cc::Z), "x")),
            label("x"),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(dead[0], Z80Op::Or(A));
        let live = run(vec![
            ins(Z80Op::Cp(AluSrc::Imm(0))),
            ins(jp(Some(Cc::Pe), "x")),
            label("x"),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(live[0], Z80Op::Cp(AluSrc::Imm(0)));
    }

    #[test]
    fn add_one_becomes_inc_only_when_carry_is_dead() {
        let dead = run(vec![ins(Z80Op::AddA(AluSrc::Imm(1))), ins(Z80Op::Ret)]);
        assert_eq!(dead, [Z80Op::Inc8(R8::A), Z80Op::Ret]);
        let live = run(vec![
            ins(Z80Op::AddA(AluSrc::Imm(1))),
            ins(jp(Some(Cc::C), "x")),
            label("x"),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(live[0], Z80Op::AddA(AluSrc::Imm(1)));
        // Carry read through a backward jump (a loop) keeps it too.
        let looped = run(vec![
            label("top"),
            ins(Z80Op::Rla),
            ins(Z80Op::Sub(AluSrc::Imm(1))),
            ins(jp(None, "top")),
        ]);
        assert_eq!(looped[1], Z80Op::Sub(AluSrc::Imm(1)));
        let dec = run(vec![
            ins(Z80Op::Sub(AluSrc::Imm(1))),
            ins(jp(Some(Cc::Nz), "x")),
            label("x"),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(dec[0], Z80Op::Dec8(R8::A));
    }

    #[test]
    fn ld_a_zero_becomes_xor_a_only_when_every_flag_is_dead() {
        let dead = run(vec![
            ins(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(dead[0], Z80Op::Xor(A));
        // The bool-materialization shape: flags set, LD A,0, then a jump.
        let live = run(vec![
            ins(Z80Op::Cp(AluSrc::Reg(R8::B))),
            ins(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }),
            ins(jp(Some(Cc::Z), "x")),
            label("x"),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(live[1], Z80Op::Ld8Imm { dst: R8::A, imm: 0 });
        // A label outside the list counts as reading everything.
        let ext = run(vec![
            ins(Z80Op::Ld8Imm { dst: R8::A, imm: 0 }),
            ins(jp(None, "elsewhere")),
        ]);
        assert_eq!(ext[0], Z80Op::Ld8Imm { dst: R8::A, imm: 0 });
    }

    #[test]
    fn add_zero_is_dropped_only_when_flags_are_dead() {
        let dead = run(vec![ins(Z80Op::AddA(AluSrc::Imm(0))), ins(Z80Op::Ret)]);
        assert_eq!(dead, [Z80Op::Ret]);
        let live = run(vec![
            ins(Z80Op::AddA(AluSrc::Imm(0))),
            ins(Z80Op::AdcA(A)),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(live[0], Z80Op::AddA(AluSrc::Imm(0)));
    }

    #[test]
    fn word_add_with_zero_low_byte_uses_the_high_half() {
        let ld = |dst, src| ins(Z80Op::Ld8 { dst, src });
        let got = run(vec![
            ld(R8::A, R8::L),
            ins(Z80Op::AddA(AluSrc::Imm(0))),
            ld(R8::L, R8::A),
            ld(R8::A, R8::H),
            ins(Z80Op::AdcA(AluSrc::Imm(0x80))),
            ld(R8::H, R8::A),
            ins(jp(Some(Cc::C), "x")),
            label("x"),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(
            got[..3],
            [
                Z80Op::Ld8 {
                    dst: R8::A,
                    src: R8::H
                },
                Z80Op::AddA(AluSrc::Imm(0x80)),
                Z80Op::Ld8 {
                    dst: R8::H,
                    src: R8::A
                },
            ]
        );
    }

    #[test]
    fn load_step_store_becomes_inc_r_only_when_a_and_carry_are_dead() {
        let ld = |dst, src| ins(Z80Op::Ld8 { dst, src });
        let dead = run(vec![
            ld(R8::A, R8::C),
            ins(Z80Op::AddA(AluSrc::Imm(1))),
            ld(R8::C, R8::A),
            ins(Z80Op::Ld8Imm { dst: R8::A, imm: 5 }),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(dead[0], Z80Op::Inc8(R8::C));
        // A is returned: the copy in A must stay.
        let live_a = run(vec![
            ld(R8::A, R8::C),
            ins(Z80Op::AddA(AluSrc::Imm(1))),
            ld(R8::C, R8::A),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(
            live_a[0],
            Z80Op::Ld8 {
                dst: R8::A,
                src: R8::C
            }
        );
    }

    #[test]
    fn hl_doubling_through_de_only_when_de_is_dead() {
        let ld = |dst, src| ins(Z80Op::Ld8 { dst, src });
        let dead = run(vec![
            ld(R8::D, R8::H),
            ld(R8::E, R8::L),
            ins(Z80Op::AddHl(Rr::De)),
            ins(Z80Op::Ld16Imm {
                dst: Rr::De,
                imm: 0,
            }),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(dead[0], Z80Op::AddHl(Rr::Hl));
        let live = run(vec![
            ld(R8::D, R8::H),
            ld(R8::E, R8::L),
            ins(Z80Op::AddHl(Rr::De)),
            ins(Z80Op::Ret),
        ]);
        assert_eq!(live[2], Z80Op::AddHl(Rr::De));
    }
}

#[cfg(test)]
mod track_tests {
    //! Register tracking and dead loads (H5 item 3).
    use super::*;
    use crate::liveness::ALL_REGS;
    use crate::source::NodeId;
    use crate::z80::{AsmInstructionId, Cc};

    fn ins(op: Z80Op) -> Z80Item {
        Z80Item::Instruction {
            id: AsmInstructionId(0),
            op,
            span: SourceSpan::point(FileId(0), 0),
            expr: None,
            synthetic: false,
            node: None,
            note: None,
        }
    }

    fn ops(items: &[Z80Item]) -> Vec<Z80Op> {
        items
            .iter()
            .filter_map(|i| match i {
                Z80Item::Instruction { op, .. } => Some(op.clone()),
                _ => None,
            })
            .collect()
    }

    fn ld(dst: R8, src: R8) -> Z80Op {
        Z80Op::Ld8 { dst, src }
    }

    #[test]
    fn a_move_back_is_dropped_but_not_across_a_label() {
        let mut items = vec![
            ins(ld(R8::B, R8::A)),
            ins(ld(R8::A, R8::B)),
            ins(Z80Op::Ret),
        ];
        track_registers(&mut items, &HashSet::new());
        assert_eq!(ops(&items), [ld(R8::B, R8::A), Z80Op::Ret]);
        let entry = Z80Item::Label {
            id: AsmInstructionId(0),
            name: "f".into(),
            span: SourceSpan::point(FileId(0), 0),
            node: NodeId(0),
        };
        let mut looped = vec![
            entry,
            ins(ld(R8::B, R8::A)),
            Z80Item::Label {
                id: AsmInstructionId(0),
                name: "top".into(),
                span: SourceSpan::point(FileId(0), 0),
                node: NodeId(0),
            },
            ins(ld(R8::A, R8::B)),
            ins(Z80Op::Jp {
                cc: Some(Cc::Z),
                target: "top".into(),
            }),
            ins(Z80Op::Ret),
        ];
        track_registers(&mut looped, &HashSet::new());
        assert_eq!(ops(&looped).len(), 4);
        // An unreferenced label is straight-line code and keeps the knowledge.
        let mut plain = looped.clone();
        plain.remove(4);
        track_registers(&mut plain, &HashSet::new());
        assert_eq!(ops(&plain), [ld(R8::B, R8::A), Z80Op::Ret]);
    }

    #[test]
    fn a_slot_reload_uses_the_register_but_a_global_reload_stays() {
        let st = |symbol: &str| Z80Op::StAbs8 {
            src: R8::C,
            symbol: symbol.into(),
        };
        let lda = |symbol: &str| Z80Op::LdAbs8 {
            dst: R8::A,
            symbol: symbol.into(),
        };
        let mut items = vec![
            ins(st("FN3_F_S")),
            ins(lda("FN3_F_S")),
            ins(st("G1_G")),
            ins(lda("G1_G")),
            ins(Z80Op::Ret),
        ];
        track_registers(&mut items, &HashSet::new());
        assert_eq!(ops(&items)[1], ld(R8::A, R8::C));
        assert_eq!(ops(&items)[3], lda("G1_G"));
        // A store through a pointer may hit a slot whose address was taken.
        let mut through = vec![
            ins(st("FN3_F_S")),
            ins(Z80Op::StHl(R8::B)),
            ins(lda("FN3_F_S")),
            ins(Z80Op::Ret),
        ];
        track_registers(&mut through, &HashSet::new());
        assert_eq!(ops(&through)[2], lda("FN3_F_S"));
    }

    #[test]
    fn a_word_slot_reload_becomes_two_ordered_moves() {
        // DE was saved; HL now holds DE's old halves swapped in: H <- E's
        // value must not be read after L is written.
        let mut items = vec![
            ins(Z80Op::StAbs16 {
                src: Rr::De,
                symbol: "FN3_F_S".into(),
            }),
            ins(ld(R8::H, R8::E)),
            ins(ld(R8::L, R8::D)),
            ins(Z80Op::Ld16Imm {
                dst: Rr::De,
                imm: 0,
            }),
            ins(Z80Op::LdAbs16 {
                dst: Rr::De,
                symbol: "FN3_F_S".into(),
            }),
            ins(Z80Op::Ret),
        ];
        track_registers(&mut items, &HashSet::new());
        assert_eq!(ops(&items)[4..6], [ld(R8::E, R8::H), ld(R8::D, R8::L)]);
    }

    #[test]
    fn a_dead_register_load_is_removed_a_live_one_stays() {
        let mut items = vec![
            ins(ld(R8::E, R8::A)),
            ins(Z80Op::Ld8Imm { dst: R8::E, imm: 1 }),
            ins(Z80Op::Ret),
        ];
        peephole(&mut items, ALL_REGS);
        assert_eq!(
            ops(&items),
            [Z80Op::Ld8Imm { dst: R8::E, imm: 1 }, Z80Op::Ret]
        );
        // At RET only the returned and preserved registers are read.
        let mut ret_a = vec![ins(ld(R8::E, R8::A)), ins(Z80Op::Ret)];
        peephole(&mut ret_a, crate::liveness::r(R8::A));
        assert_eq!(ops(&ret_a), [Z80Op::Ret]);
        let mut ret_e = vec![ins(ld(R8::E, R8::A)), ins(Z80Op::Ret)];
        peephole(&mut ret_e, crate::liveness::r(R8::E));
        assert_eq!(ops(&ret_e), [ld(R8::E, R8::A), Z80Op::Ret]);
    }
}
