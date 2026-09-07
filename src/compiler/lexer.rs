//! Byte-offset lexer with located diagnostics.

use super::diagnostic::{DiagCode, Diagnostic};
use super::source::{FileId, SourceFile, SourceSpan};
use super::token::{Token, TokenKind, keyword};

pub struct LexerOutput {
    pub tokens: Vec<Token>,
    pub integers: Vec<IntegerLit>,
    pub chars: Vec<u8>,
    pub strings: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerLit {
    pub text: String,
    pub value: Option<u32>,
    pub hex: bool,
}

struct Lexer<'a> {
    file: FileId,
    text: &'a str,
    bytes: &'a [u8],
    pos: usize,
    diagnostics: &'a mut Vec<Diagnostic>,
    integers: Vec<IntegerLit>,
    chars: Vec<u8>,
    strings: Vec<Vec<u8>>,
}

pub fn lex(file: &SourceFile, diagnostics: &mut Vec<Diagnostic>) -> LexerOutput {
    let mut lexer = Lexer {
        file: file.id,
        text: &file.text,
        bytes: file.text.as_bytes(),
        pos: 0,
        diagnostics,
        integers: Vec::new(),
        chars: Vec::new(),
        strings: Vec::new(),
    };
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        let eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if eof {
            break;
        }
    }
    LexerOutput {
        tokens,
        integers: lexer.integers,
        chars: lexer.chars,
        strings: lexer.strings,
    }
}

impl<'a> Lexer<'a> {
    fn span(&self, start: usize, end: usize) -> SourceSpan {
        SourceSpan::new(self.file, start as u32, end as u32)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn match_byte(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn error(&mut self, code: DiagCode, start: usize, end: usize, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(code, self.span(start, end), message));
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\n' | b'\r') => {
                    self.pos += 1;
                }
                Some(b'/') if self.bytes.get(self.pos + 1) == Some(&b'/') => {
                    self.pos += 2;
                    while let Some(b) = self.peek() {
                        self.pos += 1;
                        if b == b'\n' {
                            break;
                        }
                    }
                }
                Some(b'/') if self.bytes.get(self.pos + 1) == Some(&b'*') => {
                    let start = self.pos;
                    self.pos += 2;
                    let mut closed = false;
                    while let Some(b) = self.peek() {
                        self.pos += 1;
                        if b == b'*' && self.peek() == Some(b'/') {
                            self.pos += 1;
                            closed = true;
                            break;
                        }
                    }
                    if !closed {
                        self.error(
                            DiagCode::LexUnterminatedBlockComment,
                            start,
                            self.pos,
                            "unterminated block comment",
                        );
                    }
                }
                _ => return,
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        let Some(b) = self.bump() else {
            return Token::new(TokenKind::Eof, self.span(start, start));
        };
        let kind = match b {
            b'@' => TokenKind::At,
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            b',' => TokenKind::Comma,
            b';' => TokenKind::Semicolon,
            b'~' => TokenKind::Tilde,
            b':' => {
                if self.match_byte(b':') {
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }
            b'.' => TokenKind::Dot,
            b'=' => {
                if self.match_byte(b'=') {
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            b'!' => {
                if self.match_byte(b'=') {
                    TokenKind::BangEq
                } else {
                    TokenKind::Bang
                }
            }
            b'<' => {
                if self.match_byte(b'<') {
                    if self.match_byte(b'=') {
                        TokenKind::LtLtEq
                    } else {
                        TokenKind::LtLt
                    }
                } else if self.match_byte(b'=') {
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            b'>' => {
                if self.match_byte(b'>') {
                    if self.match_byte(b'=') {
                        TokenKind::GtGtEq
                    } else {
                        TokenKind::GtGt
                    }
                } else if self.match_byte(b'=') {
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            b'+' => {
                if self.match_byte(b'+') {
                    TokenKind::PlusPlus
                } else if self.match_byte(b'=') {
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            b'-' => {
                if self.match_byte(b'>') {
                    TokenKind::Arrow
                } else if self.match_byte(b'-') {
                    TokenKind::MinusMinus
                } else if self.match_byte(b'=') {
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            b'*' => {
                if self.match_byte(b'=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            b'/' => {
                if self.match_byte(b'=') {
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            b'%' => {
                if self.match_byte(b'=') {
                    TokenKind::PercentEq
                } else {
                    TokenKind::Percent
                }
            }
            b'&' => {
                if self.match_byte(b'&') {
                    TokenKind::AmpAmp
                } else if self.match_byte(b'=') {
                    TokenKind::AmpEq
                } else {
                    TokenKind::Amp
                }
            }
            b'|' => {
                if self.match_byte(b'|') {
                    TokenKind::PipePipe
                } else if self.match_byte(b'=') {
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe
                }
            }
            b'^' => {
                if self.match_byte(b'=') {
                    TokenKind::CaretEq
                } else {
                    TokenKind::Caret
                }
            }
            b'\'' => return self.char_literal(start),
            b'"' => return self.string_literal(start),
            b'0'..=b'9' => return self.number(start, b),
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => return self.ident_or_keyword(start),
            _ => {
                let ch = if b < 0x80 {
                    (b as char).escape_default().to_string()
                } else {
                    // Consume the rest of this UTF-8 scalar so spans stay on character boundaries.
                    let rest = self.text[start..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    self.pos = start + rest;
                    self.text[start..self.pos].escape_default().to_string()
                };
                self.error(
                    DiagCode::LexUnexpectedCharacter,
                    start,
                    self.pos,
                    format!("unexpected character '{ch}'"),
                );
                return self.next_token();
            }
        };
        Token::new(kind, self.span(start, self.pos))
    }

    fn ident_or_keyword(&mut self, start: usize) -> Token {
        while matches!(
            self.peek(),
            Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
        ) {
            self.pos += 1;
        }
        if let Some(b) = self.peek() {
            if b >= 0x80 {
                let rest = self.text[self.pos..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(1);
                self.error(
                    DiagCode::LexInvalidIdentifier,
                    start,
                    self.pos + rest,
                    "identifiers must use ASCII letters, digits, and underscore",
                );
                self.pos += rest;
                while matches!(
                    self.peek(),
                    Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
                ) || self.peek().is_some_and(|b| b >= 0x80)
                {
                    if self.peek().is_some_and(|b| b >= 0x80) {
                        let rest = self.text[self.pos..]
                            .chars()
                            .next()
                            .map(|c| c.len_utf8())
                            .unwrap_or(1);
                        self.pos += rest;
                    } else {
                        self.pos += 1;
                    }
                }
            }
        }
        let ident = &self.text[start..self.pos.min(self.text.len())];
        let ascii_ident: String = ident.chars().filter(|c| c.is_ascii()).collect();
        let kind = keyword(&ascii_ident).unwrap_or(TokenKind::Ident);
        Token::new(kind, self.span(start, self.pos))
    }

    fn number(&mut self, start: usize, first: u8) -> Token {
        let hex = first == b'0' && matches!(self.peek(), Some(b'x' | b'X'));
        if hex {
            self.pos += 1;
            let digits_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f')) {
                self.pos += 1;
            }
            let text = self.text[start..self.pos].to_string();
            if self.pos == digits_start {
                self.error(
                    DiagCode::LexMalformedLiteral,
                    start,
                    self.pos,
                    "hexadecimal literal needs at least one digit",
                );
                self.integers.push(IntegerLit {
                    text,
                    value: None,
                    hex: true,
                });
                return Token::new(TokenKind::Integer, self.span(start, self.pos));
            }
            if self
                .peek()
                .is_some_and(|b| matches!(b, b'G'..=b'Z' | b'g'..=b'z' | b'_'))
            {
                let extra_start = self.pos;
                while matches!(
                    self.peek(),
                    Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
                ) {
                    self.pos += 1;
                }
                self.error(
                    DiagCode::LexMalformedLiteral,
                    extra_start,
                    self.pos,
                    "invalid hexadecimal digit",
                );
            }
            let digits = &self.text[digits_start..self.pos.min(self.text.len())];
            let hex_digits: String = digits.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            let value = parse_hex_u32(&hex_digits);
            if value.is_none() {
                self.error(
                    DiagCode::LexIntegerOverflow,
                    start,
                    self.pos,
                    "integer literal does not fit in 32 bits",
                );
            }
            self.integers.push(IntegerLit {
                text,
                value,
                hex: true,
            });
            return Token::new(TokenKind::Integer, self.span(start, self.pos));
        }

        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self
            .peek()
            .is_some_and(|b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'_'))
        {
            let extra_start = self.pos;
            while matches!(
                self.peek(),
                Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
            ) {
                self.pos += 1;
            }
            self.error(
                DiagCode::LexMalformedLiteral,
                extra_start,
                self.pos,
                "invalid decimal digit",
            );
        }
        let text = self.text[start..self.pos].to_string();
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        let value = parse_dec_u32(&digits);
        if value.is_none() {
            self.error(
                DiagCode::LexIntegerOverflow,
                start,
                self.pos,
                "integer literal does not fit in 32 bits",
            );
        }
        self.integers.push(IntegerLit {
            text,
            value,
            hex: false,
        });
        Token::new(TokenKind::Integer, self.span(start, self.pos))
    }

    fn char_literal(&mut self, start: usize) -> Token {
        match self.read_escape_or_byte(true) {
            None => {
                if self.peek() == Some(b'\'') {
                    self.pos += 1;
                    self.error(
                        DiagCode::LexMalformedLiteral,
                        start,
                        self.pos,
                        "empty character literal",
                    );
                } else if self.peek().is_none() || self.peek() == Some(b'\n') {
                    self.error(
                        DiagCode::LexUnterminatedChar,
                        start,
                        self.pos,
                        "unterminated character literal",
                    );
                }
                self.chars.push(0);
                self.skip_to_char_end();
                Token::new(TokenKind::Char, self.span(start, self.pos))
            }
            Some(value) => {
                if self.match_byte(b'\'') {
                    self.chars.push(value);
                    Token::new(TokenKind::Char, self.span(start, self.pos))
                } else {
                    let closed = self.bytes[self.pos..].contains(&b'\'')
                        && self.bytes[self.pos..]
                            .iter()
                            .take_while(|b| **b != b'\n')
                            .any(|b| *b == b'\'');
                    if !closed {
                        self.error(
                            DiagCode::LexUnterminatedChar,
                            start,
                            self.pos,
                            "unterminated character literal",
                        );
                    } else {
                        self.error(
                            DiagCode::LexMalformedLiteral,
                            start,
                            self.pos,
                            "character literal must contain exactly one character",
                        );
                    }
                    self.chars.push(value);
                    self.skip_to_char_end();
                    Token::new(TokenKind::Char, self.span(start, self.pos))
                }
            }
        }
    }

    fn skip_to_char_end(&mut self) {
        while let Some(b) = self.peek() {
            if b == b'\n' {
                break;
            }
            self.pos += 1;
            if b == b'\'' {
                break;
            }
        }
    }

    fn string_literal(&mut self, start: usize) -> Token {
        let mut bytes = Vec::new();
        loop {
            match self.peek() {
                None | Some(b'\n') => {
                    self.error(
                        DiagCode::LexUnterminatedString,
                        start,
                        self.pos,
                        "unterminated string literal",
                    );
                    break;
                }
                Some(b'"') => {
                    self.pos += 1;
                    break;
                }
                Some(_) => match self.read_escape_or_byte(false) {
                    Some(b) => bytes.push(b),
                    None => {}
                },
            }
        }
        self.strings.push(bytes);
        Token::new(TokenKind::String, self.span(start, self.pos))
    }

    fn read_escape_or_byte(&mut self, in_char: bool) -> Option<u8> {
        let start = self.pos;
        let b = self.bump()?;
        if b == b'\\' {
            let esc_start = start;
            let Some(esc) = self.bump() else {
                self.error(
                    if in_char {
                        DiagCode::LexUnterminatedChar
                    } else {
                        DiagCode::LexUnterminatedString
                    },
                    esc_start,
                    self.pos,
                    "unterminated escape sequence",
                );
                return None;
            };
            return match esc {
                b'n' => Some(b'\n'),
                b'r' => Some(b'\r'),
                b't' => Some(b'\t'),
                b'0' => Some(0),
                b'\\' => Some(b'\\'),
                b'\'' => Some(b'\''),
                b'"' => Some(b'"'),
                b'x' => self.hex_escape(esc_start),
                _ => {
                    self.error(
                        DiagCode::LexInvalidEscape,
                        esc_start,
                        self.pos,
                        format!("invalid escape '\\{}'", esc as char),
                    );
                    None
                }
            };
        }
        if b == b'\'' && in_char {
            self.pos = start;
            return None;
        }
        if b == b'"' && !in_char {
            self.pos = start;
            return None;
        }
        if b >= 0x80 {
            let rest = self.text[start..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.pos = start + rest;
            self.error(
                DiagCode::LexMalformedLiteral,
                start,
                self.pos,
                "string and character literals currently accept printable ASCII, common escapes, and \\xNN bytes",
            );
            return None;
        }
        if !matches!(b, 0x20..=0x7E) && b != b'\t' {
            self.error(
                DiagCode::LexMalformedLiteral,
                start,
                self.pos,
                "string and character literals currently accept printable ASCII, common escapes, and \\xNN bytes",
            );
            return None;
        }
        Some(b)
    }

    fn hex_escape(&mut self, start: usize) -> Option<u8> {
        let mut value = 0u8;
        for i in 0..2 {
            let Some(b) = self.peek() else {
                self.error(
                    DiagCode::LexInvalidEscape,
                    start,
                    self.pos,
                    "\\x escape needs two hexadecimal digits",
                );
                return None;
            };
            let Some(digit) = hex_digit(b) else {
                if i == 0 {
                    self.error(
                        DiagCode::LexInvalidEscape,
                        start,
                        self.pos,
                        "\\x escape needs two hexadecimal digits",
                    );
                    return None;
                }
                self.error(
                    DiagCode::LexInvalidEscape,
                    start,
                    self.pos,
                    "\\x escape needs two hexadecimal digits",
                );
                return None;
            };
            self.pos += 1;
            value = value * 16 + digit;
        }
        Some(value)
    }
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn parse_dec_u32(digits: &str) -> Option<u32> {
    if digits.is_empty() {
        return None;
    }
    let mut value: u32 = 0;
    for b in digits.bytes() {
        let digit = u32::from(b - b'0');
        value = value.checked_mul(10)?.checked_add(digit)?;
    }
    Some(value)
}

fn parse_hex_u32(digits: &str) -> Option<u32> {
    if digits.is_empty() {
        return None;
    }
    let mut value: u32 = 0;
    for b in digits.bytes() {
        let digit = u32::from(hex_digit(b)?);
        value = value.checked_mul(16)?.checked_add(digit)?;
    }
    Some(value)
}
