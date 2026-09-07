//! C80 compiler library: source model, parser, typed IR, and Z80 codegen.

mod abi;
mod ast;
mod diagnostic;
mod inline_asm;
mod ir;
mod lexer;
mod lower;
mod parser;
mod semantics;
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
pub use diagnostic::{DiagCode, Diagnostic, RelatedSpan, Severity};
pub use ir::{IrBinary, IrOp, TypedFunction, TypedProgram, function_by_name};
pub use lower::DEFAULT_CODE_ORIGIN;
pub use source::{FileId, IdGen, NodeId, SourceFile, SourceMap, SourceSpan};
pub use source_map::{
    BuildIdentity, CompilerMap, CompilerSymbol, FnStack, NoCodeReason, NoCodeSpan, SpanCost,
    StackReport, SymbolKind,
};
pub use types::CType;
pub use z80::{
    AsmInstructionId, GeneratedFunction, GeneratedGlobal, GeneratedProgram, MappedInstruction,
    MappedKind, R8, RegHome, Rr, StackProvenance, StaticTiming, Z80Item, Z80Op,
};

use parser::parse_file;
use semantics::analyze_unit;

/// In-memory compilation input. The core never reads the filesystem.
pub struct CompileInput<'a> {
    pub files: Vec<SourceInput<'a>>,
    pub origin: u16,
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
        globals: Vec::new(),
        functions: Vec::new(),
    };
    for unit in &units {
        let part = analyze_unit(unit, &mut diagnostics);
        program.globals.extend(part.globals);
        program.functions.extend(part.functions);
    }
    let program = if diagnostics.iter().any(Diagnostic::is_error) {
        None
    } else {
        Some(program)
    };
    let code = program.as_ref().and_then(|program| {
        lower::lower_program(program, input.origin, &mut ids, &mut diagnostics)
    });
    CompilationResult {
        sources,
        diagnostics,
        units,
        program,
        code,
    }
}

pub fn compile_source(name: &str, text: &str) -> CompilationResult {
    compile(CompileInput {
        files: vec![SourceInput { name, text }],
        origin: DEFAULT_CODE_ORIGIN,
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
