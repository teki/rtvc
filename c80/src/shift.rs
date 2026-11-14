//! Instruction sequences for shifts (H3).
//!
//! F001: the count is `u8`; `<<` wraps at the operand width; `>>` is logical
//! for unsigned and arithmetic for signed operands; a count at or above the
//! width gives 0 (or all sign bits for a signed `>>`). Constant counts pick
//! the fastest sequence (T-states, then bytes) from a few templates;
//! variable counts run one step per count behind a zero guard.

use super::z80::{AluSrc, R8, Rr, Z80Op};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftKind {
    /// `<<` (any integer type).
    Shl,
    /// `>>` on an unsigned type.
    Shr,
    /// `>>` on a signed type.
    Sar,
}

/// T-states, then bytes, of a straight-line sequence.
pub fn seq_cost(ops: &[Z80Op]) -> (u32, u32) {
    ops.iter().fold((0, 0), |(t, b), op| {
        (t + op.approx_tstates(), b + op.size_bytes())
    })
}

fn best(cands: Vec<Vec<Z80Op>>) -> Vec<Z80Op> {
    cands
        .into_iter()
        .min_by_key(|ops| seq_cost(ops))
        .unwrap_or_default()
}

fn rep(n: u8, step: &[Z80Op]) -> Vec<Z80Op> {
    let mut out = Vec::new();
    for _ in 0..n {
        out.extend_from_slice(step);
    }
    out
}

fn ld(dst: R8, src: R8) -> Z80Op {
    Z80Op::Ld8 { dst, src }
}

fn ld_imm(dst: R8, imm: u8) -> Z80Op {
    Z80Op::Ld8Imm { dst, imm }
}

const A: AluSrc = AluSrc::Reg(R8::A);

/// One step of a byte shift of `r` in place.
pub fn byte_step(kind: ShiftKind, r: R8) -> Vec<Z80Op> {
    match (kind, r) {
        (ShiftKind::Shl, R8::A) => vec![Z80Op::AddA(A)],
        (ShiftKind::Shl, r) => vec![Z80Op::Sla(r)],
        (ShiftKind::Shr, r) => vec![Z80Op::Srl(r)],
        (ShiftKind::Sar, r) => vec![Z80Op::Sra(r)],
    }
}

/// One step of a word shift of `rr` in place.
pub fn word_step(kind: ShiftKind, rr: Rr) -> Vec<Z80Op> {
    let (h, l) = rr.halves();
    match (kind, rr) {
        (ShiftKind::Shl, Rr::Hl) => vec![Z80Op::AddHl(Rr::Hl)],
        (ShiftKind::Shl, _) => vec![Z80Op::Sla(l), Z80Op::Rl8(h)],
        (ShiftKind::Shr, _) => vec![Z80Op::Srl(h), Z80Op::Rr8(l)],
        (ShiftKind::Sar, _) => vec![Z80Op::Sra(h), Z80Op::Rr8(l)],
    }
}

/// `A` shifted by a constant `k` (1..=7; a signed count of 7 or more is
/// passed as 7), result in `A`. Empty for `k == 0`.
pub fn a_const(kind: ShiftKind, k: u8) -> Vec<Z80Op> {
    if k == 0 {
        return Vec::new();
    }
    let k = k.min(7);
    let mut cands = vec![rep(k, &byte_step(kind, R8::A))];
    // Rotate the byte and mask the bits that wrapped around.
    let rot = |n: u8, right: bool| rep(n, &[if right { Z80Op::Rrca } else { Z80Op::Rlca }]);
    let logical_rot = |kind: ShiftKind| -> Vec<Vec<Z80Op>> {
        let (mask, right_n, left_n) = match kind {
            ShiftKind::Shl => (0xFFu8 << k, 8 - k, k),
            _ => (0xFFu8 >> k, k, 8 - k),
        };
        [rot(right_n, true), rot(left_n, false)]
            .into_iter()
            .map(|mut ops| {
                ops.push(Z80Op::And(AluSrc::Imm(mask)));
                ops
            })
            .collect()
    };
    match kind {
        ShiftKind::Shl | ShiftKind::Shr => cands.extend(logical_rot(kind)),
        ShiftKind::Sar => {
            if k == 7 {
                // Bit 7 into carry, then 0 or 0xFF.
                cands.push(vec![Z80Op::AddA(A), Z80Op::SbcA(A)]);
            } else {
                // Logical shift, then sign-extend from bit 7-k:
                // (v ^ m) - m with m = 0x80 >> k.
                let m = 0x80u8 >> k;
                for mut ops in logical_rot(ShiftKind::Shr) {
                    ops.push(Z80Op::Xor(AluSrc::Imm(m)));
                    ops.push(Z80Op::Sub(AluSrc::Imm(m)));
                    cands.push(ops);
                }
            }
        }
    }
    best(cands)
}

/// A byte in `r` shifted by a constant `k` (1..=7, signed counts clamped to
/// 7), result in `r`. `a_free`: `A` may be used as scratch (always true when
/// `r` is `A`).
pub fn byte_const(kind: ShiftKind, k: u8, r: R8, a_free: bool) -> Vec<Z80Op> {
    if r == R8::A {
        return a_const(kind, k);
    }
    let k = k.min(7);
    let mut cands = vec![rep(k, &byte_step(kind, r))];
    if a_free {
        let mut ops = vec![ld(R8::A, r)];
        ops.extend(a_const(kind, k));
        ops.push(ld(r, R8::A));
        cands.push(ops);
    }
    best(cands)
}

/// A word in `rr` shifted by a constant `k` (1..=15; a signed count of 15
/// or more is passed as 15), result in `rr`. `a_free`: `A` may be used as
/// scratch.
pub fn word_const(kind: ShiftKind, k: u8, rr: Rr, a_free: bool) -> Vec<Z80Op> {
    if k == 0 {
        return Vec::new();
    }
    let k = k.min(15);
    let (h, l) = rr.halves();
    let mut cands = Vec::new();
    if k < 8 {
        cands.push(rep(k, &word_step(kind, rr)));
    }
    if k >= 8 {
        let rest = k - 8;
        match kind {
            ShiftKind::Shl => {
                let mut ops = vec![ld(h, l), ld_imm(l, 0)];
                ops.extend(rep(rest, &[Z80Op::Sla(h)]));
                cands.push(ops);
                if a_free {
                    let mut ops = vec![ld(R8::A, l)];
                    ops.extend(a_const(kind, rest));
                    ops.extend([ld(h, R8::A), ld_imm(l, 0)]);
                    cands.push(ops);
                }
            }
            ShiftKind::Shr => {
                let mut ops = vec![ld(l, h), ld_imm(h, 0)];
                ops.extend(rep(rest, &[Z80Op::Srl(l)]));
                cands.push(ops);
                if a_free {
                    let mut ops = vec![ld(R8::A, h)];
                    ops.extend(a_const(kind, rest));
                    ops.extend([ld(l, R8::A), ld_imm(h, 0)]);
                    cands.push(ops);
                }
            }
            ShiftKind::Sar => {
                let mut ops = vec![ld(l, h)];
                ops.extend(rep(rest, &[Z80Op::Sra(l)]));
                ops.extend(rep(7, &[Z80Op::Sra(h)]));
                cands.push(ops);
                if rr == Rr::Hl && k == 15 {
                    // Sign into carry, then 0 or 0xFFFF.
                    cands.push(vec![Z80Op::AddHl(Rr::Hl), Z80Op::SbcHl(Rr::Hl)]);
                }
                if a_free {
                    if k == 15 {
                        cands.push(vec![
                            ld(R8::A, h),
                            Z80Op::Rla,
                            Z80Op::SbcA(A),
                            ld(h, R8::A),
                            ld(l, R8::A),
                        ]);
                    } else {
                        // Low byte: the high byte shifted arithmetically;
                        // high byte: its sign.
                        let mut ops = vec![ld(R8::A, h)];
                        ops.extend(a_const(kind, rest));
                        ops.extend([ld(l, R8::A), Z80Op::Rla, Z80Op::SbcA(A), ld(h, R8::A)]);
                        cands.push(ops);
                    }
                }
            }
        }
    } else if a_free {
        // Shift the other way by 8-k, catching the bits that leave in A,
        // then move bytes: one 8-bit move replaces k single-bit steps.
        let n = 8 - k;
        let left = word_step(ShiftKind::Shl, rr);
        match kind {
            ShiftKind::Shl => {
                let mut ops = vec![Z80Op::Xor(A)];
                ops.extend(rep(n, &[Z80Op::Srl(h), Z80Op::Rr8(l), Z80Op::Rra]));
                ops.extend([ld(h, l), ld(l, R8::A)]);
                cands.push(ops);
            }
            ShiftKind::Shr | ShiftKind::Sar => {
                let mut ops = if kind == ShiftKind::Shr {
                    vec![Z80Op::Xor(A)]
                } else {
                    vec![ld(R8::A, h), Z80Op::Rla, Z80Op::SbcA(A)]
                };
                let mut step = left.clone();
                step.push(Z80Op::Rla);
                ops.extend(rep(n, &step));
                ops.extend([ld(l, h), ld(h, R8::A)]);
                cands.push(ops);
            }
        }
    }
    best(cands)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run a straight-line sequence on a tiny register model.
    fn run(ops: &[Z80Op], regs: &mut [u8; 7]) {
        let mut carry = false;
        let ix = |r: R8| r.index();
        for op in ops {
            match op {
                Z80Op::Ld8 { dst, src } => regs[ix(*dst)] = regs[ix(*src)],
                Z80Op::Ld8Imm { dst, imm } => regs[ix(*dst)] = *imm,
                Z80Op::AddA(AluSrc::Reg(R8::A)) => {
                    let a = regs[0];
                    carry = a & 0x80 != 0;
                    regs[0] = a.wrapping_add(a);
                }
                Z80Op::SbcA(AluSrc::Reg(R8::A)) => regs[0] = if carry { 0xFF } else { 0 },
                Z80Op::Xor(AluSrc::Reg(R8::A)) => {
                    regs[0] = 0;
                    carry = false;
                }
                Z80Op::And(AluSrc::Imm(n)) => {
                    regs[0] &= n;
                    carry = false;
                }
                Z80Op::Xor(AluSrc::Imm(n)) => {
                    regs[0] ^= n;
                    carry = false;
                }
                Z80Op::Sub(AluSrc::Imm(n)) => {
                    carry = regs[0] < *n;
                    regs[0] = regs[0].wrapping_sub(*n);
                }
                Z80Op::Rlca => {
                    regs[0] = regs[0].rotate_left(1);
                    carry = regs[0] & 1 != 0;
                }
                Z80Op::Rrca => {
                    regs[0] = regs[0].rotate_right(1);
                    carry = regs[0] & 0x80 != 0;
                }
                Z80Op::Rla => {
                    let c = regs[0] & 0x80 != 0;
                    regs[0] = (regs[0] << 1) | u8::from(carry);
                    carry = c;
                }
                Z80Op::Rra => {
                    let c = regs[0] & 1 != 0;
                    regs[0] = (regs[0] >> 1) | (u8::from(carry) << 7);
                    carry = c;
                }
                Z80Op::Sla(r) => {
                    let v = regs[ix(*r)];
                    carry = v & 0x80 != 0;
                    regs[ix(*r)] = v << 1;
                }
                Z80Op::Srl(r) => {
                    let v = regs[ix(*r)];
                    carry = v & 1 != 0;
                    regs[ix(*r)] = v >> 1;
                }
                Z80Op::Sra(r) => {
                    let v = regs[ix(*r)];
                    carry = v & 1 != 0;
                    regs[ix(*r)] = ((v as i8) >> 1) as u8;
                }
                Z80Op::Rl8(r) => {
                    let v = regs[ix(*r)];
                    regs[ix(*r)] = (v << 1) | u8::from(carry);
                    carry = v & 0x80 != 0;
                }
                Z80Op::Rr8(r) => {
                    let v = regs[ix(*r)];
                    regs[ix(*r)] = (v >> 1) | (u8::from(carry) << 7);
                    carry = v & 1 != 0;
                }
                Z80Op::AddHl(Rr::Hl) => {
                    let hl = u16::from_be_bytes([regs[5], regs[6]]);
                    carry = hl & 0x8000 != 0;
                    let [h, l] = hl.wrapping_add(hl).to_be_bytes();
                    regs[5] = h;
                    regs[6] = l;
                }
                Z80Op::SbcHl(Rr::Hl) => {
                    let v = if carry { 0xFF } else { 0 };
                    regs[5] = v;
                    regs[6] = v;
                }
                other => panic!("model lacks {other:?}"),
            }
        }
    }

    fn expect(kind: ShiftKind, bits: u32, x: u16, k: u32) -> u16 {
        let mask = if bits == 8 { 0xFF } else { 0xFFFF };
        let x = x & mask;
        if k >= bits {
            return match kind {
                ShiftKind::Sar if x & (1 << (bits - 1)) != 0 => mask,
                _ => 0,
            };
        }
        match kind {
            ShiftKind::Shl => (x << k) & mask,
            ShiftKind::Shr => x >> k,
            ShiftKind::Sar => {
                let sx = if bits == 8 {
                    i32::from(x as u8 as i8)
                } else {
                    i32::from(x as i16)
                };
                (sx >> k) as u16 & mask
            }
        }
    }

    #[test]
    fn byte_sequences_compute_f001() {
        for kind in [ShiftKind::Shl, ShiftKind::Shr, ShiftKind::Sar] {
            for k in 1..=7u8 {
                for r in [R8::A, R8::B, R8::E] {
                    for a_free in [false, true] {
                        let ops = byte_const(kind, k, r, a_free || r == R8::A);
                        for x in 0..=255u8 {
                            let mut regs = [0x5Au8; 7];
                            regs[r.index()] = x;
                            run(&ops, &mut regs);
                            let want = expect(kind, 8, u16::from(x), u32::from(k));
                            assert_eq!(
                                u16::from(regs[r.index()]),
                                want,
                                "{kind:?} {k} in {r:?} x={x:#x}: {ops:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn word_sequences_compute_f001() {
        for kind in [ShiftKind::Shl, ShiftKind::Shr, ShiftKind::Sar] {
            for k in 1..=15u8 {
                for rr in [Rr::Hl, Rr::De, Rr::Bc] {
                    for a_free in [false, true] {
                        let ops = word_const(kind, k, rr, a_free);
                        let (h, l) = rr.halves();
                        for x in [
                            0u16, 1, 0x7F, 0x80, 0xFF, 0x100, 0x1234, 0x7FFF, 0x8000, 0x8001,
                            0xA5C3, 0xFFFF, 0x0F0F, 0xF0F0,
                        ] {
                            let mut regs = [0x5Au8; 7];
                            regs[h.index()] = (x >> 8) as u8;
                            regs[l.index()] = x as u8;
                            run(&ops, &mut regs);
                            let got = u16::from_be_bytes([regs[h.index()], regs[l.index()]]);
                            assert_eq!(
                                got,
                                expect(kind, 16, x, u32::from(k)),
                                "{kind:?} {k} in {rr:?} a_free={a_free} x={x:#x}: {ops:?}"
                            );
                            if !a_free {
                                assert!(ops.iter().all(|op| !op.writes().contains(&R8::A)));
                            }
                        }
                    }
                }
            }
        }
    }

    /// The chosen constant-count sequences (H3 progress table).
    #[test]
    fn print_constant_shift_table() {
        let name = |k: ShiftKind| match k {
            ShiftKind::Shl => "<<",
            ShiftKind::Shr => ">> unsigned",
            ShiftKind::Sar => ">> signed",
        };
        for kind in [ShiftKind::Shl, ShiftKind::Shr, ShiftKind::Sar] {
            for k in 1..=7u8 {
                let ops = a_const(kind, k);
                let (t, b) = seq_cost(&ops);
                let text: Vec<String> = ops.iter().map(Z80Op::render).collect();
                println!(
                    "byte A {} {k}: {t} T, {b} B: {}",
                    name(kind),
                    text.join(" / ")
                );
            }
            for k in 1..=15u8 {
                for a_free in [true, false] {
                    let ops = word_const(kind, k, Rr::Hl, a_free);
                    let (t, b) = seq_cost(&ops);
                    let text: Vec<String> = ops.iter().map(Z80Op::render).collect();
                    println!(
                        "word HL{} {} {k}: {t} T, {b} B: {}",
                        if a_free { "" } else { " (A busy)" },
                        name(kind),
                        text.join(" / ")
                    );
                }
            }
        }
    }
}
