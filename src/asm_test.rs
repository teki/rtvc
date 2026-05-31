mod asm;
mod bus;
mod dasm;
mod z80_tables;

use asm::assemble_line;
use bus::FakeBus;
use dasm::disassemble;

fn main() {
    let mut mmu = FakeBus::new();

    let tests = vec![
        // No operands
        "NOP",
        "HALT",
        "EXX",
        "RETN",
        "RLD",
        "NEG",
        "LDI",
        "LDIR",
        "CPI",
        "INI",
        "OUTI",
        "OTIR",
        // 8-bit immediate
        "LD B,0x56",
        "ADD A,0x12",
        "ADC A,0x34",
        "SUB 0x78",
        "SBC A,0x9A",
        "AND 0xFF",
        "OR 0x0F",
        "XOR 0xF0",
        "CP 0x10",
        // 16-bit immediate
        "LD BC,0x1234",
        "LD DE,0x5678",
        "LD HL,0x9ABC",
        "LD SP,0xDEF0",
        "LD IX,0x5678",
        "LD IY,0x1234",
        // Relative jumps
        "JR -5",
        "JR NZ,+20",
        "JR Z,-10",
        "DJNZ +10",
        // Absolute jumps & calls
        "JP 0x1234",
        "JP NZ,0x1234",
        "JP (HL)",
        "JP IX",
        "CALL 0x1234",
        "CALL Z,0x1234",
        "RET",
        "RET NZ",
        // RST & IM
        "RST 0x00",
        "RST 8",
        "RST 0x10",
        "RST 0x38",
        "IM 0",
        "IM 1",
        "IM 2",
        // Memory access
        "LD A,(0x1234)",
        "LD (0x1234),A",
        "LD HL,(0x1234)",
        "LD (0x1234),HL",
        "LD BC,(0x1234)",
        "LD (0x1234),BC",
        // I/O
        "IN A,(0xFE)",
        "IN A,(C)",
        "IN B,(C)",
        "OUT (0xFE),A",
        "OUT (C),A",
        "OUT (C),B",
        "OUT (C),0",
        // Indexed addressing (DD/FD)
        "LD A,(IX+5)",
        "LD (IY-2),0xAA",
        "LD B,(IX+1)",
        "LD (IY+3),B",
        "INC (IX+1)",
        "DEC (IY-1)",
        "ADD IX,BC",
        "ADD IY,DE",
        "LD IXh,0x12",
        "LD IYl,0x34",
        // CB prefix (rotates/shifts/bit ops on registers)
        "RLC B",
        "RRC C",
        "RL D",
        "RR E",
        "SLA H",
        "SRA L",
        "SRL A",
        "BIT 0,B",
        "BIT 7,(HL)",
        "SET 3,C",
        "RES 5,A",
        // DDCB/FDCB prefix (indexed bit ops & rotates)
        "BIT 0,(IX-2)",
        "RES 3,(IY+4)",
        "SET 7,(IX+1)",
        "LD B,RLC (IX+1)",
        "RLC (IY-3)",
        "LD A,RRC (IX+5)",
        // Stack
        "PUSH BC",
        "PUSH IX",
        "POP DE",
        "POP IY",
        // Exchange
        "EX AF,AF'",
        "EX DE,HL",
        "EX (SP),HL",
        "EX (SP),IX",
        // 16-bit arithmetic
        "ADD HL,BC",
        "SBC HL,DE",
        "ADC HL,HL",
        // Register-to-register
        "LD A,B",
        "LD C,D",
        "LD E,H",
        "LD L,A",
        "LD B,C",
        // INC/DEC
        "INC B",
        "INC BC",
        "DEC C",
        "DEC DE",
        "INC (HL)",
        "INC IX",
        "DEC IY",
        // Misc ED
        "LD A,I",
        "LD I,A",
        "LD A,R",
        "LD R,A",
        "RRD",
        "CPD",
        "IND",
        "OUTD",
        "OTDR",
    ];

    let mut addr = 0x1000u16;
    println!("=== Assembly ===");
    for line in &tests {
        match assemble_line(&mut mmu, addr, line) {
            Ok(res) => {
                let hex = res
                    .bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{:04X}: {:30} -> {}", res.addr, line, hex);
                addr += res.bytes.len() as u16;
            }
            Err(e) => {
                println!("{:04X}: {:30} -> ERROR: {}", addr, line, e);
                addr += 1;
            }
        }
    }

    println!("\n=== Disassembly ===");
    let mut dasm = disassemble(&mut mmu, 0x1000);
    while let Some(inst) = dasm.next() {
        if inst.addr >= addr {
            break;
        }
        let hex = inst
            .bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{:04X}: {:20} | {}", inst.addr, hex, inst.text);
    }
}
