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
    Ld8Imm {
        dst: R8,
        imm: u8,
    },
    Ld8 {
        dst: R8,
        src: R8,
    },
    Ld16Imm {
        dst: Rr,
        imm: u16,
    },
    Ld16Sym {
        dst: Rr,
        symbol: String,
    },
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
    Jp {
        cc: Option<Cc>,
        target: String,
    },
    Jr {
        cc: Option<Cc>,
        target: String,
    },
    LdAbs8 {
        dst: R8,
        symbol: String,
    },
    StAbs8 {
        src: R8,
        symbol: String,
    },
    LdAbs16 {
        dst: Rr,
        symbol: String,
    },
    StAbs16 {
        src: Rr,
        symbol: String,
    },
    /// `clobbers`: registers the callee may change (H2c): everything, unless
    /// the callee has a declared or inferred contract.
    Call {
        target: String,
        clobbers: Vec<R8>,
    },
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
    Ld8Ix {
        dst: R8,
        disp: i8,
    },
    St8Ix {
        src: R8,
        disp: i8,
    },
    Ld8IxImm {
        disp: i8,
        imm: u8,
    },
    LdHl(R8),
    StHl(R8),
    IncHl,
    /// `LD A,(BC)` / `LD A,(DE)` (H5): a byte load through a pointer that
    /// already lives in `BC` or `DE`.
    LdAInd(Rr),
    /// `LD (BC),A` / `LD (DE),A`.
    StIndA(Rr),
    /// `INC BC` / `INC DE` (6 T, flags unchanged).
    Inc16(Rr),
    Rla,
    InImm {
        port: u8,
    },
    OutImm {
        port: u8,
    },
    InC,
    OutC,
    Di,
    Ei,
    Ldir,
    /// `SLA r`, `SRL r`, `SRA r`, `RL r`, `RR r` (CB-prefixed, 8 T).
    Sla(R8),
    Srl(R8),
    Sra(R8),
    Rl8(R8),
    Rr8(R8),
    /// Accumulator rotates (4 T).
    Rlca,
    Rrca,
    Rra,
    Inc8(R8),
    Dec8(R8),
    /// `DJNZ target` (13/8 T); only emitted behind a zero-count guard.
    Djnz {
        target: String,
    },
}

impl Z80Op {
    /// General-purpose 8-bit registers this instruction may change. Flags, SP,
    /// IX, and memory are not tracked. `CALL` writes its callee's clobbers; `OR A`,
    /// `AND A`, `CP`, and `LD r,r` with equal operands change no register.
    pub fn writes(&self) -> Vec<R8> {
        fn pair(rr: Rr) -> Vec<R8> {
            let (h, l) = rr.halves();
            vec![h, l]
        }
        match self {
            Self::Ld8 { dst, src } if dst == src => Vec::new(),
            Self::Ld8 { dst, .. }
            | Self::Ld8Imm { dst, .. }
            | Self::LdAbs8 { dst, .. }
            | Self::Ld8Ix { dst, .. }
            | Self::LdHl(dst) => vec![*dst],
            Self::LdAInd(_) => vec![R8::A],
            Self::Inc16(rr) => pair(*rr),
            Self::Ld16Imm { dst, .. }
            | Self::Ld16Sym { dst, .. }
            | Self::LdAbs16 { dst, .. }
            | Self::Pop(dst) => pair(*dst),
            Self::Or(AluSrc::Reg(R8::A)) | Self::And(AluSrc::Reg(R8::A)) => Vec::new(),
            Self::AddA(_)
            | Self::Sub(_)
            | Self::And(_)
            | Self::Xor(_)
            | Self::Or(_)
            | Self::SbcA(_)
            | Self::AdcA(_)
            | Self::Cpl
            | Self::Neg
            | Self::Rla
            | Self::Rlca
            | Self::Rrca
            | Self::Rra
            | Self::PopAf
            | Self::InImm { .. }
            | Self::InC => vec![R8::A],
            Self::AddHl(_) | Self::SbcHl(_) | Self::AddHlSp | Self::IncHl => pair(Rr::Hl),
            Self::ExDeHl => vec![R8::D, R8::E, R8::H, R8::L],
            Self::Ldir => vec![R8::B, R8::C, R8::D, R8::E, R8::H, R8::L],
            Self::Call { clobbers, .. } => clobbers.clone(),
            Self::Sla(r)
            | Self::Srl(r)
            | Self::Sra(r)
            | Self::Rl8(r)
            | Self::Rr8(r)
            | Self::Inc8(r)
            | Self::Dec8(r) => vec![*r],
            Self::Djnz { .. } => vec![R8::B],
            Self::Ret
            | Self::Cp(_)
            | Self::Jp { .. }
            | Self::Jr { .. }
            | Self::StAbs8 { .. }
            | Self::StAbs16 { .. }
            | Self::Push(_)
            | Self::PushAf
            | Self::PushIx
            | Self::PopIx
            | Self::LdIxImm(_)
            | Self::AddIxSp
            | Self::LdSpHl
            | Self::LdSpIx
            | Self::IncSp
            | Self::DecSp
            | Self::St8Ix { .. }
            | Self::Ld8IxImm { .. }
            | Self::StHl(_)
            | Self::StIndA(_)
            | Self::OutImm { .. }
            | Self::OutC
            | Self::Di
            | Self::Ei => Vec::new(),
        }
    }

    /// T-states of the instructions fallback code emits; `None` otherwise.
    pub fn spill_tstates(&self) -> Option<u32> {
        Some(match self {
            Self::LdAbs8 { .. } | Self::StAbs8 { .. } => 13,
            Self::LdAbs16 { dst: Rr::Hl, .. } | Self::StAbs16 { src: Rr::Hl, .. } => 16,
            Self::LdAbs16 { .. } | Self::StAbs16 { .. } => 20,
            Self::Ld8 { .. } => 4,
            Self::Ld8Imm { .. } => 7,
            Self::Ld16Imm { .. } | Self::Ld16Sym { .. } => 10,
            Self::Push(_) | Self::PushAf => 11,
            Self::Pop(_) | Self::PopAf => 10,
            Self::Ld8Ix { .. } | Self::St8Ix { .. } => 19,
            _ => return None,
        })
    }

    /// Approximate T-states (taken branches, one `LDIR` step) for comparing
    /// alternative lowerings of one function; not a timing report.
    pub fn approx_tstates(&self) -> u32 {
        let alu = |src: &AluSrc| match src {
            AluSrc::Reg(_) => 4,
            AluSrc::Imm(_) => 7,
        };
        match self {
            Self::Ld8 { .. }
            | Self::Cpl
            | Self::ExDeHl
            | Self::Rla
            | Self::Rlca
            | Self::Rrca
            | Self::Rra
            | Self::Inc8(_)
            | Self::Dec8(_)
            | Self::Di
            | Self::Ei => 4,
            Self::Sla(_) | Self::Srl(_) | Self::Sra(_) | Self::Rl8(_) | Self::Rr8(_) => 8,
            Self::Djnz { .. } => 13,
            Self::AddA(s)
            | Self::Sub(s)
            | Self::And(s)
            | Self::Xor(s)
            | Self::Or(s)
            | Self::Cp(s)
            | Self::SbcA(s)
            | Self::AdcA(s) => alu(s),
            Self::Ld8Imm { .. }
            | Self::LdHl(_)
            | Self::StHl(_)
            | Self::LdAInd(_)
            | Self::StIndA(_) => 7,
            Self::Neg => 8,
            Self::Ret | Self::Ld16Imm { .. } | Self::Ld16Sym { .. } | Self::Jp { .. } => 10,
            Self::Pop(_) | Self::PopAf | Self::LdSpIx => 10,
            Self::AddHl(_) | Self::AddHlSp | Self::Push(_) | Self::PushAf => 11,
            Self::InImm { .. } | Self::OutImm { .. } => 11,
            Self::Jr { .. } | Self::InC | Self::OutC => 12,
            Self::LdSpHl | Self::IncSp | Self::DecSp | Self::IncHl | Self::Inc16(_) => 6,
            Self::LdAbs8 { .. } | Self::StAbs8 { .. } => 13,
            Self::LdAbs16 { dst: Rr::Hl, .. } | Self::StAbs16 { src: Rr::Hl, .. } => 16,
            Self::LdAbs16 { .. } | Self::StAbs16 { .. } => 20,
            Self::LdIxImm(_) | Self::PopIx => 14,
            Self::SbcHl(_) | Self::PushIx | Self::AddIxSp => 15,
            Self::Call { .. } => 17,
            Self::Ld8Ix { .. } | Self::St8Ix { .. } | Self::Ld8IxImm { .. } => 19,
            Self::Ldir => 21,
        }
    }

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
            Self::Call { target, .. } => format!("CALL {target}"),
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
            Self::LdAInd(rr) => format!("LD A,({})", rr.name()),
            Self::StIndA(rr) => format!("LD ({}),A", rr.name()),
            Self::Inc16(rr) => format!("INC {}", rr.name()),
            Self::Rla => "RLA".to_string(),
            Self::InImm { port } => format!("IN A,({port})"),
            Self::OutImm { port } => format!("OUT ({port}),A"),
            Self::InC => "IN A,(C)".to_string(),
            Self::OutC => "OUT (C),A".to_string(),
            Self::Di => "DI".to_string(),
            Self::Ei => "EI".to_string(),
            Self::Ldir => "LDIR".to_string(),
            Self::Sla(r) => format!("SLA {}", r.name()),
            Self::Srl(r) => format!("SRL {}", r.name()),
            Self::Sra(r) => format!("SRA {}", r.name()),
            Self::Rl8(r) => format!("RL {}", r.name()),
            Self::Rr8(r) => format!("RR {}", r.name()),
            Self::Rlca => "RLCA".to_string(),
            Self::Rrca => "RRCA".to_string(),
            Self::Rra => "RRA".to_string(),
            Self::Inc8(r) => format!("INC {}", r.name()),
            Self::Dec8(r) => format!("DEC {}", r.name()),
            Self::Djnz { target } => format!("DJNZ {target}"),
        }
    }

    /// Encoded length in bytes, for choosing between equally fast
    /// alternative sequences (the assembler stays the authority).
    pub fn size_bytes(&self) -> u32 {
        let alu = |src: &AluSrc| match src {
            AluSrc::Reg(_) => 1,
            AluSrc::Imm(_) => 2,
        };
        match self {
            Self::Ret
            | Self::Ld8 { .. }
            | Self::AddHl(_)
            | Self::Cpl
            | Self::ExDeHl
            | Self::Push(_)
            | Self::Pop(_)
            | Self::PushAf
            | Self::PopAf
            | Self::AddHlSp
            | Self::LdSpHl
            | Self::IncSp
            | Self::DecSp
            | Self::LdHl(_)
            | Self::StHl(_)
            | Self::IncHl
            | Self::LdAInd(_)
            | Self::StIndA(_)
            | Self::Inc16(_)
            | Self::Rla
            | Self::Rlca
            | Self::Rrca
            | Self::Rra
            | Self::Inc8(_)
            | Self::Dec8(_)
            | Self::Di
            | Self::Ei => 1,
            Self::AddA(s)
            | Self::Sub(s)
            | Self::And(s)
            | Self::Xor(s)
            | Self::Or(s)
            | Self::Cp(s)
            | Self::SbcA(s)
            | Self::AdcA(s) => alu(s),
            Self::Ld8Imm { .. }
            | Self::SbcHl(_)
            | Self::Neg
            | Self::Jr { .. }
            | Self::PushIx
            | Self::PopIx
            | Self::AddIxSp
            | Self::LdSpIx
            | Self::InImm { .. }
            | Self::OutImm { .. }
            | Self::InC
            | Self::OutC
            | Self::Ldir
            | Self::Sla(_)
            | Self::Srl(_)
            | Self::Sra(_)
            | Self::Rl8(_)
            | Self::Rr8(_)
            | Self::Djnz { .. } => 2,
            Self::Ld16Imm { .. }
            | Self::Ld16Sym { .. }
            | Self::Jp { .. }
            | Self::Call { .. }
            | Self::LdAbs8 { .. }
            | Self::StAbs8 { .. }
            | Self::Ld8Ix { .. }
            | Self::St8Ix { .. } => 3,
            Self::LdAbs16 { dst: Rr::Hl, .. } | Self::StAbs16 { src: Rr::Hl, .. } => 3,
            Self::LdAbs16 { .. }
            | Self::StAbs16 { .. }
            | Self::LdIxImm(_)
            | Self::Ld8IxImm { .. } => 4,
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
        /// Listing comment (`; spill` on compiler-inserted fallback code).
        note: Option<&'static str>,
    },
    Data {
        id: AsmInstructionId,
        text: String,
        span: SourceSpan,
        node: NodeId,
        /// Compiler-made data (a function's static slot area).
        synthetic: bool,
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
            Self::Instruction { op, note, .. } => match note {
                Some(note) => format!("        {} ; {note}", op.render()),
                None => format!("        {}", op.render()),
            },
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
    /// Expanded calls this instruction came through, outermost first (H3i).
    /// `span` is then in the callee's source; each frame names its call site.
    pub inlined: Vec<super::ir::InlineFrame>,
}

/// Per-function cost of the H2 fallbacks: memory slots and saves around
/// calls. All zero for a function that keeps every value in registers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpillReport {
    /// Static slot bytes emitted after the function (zero in the image).
    pub slot_bytes: u16,
    /// Bytes pushed to save live registers, one entry per call, `LDIR`, or
    /// inline `asm` site that needed a save (in listing order).
    pub saves: Vec<u16>,
    /// T-states of all `; spill` instructions, each counted once.
    pub tstates: u32,
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
    /// Explicit `@fastcall`: the body was checked to use registers only
    /// (H2b). The ABI is the same as the default.
    pub strict_registers: bool,
    /// Registers this function returns with their entry values (H2c), and
    /// where that promise comes from. Empty with `Contract::None`.
    pub preserves: Vec<R8>,
    pub contract: Contract,
    pub param_homes: Vec<RegHome>,
    pub param_types: Vec<CType>,
    pub ret: Option<RegHome>,
    pub stack_bound: u16,
    pub stack_provenance: StackProvenance,
    pub frame_bytes: u16,
    pub spill: SpillReport,
    pub instruction_ids: Vec<AsmInstructionId>,
    pub mapped: Vec<MappedInstruction>,
    /// Calls expanded in this function (H3i), with the bytes each expansion
    /// takes here: the inlined growth, separate from the function's own code.
    pub inlined: Vec<InlineReport>,
}

/// One expanded call site in a function (H3i).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineReport {
    pub callee: String,
    pub call: SourceSpan,
    /// Chosen by automatic inlining, not `@inline`.
    pub auto: bool,
    pub bytes: u16,
}

/// Source of a function's register contract (H2c, F-028).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contract {
    /// No contract: callers assume every register changes.
    None,
    /// `@preserves(...)`, checked against the body.
    Declared,
    /// A private function's actual writes, used only by callers in its unit.
    Inferred,
}

impl Contract {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Declared => "declared",
            Self::Inferred => "inferred",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedGlobal {
    pub name: String,
    pub label: String,
    pub addr: u16,
    pub size: u16,
    pub ty: super::types::CType,
    pub span: SourceSpan,
    /// Words of the data that hold another global's address (H3a).
    pub refs: Vec<DataRef>,
}

/// A data word holding `target`'s link-time address plus `addend` (H3a), at
/// byte `offset` of its global; emitted as `DW label+addend`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRef {
    pub offset: u16,
    /// The target's C80 name (`fn.local` for a function-local static).
    pub target: String,
    pub label: String,
    pub addend: u16,
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
    // Every generated label starts with a prefix holding a letter that is not
    // a hex digit (`FN`, `G`, `IA`, `U`), so the helper assembler can never
    // read one as a number (F-035: it drops `_` and accepts a trailing `H`
    // or `B` radix suffix) or as a register or condition name. The id makes
    // the label unique whatever the source name is.
    let mut label = format!("FN{}_{name}", func_id.0.0);
    label.make_ascii_uppercase();
    label
}

pub fn asm_global_label(id: super::ir::GlobalId, name: &str) -> String {
    // A `static` local is named `function.local`.
    let mut label = format!("G{}_{}", id.0.0, name.replace('.', "_"));
    label.make_ascii_uppercase();
    label
}

pub fn asm_block_label(func_label: &str, block: super::ir::BlockId) -> String {
    format!("{func_label}_B{}", block.0)
}
