//! Constant folding of literal-only subexpressions (H5 item 6, optimized
//! builds). Within one block, a `Unary`, `Binary` or integer `Cast` whose
//! operands are all `Const` becomes a `Const` with C80's result (wrapping at
//! the result width, F001 shifts, signed compares by operand type), and a
//! `Branch` on a constant becomes a `Jump`. Operands defined in another block
//! are never treated as constants, so short-circuit joins are untouched.

use std::collections::HashMap;

use super::ir::{IrBinary, IrOp, IrUnary, TypedProgram, VReg};
use super::types::CType;

fn integer(ty: CType) -> bool {
    matches!(
        ty,
        CType::Bool | CType::U8 | CType::I8 | CType::U16 | CType::I16
    )
}

/// The value as a signed or unsigned integer of its type.
fn value(bits: u16, ty: CType) -> i32 {
    match ty {
        CType::I8 => i32::from(bits as u8 as i8),
        CType::I16 => i32::from(bits as i16),
        _ => i32::from(bits & ty.mask()),
    }
}

fn unary(op: IrUnary, bits: u16, ty: CType) -> u16 {
    let m = ty.mask();
    match op {
        IrUnary::Plus => bits & m,
        IrUnary::Neg => bits.wrapping_neg() & m,
        IrUnary::BitNot => !bits & m,
        IrUnary::Not => u16::from(bits & m == 0),
    }
}

fn binary(op: IrBinary, ty: CType, l: (u16, CType), r: (u16, CType)) -> Option<u16> {
    let m = ty.mask();
    let width = ty.bit_width()?;
    Some(match op {
        IrBinary::Add => l.0.wrapping_add(r.0) & m,
        IrBinary::Sub => l.0.wrapping_sub(r.0) & m,
        IrBinary::BitAnd => l.0 & r.0 & m,
        IrBinary::BitOr => (l.0 | r.0) & m,
        IrBinary::BitXor => (l.0 ^ r.0) & m,
        IrBinary::And => u16::from(l.0 & l.1.mask() != 0 && r.0 & r.1.mask() != 0),
        IrBinary::Or => u16::from(l.0 & l.1.mask() != 0 || r.0 & r.1.mask() != 0),
        IrBinary::Shl | IrBinary::Shr => {
            let count = u32::from(r.0 & 0xFF);
            let v = l.0 & m;
            if op == IrBinary::Shl {
                if count >= width { 0 } else { (v << count) & m }
            } else if ty.is_signed() {
                let s = value(v, ty);
                (s >> count.min(width - 1)) as u16 & m
            } else if count >= width {
                0
            } else {
                v >> count
            }
        }
        IrBinary::Eq | IrBinary::Ne | IrBinary::Lt | IrBinary::Le | IrBinary::Gt | IrBinary::Ge => {
            if l.1 != r.1 {
                return None;
            }
            let (a, b) = (value(l.0, l.1), value(r.0, r.1));
            u16::from(match op {
                IrBinary::Eq => a == b,
                IrBinary::Ne => a != b,
                IrBinary::Lt => a < b,
                IrBinary::Le => a <= b,
                IrBinary::Gt => a > b,
                _ => a >= b,
            })
        }
    })
}

fn cast(bits: u16, from: CType, to: CType) -> u16 {
    if to == CType::Bool {
        return u16::from(bits & from.mask() != 0);
    }
    (value(bits, from) as u16) & to.mask()
}

/// Drops blocks no path from the entry reaches (optimized builds): the
/// right operand of `true || x` / `false && x`, code after `return` or
/// `break`. Returns the number of blocks dropped.
pub(crate) fn prune_unreachable(program: &mut TypedProgram) -> usize {
    let mut dropped = 0;
    for func in &mut program.functions {
        let Some(entry) = func.blocks.first().map(|b| b.id) else {
            continue;
        };
        let mut reached = vec![entry];
        let mut k = 0;
        while k < reached.len() {
            let id = reached[k];
            k += 1;
            let Some(block) = func.blocks.iter().find(|b| b.id == id) else {
                continue;
            };
            for op in &block.ops {
                let next: &[_] = match op {
                    IrOp::Jump { blk, .. } => &[*blk],
                    IrOp::Branch {
                        true_blk,
                        false_blk,
                        ..
                    } => &[*true_blk, *false_blk],
                    _ => &[],
                };
                for b in next {
                    if !reached.contains(b) {
                        reached.push(*b);
                    }
                }
            }
        }
        let before = func.blocks.len();
        func.blocks.retain(|b| reached.contains(&b.id));
        dropped += before - func.blocks.len();
    }
    dropped
}

/// Folds every function in place. Returns the number of ops folded.
pub(crate) fn fold_constants(program: &mut TypedProgram) -> usize {
    let mut folded = 0;
    for func in &mut program.functions {
        // A `&&`/`||` value join (a block starting with `Binary And/Or`)
        // receives its right operand in A from the branch that reaches it;
        // a branch into one keeps its shape.
        let joins: Vec<_> = func
            .blocks
            .iter()
            .filter(|b| {
                matches!(
                    b.ops.first(),
                    Some(IrOp::Binary {
                        op: IrBinary::And | IrBinary::Or,
                        ..
                    })
                )
            })
            .map(|b| b.id)
            .collect();
        for block in &mut func.blocks {
            let mut known: HashMap<VReg, (u16, CType)> = HashMap::new();
            for op in &mut block.ops {
                let replacement = match &*op {
                    IrOp::Const { dst, ty, bits, .. } => {
                        if integer(*ty) {
                            known.insert(*dst, (*bits, *ty));
                        }
                        None
                    }
                    IrOp::Unary {
                        dst,
                        ty,
                        op: u,
                        src,
                        span,
                    } if integer(*ty) => known
                        .get(src)
                        .filter(|s| integer(s.1))
                        .map(|&(b, _)| (*dst, *ty, unary(*u, b, *ty), *span)),
                    IrOp::Binary {
                        dst,
                        ty,
                        op: b,
                        lhs,
                        rhs,
                        span,
                    } if integer(*ty) => match (known.get(lhs), known.get(rhs)) {
                        (Some(&l), Some(&r)) if integer(l.1) && integer(r.1) => {
                            binary(*b, *ty, l, r).map(|v| (*dst, *ty, v, *span))
                        }
                        _ => None,
                    },
                    IrOp::Cast {
                        dst,
                        to,
                        from,
                        src,
                        span,
                    } if integer(*to) && integer(*from) => known
                        .get(src)
                        .map(|&(b, _)| (*dst, *to, cast(b, *from, *to), *span)),
                    IrOp::Branch {
                        cond,
                        true_blk,
                        false_blk,
                        span,
                    } => {
                        if let Some(&(b, ty)) = known
                            .get(cond)
                            .filter(|_| !joins.contains(true_blk) && !joins.contains(false_blk))
                        {
                            let blk = if b & ty.mask() != 0 {
                                *true_blk
                            } else {
                                *false_blk
                            };
                            *op = IrOp::Jump { blk, span: *span };
                            folded += 1;
                        }
                        None
                    }
                    _ => None,
                };
                if let Some((dst, ty, bits, span)) = replacement {
                    *op = IrOp::Const {
                        dst,
                        ty,
                        bits,
                        span,
                    };
                    known.insert(dst, (bits, ty));
                    folded += 1;
                }
            }
        }
    }
    folded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_follow_c80_semantics() {
        use CType::*;
        assert_eq!(binary(IrBinary::Add, U8, (255, U8), (1, U8)), Some(0));
        assert_eq!(binary(IrBinary::Sub, U16, (0, U16), (1, U16)), Some(0xFFFF));
        assert_eq!(binary(IrBinary::Shl, U8, (1, U8), (8, U8)), Some(0));
        assert_eq!(binary(IrBinary::Shr, I8, (0x80, I8), (9, U8)), Some(0xFF));
        assert_eq!(
            binary(IrBinary::Shr, I16, (0x8000, I16), (4, U8)),
            Some(0xF800)
        );
        assert_eq!(binary(IrBinary::Shr, U16, (0x8000, U16), (16, U8)), Some(0));
        assert_eq!(binary(IrBinary::Lt, Bool, (0xFF, I8), (1, I8)), Some(1));
        assert_eq!(binary(IrBinary::Lt, Bool, (0xFF, U8), (1, U8)), Some(0));
        assert_eq!(binary(IrBinary::Lt, Bool, (0xFF, U8), (1, I8)), None);
        assert_eq!(unary(IrUnary::Neg, 7, I8), 0xF9);
        assert_eq!(unary(IrUnary::Not, 0, Bool), 1);
        assert_eq!(cast(0xFF, I8, I16), 0xFFFF);
        assert_eq!(cast(0xFF, U8, U16), 0x00FF);
        assert_eq!(cast(0x1234, U16, U8), 0x34);
        assert_eq!(cast(0x0100, U16, Bool), 1);
    }
}
