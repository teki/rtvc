//! Structured Z80 items, register names, and assembler rendering.

use super::ir::FuncId;
use super::source::{NodeId, SourceSpan};
use crate::asm::AssembledProgram;
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsmInstructionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl R8 {
    pub const ALL: [R8; 7] = [R8::A, R8::B, R8::C, R8::D, R8::E, R8::H, R8::L];

    pub fn name(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::H => "H",
            Self::L => "L",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
            Self::H => 5,
            Self::L => 6,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rr {
    Bc,
    De,
    Hl,
}

impl Rr {
    pub const ALL: [Rr; 3] = [Rr::Hl, Rr::De, Rr::Bc];

    pub fn name(self) -> &'static str {
        match self {
            Self::Bc => "BC",
            Self::De => "DE",
            Self::Hl => "HL",
        }
    }

    /// High, then low.
    pub fn halves(self) -> (R8, R8) {
        match self {
            Self::Bc => (R8::B, R8::C),
            Self::De => (R8::D, R8::E),
            Self::Hl => (R8::H, R8::L),
        }
    }

    pub fn contains(self, r: R8) -> bool {
        let (h, l) = self.halves();
        h == r || l == r
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegHome {
    Byte(R8),
    Word(Rr),
}

impl RegHome {
    pub fn width(self) -> u8 {
        match self {
            Self::Byte(_) => 1,
            Self::Word(_) => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AluSrc {
    Reg(R8),
    Imm(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cc {
    Nz,
    Z,
    Nc,
    C,
    Po,
    Pe,
    P,
    M,
}

impl Cc {
    pub fn name(self) -> &'static str {
        match self {
            Self::Nz => "NZ",
            Self::Z => "Z",
            Self::Nc => "NC",
            Self::C => "C",
            Self::Po => "PO",
            Self::Pe => "PE",
            Self::P => "P",
            Self::M => "M",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Z80Op {
    Ret,
    Ld8Imm { dst: R8, imm: u8 },
    Ld8 { dst: R8, src: R8 },
    Ld16Imm { dst: Rr, imm: u16 },
    AddA(AluSrc),
    Sub(AluSrc),
    And(AluSrc),
    Xor(AluSrc),
    Or(AluSrc),
    Cp(AluSrc),
    AddHl(Rr),
    SbcA(AluSrc),
    SbcHl(Rr),
    Cpl,
    Neg,
    ExDeHl,
    Jp { cc: Option<Cc>, target: String },
    LdAbs8 { dst: R8, symbol: String },
    StAbs8 { src: R8, symbol: String },
    LdAbs16 { dst: Rr, symbol: String },
    StAbs16 { src: Rr, symbol: String },
}

impl Z80Op {
    pub fn render(&self) -> String {
        match self {
            Self::Ret => "RET".to_string(),
            Self::Ld8Imm { dst, imm } => format!("LD {},{}", dst.name(), imm),
            Self::Ld8 { dst, src } => format!("LD {},{}", dst.name(), src.name()),
            Self::Ld16Imm { dst, imm } => format!("LD {},{}", dst.name(), imm),
            Self::AddA(src) => format!("ADD A,{}", alu_src(src)),
            Self::Sub(src) => format!("SUB {}", alu_src(src)),
            Self::And(src) => format!("AND {}", alu_src(src)),
            Self::Xor(src) => format!("XOR {}", alu_src(src)),
            Self::Or(src) => format!("OR {}", alu_src(src)),
            Self::Cp(src) => format!("CP {}", alu_src(src)),
            Self::AddHl(src) => format!("ADD HL,{}", src.name()),
            Self::SbcA(src) => format!("SBC A,{}", alu_src(src)),
            Self::SbcHl(src) => format!("SBC HL,{}", src.name()),
            Self::Cpl => "CPL".to_string(),
            Self::Neg => "NEG".to_string(),
            Self::ExDeHl => "EX DE,HL".to_string(),
            Self::Jp { cc: None, target } => format!("JP {target}"),
            Self::Jp {
                cc: Some(cc),
                target,
            } => format!("JP {},{}", cc.name(), target),
            Self::LdAbs8 { dst, symbol } => format!("LD {},({})", dst.name(), symbol),
            Self::StAbs8 { src, symbol } => format!("LD ({}),{}", symbol, src.name()),
            Self::LdAbs16 { dst, symbol } => format!("LD {},({})", dst.name(), symbol),
            Self::StAbs16 { src, symbol } => format!("LD ({}),{}", symbol, src.name()),
        }
    }
}

fn alu_src(src: &AluSrc) -> String {
    match src {
        AluSrc::Reg(r) => r.name().to_string(),
        AluSrc::Imm(n) => n.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Z80Item {
    Label {
        id: AsmInstructionId,
        name: String,
        span: SourceSpan,
        node: NodeId,
    },
    Instruction {
        id: AsmInstructionId,
        op: Z80Op,
        span: SourceSpan,
        node: Option<NodeId>,
    },
    Data {
        id: AsmInstructionId,
        text: String,
        span: SourceSpan,
        node: NodeId,
    },
}

impl Z80Item {
    pub fn id(&self) -> AsmInstructionId {
        match self {
            Self::Label { id, .. } | Self::Instruction { id, .. } | Self::Data { id, .. } => *id,
        }
    }

    pub fn is_instruction(&self) -> bool {
        matches!(self, Self::Instruction { .. })
    }

    pub fn emits_bytes(&self) -> bool {
        matches!(self, Self::Instruction { .. } | Self::Data { .. })
    }

    fn render_line(&self) -> String {
        match self {
            Self::Label { name, .. } => format!("{name}:"),
            Self::Instruction { op, .. } => format!("        {}", op.render()),
            Self::Data { text, .. } => format!("        {text}"),
        }
    }
}

pub fn render_items(items: &[Z80Item]) -> String {
    let mut out = String::new();
    for item in items {
        let _ = writeln!(out, "{}", item.render_line());
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedInstruction {
    pub id: AsmInstructionId,
    pub address: u16,
    pub bytes: Vec<u8>,
    pub text: String,
    pub t_states: Option<&'static str>,
    pub span: SourceSpan,
    pub node: Option<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFunction {
    pub name: String,
    pub label: String,
    pub id: FuncId,
    pub span: SourceSpan,
    pub addr: u16,
    pub size: u16,
    pub param_homes: Vec<RegHome>,
    pub ret: Option<RegHome>,
    pub instruction_ids: Vec<AsmInstructionId>,
    pub mapped: Vec<MappedInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedGlobal {
    pub name: String,
    pub label: String,
    pub addr: u16,
    pub size: u16,
    pub ty: super::types::CType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedProgram {
    pub origin: u16,
    pub items: Vec<Z80Item>,
    pub assembly: String,
    pub assembled: AssembledProgram,
    pub functions: Vec<GeneratedFunction>,
    pub globals: Vec<GeneratedGlobal>,
}

impl GeneratedProgram {
    pub fn function(&self, name: &str) -> Option<&GeneratedFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn global(&self, name: &str) -> Option<&GeneratedGlobal> {
        self.globals.iter().find(|g| g.name == name)
    }

    pub fn function_bytes(&self, name: &str) -> Option<&[u8]> {
        let func = self.function(name)?;
        let start = func.addr.wrapping_sub(self.assembled.origin) as usize;
        let end = start + func.size as usize;
        self.assembled.bytes.get(start..end)
    }
}

pub fn asm_label(func_id: FuncId, name: &str) -> String {
    let mut label = format!("F{}_{name}", func_id.0.0);
    label.make_ascii_uppercase();
    label
}

pub fn asm_global_label(id: super::ir::GlobalId, name: &str) -> String {
    let mut label = format!("G{}_{name}", id.0.0);
    label.make_ascii_uppercase();
    label
}

pub fn asm_block_label(func_label: &str, block: super::ir::BlockId) -> String {
    format!("{func_label}_B{}", block.0)
}
