//! Located C80 syntax tree for the E01 scalar/function subset.

use super::lexer::IntegerLit;
use super::source::{NodeId, SourceSpan};
use super::token::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Void,
    Bool,
    U8,
    I8,
    U16,
    I16,
    Str,
    Ptr(Box<TypeExpr>),
}

impl TypeKind {
    pub fn from_token(kind: TokenKind) -> Option<Self> {
        Some(match kind {
            TokenKind::Void => Self::Void,
            TokenKind::Bool => Self::Bool,
            TokenKind::U8 => Self::U8,
            TokenKind::I8 => Self::I8,
            TokenKind::U16 => Self::U16,
            TokenKind::I16 => Self::I16,
            TokenKind::Str => Self::Str,
            _ => return None,
        })
    }

    pub fn as_str(&self) -> String {
        match self {
            Self::Void => "void".to_string(),
            Self::Bool => "bool".to_string(),
            Self::U8 => "u8".to_string(),
            Self::I8 => "i8".to_string(),
            Self::U16 => "u16".to_string(),
            Self::I16 => "i16".to_string(),
            Self::Str => "str".to_string(),
            Self::Ptr(inner) => format!("ptr<{}>", inner.kind.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExpr {
    pub kind: TypeKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationUnit {
    pub id: NodeId,
    pub span: SourceSpan,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Function(Function),
    Decl(VarDecl),
    Import(Import),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub id: NodeId,
    pub span: SourceSpan,
    pub name: Ident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallConv {
    Register,
    Stack,
}

impl Default for CallConv {
    fn default() -> Self {
        Self::Register
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub id: NodeId,
    pub span: SourceSpan,
    pub is_pub: bool,
    pub conv: CallConv,
    pub return_ty: TypeExpr,
    pub name: Ident,
    pub params: Vec<Param>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub id: NodeId,
    pub span: SourceSpan,
    pub ty: TypeExpr,
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarDecl {
    pub id: NodeId,
    pub span: SourceSpan,
    pub is_pub: bool,
    pub is_const: bool,
    pub ty: TypeExpr,
    pub name: Ident,
    pub array_len: Option<Expr>,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: NodeId,
    pub span: SourceSpan,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Block(Block),
    Decl(VarDecl),
    Expr(ExprStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(ReturnStmt),
    Break { id: NodeId, span: SourceSpan },
    Continue { id: NodeId, span: SourceSpan },
    Asm(AsmStmt),
    Error { id: NodeId, span: SourceSpan },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsmGpr {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Bc,
    De,
    Hl,
}

impl AsmGpr {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "a" => Self::A,
            "b" => Self::B,
            "c" => Self::C,
            "d" => Self::D,
            "e" => Self::E,
            "h" => Self::H,
            "l" => Self::L,
            "bc" => Self::Bc,
            "de" => Self::De,
            "hl" => Self::Hl,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
            Self::D => "d",
            Self::E => "e",
            Self::H => "h",
            Self::L => "l",
            Self::Bc => "bc",
            Self::De => "de",
            Self::Hl => "hl",
        }
    }

    pub fn is_pair(self) -> bool {
        matches!(self, Self::Bc | Self::De | Self::Hl)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsmOutReg {
    Gpr(AsmGpr),
    Carry,
    Zero,
}

impl AsmOutReg {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "carry" => Self::Carry,
            "zero" => Self::Zero,
            other => Self::Gpr(AsmGpr::parse(other)?),
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gpr(r) => r.as_str(),
            Self::Carry => "carry",
            Self::Zero => "zero",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsmClobber {
    Gpr(AsmGpr),
    Flags,
    Memory,
}

impl AsmClobber {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "flags" => Self::Flags,
            "memory" => Self::Memory,
            other => Self::Gpr(AsmGpr::parse(other)?),
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gpr(r) => r.as_str(),
            Self::Flags => "flags",
            Self::Memory => "memory",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmClause {
    In {
        reg: AsmGpr,
        expr: Expr,
        span: SourceSpan,
    },
    Out {
        reg: AsmOutReg,
        dest: Expr,
        span: SourceSpan,
    },
    Inout {
        reg: AsmGpr,
        dest: Expr,
        span: SourceSpan,
    },
    Clobber {
        names: Vec<(AsmClobber, SourceSpan)>,
        span: SourceSpan,
    },
    Stack {
        bytes: u16,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmStmt {
    pub id: NodeId,
    pub span: SourceSpan,
    pub has_header: bool,
    pub clauses: Vec<AsmClause>,
    pub body: String,
    pub body_span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprStmt {
    pub id: NodeId,
    pub span: SourceSpan,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStmt {
    pub id: NodeId,
    pub span: SourceSpan,
    pub cond: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStmt {
    pub id: NodeId,
    pub span: SourceSpan,
    pub cond: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStmt {
    pub id: NodeId,
    pub span: SourceSpan,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub id: NodeId,
    pub span: SourceSpan,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Name(Ident),
    Qualified {
        unit: Ident,
        name: Ident,
    },
    Int(IntegerLit),
    Char(u8),
    String(Vec<u8>),
    Bool(bool),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Assign {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Field {
        base: Box<Expr>,
        name: Ident,
    },
    Cast {
        ty: TypeExpr,
        expr: Box<Expr>,
    },
    Sizeof {
        ty: TypeExpr,
    },
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    BitNot,
    Deref,
    AddrOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Or,
    And,
    BitOr,
    BitXor,
    BitAnd,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Shl,
    Shr,
    Add,
    Sub,
}

impl BinaryOp {
    pub fn from_token(kind: TokenKind) -> Option<Self> {
        Some(match kind {
            TokenKind::PipePipe => Self::Or,
            TokenKind::AmpAmp => Self::And,
            TokenKind::Pipe => Self::BitOr,
            TokenKind::Caret => Self::BitXor,
            TokenKind::Amp => Self::BitAnd,
            TokenKind::EqEq => Self::Eq,
            TokenKind::BangEq => Self::Ne,
            TokenKind::Lt => Self::Lt,
            TokenKind::LtEq => Self::Le,
            TokenKind::Gt => Self::Gt,
            TokenKind::GtEq => Self::Ge,
            TokenKind::LtLt => Self::Shl,
            TokenKind::GtGt => Self::Shr,
            TokenKind::Plus => Self::Add,
            TokenKind::Minus => Self::Sub,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Or => "||",
            Self::And => "&&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::BitAnd => "&",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Shl => "<<",
            Self::Shr => ">>",
            Self::Add => "+",
            Self::Sub => "-",
        }
    }
}
