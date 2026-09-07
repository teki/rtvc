//! Recoverable recursive-descent / Pratt parser for the E01 scalar subset.

use super::ast::*;
use super::diagnostic::{DiagCode, Diagnostic};
use super::lexer::{IntegerLit, lex};
use super::source::{FileId, IdGen, SourceFile, SourceSpan};
use super::token::{Token, TokenKind};

const MAX_DIAGNOSTICS: usize = 64;

pub fn parse_file(
    file: &SourceFile,
    ids: &mut IdGen,
    diagnostics: &mut Vec<Diagnostic>,
) -> TranslationUnit {
    let lexed = lex(file, diagnostics);
    let mut parser = Parser {
        file: file.id,
        source: file,
        tokens: lexed.tokens,
        integers: lexed.integers,
        chars: lexed.chars,
        strings: lexed.strings,
        pos: 0,
        ids,
        diagnostics,
        error_tokens: 0,
    };
    parser.parse_unit()
}

struct Parser<'a> {
    file: FileId,
    source: &'a SourceFile,
    tokens: Vec<Token>,
    integers: Vec<IntegerLit>,
    chars: Vec<u8>,
    strings: Vec<Vec<u8>>,
    pos: usize,
    ids: &'a mut IdGen,
    diagnostics: &'a mut Vec<Diagnostic>,
    error_tokens: usize,
}

impl<'a> Parser<'a> {
    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn kind(&self) -> TokenKind {
        self.current().kind
    }

    fn span(&self) -> SourceSpan {
        self.current().span
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.kind() == kind
    }

    fn at_eof(&self) -> bool {
        self.kind() == TokenKind::Eof
    }

    fn bump(&mut self) -> Token {
        let tok = self.current().clone();
        if tok.kind != TokenKind::Eof && self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn ident_text(&self, span: SourceSpan) -> String {
        self.source.slice(span).to_string()
    }

    fn nth(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    fn integer_index_of(&self, token_index: usize) -> usize {
        self.tokens[..token_index]
            .iter()
            .filter(|t| t.kind == TokenKind::Integer)
            .count()
    }

    fn char_index_of(&self, token_index: usize) -> usize {
        self.tokens[..token_index]
            .iter()
            .filter(|t| t.kind == TokenKind::Char)
            .count()
    }

    fn string_index_of(&self, token_index: usize) -> usize {
        self.tokens[..token_index]
            .iter()
            .filter(|t| t.kind == TokenKind::String)
            .count()
    }

    fn emit(&mut self, code: DiagCode, span: SourceSpan, message: impl Into<String>) {
        if self.diagnostics.iter().filter(|d| d.is_error()).count() >= MAX_DIAGNOSTICS {
            return;
        }
        self.diagnostics
            .push(Diagnostic::error(code, span, message));
    }

    fn expect(&mut self, kind: TokenKind, what: &str) -> Option<Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            self.emit(
                DiagCode::ParseExpected,
                self.span(),
                format!("expected {what}, found {}", self.kind().describe()),
            );
            None
        }
    }

    fn parse_unit(&mut self) -> TranslationUnit {
        let start = self.span().start;
        let mut items = Vec::new();
        while !self.at_eof() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else if !self.at_eof() {
                self.recover_item();
            }
        }
        let end = self.span().end;
        TranslationUnit {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, end),
            items,
        }
    }

    fn parse_item(&mut self) -> Option<Item> {
        match self.kind() {
            TokenKind::Import
            | TokenKind::Struct
            | TokenKind::Asm
            | TokenKind::For
            | TokenKind::Do
            | TokenKind::Ptr
            | TokenKind::Str => {
                let tok = self.bump();
                self.emit(
                    DiagCode::ParseUnsupported,
                    tok.span,
                    format!(
                        "{} is not accepted in this compiler slice",
                        tok.kind.describe()
                    ),
                );
                self.skip_until_item_sync();
                None
            }
            TokenKind::At
            | TokenKind::Pub
            | TokenKind::Const
            | TokenKind::Ident
            | TokenKind::Void
            | TokenKind::Bool
            | TokenKind::U8
            | TokenKind::I8
            | TokenKind::U16
            | TokenKind::I16 => self.parse_decl_or_function(),
            TokenKind::RBrace => {
                let tok = self.bump();
                self.emit(DiagCode::ParseExpected, tok.span, "unexpected '}'");
                None
            }
            _ => {
                let tok = self.bump();
                self.emit(
                    DiagCode::ParseExpected,
                    tok.span,
                    format!("expected declaration, found {}", tok.kind.describe()),
                );
                None
            }
        }
    }

    fn parse_decl_or_function(&mut self) -> Option<Item> {
        let start = self.span().start;
        let conv_before = self.parse_call_conv();
        let is_pub = if self.at(TokenKind::Pub) {
            self.bump();
            true
        } else {
            false
        };
        let conv_after = self.parse_call_conv();
        let conv = match (conv_before, conv_after) {
            (Some(a), Some(b)) if a != b => {
                self.emit(
                    DiagCode::ParseExpected,
                    self.span(),
                    "conflicting calling-convention attributes",
                );
                a
            }
            (Some(a), Some(_)) => {
                self.emit(
                    DiagCode::ParseExpected,
                    self.span(),
                    "duplicate calling-convention attribute",
                );
                a
            }
            (Some(c), None) | (None, Some(c)) => c,
            (None, None) => CallConv::Register,
        };
        let is_const = if self.at(TokenKind::Const) {
            self.bump();
            true
        } else {
            false
        };
        let ty = self.parse_type()?;
        let name = self.parse_ident()?;
        if self.at(TokenKind::LParen) && !is_const {
            let func = self.parse_function_rest(start, is_pub, conv, ty, name)?;
            return Some(Item::Function(func));
        }
        if conv_before.is_some() || conv_after.is_some() {
            self.emit(
                DiagCode::ParseExpected,
                name.span,
                "calling-convention attributes apply only to functions",
            );
        }
        let init = if self.at(TokenKind::Eq) {
            self.bump();
            Some(self.parse_expr())
        } else {
            None
        };
        let semi = self.expect(TokenKind::Semicolon, "';'");
        let end = semi.map(|t| t.span.end).unwrap_or(self.span().start);
        Some(Item::Decl(VarDecl {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, end),
            is_pub,
            is_const,
            ty,
            name,
            init,
        }))
    }

    fn parse_call_conv(&mut self) -> Option<CallConv> {
        let mut conv = None;
        while self.at(TokenKind::At) {
            let at = self.bump();
            let Some(ident) = self.parse_ident() else {
                continue;
            };
            let next = match ident.name.as_str() {
                "stackcall" => CallConv::Stack,
                "fastcall" => CallConv::Register,
                other => {
                    self.emit(
                        DiagCode::ParseExpected,
                        ident.span,
                        format!("unknown attribute '@{other}'"),
                    );
                    continue;
                }
            };
            match conv {
                None => conv = Some(next),
                Some(prev) if prev == next => self.emit(
                    DiagCode::ParseExpected,
                    at.span,
                    "duplicate calling-convention attribute",
                ),
                Some(_) => self.emit(
                    DiagCode::ParseExpected,
                    at.span,
                    "conflicting calling-convention attributes",
                ),
            }
        }
        conv
    }

    fn parse_function_rest(
        &mut self,
        start: u32,
        is_pub: bool,
        conv: CallConv,
        return_ty: TypeExpr,
        name: Ident,
    ) -> Option<Function> {
        self.bump(); // (
        let mut params = Vec::new();
        if !self.at(TokenKind::RParen) && !self.at(TokenKind::LBrace) && !self.at_eof() {
            loop {
                if let Some(param) = self.parse_param() {
                    params.push(param);
                } else {
                    self.skip_until(&[
                        TokenKind::Comma,
                        TokenKind::RParen,
                        TokenKind::LBrace,
                        TokenKind::Semicolon,
                    ]);
                }
                if self.at(TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }
        if self.expect(TokenKind::RParen, "')'").is_none() {
            self.skip_until(&[TokenKind::LBrace, TokenKind::Semicolon, TokenKind::RBrace]);
        }
        let body = self.parse_block();
        let end = body.span.end;
        Some(Function {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, end),
            is_pub,
            conv,
            return_ty,
            name,
            params,
            body,
        })
    }

    fn parse_param(&mut self) -> Option<Param> {
        let start = self.span().start;
        let ty = self.parse_type()?;
        let name = self.parse_ident()?;
        Some(Param {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, name.span.end),
            ty,
            name,
        })
    }

    fn parse_type(&mut self) -> Option<TypeExpr> {
        if self.kind().is_type_start() {
            let tok = self.bump();
            Some(TypeExpr {
                kind: TypeKind::from_token(tok.kind).unwrap(),
                span: tok.span,
            })
        } else {
            self.emit(
                DiagCode::ParseExpected,
                self.span(),
                format!("expected type, found {}", self.kind().describe()),
            );
            None
        }
    }

    fn parse_ident(&mut self) -> Option<Ident> {
        if self.at(TokenKind::Ident) {
            let tok = self.bump();
            Some(Ident {
                name: self.ident_text(tok.span),
                span: tok.span,
            })
        } else {
            self.emit(
                DiagCode::ParseExpected,
                self.span(),
                format!("expected identifier, found {}", self.kind().describe()),
            );
            None
        }
    }

    fn parse_block(&mut self) -> Block {
        let start_tok = match self.expect(TokenKind::LBrace, "'{'") {
            Some(tok) => tok,
            None => {
                self.emit(DiagCode::ParseIncomplete, self.span(), "incomplete block");
                return Block {
                    id: self.ids.next(),
                    span: self.span(),
                    stmts: Vec::new(),
                };
            }
        };
        let mut stmts = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at_eof() {
            if self.at_item_starter() && stmts_look_like_next_function(self) {
                self.emit(
                    DiagCode::ParseIncomplete,
                    self.span(),
                    "unclosed block before the next declaration",
                );
                break;
            }
            stmts.push(self.parse_stmt());
        }
        let end = if self.at(TokenKind::RBrace) {
            self.bump().span.end
        } else {
            self.emit(DiagCode::ParseIncomplete, self.span(), "unclosed block");
            self.span().start
        };
        Block {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start_tok.span.start, end),
            stmts,
        }
    }

    fn at_item_starter(&self) -> bool {
        matches!(
            self.kind(),
            TokenKind::Pub
                | TokenKind::Const
                | TokenKind::Void
                | TokenKind::Bool
                | TokenKind::U8
                | TokenKind::I8
                | TokenKind::U16
                | TokenKind::I16
        )
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.kind() {
            TokenKind::LBrace => Stmt::Block(self.parse_block()),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => {
                let tok = self.bump();
                let semi = self.expect(TokenKind::Semicolon, "';'");
                Stmt::Break {
                    id: self.ids.next(),
                    span: SourceSpan::new(
                        self.file,
                        tok.span.start,
                        semi.map(|t| t.span.end).unwrap_or(tok.span.end),
                    ),
                }
            }
            TokenKind::Continue => {
                let tok = self.bump();
                let semi = self.expect(TokenKind::Semicolon, "';'");
                Stmt::Continue {
                    id: self.ids.next(),
                    span: SourceSpan::new(
                        self.file,
                        tok.span.start,
                        semi.map(|t| t.span.end).unwrap_or(tok.span.end),
                    ),
                }
            }
            TokenKind::Const => {
                if let Some(decl) = self.parse_local_decl() {
                    Stmt::Decl(decl)
                } else {
                    self.recover_stmt()
                }
            }
            TokenKind::Void
            | TokenKind::Bool
            | TokenKind::U8
            | TokenKind::I8
            | TokenKind::U16
            | TokenKind::I16 => {
                if self.nth(1) == TokenKind::Ident {
                    if let Some(decl) = self.parse_local_decl() {
                        Stmt::Decl(decl)
                    } else {
                        self.recover_stmt()
                    }
                } else {
                    let expr = self.parse_expr();
                    let semi = self.expect(TokenKind::Semicolon, "';' after expression");
                    let end = semi.map(|t| t.span.end).unwrap_or(expr.span.end);
                    Stmt::Expr(ExprStmt {
                        id: self.ids.next(),
                        span: SourceSpan::new(self.file, expr.span.start, end),
                        expr,
                    })
                }
            }
            TokenKind::Semicolon => {
                let tok = self.bump();
                Stmt::Error {
                    id: self.ids.next(),
                    span: tok.span,
                }
            }
            _ => {
                let expr = self.parse_expr();
                let semi = self.expect(TokenKind::Semicolon, "';' after expression");
                let end = semi.map(|t| t.span.end).unwrap_or(expr.span.end);
                Stmt::Expr(ExprStmt {
                    id: self.ids.next(),
                    span: SourceSpan::new(self.file, expr.span.start, end),
                    expr,
                })
            }
        }
    }

    fn parse_local_decl(&mut self) -> Option<VarDecl> {
        let start = self.span().start;
        let is_const = if self.at(TokenKind::Const) {
            self.bump();
            true
        } else {
            false
        };
        let ty = self.parse_type()?;
        let name = self.parse_ident()?;
        let init = if self.at(TokenKind::Eq) {
            self.bump();
            Some(self.parse_expr())
        } else {
            None
        };
        let semi = self.expect(TokenKind::Semicolon, "';'");
        let end = semi.map(|t| t.span.end).unwrap_or(name.span.end);
        Some(VarDecl {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, end),
            is_pub: false,
            is_const,
            ty,
            name,
            init,
        })
    }

    fn parse_if(&mut self) -> Stmt {
        let start = self.bump().span.start;
        self.expect(TokenKind::LParen, "'('");
        let cond = self.parse_expr();
        if self.expect(TokenKind::RParen, "')'").is_none() {
            self.skip_until(&[TokenKind::RParen, TokenKind::LBrace, TokenKind::Semicolon]);
            if self.at(TokenKind::RParen) {
                self.bump();
            }
        }
        let then_branch = Box::new(self.parse_stmt());
        let else_branch = if self.at(TokenKind::Else) {
            self.bump();
            Some(Box::new(self.parse_stmt()))
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map(|s| stmt_span(s).end)
            .unwrap_or_else(|| stmt_span(&then_branch).end);
        Stmt::If(IfStmt {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, end),
            cond,
            then_branch,
            else_branch,
        })
    }

    fn parse_while(&mut self) -> Stmt {
        let start = self.bump().span.start;
        self.expect(TokenKind::LParen, "'('");
        let cond = self.parse_expr();
        if self.expect(TokenKind::RParen, "')'").is_none() {
            self.skip_until(&[TokenKind::RParen, TokenKind::LBrace, TokenKind::Semicolon]);
            if self.at(TokenKind::RParen) {
                self.bump();
            }
        }
        let body = Box::new(self.parse_stmt());
        let end = stmt_span(&body).end;
        Stmt::While(WhileStmt {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, end),
            cond,
            body,
        })
    }

    fn parse_return(&mut self) -> Stmt {
        let start_tok = self.bump();
        let value = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr())
        };
        let semi = self.expect(TokenKind::Semicolon, "';'");
        let end = semi
            .map(|t| t.span.end)
            .or_else(|| value.as_ref().map(|e| e.span.end))
            .unwrap_or(start_tok.span.end);
        Stmt::Return(ReturnStmt {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start_tok.span.start, end),
            value,
        })
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_prefix();
        loop {
            if let Some((l_bp, r_bp, kind)) = infix_binding(self.kind()) {
                if l_bp < min_bp {
                    break;
                }
                if kind == TokenKind::Eq {
                    self.bump();
                    let rhs = self.parse_expr_bp(r_bp);
                    let span = lhs.span.merge(rhs.span);
                    lhs = Expr {
                        id: self.ids.next(),
                        span,
                        kind: ExprKind::Assign {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                    };
                    continue;
                }
                if matches!(
                    kind,
                    TokenKind::Star
                        | TokenKind::Slash
                        | TokenKind::Percent
                        | TokenKind::PlusEq
                        | TokenKind::MinusEq
                        | TokenKind::StarEq
                        | TokenKind::SlashEq
                        | TokenKind::PercentEq
                        | TokenKind::AmpEq
                        | TokenKind::PipeEq
                        | TokenKind::CaretEq
                        | TokenKind::LtLtEq
                        | TokenKind::GtGtEq
                ) {
                    let op = self.bump();
                    self.emit(
                        DiagCode::ParseUnsupported,
                        op.span,
                        format!(
                            "{} is not accepted in this compiler slice",
                            op.kind.describe()
                        ),
                    );
                    let rhs = self.parse_expr_bp(r_bp);
                    let span = lhs.span.merge(rhs.span);
                    lhs = Expr {
                        id: self.ids.next(),
                        span,
                        kind: ExprKind::Error,
                    };
                    continue;
                }
                let op_tok = self.bump();
                let op = BinaryOp::from_token(op_tok.kind).unwrap();
                let rhs = self.parse_expr_bp(r_bp);
                let span = lhs.span.merge(rhs.span);
                lhs = Expr {
                    id: self.ids.next(),
                    span,
                    kind: ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                };
                continue;
            }
            match self.kind() {
                TokenKind::LParen if 30 >= min_bp => {
                    self.bump();
                    let args = self.parse_arg_list();
                    let end = self
                        .expect(TokenKind::RParen, "')'")
                        .map(|t| t.span.end)
                        .unwrap_or(self.span().start);
                    let span = SourceSpan::new(self.file, lhs.span.start, end);
                    lhs = Expr {
                        id: self.ids.next(),
                        span,
                        kind: ExprKind::Call {
                            callee: Box::new(lhs),
                            args,
                        },
                    };
                }
                TokenKind::LBracket if 30 >= min_bp => {
                    self.bump();
                    let index = self.parse_expr();
                    let end = self
                        .expect(TokenKind::RBracket, "']'")
                        .map(|t| t.span.end)
                        .unwrap_or(index.span.end);
                    let span = SourceSpan::new(self.file, lhs.span.start, end);
                    lhs = Expr {
                        id: self.ids.next(),
                        span,
                        kind: ExprKind::Index {
                            base: Box::new(lhs),
                            index: Box::new(index),
                        },
                    };
                }
                TokenKind::Dot if 30 >= min_bp => {
                    self.bump();
                    if let Some(name) = self.parse_ident() {
                        let span = lhs.span.merge(name.span);
                        lhs = Expr {
                            id: self.ids.next(),
                            span,
                            kind: ExprKind::Field {
                                base: Box::new(lhs),
                                name,
                            },
                        };
                    } else {
                        break;
                    }
                }
                TokenKind::ColonColon => {
                    let tok = self.bump();
                    self.emit(
                        DiagCode::ParseUnsupported,
                        tok.span,
                        "qualified names are not accepted in this compiler slice",
                    );
                    let _ = self.parse_ident();
                    lhs = Expr {
                        id: self.ids.next(),
                        span: lhs.span.merge(tok.span),
                        kind: ExprKind::Error,
                    };
                }
                TokenKind::PlusPlus | TokenKind::MinusMinus | TokenKind::Arrow => {
                    let tok = self.bump();
                    self.emit(
                        DiagCode::ParseUnsupported,
                        tok.span,
                        format!(
                            "{} is not accepted in this compiler slice",
                            tok.kind.describe()
                        ),
                    );
                    lhs = Expr {
                        id: self.ids.next(),
                        span: lhs.span.merge(tok.span),
                        kind: ExprKind::Error,
                    };
                }
                _ => break,
            }
        }
        lhs
    }

    fn parse_arg_list(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.at(TokenKind::RParen) {
            return args;
        }
        loop {
            args.push(self.parse_expr());
            if self.at(TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        args
    }

    fn parse_prefix(&mut self) -> Expr {
        match self.kind() {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Bang | TokenKind::Tilde => {
                let tok = self.bump();
                let op = match tok.kind {
                    TokenKind::Plus => UnaryOp::Plus,
                    TokenKind::Minus => UnaryOp::Minus,
                    TokenKind::Bang => UnaryOp::Not,
                    _ => UnaryOp::BitNot,
                };
                let expr = self.parse_expr_bp(28);
                Expr {
                    id: self.ids.next(),
                    span: tok.span.merge(expr.span),
                    kind: ExprKind::Unary {
                        op,
                        expr: Box::new(expr),
                    },
                }
            }
            TokenKind::Star | TokenKind::Amp => {
                let tok = self.bump();
                self.emit(
                    DiagCode::ParseUnsupported,
                    tok.span,
                    format!(
                        "{} is not accepted in this compiler slice",
                        tok.kind.describe()
                    ),
                );
                let expr = self.parse_expr_bp(28);
                Expr {
                    id: self.ids.next(),
                    span: tok.span.merge(expr.span),
                    kind: ExprKind::Error,
                }
            }
            TokenKind::Void
            | TokenKind::Bool
            | TokenKind::U8
            | TokenKind::I8
            | TokenKind::U16
            | TokenKind::I16 => {
                if self.nth(1) == TokenKind::Ident && self.nth(2) == TokenKind::LParen {
                    let span = self.span();
                    self.emit(
                        DiagCode::ParseExpected,
                        span,
                        format!("expected expression, found {}", self.kind().describe()),
                    );
                    self.error_expr(span)
                } else {
                    self.parse_cast()
                }
            }
            TokenKind::Ident => {
                let tok = self.bump();
                Expr {
                    id: self.ids.next(),
                    span: tok.span,
                    kind: ExprKind::Name(Ident {
                        name: self.ident_text(tok.span),
                        span: tok.span,
                    }),
                }
            }
            TokenKind::Integer => {
                let idx = self.pos;
                let tok = self.bump();
                let lit = self.integers[self.integer_index_of(idx)].clone();
                Expr {
                    id: self.ids.next(),
                    span: tok.span,
                    kind: ExprKind::Int(lit),
                }
            }
            TokenKind::Char => {
                let idx = self.pos;
                let tok = self.bump();
                let value = self
                    .chars
                    .get(self.char_index_of(idx))
                    .copied()
                    .unwrap_or(0);
                Expr {
                    id: self.ids.next(),
                    span: tok.span,
                    kind: ExprKind::Char(value),
                }
            }
            TokenKind::String => {
                let idx = self.pos;
                let tok = self.bump();
                let value = self
                    .strings
                    .get(self.string_index_of(idx))
                    .cloned()
                    .unwrap_or_default();
                Expr {
                    id: self.ids.next(),
                    span: tok.span,
                    kind: ExprKind::String(value),
                }
            }
            TokenKind::True | TokenKind::False => {
                let tok = self.bump();
                Expr {
                    id: self.ids.next(),
                    span: tok.span,
                    kind: ExprKind::Bool(tok.kind == TokenKind::True),
                }
            }
            TokenKind::LParen => {
                let start = self.bump().span.start;
                let expr = self.parse_expr();
                let end = self
                    .expect(TokenKind::RParen, "')'")
                    .map(|t| t.span.end)
                    .unwrap_or(expr.span.end);
                Expr {
                    id: self.ids.next(),
                    span: SourceSpan::new(self.file, start, end),
                    kind: expr.kind,
                }
            }
            TokenKind::Sizeof => self.parse_sizeof(),
            TokenKind::Ptr | TokenKind::Str => {
                let tok = self.bump();
                self.emit(
                    DiagCode::ParseUnsupported,
                    tok.span,
                    format!(
                        "{} is not accepted in this compiler slice",
                        tok.kind.describe()
                    ),
                );
                self.error_expr(tok.span)
            }
            _ => {
                let span = self.span();
                self.emit(
                    DiagCode::ParseExpected,
                    span,
                    format!("expected expression, found {}", self.kind().describe()),
                );
                if !matches!(
                    self.kind(),
                    TokenKind::Semicolon
                        | TokenKind::RBrace
                        | TokenKind::RParen
                        | TokenKind::RBracket
                        | TokenKind::Comma
                        | TokenKind::Eof
                ) {
                    self.bump();
                }
                self.error_expr(span)
            }
        }
    }

    fn parse_cast(&mut self) -> Expr {
        let ty_tok = self.bump();
        let ty = TypeExpr {
            kind: TypeKind::from_token(ty_tok.kind).unwrap(),
            span: ty_tok.span,
        };
        if !self.at(TokenKind::LParen) {
            self.emit(
                DiagCode::ParseExpected,
                self.span(),
                format!("expected '(' after {}", ty.kind.as_str()),
            );
            return self.error_expr(ty.span);
        }
        self.bump();
        let expr = self.parse_expr();
        let end = self
            .expect(TokenKind::RParen, "')'")
            .map(|t| t.span.end)
            .unwrap_or(expr.span.end);
        Expr {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, ty.span.start, end),
            kind: ExprKind::Cast {
                ty,
                expr: Box::new(expr),
            },
        }
    }

    fn parse_sizeof(&mut self) -> Expr {
        let start = self.bump().span.start;
        self.expect(TokenKind::LParen, "'('");
        let ty = match self.parse_type() {
            Some(ty) => ty,
            None => {
                self.skip_until(&[TokenKind::RParen, TokenKind::Semicolon]);
                if self.at(TokenKind::RParen) {
                    self.bump();
                }
                return self.error_expr(SourceSpan::new(self.file, start, self.span().start));
            }
        };
        let end = self
            .expect(TokenKind::RParen, "')'")
            .map(|t| t.span.end)
            .unwrap_or(ty.span.end);
        Expr {
            id: self.ids.next(),
            span: SourceSpan::new(self.file, start, end),
            kind: ExprKind::Sizeof { ty },
        }
    }

    fn error_expr(&mut self, span: SourceSpan) -> Expr {
        Expr {
            id: self.ids.next(),
            span,
            kind: ExprKind::Error,
        }
    }

    fn recover_item(&mut self) {
        self.error_tokens += 1;
        if self.error_tokens > 256 {
            self.pos = self.tokens.len().saturating_sub(1);
            return;
        }
        self.skip_until_item_sync();
    }

    fn recover_stmt(&mut self) -> Stmt {
        let start = self.span();
        self.skip_until(&[
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Pub,
            TokenKind::Const,
        ]);
        if self.at(TokenKind::Semicolon) {
            self.bump();
        }
        Stmt::Error {
            id: self.ids.next(),
            span: start,
        }
    }

    fn skip_until_item_sync(&mut self) {
        let mut depth = 0u32;
        while !self.at_eof() {
            match self.kind() {
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace => {
                    self.bump();
                    if depth == 0 {
                        return;
                    }
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return;
                    }
                }
                TokenKind::Semicolon if depth == 0 => {
                    self.bump();
                    return;
                }
                TokenKind::Pub | TokenKind::Const | TokenKind::At if depth == 0 && self.pos > 0 => {
                    return;
                }
                k if depth == 0 && k.is_type_start() && self.pos > 0 => return,
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn skip_until(&mut self, kinds: &[TokenKind]) {
        let mut depth = 0u32;
        while !self.at_eof() {
            let kind = self.kind();
            if depth == 0 && kinds.contains(&kind) {
                return;
            }
            match kind {
                TokenKind::LBrace | TokenKind::LParen | TokenKind::LBracket => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace | TokenKind::RParen | TokenKind::RBracket => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }
}

fn stmts_look_like_next_function(parser: &Parser<'_>) -> bool {
    // `void name(` or `pub void name(` at the start of what would be a new unit item.
    let mut i = parser.pos;
    if parser.tokens.get(i).map(|t| t.kind) == Some(TokenKind::Pub) {
        i += 1;
    }
    let Some(ty) = parser.tokens.get(i) else {
        return false;
    };
    if !ty.kind.is_type_start() {
        return false;
    }
    let Some(name) = parser.tokens.get(i + 1) else {
        return false;
    };
    if name.kind != TokenKind::Ident {
        return false;
    }
    parser.tokens.get(i + 2).map(|t| t.kind) == Some(TokenKind::LParen)
}

fn stmt_span(stmt: &Stmt) -> SourceSpan {
    match stmt {
        Stmt::Block(b) => b.span,
        Stmt::Decl(d) => d.span,
        Stmt::Expr(e) => e.span,
        Stmt::If(s) => s.span,
        Stmt::While(s) => s.span,
        Stmt::Return(s) => s.span,
        Stmt::Break { span, .. } | Stmt::Continue { span, .. } | Stmt::Error { span, .. } => *span,
    }
}

fn infix_binding(kind: TokenKind) -> Option<(u8, u8, TokenKind)> {
    // Pratt binding powers: assignment weakest, +/− stronger than shifts, etc.
    // Left-assoc uses (bp, bp+1); right-assoc assignment uses (bp, bp).
    Some(match kind {
        TokenKind::Eq
        | TokenKind::PlusEq
        | TokenKind::MinusEq
        | TokenKind::StarEq
        | TokenKind::SlashEq
        | TokenKind::PercentEq
        | TokenKind::AmpEq
        | TokenKind::PipeEq
        | TokenKind::CaretEq
        | TokenKind::LtLtEq
        | TokenKind::GtGtEq => (1, 1, kind),
        TokenKind::PipePipe => (3, 4, kind),
        TokenKind::AmpAmp => (5, 6, kind),
        TokenKind::Pipe => (7, 8, kind),
        TokenKind::Caret => (9, 10, kind),
        TokenKind::Amp => (11, 12, kind),
        TokenKind::EqEq | TokenKind::BangEq => (13, 14, kind),
        TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq => (15, 16, kind),
        TokenKind::LtLt | TokenKind::GtGt => (17, 18, kind),
        TokenKind::Plus | TokenKind::Minus => (19, 20, kind),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (21, 22, kind),
        _ => return None,
    })
}
