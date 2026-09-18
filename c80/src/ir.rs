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
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunction {
    pub id: FuncId,
    pub name: String,
    pub is_pub: bool,
    pub conv: CallConv,
    pub ret: CType,
    pub params: Vec<TypedParam>,
    pub locals: Vec<TypedLocal>,
    pub blocks: Vec<IrBlock>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub id: LocalId,
    pub name: String,
    pub ty: CType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocal {
    pub id: LocalId,
    pub name: String,
    pub ty: CType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBlock {
    pub id: BlockId,
    pub ops: Vec<IrOp>,
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
