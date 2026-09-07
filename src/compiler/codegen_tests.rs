use super::*;
use crate::asm::assemble_line;
use crate::bus::FakeBus;
use crate::compiler::abi::{assign_params, return_home};
use crate::compiler::harness::{AccessKind, ExecConfig, execute_function, execute_function_with};
use crate::disasm::disassemble_at;

fn compile_ok(src: &str) -> CompilationResult {
    let result = compile_source("test.c80", src);
    assert!(!result.has_errors(), "{:?}\n{src}", result.diagnostics);
    assert!(result.program.is_some(), "expected typed IR");
    assert!(result.code.is_some(), "expected generated code for {src}");
    result
}

fn codes(result: &CompilationResult) -> Vec<&'static str> {
    result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code.as_str())
        .collect()
}

fn func_bytes<'a>(result: &'a CompilationResult, name: &str) -> &'a [u8] {
    result
        .code
        .as_ref()
        .and_then(|code| code.function_bytes(name))
        .unwrap_or_else(|| panic!("missing bytes for {name}"))
}

fn func_tstates(result: &CompilationResult, name: &str) -> Vec<&'static str> {
    result
        .code
        .as_ref()
        .and_then(|code| code.function(name))
        .unwrap()
        .mapped
        .iter()
        .map(|m| m.t_states.unwrap_or(""))
        .collect()
}

fn assembly_of(result: &CompilationResult) -> &str {
    &result.code.as_ref().unwrap().assembly
}

#[test]
fn abi_word_then_byte_skipping_reserved_halves() {
    assert_eq!(
        assign_params(&[CType::U8]),
        Some(vec![RegHome::Byte(R8::A)])
    );
    assert_eq!(
        assign_params(&[CType::U16, CType::U16]),
        Some(vec![RegHome::Word(Rr::Hl), RegHome::Word(Rr::De)])
    );
    assert_eq!(
        assign_params(&[CType::U16, CType::U8]),
        Some(vec![RegHome::Word(Rr::Hl), RegHome::Byte(R8::A)])
    );
    assert_eq!(
        assign_params(&[CType::U8, CType::U16]),
        Some(vec![RegHome::Byte(R8::A), RegHome::Word(Rr::Hl)])
    );
    assert_eq!(
        assign_params(&[CType::U8, CType::U8]),
        Some(vec![RegHome::Byte(R8::A), RegHome::Byte(R8::C)])
    );
    assert_eq!(
        assign_params(&[CType::U16, CType::U16, CType::U16]),
        Some(vec![
            RegHome::Word(Rr::Hl),
            RegHome::Word(Rr::De),
            RegHome::Word(Rr::Bc)
        ])
    );
    assert_eq!(assign_params(&[CType::U16; 4]), None);
    assert_eq!(return_home(CType::U8), Some(RegHome::Byte(R8::A)));
    assert_eq!(return_home(CType::I16), Some(RegHome::Word(Rr::Hl)));
}

#[test]
fn byte_identity_is_ret() {
    let result = compile_ok("u8 id(u8 x) { return x; }");
    let bytes = func_bytes(&result, "id");
    assert_eq!(bytes, &[0xC9]);
    let text = assembly_of(&result).to_ascii_uppercase();
    assert!(text.contains("RET"), "{text}");
    assert!(!text.contains("IX"), "{text}");
    assert!(!text.contains("IY"), "{text}");
    assert!(!text.contains("PUSH"), "{text}");
    assert_eq!(func_tstates(&result, "id"), ["10"]);

    for x in [0u16, 1, 127, 128, 255] {
        let exec = execute_function(result.code.as_ref().unwrap(), "id", &[x]).unwrap();
        assert_eq!(exec.return_byte(), x as u8, "id({x})");
        assert_eq!(exec.sp, crate::compiler::harness::DEFAULT_SP);
        assert_eq!(exec.ix, 0x1111);
        assert_eq!(exec.iy, 0x2222);
        assert_eq!(exec.tstates, 10);
        let data: Vec<_> = exec.data_accesses().collect();
        assert!(
            data.iter().all(|a| a.kind != AccessKind::DataWrite),
            "{data:?}"
        );
        assert!(
            data.iter()
                .all(|a| a.kind != AccessKind::DataRead || a.addr >= 0xFE00),
            "non-stack data reads: {data:?}"
        );
    }
}

#[test]
fn word_add_is_add_hl_de() {
    let result = compile_ok("u16 add(u16 a, u16 b) { return a + b; }");
    let bytes = func_bytes(&result, "add");
    assert_eq!(bytes, &[0x19, 0xC9], "assembly:\n{}", assembly_of(&result));
    let text = assembly_of(&result).to_ascii_uppercase();
    assert!(text.contains("ADD HL,DE"), "{text}");
    assert!(text.contains("RET"), "{text}");
    assert!(!text.contains("IX"), "{text}");
    assert!(!text.contains("PUSH"), "{text}");
    assert_eq!(func_tstates(&result, "add"), ["11", "10"]);

    let cases = [
        (0u16, 0u16, 0u16),
        (1, 2, 3),
        (0xFFFF, 1, 0),
        (0x7FFF, 1, 0x8000),
        (0x8000, 0x8000, 0),
        (0xFFFF, 0xFFFF, 0xFFFE),
    ];
    for (a, b, expect) in cases {
        let exec = execute_function(result.code.as_ref().unwrap(), "add", &[a, b]).unwrap();
        assert_eq!(exec.return_word(), expect, "add({a:#06x},{b:#06x})");
        assert_eq!(exec.tstates, 21);
        assert_eq!(exec.sp, crate::compiler::harness::DEFAULT_SP);
        assert_eq!(exec.ix, 0x1111);
    }
}

#[test]
fn wrap_and_sign_boundary_leaves_execute() {
    let result = compile_ok(
        r#"
u8 wrap() { return 255 + 1; }
i8 min() { return -128; }
i16 sadd(i16 a, i16 b) { return a + b; }
"#,
    );
    let wrap = execute_function(result.code.as_ref().unwrap(), "wrap", &[]).unwrap();
    assert_eq!(wrap.return_byte(), 0);
    let min = execute_function(result.code.as_ref().unwrap(), "min", &[]).unwrap();
    assert_eq!(min.return_byte(), 0x80);
    let sadd = execute_function(result.code.as_ref().unwrap(), "sadd", &[0x7FFF, 1]).unwrap();
    assert_eq!(sadd.return_word(), 0x8000);
}

#[test]
fn round_trip_bytes_via_assembler_disassembler() {
    let result = compile_ok("u16 add(u16 a, u16 b) { return a + b; }");
    let code = result.code.as_ref().unwrap();
    let mut bus = FakeBus::new();
    for (i, b) in code.assembled.bytes.iter().enumerate() {
        bus.mem[code.assembled.origin.wrapping_add(i as u16) as usize] = *b;
    }
    let mut addr = code.assembled.origin;
    let end = addr.wrapping_add(code.assembled.bytes.len() as u16);
    let mut recovered = Vec::new();
    while addr != end {
        let d = disassemble_at(&mut bus, addr);
        let bytes = assemble_line(&d.text, addr).expect(&d.text);
        assert_eq!(bytes, d.bytes, "{}", d.text);
        recovered.extend_from_slice(&bytes);
        addr = addr.wrapping_add(d.len as u16);
    }
    assert_eq!(recovered, code.assembled.bytes);
}

#[test]
fn deterministic_assembly_and_ids() {
    let src = "u8 id(u8 x) { return x; }\nu16 add(u16 a, u16 b) { return a + b; }\n";
    let a = compile_ok(src);
    let b = compile_ok(src);
    let ca = a.code.as_ref().unwrap();
    let cb = b.code.as_ref().unwrap();
    assert_eq!(ca.assembly, cb.assembly);
    assert_eq!(ca.assembled.bytes, cb.assembled.bytes);
    let ids_a: Vec<_> = ca
        .functions
        .iter()
        .flat_map(|f| f.instruction_ids.iter().map(|id| id.0))
        .collect();
    let ids_b: Vec<_> = cb
        .functions
        .iter()
        .flat_map(|f| f.instruction_ids.iter().map(|id| id.0))
        .collect();
    assert_eq!(ids_a, ids_b);
    assert!(!ids_a.is_empty());
}

#[test]
fn if_else_selects_assigned_local() {
    let result = compile_ok(
        r#"
u8 all(bool c) {
    u8 x;
    if (c) { x = 1; } else { x = 2; }
    return x;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "all", &[1]).unwrap().return_byte(),
        1
    );
    assert_eq!(
        execute_function(code, "all", &[0]).unwrap().return_byte(),
        2
    );
}

#[test]
fn while_break_and_leaf_share_a_unit() {
    let result = compile_ok(
        r#"
void loop(bool c) {
    while (c) { break; }
}
u8 val() { return 1; }
"#,
    );
    assert!(result.code.as_ref().unwrap().function("val").is_some());
    assert!(result.code.as_ref().unwrap().function("loop").is_some());
    let exec = execute_function(result.code.as_ref().unwrap(), "val", &[]).unwrap();
    assert_eq!(exec.return_byte(), 1);
    execute_function(result.code.as_ref().unwrap(), "loop", &[0]).unwrap();
    execute_function(result.code.as_ref().unwrap(), "loop", &[1]).unwrap();
}

#[test]
fn register_signature_that_does_not_fit_is_diagnosed() {
    let result = compile_source(
        "test.c80",
        "u16 too_many(u16 a, u16 b, u16 c, u16 d) { return a; }",
    );
    assert!(result.has_errors());
    assert!(
        codes(&result).contains(&"cg-unsupported"),
        "{:?}",
        codes(&result)
    );
    assert!(result.program.is_some());
    assert!(result.code.is_none());
}

#[test]
fn harness_timeout_is_failure() {
    let result = compile_ok("u8 id(u8 x) { return x; }");
    let err = execute_function_with(
        result.code.as_ref().unwrap(),
        "id",
        &[1],
        &ExecConfig {
            insn_limit: 0,
            tstate_limit: 1_000_000,
            ..ExecConfig::default()
        },
    )
    .unwrap_err();
    assert!(err.0.contains("timed out"), "{err}");
}

#[test]
fn casts_and_locals_execute() {
    let result = compile_ok(
        r#"
u16 widen(u8 x) { return u16(x); }
u8 narrow(u16 x) { return u8(x); }
bool flag(u8 x) { return bool(x); }
u8 from_bool(bool b) { return u8(b); }
u8 local() { u8 a = 255; return a; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "widen", &[0xAB])
            .unwrap()
            .return_word(),
        0x00AB
    );
    assert_eq!(
        execute_function(code, "narrow", &[0x12AB])
            .unwrap()
            .return_byte(),
        0xAB
    );
    assert_eq!(
        execute_function(code, "flag", &[0]).unwrap().return_byte(),
        0
    );
    assert_eq!(
        execute_function(code, "flag", &[3]).unwrap().return_byte(),
        1
    );
    assert_eq!(
        execute_function(code, "from_bool", &[1])
            .unwrap()
            .return_byte(),
        1
    );
    assert_eq!(
        execute_function(code, "local", &[]).unwrap().return_byte(),
        255
    );
}

fn global_addr(code: &GeneratedProgram, name: &str) -> u16 {
    code.global(name)
        .unwrap_or_else(|| panic!("missing global {name}"))
        .addr
}

#[test]
fn signed_comparisons_across_overflow() {
    let result = compile_ok(
        r#"
bool slt8(i8 a, i8 b) { return a < b; }
bool slt16(i16 a, i16 b) { return a < b; }
bool sgt8(i8 a, i8 b) { return a > b; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let cases8 = [
        (0x80u16, 0x7Fu16, 1u8),
        (0x7F, 0x80, 0),
        (0xFF, 0, 1),
        (0, 0xFF, 0),
        (0, 1, 1),
        (5, 5, 0),
    ];
    for (a, b, expect) in cases8 {
        assert_eq!(
            execute_function(code, "slt8", &[a, b])
                .unwrap()
                .return_byte(),
            expect,
            "slt8({a:#x},{b:#x})"
        );
    }
    assert_eq!(
        execute_function(code, "slt16", &[0x8000, 0x7FFF])
            .unwrap()
            .return_byte(),
        1
    );
    assert_eq!(
        execute_function(code, "slt16", &[0x7FFF, 0x8000])
            .unwrap()
            .return_byte(),
        0
    );
    assert_eq!(
        execute_function(code, "sgt8", &[0x7F, 0x80])
            .unwrap()
            .return_byte(),
        1
    );
}

#[test]
fn short_circuit_paths_and_suppressed_stores() {
    let result = compile_ok(
        r#"
u8 g;
bool sc_and(bool a) { return a && (g = 1) != 0; }
bool sc_or(bool a) { return a || (g = 1) != 0; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let g = global_addr(code, "g");
    let and_false = execute_function(code, "sc_and", &[0]).unwrap();
    assert_eq!(and_false.return_byte(), 0);
    assert!(
        !and_false
            .data_accesses()
            .any(|a| a.kind == AccessKind::DataWrite && a.addr == g),
        "false && must not store g"
    );
    let and_true = execute_function(code, "sc_and", &[1]).unwrap();
    assert_eq!(and_true.return_byte(), 1);
    assert!(
        and_true
            .data_accesses()
            .any(|a| a.kind == AccessKind::DataWrite && a.addr == g && a.value == 1)
    );
    let or_true = execute_function(code, "sc_or", &[1]).unwrap();
    assert_eq!(or_true.return_byte(), 1);
    assert!(
        !or_true
            .data_accesses()
            .any(|a| a.kind == AccessKind::DataWrite && a.addr == g)
    );
    let or_false = execute_function(code, "sc_or", &[0]).unwrap();
    assert_eq!(or_false.return_byte(), 1);
    assert!(
        or_false
            .data_accesses()
            .any(|a| a.kind == AccessKind::DataWrite && a.addr == g)
    );
}

#[test]
fn zero_one_many_iterations_and_nested_branches() {
    let result = compile_ok(
        r#"
u8 count(u8 n) {
    u8 s = 0;
    while (n != 0) {
        s = s + 1;
        n = n - 1;
    }
    return s;
}
u8 nest(bool a, bool b) {
    if (a) {
        if (b) { return 1; }
        return 2;
    }
    return 3;
}
u8 cont(u8 n) {
    u8 s = 0;
    while (n != 0) {
        n = n - 1;
        if (n == 2) { continue; }
        s = s + 1;
    }
    return s;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    let text = assembly_of(&result).to_ascii_uppercase();
    assert!(!text.contains("IX"), "{text}");
    assert!(!text.contains("PUSH"), "{text}");
    let zero = execute_function(code, "count", &[0]).unwrap();
    assert_eq!(zero.return_byte(), 0);
    let one = execute_function(code, "count", &[1]).unwrap();
    assert_eq!(one.return_byte(), 1);
    let many = execute_function(code, "count", &[5]).unwrap();
    assert_eq!(many.return_byte(), 5);
    assert!(many.tstates > one.tstates);
    assert_eq!(zero.tstates, 66);
    assert_eq!(one.tstates, 127);
    assert_eq!(many.tstates, 371);
    assert_eq!(
        execute_function(code, "nest", &[1, 1])
            .unwrap()
            .return_byte(),
        1
    );
    assert_eq!(
        execute_function(code, "nest", &[1, 0])
            .unwrap()
            .return_byte(),
        2
    );
    assert_eq!(
        execute_function(code, "nest", &[0, 1])
            .unwrap()
            .return_byte(),
        3
    );
    assert_eq!(
        execute_function(code, "cont", &[4]).unwrap().return_byte(),
        3
    );
}

#[test]
fn long_branches_use_jp_not_jr() {
    let mut body = String::from("u8 g;\nu8 far(bool c) {\n    if (c) {\n");
    for _ in 0..80 {
        body.push_str("        g = g + 1;\n");
    }
    body.push_str("    }\n    return g;\n}\n");
    let result = compile_ok(&body);
    let text = assembly_of(&result).to_ascii_uppercase();
    assert!(text.contains("JP"), "{text}");
    assert!(!text.contains("JR"), "{text}");
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "far", &[0]).unwrap().return_byte(),
        0
    );
    assert_eq!(
        execute_function(code, "far", &[1]).unwrap().return_byte(),
        80
    );
}

#[test]
fn repeated_global_writes_and_polling_rereads() {
    let result = compile_ok(
        r#"
u8 g;
void writes() { g = 1; g = 2; g = 3; }
u8 poll() { u8 a = g; u8 b = g; return a + b; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let g = global_addr(code, "g");
    let writes = execute_function(code, "writes", &[]).unwrap();
    let g_writes: Vec<u8> = writes
        .data_accesses()
        .filter(|a| a.kind == AccessKind::DataWrite && a.addr == g)
        .map(|a| a.value)
        .collect();
    assert_eq!(
        g_writes,
        [1, 2, 3],
        "{:?}",
        writes.data_accesses().collect::<Vec<_>>()
    );

    let polled = execute_function_with(
        code,
        "poll",
        &[],
        &ExecConfig {
            scripted_reads: vec![(g, vec![3, 5])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(polled.return_byte(), 8);
    let g_reads: Vec<_> = polled
        .data_accesses()
        .filter(|a| a.kind == AccessKind::DataRead && a.addr == g)
        .collect();
    assert_eq!(g_reads.len(), 2, "{g_reads:?}");
}
