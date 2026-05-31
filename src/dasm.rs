#![allow(dead_code)]

use crate::mmu::CpuBus;
use crate::z80_tables::opcode_to_mnemonic;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub addr: u16,
    pub bytes: Vec<u8>,
    pub text: String,
}

pub struct Dasm<'a, M: CpuBus> {
    mmu: &'a mut M,
    pos: u16,
}

impl<'a, M: CpuBus> Dasm<'a, M> {
    pub fn new(mmu: &'a mut M, addr: u16) -> Self {
        Dasm { mmu, pos: addr }
    }
}

impl<'a, M: CpuBus> Iterator for Dasm<'a, M> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let addr = self.pos;
        let b1 = self.mmu.r8(self.pos) as u32;

        let (key, mut bytes, operand_start, display_base, is_ddc) = match b1 {
            0xCB => {
                let b2 = self.mmu.r8(self.pos.wrapping_add(1)) as u32;
                let key = 0xCB00 | b2;
                (
                    key,
                    vec![b1 as u8, b2 as u8],
                    self.pos.wrapping_add(2),
                    vec![b1 as u8, b2 as u8],
                    false,
                )
            }
            0xED => {
                let b2 = self.mmu.r8(self.pos.wrapping_add(1)) as u32;
                let key = 0xED00 | b2;
                (
                    key,
                    vec![b1 as u8, b2 as u8],
                    self.pos.wrapping_add(2),
                    vec![b1 as u8, b2 as u8],
                    false,
                )
            }
            0xDD | 0xFD => {
                let b2 = self.mmu.r8(self.pos.wrapping_add(1)) as u32;
                if b2 == 0xCB {
                    // DDCB / FDCB: DD/FD, CB, displacement, opcode
                    let disp = self.mmu.r8(self.pos.wrapping_add(2));
                    let b3 = self.mmu.r8(self.pos.wrapping_add(3)) as u32;
                    let key = (b1 << 16) | (b2 << 8) | b3;
                    (
                        key,
                        vec![b1 as u8, b2 as u8, disp, b3 as u8],
                        self.pos.wrapping_add(4), // no extra operands
                        vec![b1 as u8, b2 as u8, b3 as u8], // display: skip displacement
                        true,
                    )
                } else {
                    let key = (b1 << 8) | b2;
                    (
                        key,
                        vec![b1 as u8, b2 as u8],
                        self.pos.wrapping_add(2),
                        vec![b1 as u8, b2 as u8],
                        false,
                    )
                }
            }
            _ => {
                let key = b1;
                (
                    key,
                    vec![b1 as u8],
                    self.pos.wrapping_add(1),
                    vec![b1 as u8],
                    false,
                )
            }
        };

        let template = match opcode_to_mnemonic().get(&key).cloned() {
            Some(t) => t,
            None => {
                self.pos = self.pos.wrapping_add(1);
                return Some(Instruction {
                    addr,
                    bytes: vec![b1 as u8],
                    text: format!("DB {:02X}h", b1),
                });
            }
        };

        // For DDCB/FDCB, replace +dd with the actual displacement value
        let template = if is_ddc {
            let disp = bytes[2] as i8;
            let disp_str = if disp >= 0 {
                format!("+{}", disp)
            } else {
                format!("{}", disp)
            };
            template.replace("+dd", &disp_str)
        } else {
            template
        };

        let (op_bytes, formatted) = format_operands(&template, self.mmu, operand_start);
        bytes.extend(op_bytes);
        self.pos = addr.wrapping_add(bytes.len() as u16);

        let text = format!(
            "{} | {}",
            display_base
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" "),
            formatted
        );

        Some(Instruction { addr, bytes, text })
    }
}

fn format_operands(template: &str, mmu: &mut impl CpuBus, mut pos: u16) -> (Vec<u8>, String) {
    let mut bytes = Vec::new();
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == 'n' && chars.peek() == Some(&'n') {
            chars.next();
            if chars.peek() == Some(&'n') {
                chars.next();
                if chars.peek() == Some(&'n') {
                    chars.next();
                    let val = mmu.r16(pos);
                    bytes.push((val & 0xFF) as u8);
                    bytes.push((val >> 8) as u8);
                    pos = pos.wrapping_add(2);
                    result.push_str(&format!("{:04X}h", val));
                } else {
                    let val = mmu.r8(pos);
                    bytes.push(val);
                    pos = pos.wrapping_add(1);
                    result.push_str(&format!("{:02X}h", val));
                    result.push('n');
                }
            } else {
                let val = mmu.r8(pos);
                bytes.push(val);
                pos = pos.wrapping_add(1);
                result.push_str(&format!("{:02X}h", val));
            }
        } else if c == 'o' && chars.peek() == Some(&'f') {
            let mut rest = String::new();
            for _ in 0..5 {
                rest.push(chars.next().unwrap());
            }
            if rest == "ffset" {
                let val = mmu.r8s(pos);
                bytes.push(val as u8);
                pos = pos.wrapping_add(1);
                result.push_str(&format!("{:+}", val));
            } else {
                result.push(c);
                result.push_str(&rest);
            }
        } else if c == 'd' && chars.peek() == Some(&'d') {
            chars.next();
            let val = mmu.r8s(pos);
            bytes.push(val as u8);
            pos = pos.wrapping_add(1);
            result.push_str(&format!("{}", val));
        } else {
            result.push(c);
        }
    }

    (bytes, result)
}

pub fn disassemble<'a, M: CpuBus>(mmu: &'a mut M, addr: u16) -> Dasm<'a, M> {
    Dasm::new(mmu, addr)
}
