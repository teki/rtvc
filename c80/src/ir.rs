//! Typed control-flow IR for scalar C80.

use super::ast::{AsmClobber, AsmGpr, CallConv};
use super::source::{NodeId, SourceSpan};
use super::types::CType;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VReg(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub NodeId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub NodeId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncId(pub NodeId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedProgram {
    pub structs: Vec<super::types::StructDef>,
    pub globals: Vec<TypedGlobal>,
    pub functions: Vec<TypedFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedGlobal {
    pub id: GlobalId,
    pub name: String,
    pub ty: CType,
    pub is_pub: bool,
    pub is_const: bool,
    pub init: Option<u16>,
    pub extra: Vec<u8>,
    /// Words of the data that hold another global's link-time address
    /// (H3a): `&g`, `&a[K]`, `&s.f` in a static initializer. The bytes at
    /// `offset` are zero in `extra`/`init`; the linker fills them.
    pub relocs: Vec<StaticReloc>,
    pub span: SourceSpan,
}

/// `DW <target label>+addend` at byte `offset` of a global's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticReloc {
    pub offset: u16,
    pub target: GlobalId,
    pub addend: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunction {
    pub id: FuncId,
    pub name: String,
    pub is_pub: bool,
    pub conv: CallConv,
    /// Explicit `@fastcall` (H2b): no slot, save, or IX fallback allowed.
    pub strict_registers: bool,
    /// `@preserves(...)` (H2c): the declared registers as an inline-asm
    /// resource mask (`RES_A`..`RES_L`, pairs expanded). 0 when undeclared.
    pub preserves: u16,
    /// Span of the `@preserves` attribute, when declared.
    pub preserves_span: Option<SourceSpan>,
    /// `@inline` (H3i): its span. Every C80 call is expanded in place.
    pub inline: Option<SourceSpan>,
    /// `@noinline`: excluded from automatic inlining.
    pub noinline: bool,
    /// Emit the out-of-line body (H3i). False for a private `@inline`
    /// function and, in optimized builds, for a private function whose every
    /// call was inlined automatically; such a function has no address.
    pub out_of_line: bool,
    /// Where expanded ops came from (H3i): `IrBlock::origin` indexes this
    /// table, 1-based (0 is the function's own code).
    pub inline_sites: Vec<InlineSite>,
    pub ret: CType,
    pub params: Vec<TypedParam>,
    pub locals: Vec<TypedLocal>,
    pub blocks: Vec<IrBlock>,
    pub span: SourceSpan,
}

/// The chain of expanded calls an op came through, outermost first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineSite {
    pub frames: Vec<InlineFrame>,
}

/// One expanded call: the callee and the call site in its caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineFrame {
    pub callee: FuncId,
    pub name: String,
    pub call: SourceSpan,
    /// Chosen by automatic inlining (optimized builds), not `@inline`.
    pub auto: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub id: LocalId,
    pub name: String,
    pub ty: CType,
    pub span: SourceSpan,
    pub pin: Option<RegPin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocal {
    pub id: LocalId,
    pub name: String,
    pub ty: CType,
    pub span: SourceSpan,
    pub pin: Option<RegPin>,
}

/// A register pin (`name @ reg`, H2b): the variable lives in `reg` for
/// function-wide op indexes `scope.0..scope.1` (its lexical scope).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegPin {
    pub reg: AsmGpr,
    pub span: SourceSpan,
    pub scope: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBlock {
    pub id: BlockId,
    pub ops: Vec<IrOp>,
    /// Per op: index into `TypedFunction::inline_sites` plus one, or 0. Empty
    /// when the block has no expanded ops (H3i).
    pub origin: Vec<u32>,
}

impl IrBlock {
    pub fn origin_of(&self, op: usize) -> u32 {
        self.origin.get(op).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrOp {
    Const {
        dst: VReg,
        ty: CType,
        bits: u16,
        span: SourceSpan,
    },
    Unary {
        dst: VReg,
        ty: CType,
        op: IrUnary,
        src: VReg,
        span: SourceSpan,
    },
    Binary {
        dst: VReg,
        ty: CType,
        op: IrBinary,
        lhs: VReg,
        rhs: VReg,
        span: SourceSpan,
    },
    Cast {
        dst: VReg,
        to: CType,
        from: CType,
        src: VReg,
        span: SourceSpan,
    },
    LoadLocal {
        dst: VReg,
        local: LocalId,
        span: SourceSpan,
    },
    StoreLocal {
        local: LocalId,
        src: VReg,
        span: SourceSpan,
    },
    LoadGlobal {
        dst: VReg,
        global: GlobalId,
        span: SourceSpan,
    },
    AddrGlobal {
        dst: VReg,
        global: GlobalId,
        span: SourceSpan,
    },
    AddrLocal {
        dst: VReg,
        local: LocalId,
        span: SourceSpan,
    },
    StoreGlobal {
        global: GlobalId,
        src: VReg,
        span: SourceSpan,
    },
    LoadIndirect {
        dst: VReg,
        ptr: VReg,
        ty: CType,
        span: SourceSpan,
    },
    StoreIndirect {
        ptr: VReg,
        src: VReg,
        ty: CType,
        span: SourceSpan,
    },
    Call {
        dst: Option<VReg>,
        func: FuncId,
        args: Vec<VReg>,
        span: SourceSpan,
    },
    PortIn {
        dst: VReg,
        port: VReg,
        span: SourceSpan,
    },
    PortOut {
        port: VReg,
        value: VReg,
        span: SourceSpan,
    },
    Di {
        span: SourceSpan,
    },
    Ei {
        span: SourceSpan,
    },
    Ldir {
        hl: VReg,
        de: VReg,
        bc: VReg,
        span: SourceSpan,
    },
    InlineAsm {
        inputs: Vec<(AsmGpr, VReg)>,
        outputs: Vec<IrAsmOutput>,
        clobbers: Vec<AsmClobber>,
        lines: Vec<String>,
        stack: Option<u16>,
        unknown_stack: bool,
        plain: bool,
        span: SourceSpan,
        node: NodeId,
    },
    Branch {
        cond: VReg,
        true_blk: BlockId,
        false_blk: BlockId,
        span: SourceSpan,
    },
    Jump {
        blk: BlockId,
        span: SourceSpan,
    },
    Return {
        value: Option<VReg>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrAsmOutput {
    pub dst: VReg,
    pub ty: CType,
    pub kind: IrAsmOutKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrAsmOutKind {
    Gpr(AsmGpr),
    Carry,
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrUnary {
    Plus,
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrBinary {
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

pub fn function_by_name<'a>(program: &'a TypedProgram, name: &str) -> Option<&'a TypedFunction> {
    program.functions.iter().find(|f| f.name == name)
}

#[allow(dead_code)]
pub fn const_bits_in(func: &TypedFunction) -> BTreeMap<VReg, u16> {
    let mut out = BTreeMap::new();
    for block in &func.blocks {
        for op in &block.ops {
            if let IrOp::Const { dst, bits, .. } = op {
                out.insert(*dst, *bits);
            }
        }
    }
    out
}
