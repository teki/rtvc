//! F002 register ABI assignment for ordinary (non-@stackcall) functions.

use super::types::CType;
use super::z80::{R8, RegHome, Rr};

const WORD_REGS: [Rr; 3] = [Rr::Hl, Rr::De, Rr::Bc];
const BYTE_ORDER: [R8; 7] = [R8::A, R8::C, R8::B, R8::E, R8::D, R8::L, R8::H];

/// Assign incoming argument homes for a register-call signature.
///
/// Word/pointer arguments take HL, then DE, then BC in declaration order among
/// those arguments. Byte/bool arguments then take A, C, B, E, D, L, H, skipping
/// halves reserved by the word assignment. Returns `None` if the signature does
/// not fit (callers should diagnose and suggest `@stackcall`).
pub fn assign_params(params: &[CType]) -> Option<Vec<RegHome>> {
    let mut homes = vec![None; params.len()];
    let mut reserved = [false; 7];
    let mut word_i = 0usize;

    for (i, ty) in params.iter().enumerate() {
        match ty.byte_width() {
            Some(2) => {
                let rr = *WORD_REGS.get(word_i)?;
                word_i += 1;
                let (hi, lo) = rr.halves();
                reserved[hi.index()] = true;
                reserved[lo.index()] = true;
                homes[i] = Some(RegHome::Word(rr));
            }
            Some(1) => {}
            _ => return None,
        }
    }

    let mut byte_i = 0usize;
    for (i, ty) in params.iter().enumerate() {
        if ty.byte_width() != Some(1) {
            continue;
        }
        loop {
            let r = *BYTE_ORDER.get(byte_i)?;
            byte_i += 1;
            if !reserved[r.index()] {
                homes[i] = Some(RegHome::Byte(r));
                break;
            }
        }
    }

    homes.into_iter().collect()
}

pub fn return_home(ty: CType) -> Option<RegHome> {
    match ty.byte_width() {
        Some(1) => Some(RegHome::Byte(R8::A)),
        Some(2) => Some(RegHome::Word(Rr::Hl)),
        _ => None,
    }
}
