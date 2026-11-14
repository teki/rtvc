//! Located compiler diagnostics.

use super::source::{SourceMap, SourceSpan};

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
    LnOverlap,
    LnMissingOrigin,
    LnStack,
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
            Self::LnOverlap => "ln-overlap",
            Self::LnMissingOrigin => "ln-missing-origin",
            Self::LnStack => "ln-stack",
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

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
        }
    }
}

/// `path:line:col` of the span start, or `None` when the span's file is not
/// in `sources` (a synthetic span).
fn location(sources: &SourceMap, span: SourceSpan) -> Option<String> {
    let file = sources.files().get(span.file.0 as usize)?;
    let (line, col) = file.line_col(span.start);
    Some(format!("{}:{line}:{col}", file.name))
}

/// The source line holding the span start, with a `^~~~` underline of the
/// span (to the end of that line for a multi-line span).
fn excerpt(sources: &SourceMap, span: SourceSpan, out: &mut String) {
    let Some(file) = sources.files().get(span.file.0 as usize) else {
        return;
    };
    let text = file.text.as_str();
    let start = (span.start as usize).min(text.len());
    let line_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
    let line_end = text[start..].find('\n').map_or(text.len(), |i| start + i);
    let line = text[line_start..line_end].trim_end_matches('\r');
    let (line_no, _) = file.line_col(span.start);
    let gutter = line_no.to_string().len();
    let end = (span.end as usize).clamp(start, line_start + line.len());
    // Keep tabs so the caret lines up under the same indentation.
    let pad: String = line[..start - line_start]
        .chars()
        .map(|c| if c == '\t' { '\t' } else { ' ' })
        .collect();
    let width = line[start - line_start..end - line_start]
        .chars()
        .count()
        .max(1);
    out.push_str(&format!("{line_no:>gutter$} | {line}\n"));
    out.push_str(&format!(
        "{:gutter$} | {pad}^{}\n",
        "",
        "~".repeat(width - 1)
    ));
}

/// Render a diagnostic for a terminal:
///
/// ```text
/// path:line:col: error code: message
///  12 |     u16 s @ hl = 0;
///     |         ^~~~~~
///   path:line:col: note: related message
/// ```
pub fn render_diagnostic(sources: &SourceMap, diag: &Diagnostic) -> String {
    let mut out = String::new();
    if let Some(loc) = location(sources, diag.span) {
        out.push_str(&loc);
        out.push_str(": ");
    }
    out.push_str(&format!(
        "{} {}: {}\n",
        diag.severity.as_str(),
        diag.code.as_str(),
        diag.message
    ));
    excerpt(sources, diag.span, &mut out);
    for rel in &diag.related {
        out.push_str("  ");
        if let Some(loc) = location(sources, rel.span) {
            out.push_str(&loc);
            out.push_str(": ");
        }
        out.push_str(&format!("note: {}\n", rel.message));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_shows_location_line_and_underline() {
        let mut sources = SourceMap::new();
        let id = sources.add("dir/f.c80", "u8 f() {\n\treturn 300;\n}\n");
        let mut diag = Diagnostic::error(
            DiagCode::TyLiteralRange,
            SourceSpan::new(id, 17, 20),
            "300 does not fit u8",
        );
        diag.related.push(RelatedSpan {
            span: SourceSpan::new(id, 0, 2),
            message: "return type".to_string(),
        });
        assert_eq!(
            render_diagnostic(&sources, &diag),
            "dir/f.c80:2:9: error ty-literal-range: 300 does not fit u8\n\
             2 | \treturn 300;\n\
             \x20 | \t       ^~~\n\
             \x20 dir/f.c80:1:1: note: return type\n"
        );
    }

    #[test]
    fn render_without_a_known_file_has_no_location() {
        let sources = SourceMap::new();
        let diag = Diagnostic::error(
            DiagCode::LnOverlap,
            SourceSpan::new(super::super::source::FileId(7), 0, 0),
            "a overlaps b",
        );
        assert_eq!(
            render_diagnostic(&sources, &diag),
            "error ln-overlap: a overlaps b\n"
        );
    }
}
