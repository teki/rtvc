//! Listing provenance, symbols, timing, and stack reports for compiled code.

use super::ast::{ForInit, Item, Stmt, TranslationUnit};
use super::source::{FileId, SourceMap, SourceSpan};
use super::types::CType;
use super::z80::{
    AsmInstructionId, GeneratedFunction, GeneratedGlobal, GeneratedProgram, MappedInstruction,
    MappedKind, StackProvenance, StaticTiming,
};
use rtvc_core::asm::AssembledProgram;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerSymbol {
    pub name: String,
    pub label: String,
    pub addr: u16,
    pub size: u16,
    pub ty: Option<CType>,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoCodeSpan {
    pub span: SourceSpan,
    pub reason: NoCodeReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoCodeReason {
    Eliminated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnStack {
    pub name: String,
    pub frame_bytes: u16,
    pub local_peak_bytes: u16,
    pub additional_bytes: Option<u16>,
    pub provenance: StackProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackReport {
    pub functions: Vec<FnStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildIdentity {
    pub crate_version: String,
    pub origin: u16,
    pub source_hash: u64,
    pub source_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanCost {
    pub local: u16,
    pub complete: bool,
    pub has_call: bool,
    pub has_loop: bool,
    pub unknown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerMap {
    pub entries: Vec<MappedInstruction>,
    pub no_code: Vec<NoCodeSpan>,
    pub symbols: Vec<CompilerSymbol>,
    pub stack: StackReport,
    pub identity: BuildIdentity,
}

impl CompilerMap {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            no_code: Vec::new(),
            symbols: Vec::new(),
            stack: StackReport {
                functions: Vec::new(),
            },
            identity: BuildIdentity {
                crate_version: env!("CARGO_PKG_VERSION").to_string(),
                origin: 0,
                source_hash: 0,
                source_names: Vec::new(),
            },
        }
    }

    pub fn from_generated(
        code: &GeneratedProgram,
        units: &[TranslationUnit],
        sources: &SourceMap,
    ) -> Self {
        let mut entries = Vec::new();
        for func in &code.functions {
            entries.extend(func.mapped.iter().cloned());
        }
        for global in &code.globals {
            if let Some(mapped) = data_entry_for_global(global, &code.assembled) {
                entries.push(mapped);
            }
        }
        let no_code = no_code_spans(units, &entries);
        let symbols = collect_symbols(&code.functions, &code.globals);
        let stack = StackReport {
            functions: code
                .functions
                .iter()
                .map(|f| FnStack {
                    name: f.name.clone(),
                    frame_bytes: f.frame_bytes,
                    local_peak_bytes: f.frame_bytes,
                    additional_bytes: match f.stack_provenance {
                        StackProvenance::Unknown => None,
                        _ => Some(f.stack_bound.saturating_sub(f.frame_bytes)),
                    },
                    provenance: f.stack_provenance,
                })
                .collect(),
        };
        Self {
            entries,
            no_code,
            symbols,
            stack,
            identity: BuildIdentity {
                crate_version: env!("CARGO_PKG_VERSION").to_string(),
                origin: code.origin,
                source_hash: hash_sources(sources, code.origin),
                source_names: sources.files().iter().map(|f| f.name.clone()).collect(),
            },
        }
    }

    pub fn by_id(&self, id: AsmInstructionId) -> Option<&MappedInstruction> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn at_address(&self, addr: u16) -> Option<&MappedInstruction> {
        self.entries.iter().find(|e| {
            let end = e.address.wrapping_add(e.bytes.len() as u16);
            if e.bytes.is_empty() {
                return false;
            }
            addr_in_range(addr, e.address, end)
        })
    }

    pub fn covering(&self, file: FileId, offset: u32) -> Vec<&MappedInstruction> {
        let mut hits: Vec<&MappedInstruction> = self
            .entries
            .iter()
            .filter(|e| covers_offset(e, file, offset))
            .collect();
        hits.sort_by_key(|e| span_len(e));
        hits
    }

    pub fn select(&self, file: FileId, offset: u32) -> Option<&MappedInstruction> {
        let hits = self.covering(file, offset);
        hits.iter()
            .copied()
            .find(|e| {
                e.expression_span
                    .is_some_and(|s| s.file == file && s.contains_offset(offset))
            })
            .or_else(|| hits.first().copied())
    }

    pub fn symbol(&self, name: &str) -> Option<&CompilerSymbol> {
        self.symbols.iter().find(|s| s.name == name)
    }

    pub fn every_byte_mapped(&self, assembled: &AssembledProgram) -> bool {
        for seg in &assembled.segments {
            for i in 0..seg.bytes.len() {
                let addr = seg.addr.wrapping_add(i as u16);
                if self.at_address(addr).is_none() {
                    return false;
                }
            }
        }
        true
    }

    pub fn span_cost(&self, entries: &[&MappedInstruction]) -> SpanCost {
        let addrs: Vec<u16> = entries.iter().map(|e| e.address).collect();
        let mut local = 0u16;
        let mut complete = true;
        let mut has_call = false;
        let mut has_loop = false;
        let mut unknown = false;
        for e in entries {
            if is_call(&e.bytes) {
                has_call = true;
                complete = false;
            }
            if let Some(target) = jump_target(e) {
                if addrs.iter().any(|&a| a == target) && !addr_in_range_forward(e.address, target) {
                    has_loop = true;
                    complete = false;
                }
            }
            match e.timing {
                StaticTiming::Exact(n) => local = local.saturating_add(n),
                StaticTiming::Branch {
                    not_taken,
                    taken: _,
                } => {
                    local = local.saturating_add(not_taken);
                    complete = false;
                }
                StaticTiming::Repeating { last, .. } => {
                    local = local.saturating_add(last);
                    complete = false;
                }
                StaticTiming::Unknown => {
                    unknown = true;
                    complete = false;
                }
            }
        }
        if has_call || has_loop || unknown {
            complete = false;
        }
        SpanCost {
            local,
            complete,
            has_call,
            has_loop,
            unknown,
        }
    }
}

impl SourceSpan {
    pub fn contains_offset(self, offset: u32) -> bool {
        offset >= self.start && offset < self.end.max(self.start.saturating_add(1))
    }

    pub fn contains_span(self, other: Self) -> bool {
        self.file == other.file && self.start <= other.start && self.end >= other.end
    }
}

pub fn static_timing(bytes: &[u8], text: Option<&str>) -> StaticTiming {
    let Some(text) = text else {
        return StaticTiming::Unknown;
    };
    if let Some((left, right)) = text.split_once('/') {
        let Ok(a) = left.parse::<u16>() else {
            return StaticTiming::Unknown;
        };
        let Ok(b) = right.parse::<u16>() else {
            return StaticTiming::Unknown;
        };
        if is_repeating_block(bytes) {
            return StaticTiming::Repeating {
                continuing: a,
                last: b,
            };
        }
        return StaticTiming::Branch {
            taken: a,
            not_taken: b,
        };
    }
    match text.parse::<u16>() {
        Ok(n) => StaticTiming::Exact(n),
        Err(_) => StaticTiming::Unknown,
    }
}

fn is_repeating_block(bytes: &[u8]) -> bool {
    matches!(bytes, [0xED, op, ..] if matches!(op, 0xB0..=0xB3 | 0xB8..=0xBB))
}

fn is_call(bytes: &[u8]) -> bool {
    match bytes.first().copied() {
        Some(0xCD) => true,
        Some(op) if op & 0xC7 == 0xC4 => true,
        Some(op) if op & 0xC7 == 0xC7 => true,
        Some(0xDD | 0xFD) => bytes.get(1).copied() == Some(0xCD),
        _ => false,
    }
}

fn jump_target(entry: &MappedInstruction) -> Option<u16> {
    match entry.bytes.as_slice() {
        [0xC3, lo, hi, ..]
        | [
            0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA,
            lo,
            hi,
            ..,
        ] => Some(u16::from(*lo) | (u16::from(*hi) << 8)),
        [0x18, e, ..] | [0x20 | 0x28 | 0x30 | 0x38, e, ..] => {
            let disp = *e as i8 as i16;
            Some(entry.address.wrapping_add(2).wrapping_add(disp as u16))
        }
        _ => None,
    }
}

fn addr_in_range(addr: u16, start: u16, end: u16) -> bool {
    if start <= end {
        addr >= start && addr < end
    } else {
        addr >= start || addr < end
    }
}

fn addr_in_range_forward(from: u16, target: u16) -> bool {
    target.wrapping_sub(from) < 0x8000 && target != from
}

fn covers_offset(entry: &MappedInstruction, file: FileId, offset: u32) -> bool {
    if let Some(span) = entry.expression_span {
        if span.file == file && span.contains_offset(offset) {
            return true;
        }
    }
    if let Some(span) = entry.statement_span {
        if span.file == file && span.contains_offset(offset) {
            return true;
        }
    }
    entry.span.file == file && entry.span.contains_offset(offset)
}

fn span_len(entry: &MappedInstruction) -> u32 {
    if let Some(span) = entry.expression_span {
        return span.len();
    }
    if let Some(span) = entry.statement_span {
        return span.len();
    }
    entry.span.len()
}

fn data_entry_for_global(
    global: &GeneratedGlobal,
    assembled: &AssembledProgram,
) -> Option<MappedInstruction> {
    if global.size == 0 {
        return None;
    }
    let bytes =
        super::z80::bytes_in_segments(assembled, global.addr, global.size as usize)?.to_vec();
    Some(MappedInstruction {
        id: AsmInstructionId(0),
        address: global.addr,
        bytes,
        text: global.label.clone(),
        t_states: None,
        timing: StaticTiming::Unknown,
        kind: MappedKind::Data,
        synthetic: false,
        expression_span: None,
        statement_span: None,
        function: None,
        span: global.span,
        node: None,
    })
}

fn collect_symbols(
    functions: &[GeneratedFunction],
    globals: &[GeneratedGlobal],
) -> Vec<CompilerSymbol> {
    let mut symbols = Vec::new();
    for func in functions {
        symbols.push(CompilerSymbol {
            name: func.name.clone(),
            label: func.label.clone(),
            addr: func.addr,
            size: func.size,
            ty: None,
            kind: SymbolKind::Function,
        });
    }
    for global in globals {
        symbols.push(CompilerSymbol {
            name: global.name.clone(),
            label: global.label.clone(),
            addr: global.addr,
            size: global.size,
            ty: Some(global.ty),
            kind: SymbolKind::Global,
        });
    }
    symbols
}

fn no_code_spans(units: &[TranslationUnit], entries: &[MappedInstruction]) -> Vec<NoCodeSpan> {
    let mut out = Vec::new();
    for unit in units {
        for item in &unit.items {
            if let Item::Function(func) = item {
                collect_no_code(&func.body.stmts, entries, &mut out);
            }
        }
    }
    out
}

fn collect_no_code(stmts: &[Stmt], entries: &[MappedInstruction], out: &mut Vec<NoCodeSpan>) {
    for stmt in stmts {
        match stmt {
            Stmt::Block(block) => collect_no_code(&block.stmts, entries, out),
            Stmt::If(s) => {
                collect_no_code(std::slice::from_ref(s.then_branch.as_ref()), entries, out);
                if let Some(els) = &s.else_branch {
                    collect_no_code(std::slice::from_ref(els.as_ref()), entries, out);
                }
            }
            Stmt::While(s) => collect_no_code(std::slice::from_ref(s.body.as_ref()), entries, out),
            Stmt::For(s) => {
                if let Some(ForInit::Decl(decl)) = &s.init {
                    if !entries
                        .iter()
                        .any(|e| !e.synthetic && span_inside(e, decl.span))
                    {
                        out.push(NoCodeSpan {
                            span: decl.span,
                            reason: NoCodeReason::Eliminated,
                        });
                    }
                }
                collect_no_code(std::slice::from_ref(s.body.as_ref()), entries, out);
            }
            Stmt::DoWhile(s) => {
                collect_no_code(std::slice::from_ref(s.body.as_ref()), entries, out)
            }
            Stmt::Decl(decl) => {
                if !entries
                    .iter()
                    .any(|e| !e.synthetic && span_inside(e, decl.span))
                {
                    out.push(NoCodeSpan {
                        span: decl.span,
                        reason: NoCodeReason::Eliminated,
                    });
                }
            }
            Stmt::Expr(_)
            | Stmt::Return(_)
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Asm(_)
            | Stmt::Error { .. } => {}
        }
    }
}

fn span_inside(entry: &MappedInstruction, outer: SourceSpan) -> bool {
    let inner = entry
        .expression_span
        .or(entry.statement_span)
        .unwrap_or(entry.span);
    outer.contains_span(inner) || inner.contains_span(outer) && inner.len() < outer.len()
}

fn hash_sources(sources: &SourceMap, origin: u16) -> u64 {
    let mut hasher = DefaultHasher::new();
    origin.hash(&mut hasher);
    for file in sources.files() {
        file.name.hash(&mut hasher);
        file.text.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_parses_exact_branch_and_repeat() {
        assert_eq!(static_timing(&[0xC9], Some("10")), StaticTiming::Exact(10));
        assert_eq!(
            static_timing(&[0x28, 0x00], Some("12/7")),
            StaticTiming::Branch {
                taken: 12,
                not_taken: 7
            }
        );
        assert_eq!(
            static_timing(&[0xED, 0xB0], Some("21/16")),
            StaticTiming::Repeating {
                continuing: 21,
                last: 16
            }
        );
        assert_eq!(
            static_timing(&[0xDD, 0xDD, 0x00], None),
            StaticTiming::Unknown
        );
    }
}
