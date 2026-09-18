//! Located tokens.

use super::source::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Ident,
    Integer,
    Char,
    String,
    // Keywords
    Void,
    Bool,
    U8,
    I8,
    U16,
    I16,
    True,
    False,
    Pub,
    Const,
    If,
    Else,
    While,
    Return,
    Break,
    Continue,
    Sizeof,
    // Reserved later syntax (lexed so they are not identifiers)
    Import,
    Struct,
    Str,
    Ptr,
    Asm,
    For,
    Do,
    // Punctuation
    At,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    ColonColon,
    Dot,
    Arrow,
    // Operators
    Eq,
    EqEq,
    Bang,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    PlusPlus,
    PlusEq,
    Minus,
    MinusMinus,
    MinusEq,
    Star,
    StarEq,
    Slash,
    SlashEq,
    Percent,
    PercentEq,
    Amp,
    AmpAmp,
    AmpEq,
    Pipe,
    PipePipe,
    PipeEq,
    Caret,
    CaretEq,
    Tilde,
    LtLt,
    LtLtEq,
    GtGt,
    GtGtEq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

pub fn keyword(ident: &str) -> Option<TokenKind> {
    Some(match ident {
        "void" => TokenKind::Void,
        "bool" => TokenKind::Bool,
        "u8" => TokenKind::U8,
        "i8" => TokenKind::I8,
        "u16" => TokenKind::U16,
        "i16" => TokenKind::I16,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "pub" => TokenKind::Pub,
        "const" => TokenKind::Const,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "return" => TokenKind::Return,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "sizeof" => TokenKind::Sizeof,
        "import" => TokenKind::Import,
        "struct" => TokenKind::Struct,
        "str" => TokenKind::Str,
        "ptr" => TokenKind::Ptr,
        "asm" => TokenKind::Asm,
        "for" => TokenKind::For,
        "do" => TokenKind::Do,
        _ => return None,
    })
}

impl TokenKind {
    pub fn describe(self) -> &'static str {
        match self {
            Self::Eof => "end of file",
            Self::Ident => "identifier",
            Self::Integer => "integer literal",
            Self::Char => "character literal",
            Self::String => "string literal",
            Self::Void => "'void'",
            Self::Bool => "'bool'",
            Self::U8 => "'u8'",
            Self::I8 => "'i8'",
            Self::U16 => "'u16'",
            Self::I16 => "'i16'",
            Self::True => "'true'",
            Self::False => "'false'",
            Self::Pub => "'pub'",
            Self::Const => "'const'",
            Self::If => "'if'",
            Self::Else => "'else'",
            Self::While => "'while'",
            Self::Return => "'return'",
            Self::Break => "'break'",
            Self::Continue => "'continue'",
            Self::Sizeof => "'sizeof'",
            Self::Import => "'import'",
            Self::Struct => "'struct'",
            Self::Str => "'str'",
            Self::Ptr => "'ptr'",
            Self::Asm => "'asm'",
            Self::For => "'for'",
            Self::Do => "'do'",
            Self::At => "'@'",
            Self::LParen => "'('",
            Self::RParen => "')'",
            Self::LBrace => "'{'",
            Self::RBrace => "'}'",
            Self::LBracket => "'['",
            Self::RBracket => "']'",
            Self::Comma => "','",
            Self::Semicolon => "';'",
            Self::Colon => "':'",
            Self::ColonColon => "'::'",
            Self::Dot => "'.'",
            Self::Arrow => "'->'",
            Self::Eq => "'='",
            Self::EqEq => "'=='",
            Self::Bang => "'!'",
            Self::BangEq => "'!='",
            Self::Lt => "'<'",
            Self::LtEq => "'<='",
            Self::Gt => "'>'",
            Self::GtEq => "'>='",
            Self::Plus => "'+'",
            Self::PlusPlus => "'++'",
            Self::PlusEq => "'+='",
            Self::Minus => "'-'",
            Self::MinusMinus => "'--'",
            Self::MinusEq => "'-='",
            Self::Star => "'*'",
            Self::StarEq => "'*='",
            Self::Slash => "'/'",
            Self::SlashEq => "'/='",
            Self::Percent => "'%'",
            Self::PercentEq => "'%='",
            Self::Amp => "'&'",
            Self::AmpAmp => "'&&'",
            Self::AmpEq => "'&='",
            Self::Pipe => "'|'",
            Self::PipePipe => "'||'",
            Self::PipeEq => "'|='",
            Self::Caret => "'^'",
            Self::CaretEq => "'^='",
            Self::Tilde => "'~'",
            Self::LtLt => "'<<'",
            Self::LtLtEq => "'<<='",
            Self::GtGt => "'>>'",
            Self::GtGtEq => "'>>='",
        }
    }

    pub fn is_type_start(self) -> bool {
        matches!(
            self,
            Self::Void
                | Self::Bool
                | Self::U8
                | Self::I8
                | Self::U16
                | Self::I16
                | Self::Ptr
                | Self::Str
                | Self::Ident
        )
    }
}
