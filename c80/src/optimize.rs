//! Provenance-preserving peephole and layout-sensitive branch shortening.

use super::diagnostic::{DiagCode, Diagnostic};
use super::lower::EmitChunk;
use super::source::{FileId, SourceSpan};
use super::z80::{Z80Item, Z80Op, render_items};
use rtvc_core::asm::assemble_program;

const MAX_JR_PASSES: usize = 256;

pub(crate) fn optimize_chunks(
    chunks: &mut [EmitChunk],
    origin: u16,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for chunk in chunks.iter_mut() {
        strip_redundant(chunk.items_mut());
    }
    shorten_jumps(chunks, origin, diagnostics);
}

fn strip_redundant(items: &mut Vec<Z80Item>) {
    for _ in 0..64 {
        let before = items.len();
        items.retain(|item| !is_identity_move(item));
        invert_cond_skip(items);
        remove_fallthrough_jumps(items);
        if items.len() == before {
            break;
        }
    }
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

fn shorten_jumps(chunks: &mut [EmitChunk], origin: u16, diagnostics: &mut Vec<Diagnostic>) {
    for _ in 0..MAX_JR_PASSES {
        let assembled = match assemble_current(chunks, origin) {
            Ok(assembled) => assembled,
            Err(_) => return,
        };
        let addrs = match emitting_addrs(chunks, &assembled) {
            Some(addrs) => addrs,
            None => return,
        };
        let mut converted = false;
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
            if !cc.jr_ok() {
                continue;
            }
            let Some(&target_addr) = assembled.symbols.get(&target) else {
                continue;
            };
            let adj_target = adjust_target_for_jp_to_jr(addr, target_addr);
            if !is_forward(addr, adj_target) || !jr_fits(addr, adj_target) {
                continue;
            }
            let snapshot = match &chunks[ci].items()[ii] {
                Z80Item::Instruction { op, .. } => op.clone(),
                _ => continue,
            };
            match &mut chunks[ci].items_mut()[ii] {
                Z80Item::Instruction { op, .. } => {
                    *op = Z80Op::Jr {
                        cc: Some(cc),
                        target: target.clone(),
                    };
                }
                _ => continue,
            }
            if assemble_current(chunks, origin).is_err() {
                if let Z80Item::Instruction { op, .. } = &mut chunks[ci].items_mut()[ii] {
                    *op = snapshot;
                }
                diagnostics.push(Diagnostic::error(
                    DiagCode::CgInternal,
                    chunk_span(chunks),
                    format!("JR conversion for '{target}' was rejected by the assembler"),
                ));
                return;
            }
            converted = true;
            break;
        }
        if !converted {
            return;
        }
    }
}

fn assemble_current(
    chunks: &[EmitChunk],
    origin: u16,
) -> Result<rtvc_core::asm::AssembledProgram, rtvc_core::asm::AsmError> {
    let mut items = Vec::new();
    for chunk in chunks {
        items.extend(chunk.items().iter().cloned());
    }
    assemble_program(&render_items(&items), origin)
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
