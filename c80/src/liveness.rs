//! Register and flag liveness over a function's item list (H5).
//!
//! The optimizer asks one question: is resource `R` read, before it is
//! written, on some path that starts right after item `k`? Paths follow
//! fall-through and every jump to a label of the same item list. Anything
//! the analysis cannot see through counts as a read, so the answer errs
//! towards "live":
//! - a jump to a label outside the list, and inline `asm` lines, read
//!   everything;
//! - `RET` reads the registers the caller gets back (`ret`: the return
//!   value and the preserved registers) and no flag: no C80 function
//!   returns a value in the flags;
//! - `CALL` reads every register (arguments) and writes the callee's
//!   clobbers and the flags: no C80 function takes flags as input.

use super::z80::{AluSrc, Cc, R8, Rr, Z80Item, Z80Op};
use std::collections::HashMap;

pub(crate) type Res = u16;

pub(crate) const FC: Res = 1 << 7;
pub(crate) const FZ: Res = 1 << 8;
pub(crate) const FS: Res = 1 << 9;
pub(crate) const FPV: Res = 1 << 10;
pub(crate) const FHN: Res = 1 << 11;
pub(crate) const ALL_FLAGS: Res = FC | FZ | FS | FPV | FHN;
pub(crate) const ALL_REGS: Res = 0x7F;

pub(crate) fn r(reg: R8) -> Res {
    1 << match reg {
        R8::A => 0,
        R8::B => 1,
        R8::C => 2,
        R8::D => 3,
        R8::E => 4,
        R8::H => 5,
        R8::L => 6,
    }
}

pub(crate) fn rr(pair: Rr) -> Res {
    let (h, l) = pair.halves();
    r(h) | r(l)
}

fn src(s: &AluSrc) -> Res {
    match s {
        AluSrc::Reg(x) => r(*x),
        AluSrc::Imm(_) => 0,
    }
}

fn cc_flag(cc: Cc) -> Res {
    match cc {
        Cc::Z | Cc::Nz => FZ,
        Cc::C | Cc::Nc => FC,
        Cc::Po | Cc::Pe => FPV,
        Cc::P | Cc::M => FS,
    }
}

/// (reads, writes) of an instruction.
pub(crate) fn effects(op: &Z80Op) -> (Res, Res) {
    use Z80Op::*;
    let a = r(R8::A);
    let hl = rr(Rr::Hl);
    match op {
        Ret => (ALL_REGS, 0),
        Ld8Imm { dst, .. } => (0, r(*dst)),
        Ld8 { dst, src } => (r(*src), r(*dst)),
        Ld16Imm { dst, .. } | Ld16Sym { dst, .. } => (0, rr(*dst)),
        AddA(s) | Sub(s) | And(s) | Xor(s) | Or(s) => (a | src(s), a | ALL_FLAGS),
        Cp(s) => (a | src(s), ALL_FLAGS),
        AdcA(s) | SbcA(s) => (a | src(s) | FC, a | ALL_FLAGS),
        AddHl(p) => (hl | rr(*p), hl | FC | FHN),
        SbcHl(p) => (hl | rr(*p) | FC, hl | ALL_FLAGS),
        Cpl => (a, a | FHN),
        Neg => (a, a | ALL_FLAGS),
        ExDeHl => (hl | rr(Rr::De), hl | rr(Rr::De)),
        Jp { cc, .. } | Jr { cc, .. } => (cc.map_or(0, cc_flag), 0),
        LdAbs8 { dst, .. } => (0, r(*dst)),
        StAbs8 { src, .. } => (r(*src), 0),
        LdAbs16 { dst, .. } => (0, rr(*dst)),
        StAbs16 { src, .. } => (rr(*src), 0),
        Call { clobbers, .. } => (ALL_REGS, clobbers.iter().fold(ALL_FLAGS, |m, x| m | r(*x))),
        Push(p) => (rr(*p), 0),
        Pop(p) => (0, rr(*p)),
        PushAf => (a | ALL_FLAGS, 0),
        PopAf => (0, a | ALL_FLAGS),
        PushIx | PopIx | LdIxImm(_) | LdSpIx | IncSp | DecSp | Di | Ei => (0, 0),
        AddIxSp => (0, FC | FHN),
        AddHlSp => (hl, hl | FC | FHN),
        LdSpHl => (hl, 0),
        Ld8Ix { dst, .. } => (0, r(*dst)),
        St8Ix { src, .. } => (r(*src), 0),
        Ld8IxImm { .. } => (0, 0),
        LdHl(dst) => (hl, r(*dst)),
        StHl(s) => (hl | r(*s), 0),
        IncHl => (hl, hl),
        LdAInd(p) => (rr(*p), a),
        StIndA(p) => (rr(*p) | a, 0),
        Inc16(p) => (rr(*p), rr(*p)),
        Rla | Rra => (a | FC, a | FC | FHN),
        Rlca | Rrca => (a, a | FC | FHN),
        InImm { .. } => (a, a),
        OutImm { .. } => (a, 0),
        InC => (rr(Rr::Bc), a | FZ | FS | FPV | FHN),
        OutC => (rr(Rr::Bc) | a, 0),
        Ldir => (
            rr(Rr::Bc) | rr(Rr::De) | hl,
            rr(Rr::Bc) | rr(Rr::De) | hl | FPV | FHN,
        ),
        Sla(x) | Srl(x) | Sra(x) => (r(*x), r(*x) | ALL_FLAGS),
        Rl8(x) | Rr8(x) => (r(*x) | FC, r(*x) | ALL_FLAGS),
        Inc8(x) | Dec8(x) => (r(*x), r(*x) | FZ | FS | FPV | FHN),
        Djnz { .. } => (r(R8::B), r(R8::B)),
    }
}

/// Label positions of an item list.
pub(crate) fn labels(items: &[Z80Item]) -> HashMap<String, usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| match it {
            Z80Item::Label { name, .. } => Some((name.clone(), i)),
            _ => None,
        })
        .collect()
}

/// The part of `want` that is live right after item `k`.
pub(crate) fn live_after(
    items: &[Z80Item],
    labels: &HashMap<String, usize>,
    ret: Res,
    k: usize,
    want: Res,
) -> Res {
    let mut live = 0;
    let mut seen: HashMap<usize, Res> = HashMap::new();
    let mut work = vec![(k + 1, want)];
    while let Some((mut pos, mut rem)) = work.pop() {
        loop {
            rem &= !live;
            if rem == 0 {
                break;
            }
            let prev = seen.entry(pos).or_insert(0);
            if rem & !*prev == 0 {
                break;
            }
            *prev |= rem;
            let Some(item) = items.get(pos) else {
                // Falling off the end: be safe.
                live |= rem;
                break;
            };
            match item {
                Z80Item::Label { .. } | Z80Item::Directive { .. } => {
                    pos += 1;
                    continue;
                }
                Z80Item::Data { .. } | Z80Item::Raw { .. } => {
                    live |= rem;
                    break;
                }
                Z80Item::Instruction { op, .. } => {
                    let (mut reads, writes) = effects(op);
                    if matches!(op, Z80Op::Ret) {
                        reads = ret;
                    }
                    live |= rem & reads;
                    rem &= !writes;
                    let target = match op {
                        Z80Op::Jp { target, .. }
                        | Z80Op::Jr { target, .. }
                        | Z80Op::Djnz { target } => Some(target),
                        _ => None,
                    };
                    let conditional = matches!(
                        op,
                        Z80Op::Jp { cc: Some(_), .. }
                            | Z80Op::Jr { cc: Some(_), .. }
                            | Z80Op::Djnz { .. }
                    );
                    if let Some(t) = target {
                        match labels.get(t) {
                            Some(&at) => work.push((at, rem)),
                            None => live |= rem,
                        }
                        if !conditional {
                            break;
                        }
                    }
                    if matches!(op, Z80Op::Ret) {
                        break;
                    }
                    pos += 1;
                }
            }
        }
    }
    live
}
