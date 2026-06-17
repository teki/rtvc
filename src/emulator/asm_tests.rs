
use super::*;
use crate::bus::FakeBus;
use crate::disasm::disassemble_at;

fn round_trip(source: &str, pc: u16, expected_bytes: &[u8], expected_text: &str) {
    let bytes = assemble_line(source, pc).unwrap();
    assert_eq!(bytes, expected_bytes);

    let mut bus = FakeBus::new();
    for (offset, byte) in bytes.iter().enumerate() {
        bus.mem[pc.wrapping_add(offset as u16) as usize] = *byte;
    }
    assert_eq!(disassemble_at(&mut bus, pc).text, expected_text);
}

#[test]
fn assembles_base_and_immediate_instructions() {
    round_trip("nop", 0, &[0x00], "NOP");
    round_trip("LD BC,1234H", 0, &[0x01, 0x34, 0x12], "LD BC,1234H");
    round_trip("ld a, 42", 0, &[0x3E, 42], "LD A,2AH");
    round_trip("XOR (HL)", 0, &[0xAE], "XOR (HL)");
    round_trip("CALL NZ,$4567", 0, &[0xC4, 0x67, 0x45], "CALL NZ,4567H");
}

#[test]
fn assembles_relative_targets_using_the_current_address() {
    round_trip("JR NZ,1000H", 0x1000, &[0x20, 0xFE], "JR NZ,1000H");
    round_trip("DJNZ 0x1081", 0x1000, &[0x10, 0x7F], "DJNZ 1081H");
    assert!(assemble_line("JR 1100H", 0x1000).is_err());
}

#[test]
fn assembles_indexed_and_bit_instructions() {
    round_trip(
        "LD (IX-2),99H",
        0x2000,
        &[0xDD, 0x36, 0xFE, 0x99],
        "LD (IX-02H),99H",
    );
    round_trip(
        "ADC A,(IY+5)",
        0x2000,
        &[0xFD, 0x8E, 0x05],
        "ADC A,(IY+05H)",
    );
    round_trip(
        "BIT 0,(IX-128)",
        0x2000,
        &[0xDD, 0xCB, 0x80, 0x46],
        "BIT 0,(IX-80H)",
    );
}

#[test]
fn assembles_ed_and_data_directives() {
    round_trip(
        "LD BC,(4000H)",
        0,
        &[0xED, 0x4B, 0x00, 0x40],
        "LD BC,(4000H)",
    );
    round_trip("IM 2", 0, &[0xED, 0x5E], "IM 2");
    round_trip("IN A,(C)", 0, &[0xED, 0x78], "IN A,(C)");
    round_trip("OUT (C),A", 0, &[0xED, 0x79], "OUT (C),A");
    round_trip("LDIR", 0, &[0xED, 0xB0], "LDIR");
    assert_eq!(
        assemble_line("DB 12H, 34, 0x56 ; bytes", 0).unwrap(),
        [0x12, 34, 0x56]
    );
}

#[test]
fn reports_bad_input() {
    assert_eq!(
        assemble_line("JR PE,1000H", 0).unwrap_err().to_string(),
        "JR condition must be NZ, Z, NC, or C, got 'PE'"
    );
    assert!(assemble_line("LD A,300", 0).is_err());
    assert!(assemble_line("WAT A,B", 0).is_err());
}
