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
    round_trip("EX AF,AF'", 0, &[0x08], "EX AF,AF'");
    round_trip("LDIR", 0, &[0xED, 0xB0], "LDIR");
    assert_eq!(
        assemble_line("DB 12H, 34, 0x56 ; bytes", 0).unwrap(),
        [0x12, 34, 0x56]
    );
    assert_eq!(
        assemble_program("DB 'A,B;C' ; string", 0).unwrap().bytes,
        [b'A', b',', b'B', b';', b'C']
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

#[test]
fn assembles_program_with_labels_and_expressions() {
    let program = assemble_program(
        r#"
                ORG 8000H
start:          LD HL,message
                LD B,message_end-message
loop:           DJNZ loop
                JP start
message:        DB "OK", 0
message_end:
            "#,
        0,
    )
    .unwrap();

    assert_eq!(program.origin, 0x8000);
    assert_eq!(program.next_addr, 0x800D);
    assert_eq!(program.symbols["START"], 0x8000);
    assert_eq!(program.symbols["LOOP"], 0x8005);
    assert_eq!(program.symbols["MESSAGE"], 0x800A);
    assert_eq!(program.symbols["MESSAGE_END"], 0x800D);
    assert_eq!(
        program.bytes,
        vec![
            0x21, 0x0A, 0x80, // LD HL,message
            0x06, 0x03, // LD B,message_end-message
            0x10, 0xFE, // DJNZ loop
            0xC3, 0x00, 0x80, // JP start
            b'O', b'K', 0,
        ]
    );
    assert_eq!(program.segments.len(), 1);
    assert_eq!(program.segments[0].addr, 0x8000);
}

#[test]
fn assembles_program_directives_and_segments() {
    let program = assemble_program(
        r#"
base EQU 4000H
            ORG base
first:      DB 1, 2
            DW first, $+2
            DS 3, 0FFH
            ORG base+10H
second:     NOP
            "#,
        0x2000,
    )
    .unwrap();

    assert_eq!(program.origin, 0x4000);
    assert_eq!(program.symbols["BASE"], 0x4000);
    assert_eq!(program.symbols["FIRST"], 0x4000);
    assert_eq!(program.symbols["SECOND"], 0x4010);
    assert_eq!(
        program.bytes,
        vec![1, 2, 0x00, 0x40, 0x04, 0x40, 0xFF, 0xFF, 0xFF, 0x00]
    );
    assert_eq!(program.segments.len(), 2);
    assert_eq!(program.segments[0].addr, 0x4000);
    assert_eq!(
        program.segments[0].bytes,
        vec![1, 2, 0x00, 0x40, 0x04, 0x40, 0xFF, 0xFF, 0xFF]
    );
    assert_eq!(program.segments[1].addr, 0x4010);
    assert_eq!(program.segments[1].bytes, vec![0x00]);
}

#[test]
fn org_declares_persistent_named_address_mappings() {
    let program = assemble_program(
        r#"
            ORG C000H, SYS0, 0000H, SYS1, 4000H
start:      JP MAIN@SYS0
main:       NOP
            ORG C100H
table:      DW TABLE@SYS0, TABLE@SYS1, MAIN
            ORG D000H, SYS2, 8000H
other:      JP TABLE@SYS0
        "#,
        0,
    )
    .unwrap();

    assert_eq!(program.symbols["START"], 0xC000);
    assert_eq!(program.symbols["MAIN"], 0xC003);
    assert_eq!(program.symbols["TABLE"], 0xC100);
    assert_eq!(program.symbols["OTHER"], 0xD000);
    assert_eq!(
        program.bytes,
        vec![
            0xC3, 0x03, 0x00, // JP MAIN@SYS0
            0x00, // MAIN: NOP
            0x00, 0x01, // TABLE@SYS0
            0x00, 0x41, // TABLE@SYS1
            0x03, 0xC0, // MAIN
            0xC3, 0x00, 0x01, // JP TABLE@SYS0
        ]
    );
    assert_eq!(
        program.mappings,
        vec![
            AsmMapping {
                name: "SYS0".to_string(),
                source_base: 0xC000,
                mapped_base: 0x0000,
            },
            AsmMapping {
                name: "SYS1".to_string(),
                source_base: 0xC000,
                mapped_base: 0x4000,
            },
            AsmMapping {
                name: "SYS2".to_string(),
                source_base: 0xD000,
                mapped_base: 0x8000,
            },
        ]
    );
}

#[test]
fn rejects_incomplete_org_mapping() {
    let error = assemble_program("ORG 8000H, SYS0\n", 0).unwrap_err();
    assert_eq!(
        error.to_string(),
        "line 1: ORG mappings require a name and mapped address pair"
    );
}

#[test]
fn mapping_expressions_work_in_equ_and_data_directives() {
    let program = assemble_program(
        "ORG C000H, SYS0, 0000H\nMAIN: NOP\nMAPPED EQU MAIN@SYS0\nDB MAIN@SYS0, MAPPED\nDW MAIN@SYS0+1\nDS MAIN@SYS0+1, MAIN@SYS0+7\n",
        0,
    )
    .unwrap();

    assert_eq!(program.symbols["MAPPED"], 0);
    assert_eq!(program.bytes, vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x07]);
}

#[test]
fn rejects_unknown_and_duplicate_mappings() {
    let error =
        assemble_program("ORG 8000H, SYS0, 0000H\nJP START@MISSING\nSTART: NOP\n", 0).unwrap_err();
    assert_eq!(error.to_string(), "line 2: unknown mapping 'MISSING'");

    let error =
        assemble_program("ORG 8000H, SYS0, 0000H\nORG 9000H, sys0, 4000H\n", 0).unwrap_err();
    assert_eq!(error.to_string(), "line 2: duplicate mapping 'SYS0'");
}

#[test]
fn rejects_the_old_map_directive() {
    let error = assemble_program("MAP SYS,0000H,C000H\n", 0).unwrap_err();
    assert_eq!(
        error.to_string(),
        "line 1: MAP is no longer supported; declare mappings on ORG"
    );
}

#[test]
fn assembles_basic_start_program() {
    let program = assemble_program(
        r#"
            BASIC_START
entry:      LD A,02H
            OUT (00H),A
            JP BASIC_START
            "#,
        0,
    )
    .unwrap();

    assert_eq!(program.origin, 0x19EF);
    assert_eq!(program.symbols["BASIC_START"], 0x1A30);
    assert_eq!(program.symbols["ENTRY"], 0x1A30);
    assert_eq!(program.segments.len(), 1);
    assert_eq!(program.segments[0].addr, 0x19EF);
    assert_eq!(
        program.segments[0].bytes[0..16],
        [
            0x0F, 0x0A, 0x00, 0x43, 0x9A, b'U', b'S', b'R', 0x96, b'6', b'7', b'0', b'4', 0x95,
            0xFF, 0x00,
        ]
    );
    assert_eq!(
        &program.segments[0].bytes[0x41..],
        &[0x3E, 0x02, 0xD3, 0x00, 0xC3, 0x30, 0x1A]
    );
    assert_eq!(program.next_addr, 0x1A37);
}

#[test]
fn assembles_program_compatibility_edges() {
    let program = assemble_program(
        r#"
                OUT (C),0
                DB "A\"B"
            "#,
        0x9000,
    )
    .unwrap();

    assert_eq!(program.bytes, vec![0xED, 0x71, b'A', b'"', b'B']);
}

#[test]
fn reports_program_errors_with_line_numbers() {
    let err = assemble_program(
        r#"
                JR missing
            "#,
        0x1000,
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "line 2: unknown symbol 'MISSING'");

    let err = assemble_program("again: NOP\nagain: RET", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 2: duplicate label 'AGAIN'");
}

#[test]
fn assembles_index_halves_and_cb_result_copies() {
    round_trip("LD IXH,12H", 0, &[0xDD, 0x26, 0x12], "LD IXH,12H");
    round_trip("ld a,iyl", 0, &[0xFD, 0x7D], "LD A,IYL");
    round_trip("LD IXL,IXH", 0, &[0xDD, 0x6C], "LD IXL,IXH");
    round_trip("LD IYH,B", 0, &[0xFD, 0x60], "LD IYH,B");
    round_trip("INC IXL", 0, &[0xDD, 0x2C], "INC IXL");
    round_trip("DEC IYH", 0, &[0xFD, 0x25], "DEC IYH");
    round_trip("ADD A,IXH", 0, &[0xDD, 0x84], "ADD A,IXH");
    round_trip("CP IYL", 0, &[0xFD, 0xBD], "CP IYL");
    round_trip(
        "RLC (IX+5),B",
        0,
        &[0xDD, 0xCB, 0x05, 0x00],
        "RLC (IX+05H),B",
    );
    round_trip(
        "SET 3,(IY-1),A",
        0,
        &[0xFD, 0xCB, 0xFF, 0xDF],
        "SET 3,(IY-01H),A",
    );
    for bad in [
        "LD IXH,IYL",
        "LD IXH,H",
        "LD L,IYH",
        "LD IXL,(HL)",
        "RLC (IX+1),(HL)",
        "RES 1,(HL),A",
    ] {
        assert!(assemble_line(bad, 0).is_err(), "{bad} should not assemble");
    }

    let program = assemble_program("ld ixh,a\ninc iyl\n", 0x4000).unwrap();
    assert_eq!(program.bytes, vec![0xDD, 0x67, 0xFD, 0x2C]);
}

#[test]
fn reassembles_every_disassembled_opcode() {
    const PREFIXES: [u8; 4] = [0xCB, 0xDD, 0xED, 0xFD];
    let mut sequences: Vec<Vec<u8>> = Vec::new();
    for op in 0..=255u8 {
        if !PREFIXES.contains(&op) {
            sequences.push(vec![op, 0x05, 0x12]);
        }
        sequences.push(vec![0xCB, op]);
        sequences.push(vec![0xED, op, 0x34, 0x12]);
        for prefix in [0xDD, 0xFD] {
            if !PREFIXES.contains(&op) {
                sequences.push(vec![prefix, op, 0x05, 0x12]);
            }
            sequences.push(vec![prefix, 0xCB, 0x05, op]);
        }
    }

    let pc = 0x1000;
    let mut failures = Vec::new();
    for sequence in sequences {
        let mut bus = FakeBus::new();
        for (offset, byte) in sequence.iter().enumerate() {
            bus.mem[pc as usize + offset] = *byte;
        }
        let original = disassemble_at(&mut bus, pc);
        if original.text.starts_with("DB ") {
            continue;
        }
        // Redundant DD/FD prefixes reassemble to the unprefixed form, so
        // compare the reassembled instruction text rather than the bytes.
        match assemble_line(&original.text, pc) {
            Ok(bytes) => {
                let mut bus = FakeBus::new();
                for (offset, byte) in bytes.iter().enumerate() {
                    bus.mem[pc as usize + offset] = *byte;
                }
                let text = disassemble_at(&mut bus, pc).text;
                if text != original.text {
                    failures.push(format!("{} -> {}", original.text, text));
                }
            }
            Err(err) => failures.push(format!("{}: {}", original.text, err.message)),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn rept_repeats_blocks_with_nesting_counters_and_zero_counts() {
    let program = assemble_program(
        r#"
                PADDING EQU 2
        START:  REPT PADDING+1
                NOP
        BODY_END: ENDR
                REPT 2, ROW
                REPT 2, col
                DB ROW, COL, "row"
                ENDR
                ENDR
                REPT 0
                HALT
                ENDR
            "#,
        0x8000,
    )
    .unwrap();

    assert_eq!(
        program.bytes,
        vec![
            0x00, 0x00, 0x00, //
            0, 0, b'r', b'o', b'w', //
            0, 1, b'r', b'o', b'w', //
            1, 0, b'r', b'o', b'w', //
            1, 1, b'r', b'o', b'w',
        ]
    );
    assert_eq!(program.symbols["START"], 0x8000);
    assert_eq!(program.symbols["BODY_END"], 0x8003);
    assert!(!program.symbols.contains_key("ROW"));
    let nop_lines: Vec<_> = program.lines.iter().filter(|line| line.line == 4).collect();
    assert_eq!(nop_lines.len(), 3);
    assert_eq!(nop_lines[2].addr, 0x8002);
    assert!(
        program
            .lines
            .iter()
            .any(|line| line.source.contains("DB 1, 0, \"row\""))
    );
}

#[test]
fn rept_counts_can_use_labels_defined_above() {
    let program = assemble_program(
        "START: NOP\n NOP\nEND: REPT END-START\n RET\n ENDR\n",
        0x4000,
    )
    .unwrap();
    assert_eq!(program.bytes, vec![0x00, 0x00, 0xC9, 0xC9]);
}

#[test]
fn rept_reports_structure_errors() {
    let err = assemble_program("NOP\nREPT 2\nNOP\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 2: REPT without ENDR");

    let err = assemble_program("NOP\nENDR\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 2: ENDR without REPT");

    let err = assemble_program("REPT LATER\nNOP\nENDR\nLATER EQU 2\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 1: unknown symbol 'LATER'");

    let err = assemble_program("REPT 2\nLOOP: NOP\nENDR\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 2: duplicate label 'LOOP'");

    let err = assemble_program("REPT 2048\nREPT 1024\nNOP\nENDR\nENDR\n", 0).unwrap_err();
    assert!(err.to_string().contains("expand to more than"));
}

#[test]
fn assert_checks_comparisons_and_values_after_all_symbols_are_known() {
    let program = assemble_program(
        r#"
                ASSERT DONE-START = 3
        START:  LD A,1
                NOP
        DONE:
                ASSERT $ == DONE
                ASSERT DONE <> START
                ASSERT DONE-START >= 3, "block is too short"
                ASSERT DONE > START
                ASSERT START <= DONE
                ASSERT START < DONE
                ASSERT DONE != 0
                ASSERT DONE
            "#,
        0x1000,
    )
    .unwrap();
    assert_eq!(program.bytes, vec![0x3E, 0x01, 0x00]);

    let err = assemble_program("A: NOP\nB: ASSERT B-A = 2\n", 0).unwrap_err();
    assert_eq!(
        err.to_string(),
        "line 2: assertion failed: B-A = 2 (left is 1, right is 2)"
    );

    let err = assemble_program("ASSERT $+3 < 2, \"too far\"\n", 0).unwrap_err();
    assert_eq!(
        err.to_string(),
        "line 1: assertion failed: too far (left is 3, right is 2)"
    );

    let err = assemble_program("ASSERT 0\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 1: assertion failed: 0 (value is 0)");

    let err = assemble_program("ASSERT 1 < 2 < 3\n", 0).unwrap_err();
    assert!(err.to_string().contains("more than one comparison"));

    let err = assemble_program("ASSERT 1, 2\n", 0).unwrap_err();
    assert!(err.to_string().contains("optional quoted message"));
}

fn eval_equ(expr: &str) -> u16 {
    let program = assemble_program(&format!("V EQU {expr}\n"), 0).unwrap();
    program.symbols["V"]
}

#[test]
fn expressions_follow_c_precedence() {
    assert_eq!(eval_equ("2+3*4"), 14);
    assert_eq!(eval_equ("(2+3)*4"), 20);
    assert_eq!(eval_equ("89*64"), 0x1640);
    assert_eq!(eval_equ("100/7"), 14);
    assert_eq!(eval_equ("100%7"), 2);
    assert_eq!(eval_equ("1<<4+1"), 32);
    assert_eq!(eval_equ("0F0H|0FH&3"), 0xF3);
    assert_eq!(eval_equ("0FFH^0FH"), 0xF0);
    assert_eq!(eval_equ("~0&0FFH"), 0xFF);
    assert_eq!(eval_equ("!0+!5"), 1);
    assert_eq!(eval_equ("-(-3)"), 3);
    assert_eq!(eval_equ("8000H>>4"), 0x800);
    assert_eq!(eval_equ("10-4-3"), 3);
    assert_eq!(eval_equ("LOW(1234H)"), 0x34);
    assert_eq!(eval_equ("HIGH(0B000H)"), 0xB0);
    assert_eq!(eval_equ("HIGH(LOW(1234H)*256)+1"), 0x35);
}

#[test]
fn expressions_work_in_operands_and_directives() {
    let program = assemble_program(
        r#"
        GLYPHS  EQU 89
        SIZE    EQU GLYPHS*64
                ORG 8000H
        START:  LD BC,SIZE
                LD A,HIGH(TABLE)
                LD A,(TABLE+2*2)
                LD A,(2+1)*(1+1)
                LD (IX+2*3),A
                OUT (60H+1*2),A
                DB LOW(START), HIGH(START), (1+1)*3
                DW TABLE+SIZE/GLYPHS
        TABLE:
                ASSERT SIZE = 1640H
                ASSERT 1<<3 = 8
            "#,
        0,
    )
    .unwrap();
    let table = program.symbols["TABLE"];
    let table_bytes = table.to_le_bytes();
    let plus_64 = (table + 64).to_le_bytes();
    let plus_4 = (table + 4).to_le_bytes();
    #[rustfmt::skip]
    let expected = vec![
            0x01, 0x40, 0x16, // LD BC,1640H
            0x3E, table_bytes[1], // LD A,HIGH(TABLE)
            0x3A, plus_4[0], plus_4[1], // LD A,(TABLE+4)
            0x3E, 6, // LD A,(2+1)*(1+1): an expression, not memory
            0xDD, 0x77, 6, // LD (IX+6),A
            0xD3, 0x62, // OUT (62H),A
            0x00, 0x80, 6, // DB
            plus_64[0], plus_64[1], // DW
    ];
    assert_eq!(program.bytes, expected);
}

#[test]
fn forward_references_in_expressions_resolve_in_pass_two() {
    let program = assemble_program(
        "LD A,HIGH(LATER)*2/WIDTH\nDB LOW(LATER)/WIDTH\nORG 1234H\nLATER: NOP\nWIDTH EQU 2\n",
        0,
    )
    .unwrap();
    assert_eq!(program.bytes[..3], [0x3E, 0x12, 0x1A]);
}

#[test]
fn expressions_report_errors() {
    let err = assemble_program("V EQU 1/0\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 1: division by zero in '1/0'");

    let err = assemble_program("V EQU 5%(2-2)\n", 0).unwrap_err();
    assert!(err.to_string().contains("division by zero"));

    let err = assemble_program("V EQU 1<<32\n", 0).unwrap_err();
    assert!(err.to_string().contains("larger than 31"));

    let err = assemble_program("V EQU (1+2\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 1: invalid expression '(1+2'");

    let err = assemble_program("V EQU 2*\n", 0).unwrap_err();
    assert!(err.to_string().contains("invalid expression"));

    let err = assemble_program("V EQU MID(1)\n", 0).unwrap_err();
    assert_eq!(err.to_string(), "line 1: unknown function 'MID'");

    let err = assemble_program("LD A,HIGH(1234H)*16\n", 0).unwrap_err();
    assert!(err.to_string().contains("outside 0..255"));
}
