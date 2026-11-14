//! C80 compiler library: source model, parser, typed IR, and Z80 codegen.

mod abi;
mod ast;
mod diagnostic;
mod fold;
mod inline;
mod inline_asm;
mod ir;
mod lexer;
mod liveness;
mod lower;
mod optimize;
mod parser;
mod semantics;
mod shift;
mod source;
mod source_map;
mod token;
mod types;
mod z80;

pub mod harness;
pub mod project;

pub use ast::{
    AsmClause, AsmClobber, AsmGpr, AsmOutReg, AsmStmt, BinaryOp, Block, CallConv, Expr, ExprKind,
    Function, Ident, Import, Item, Param, Stmt, TranslationUnit, TypeExpr, TypeKind, UnaryOp,
    VarDecl,
};
pub use diagnostic::{DiagCode, Diagnostic, RelatedSpan, Severity, render_diagnostic};
pub use ir::{IrBinary, IrOp, TypedFunction, TypedProgram, function_by_name};
pub use lower::DEFAULT_CODE_ORIGIN;
pub use project::CompiledBasic;
pub use source::{FileId, IdGen, NodeId, SourceFile, SourceMap, SourceSpan};
pub use source_map::{
    BuildIdentity, CompilerMap, CompilerSymbol, FnStack, MappedDataRef, NoCodeReason, NoCodeSpan,
    SpanCost, StackReport, SymbolKind,
};
pub use types::CType;
pub use z80::{
    AsmInstructionId, Contract, DataRef, GeneratedFunction, GeneratedGlobal, GeneratedProgram,
    InlineReport, MappedInstruction, MappedKind, R8, RegHome, Rr, SpillReport, StackProvenance,
    StaticTiming, Z80Item, Z80Op,
};

use parser::parse_file;
use semantics::analyze_unit;

/// Port I/O lowering profile. TVC uses immediate `IN A,(n)` / `OUT (n),A`
/// for constant ports; GenericZ80 and Zx82 zero-extend the byte port to BC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C80Target {
    GenericZ80,
    Tvc,
    Zx82,
}

impl C80Target {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GenericZ80 => "generic-z80",
            Self::Tvc => "tvc",
            Self::Zx82 => "zx82",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "generic-z80" => Self::GenericZ80,
            "tvc" => Self::Tvc,
            "zx82" => Self::Zx82,
            _ => return None,
        })
    }

    pub fn uses_immediate_port(self) -> bool {
        matches!(self, Self::Tvc)
    }
}

impl From<project::ProjectTarget> for C80Target {
    fn from(target: project::ProjectTarget) -> Self {
        match target {
            project::ProjectTarget::GenericZ80 => Self::GenericZ80,
            project::ProjectTarget::Tvc => Self::Tvc,
            project::ProjectTarget::Zx82 => Self::Zx82,
        }
    }
}

/// In-memory compilation input. The core never reads the filesystem.
pub struct CompileInput<'a> {
    pub files: Vec<SourceInput<'a>>,
    pub origin: u16,
    /// When true (the default for `compile_source` and the CLI), run the
    /// optimizer: constant folding, automatic inlining, liveness-gated
    /// peepholes, register tracking, jump threading, loop rotation, `DJNZ`,
    /// tail calls, and `JP` to `JR` shortening (info/c80.md, Optimization).
    pub optimize: bool,
    pub target: C80Target,
}

pub struct SourceInput<'a> {
    pub name: &'a str,
    pub text: &'a str,
}

pub struct CompilationResult {
    pub sources: SourceMap,
    pub diagnostics: Vec<Diagnostic>,
    pub units: Vec<TranslationUnit>,
    pub program: Option<TypedProgram>,
    pub code: Option<GeneratedProgram>,
    pub basic: Option<CompiledBasic>,
}

impl CompilationResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.is_error()).count()
    }

    pub fn map(&self) -> Option<CompilerMap> {
        let code = self.code.as_ref()?;
        Some(CompilerMap::from_generated(
            code,
            &self.units,
            &self.sources,
        ))
    }
}

/// Compile in-memory C80 sources to AST, diagnostics, typed IR, and Z80 when possible.
///
/// Parse/type errors yield `program: None` and `code: None`. Codegen errors
/// yield `code: None`. Scalar functions, control flow, globals, calls, arrays,
/// pointers, and strings produce assembled code when lowering succeeds.
pub fn compile(input: CompileInput<'_>) -> CompilationResult {
    compile_impl(input, false)
}

/// [`compile`], but every function keeps its out-of-line body even when
/// all its calls were inlined (H3i). For test harnesses that call such a
/// function by name; the other functions' code is unchanged.
#[doc(hidden)]
pub fn compile_keeping_bodies(input: CompileInput<'_>) -> CompilationResult {
    compile_impl(input, true)
}

fn compile_impl(input: CompileInput<'_>, keep_bodies: bool) -> CompilationResult {
    let mut sources = SourceMap::new();
    let mut diagnostics = Vec::new();
    let mut ids = IdGen::new();
    let mut units = Vec::new();
    for file in input.files {
        let id = sources.add(file.name, file.text);
        let unit = parse_file(sources.get(id), &mut ids, &mut diagnostics);
        units.push(unit);
    }
    let mut program = TypedProgram {
        structs: Vec::new(),
        globals: Vec::new(),
        functions: Vec::new(),
    };
    for unit in &units {
        let part = analyze_unit(unit, &mut diagnostics);
        program.structs.extend(part.structs);
        program.globals.extend(part.globals);
        program.functions.extend(part.functions);
    }
    let program = if diagnostics.iter().any(Diagnostic::is_error) {
        None
    } else {
        Some(program)
    };
    let program = program.and_then(|mut program| {
        inline_program(
            &mut program,
            &mut ids,
            &mut diagnostics,
            input.optimize,
            input.target,
            keep_bodies,
        )
        .then_some(program)
    });
    let code = program.as_ref().and_then(|program| {
        lower::lower_program(
            program,
            input.origin,
            &mut ids,
            &mut diagnostics,
            input.optimize,
            input.target,
        )
    });
    CompilationResult {
        sources,
        diagnostics,
        units,
        program,
        code,
        basic: None,
    }
}

/// Expand `@inline` calls, then (optimized builds) automatic inlining (H3i).
/// False when a call could not be expanded.
pub(crate) fn inline_program(
    program: &mut TypedProgram,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
    optimize: bool,
    target: C80Target,
    keep_bodies: bool,
) -> bool {
    if optimize {
        fold::prune_unreachable(program);
    }
    inline::carry_block_values(program, ids);
    if !inline::expand_inline_calls(program, ids, diagnostics) {
        return false;
    }
    let before = inline::call_counts(program);
    if optimize {
        let snapshot = ids.clone();
        inline::auto_inline(program, ids, &mut |p| {
            lower::measure_functions(p, &snapshot, target)
        });
    }
    // A private function is reachable only through C80 calls, so once none
    // is left (every one expanded) its body would be dead code.
    if optimize {
        fold::fold_constants(program);
    }
    let after = inline::call_counts(program);
    for f in &mut program.functions {
        let absorbed = before.get(&f.id).is_some_and(|&n| n > 0) && !after.contains_key(&f.id);
        f.out_of_line = keep_bodies || f.is_pub || (f.inline.is_none() && !absorbed);
    }
    true
}

pub fn compile_source(name: &str, text: &str) -> CompilationResult {
    compile(CompileInput {
        files: vec![SourceInput { name, text }],
        origin: DEFAULT_CODE_ORIGIN,
        optimize: true,
        target: C80Target::GenericZ80,
    })
}

#[cfg(test)]
#[path = "parse_tests.rs"]
mod parse_tests;

#[cfg(test)]
#[path = "semantics_tests.rs"]
mod semantics_tests;

#[cfg(test)]
#[path = "codegen_tests.rs"]
mod codegen_tests;

#[cfg(test)]
#[path = "conformance_tests.rs"]
mod conformance_tests;

#[cfg(test)]
#[path = "inline_tests.rs"]
mod inline_tests;

#[cfg(test)]
#[path = "quality_tests.rs"]
mod quality_tests;
