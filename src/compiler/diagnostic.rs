//! Located compiler diagnostics.

use super::source::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagCode {
    LexUnexpectedCharacter,
    LexInvalidIdentifier,
    LexUnterminatedBlockComment,
    LexUnterminatedString,
    LexUnterminatedChar,
    LexInvalidEscape,
    LexMalformedLiteral,
    LexIntegerOverflow,
    ParseExpected,
    ParseIncomplete,
    ParseUnsupported,
    TyMismatch,
    TyLiteralRange,
    TyDuplicateName,
    TyUnresolvedName,
    TyNotLvalue,
    TyUseBeforeAssign,
    TyMissingReturn,
    TyRecursion,
    TyInvalidConversion,
    TyVoidValue,
    TyAssignConst,
    TyReturnLocalAddr,
    CgUnsupported,
    CgInternal,
}

impl DiagCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LexUnexpectedCharacter => "lex-unexpected-character",
            Self::LexInvalidIdentifier => "lex-invalid-identifier",
            Self::LexUnterminatedBlockComment => "lex-unterminated-block-comment",
            Self::LexUnterminatedString => "lex-unterminated-string",
            Self::LexUnterminatedChar => "lex-unterminated-char",
            Self::LexInvalidEscape => "lex-invalid-escape",
            Self::LexMalformedLiteral => "lex-malformed-literal",
            Self::LexIntegerOverflow => "lex-integer-overflow",
            Self::ParseExpected => "parse-expected",
            Self::ParseIncomplete => "parse-incomplete",
            Self::ParseUnsupported => "parse-unsupported",
            Self::TyMismatch => "ty-mismatch",
            Self::TyLiteralRange => "ty-literal-range",
            Self::TyDuplicateName => "ty-duplicate-name",
            Self::TyUnresolvedName => "ty-unresolved-name",
            Self::TyNotLvalue => "ty-not-lvalue",
            Self::TyUseBeforeAssign => "ty-use-before-assign",
            Self::TyMissingReturn => "ty-missing-return",
            Self::TyRecursion => "ty-recursion",
            Self::TyInvalidConversion => "ty-invalid-conversion",
            Self::TyVoidValue => "ty-void-value",
            Self::TyAssignConst => "ty-assign-const",
            Self::TyReturnLocalAddr => "ty-return-local-addr",
            Self::CgUnsupported => "cg-unsupported",
            Self::CgInternal => "cg-internal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedSpan {
    pub span: SourceSpan,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagCode,
    pub message: String,
    pub span: SourceSpan,
    pub related: Vec<RelatedSpan>,
}

impl Diagnostic {
    pub fn error(code: DiagCode, span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }

    pub fn warning(code: DiagCode, span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}
