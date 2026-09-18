//! Structured Z80 items, register names, and assembler rendering.

use super::ast::CallConv;
use super::ir::FuncId;
use super::source::{NodeId, SourceSpan};
use super::types::CType;
use rtvc_core::asm::AssembledProgram;
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

    pub fn jr_ok(self) -> bool {
        matches!(self, Self::Nz | Self::Z | Self::Nc | Self::C)
    }

    pub fn invert(self) -> Option<Cc> {
        Some(match self {
            Self::Nz => Self::Z,
            Self::Z => Self::Nz,
            Self::Nc => Self::C,
            Self::C => Self::Nc,
            Self::Po => Self::Pe,
            Self::Pe => Self::Po,
            Self::P => Self::M,
            Self::M => Self::P,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Z80Op {
    Ret,
    Ld8Imm { dst: R8, imm: u8 },
    Ld8 { dst: R8, src: R8 },
    Ld16Imm { dst: Rr, imm: u16 },
    Ld16Sym { dst: Rr, symbol: String },
    AddA(AluSrc),
    Sub(AluSrc),
    And(AluSrc),
    Xor(AluSrc),
    Or(AluSrc),
    Cp(AluSrc),
    AddHl(Rr),
    SbcA(AluSrc),
    AdcA(AluSrc),
    SbcHl(Rr),
    Cpl,
    Neg,
    ExDeHl,
    Jp { cc: Option<Cc>, target: String },
    Jr { cc: Option<Cc>, target: String },
    LdAbs8 { dst: R8, symbol: String },
    StAbs8 { src: R8, symbol: String },
    LdAbs16 { dst: Rr, symbol: String },
    StAbs16 { src: Rr, symbol: String },
    Call { target: String },
    Push(Rr),
    Pop(Rr),
    PushAf,
    PopAf,
    PushIx,
    PopIx,
    LdIxImm(u16),
    AddIxSp,
    AddHlSp,
    LdSpHl,
    LdSpIx,
    IncSp,
    DecSp,
    Ld8Ix { dst: R8, disp: i8 },
    St8Ix { src: R8, disp: i8 },
    Ld8IxImm { disp: i8, imm: u8 },
    LdHl(R8),
    StHl(R8),
    IncHl,
    Rla,
    InImm { port: u8 },
    OutImm { port: u8 },
    InC,
    OutC,
    Di,
    Ei,
    Ldir,
}

impl Z80Op {
    pub fn render(&self) -> String {
        match self {
            Self::Ret => "RET".to_string(),
            Self::Ld8Imm { dst, imm } => format!("LD {},{}", dst.name(), imm),
            Self::Ld8 { dst, src } => format!("LD {},{}", dst.name(), src.name()),
            Self::Ld16Imm { dst, imm } => format!("LD {},{}", dst.name(), imm),
            Self::Ld16Sym { dst, symbol } => format!("LD {},{}", dst.name(), symbol),
            Self::AddA(src) => format!("ADD A,{}", alu_src(src)),
            Self::Sub(src) => format!("SUB {}", alu_src(src)),
            Self::And(src) => format!("AND {}", alu_src(src)),
            Self::Xor(src) => format!("XOR {}", alu_src(src)),
            Self::Or(src) => format!("OR {}", alu_src(src)),
            Self::Cp(src) => format!("CP {}", alu_src(src)),
            Self::AddHl(src) => format!("ADD HL,{}", src.name()),
            Self::SbcA(src) => format!("SBC A,{}", alu_src(src)),
            Self::AdcA(src) => format!("ADC A,{}", alu_src(src)),
            Self::SbcHl(src) => format!("SBC HL,{}", src.name()),
            Self::Cpl => "CPL".to_string(),
            Self::Neg => "NEG".to_string(),
            Self::ExDeHl => "EX DE,HL".to_string(),
            Self::Jp { cc: None, target } => format!("JP {target}"),
            Self::Jp {
                cc: Some(cc),
                target,
            } => format!("JP {},{}", cc.name(), target),
            Self::Jr { cc: None, target } => format!("JR {target}"),
            Self::Jr {
                cc: Some(cc),
                target,
            } => format!("JR {},{}", cc.name(), target),
            Self::LdAbs8 { dst, symbol } => format!("LD {},({})", dst.name(), symbol),
            Self::StAbs8 { src, symbol } => format!("LD ({}),{}", symbol, src.name()),
            Self::LdAbs16 { dst, symbol } => format!("LD {},({})", dst.name(), symbol),
            Self::StAbs16 { src, symbol } => format!("LD ({}),{}", symbol, src.name()),
            Self::Call { target } => format!("CALL {target}"),
            Self::Push(rr) => format!("PUSH {}", rr.name()),
            Self::Pop(rr) => format!("POP {}", rr.name()),
            Self::PushAf => "PUSH AF".to_string(),
            Self::PopAf => "POP AF".to_string(),
            Self::PushIx => "PUSH IX".to_string(),
            Self::PopIx => "POP IX".to_string(),
            Self::LdIxImm(imm) => format!("LD IX,{imm}"),
            Self::AddIxSp => "ADD IX,SP".to_string(),
            Self::AddHlSp => "ADD HL,SP".to_string(),
            Self::LdSpHl => "LD SP,HL".to_string(),
            Self::LdSpIx => "LD SP,IX".to_string(),
            Self::IncSp => "INC SP".to_string(),
            Self::DecSp => "DEC SP".to_string(),
            Self::Ld8Ix { dst, disp } => format!("LD {},{}", dst.name(), ix_addr(*disp)),
            Self::St8Ix { src, disp } => format!("LD {},{}", ix_addr(*disp), src.name()),
            Self::Ld8IxImm { disp, imm } => format!("LD {},{imm}", ix_addr(*disp)),
            Self::LdHl(r) => format!("LD {},(HL)", r.name()),
            Self::StHl(r) => format!("LD (HL),{}", r.name()),
            Self::IncHl => "INC HL".to_string(),
            Self::Rla => "RLA".to_string(),
            Self::InImm { port } => format!("IN A,({port})"),
            Self::OutImm { port } => format!("OUT ({port}),A"),
            Self::InC => "IN A,(C)".to_string(),
            Self::OutC => "OUT (C),A".to_string(),
            Self::Di => "DI".to_string(),
            Self::Ei => "EI".to_string(),
            Self::Ldir => "LDIR".to_string(),
        }
    }
}

fn alu_src(src: &AluSrc) -> String {
    match src {
        AluSrc::Reg(r) => r.name().to_string(),
        AluSrc::Imm(n) => n.to_string(),
    }
}

fn ix_addr(disp: i8) -> String {
    if disp >= 0 {
        format!("(IX+{disp})")
    } else {
        format!("(IX{disp})")
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
        expr: Option<SourceSpan>,
        synthetic: bool,
        node: Option<NodeId>,
    },
    Data {
        id: AsmInstructionId,
        text: String,
        span: SourceSpan,
        node: NodeId,
    },
    Raw {
        id: AsmInstructionId,
        text: String,
        span: SourceSpan,
        expr: Option<SourceSpan>,
        synthetic: bool,
        node: Option<NodeId>,
    },
    Directive {
        id: AsmInstructionId,
        text: String,
        span: SourceSpan,
        node: Option<NodeId>,
    },
}

impl Z80Item {
    pub fn id(&self) -> AsmInstructionId {
        match self {
            Self::Label { id, .. }
            | Self::Instruction { id, .. }
            | Self::Data { id, .. }
            | Self::Raw { id, .. }
            | Self::Directive { id, .. } => *id,
        }
    }

    pub fn is_instruction(&self) -> bool {
        matches!(self, Self::Instruction { .. } | Self::Raw { .. })
    }

    pub fn emits_bytes(&self) -> bool {
        matches!(
            self,
            Self::Instruction { .. } | Self::Data { .. } | Self::Raw { .. }
        )
    }

    fn render_line(&self) -> String {
        match self {
            Self::Label { name, .. } => format!("{name}:"),
            Self::Instruction { op, .. } => format!("        {}", op.render()),
            Self::Data { text, .. } | Self::Raw { text, .. } | Self::Directive { text, .. } => {
                format!("        {text}")
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticTiming {
    Exact(u16),
    Branch { not_taken: u16, taken: u16 },
    Repeating { continuing: u16, last: u16 },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedKind {
    Instruction,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackProvenance {
    Proven,
    Declared,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedInstruction {
    pub id: AsmInstructionId,
    pub address: u16,
    pub bytes: Vec<u8>,
    pub text: String,
    pub t_states: Option<&'static str>,
    pub timing: StaticTiming,
    pub kind: MappedKind,
    pub synthetic: bool,
    pub expression_span: Option<SourceSpan>,
    pub statement_span: Option<SourceSpan>,
    pub function: Option<FuncId>,
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
    pub conv: CallConv,
    pub param_homes: Vec<RegHome>,
    pub param_types: Vec<CType>,
    pub ret: Option<RegHome>,
    pub stack_bound: u16,
    pub stack_provenance: StackProvenance,
    pub frame_bytes: u16,
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
    pub span: SourceSpan,
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
        bytes_in_segments(&self.assembled, func.addr, func.size as usize)
    }
}

pub fn bytes_in_segments(assembled: &AssembledProgram, addr: u16, len: usize) -> Option<&[u8]> {
    if len == 0 {
        return Some(&[]);
    }
    for seg in &assembled.segments {
        let off = addr.wrapping_sub(seg.addr) as usize;
        if addr >= seg.addr
            && off
                .checked_add(len)
                .is_some_and(|end| end <= seg.bytes.len())
        {
            return Some(&seg.bytes[off..off + len]);
        }
    }
    None
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
