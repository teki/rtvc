use super::*;
use rtvc_core::asm::assemble_line;
use rtvc_core::bus::FakeBus;
use crate::abi::{assign_params, return_home};
use crate::harness::{AccessKind, ExecConfig, execute_function, execute_function_with};
use rtvc_core::disasm::disassemble_at;

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
        assert_eq!(exec.sp, crate::harness::DEFAULT_SP);
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
        assert_eq!(exec.sp, crate::harness::DEFAULT_SP);
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
    assert_eq!(zero.tstates, 48);
    assert_eq!(one.tstates, 106);
    assert_eq!(many.tstates, 338);
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
    assert!(
        text.contains("JP ") || text.contains("JP,"),
        "80 increments must keep a long JP:\n{text}"
    );
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

fn assert_ix_iy_sp(exec: &crate::harness::ExecResult) {
    assert_eq!(exec.sp, crate::harness::DEFAULT_SP);
    assert_eq!(exec.ix, 0x1111);
    assert_eq!(exec.iy, 0x2222);
}

#[test]
fn register_calls_execute_and_leaves_stay_frame_free() {
    let result = compile_ok(
        r#"
u8 id(u8 x) { return x; }
u8 add8(u8 a, u8 b) { return a + b; }
u16 add(u16 a, u16 b) { return a + b; }
u8 nest(u8 x) { return add8(x, id(1)); }
u16 swapped(u16 a, u16 b) { return add(b, a); }
u8 sibling(u8 x) { u8 a = id(x); u8 b = id(1); return add8(a, b); }
u8 nested_args(u8 x) { return add8(id(x), id(2)); }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let id_text = assembly_of(&result).to_ascii_uppercase();
    let id_asm = code.function("id").unwrap();
    let id_fn_asm = code
        .function("id")
        .unwrap()
        .mapped
        .iter()
        .map(|m| m.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(id_fn_asm.contains("RET"), "{id_fn_asm}");
    assert!(!id_fn_asm.contains("IX"), "{id_fn_asm}");
    assert!(!id_fn_asm.contains("PUSH"), "{id_fn_asm}");
    assert_eq!(func_bytes(&result, "id"), &[0xC9]);

    let exec_id = execute_function(code, "id", &[9]).unwrap();
    assert_eq!(exec_id.return_byte(), 9);
    assert_ix_iy_sp(&exec_id);
    assert_eq!(exec_id.sp_used, 0);
    assert_eq!(code.function("id").unwrap().stack_bound, 0);

    let nest = execute_function(code, "nest", &[5]).unwrap();
    assert_eq!(nest.return_byte(), 6);
    assert_ix_iy_sp(&nest);
    let nest_fn = code.function("nest").unwrap();
    assert!(
        nest.sp_used <= nest_fn.stack_bound,
        "{} > {}",
        nest.sp_used,
        nest_fn.stack_bound
    );

    let swapped = execute_function(code, "swapped", &[0x0011, 0x2200]).unwrap();
    assert_eq!(swapped.return_word(), 0x2211);
    assert_ix_iy_sp(&swapped);

    let sib = execute_function(code, "sibling", &[3]).unwrap();
    assert_eq!(sib.return_byte(), 4);
    assert_ix_iy_sp(&sib);
    let sib_fn = code.function("sibling").unwrap();
    assert!(sib.sp_used <= sib_fn.stack_bound);

    let nested = execute_function(code, "nested_args", &[7]).unwrap();
    assert_eq!(nested.return_byte(), 9);
    assert_ix_iy_sp(&nested);
    let _ = id_text;
    let _ = id_asm;
}

#[test]
fn stackcall_args_and_cleanup() {
    let result = compile_ok(
        r#"
@stackcall pub u16 add(u16 a, u16 b) { return a + b; }
@stackcall u8 mix(u8 a, u16 b, u8 c) { return a + c; }
@stackcall i8 s8(i8 x, i8 y) { return x + y; }
u16 use_add() { return add(2, 3); }
u8 use_mix() { return mix(1, 0x1111, 4); }
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(code.function("add").unwrap().conv, CallConv::Stack);
    let add = execute_function(code, "add", &[2, 3]).unwrap();
    assert_eq!(add.return_word(), 5);
    assert_ix_iy_sp(&add);
    assert!(add.sp_used <= code.function("add").unwrap().stack_bound);

    let mix = execute_function(code, "mix", &[1, 0x1111, 4]).unwrap();
    assert_eq!(mix.return_byte(), 5);
    assert_ix_iy_sp(&mix);

    let s8 = execute_function(code, "s8", &[0xFF, 1]).unwrap();
    assert_eq!(s8.return_byte() as i8, 0);

    let via = execute_function(code, "use_add", &[]).unwrap();
    assert_eq!(via.return_word(), 5);
    assert_ix_iy_sp(&via);
    let via_fn = code.function("use_add").unwrap();
    assert!(
        via.sp_used <= via_fn.stack_bound,
        "{} > {}",
        via.sp_used,
        via_fn.stack_bound
    );
    assert!(
        via_fn.stack_bound >= 6,
        "caller should count two stack slots plus CALL"
    );

    let mix_via = execute_function(code, "use_mix", &[]).unwrap();
    assert_eq!(mix_via.return_byte(), 5);
    assert_ix_iy_sp(&mix_via);
}

#[test]
fn stackcall_fits_rejected_register_signature() {
    let result =
        compile_ok("@stackcall u16 too_many(u16 a, u16 b, u16 c, u16 d) { return a + d; }");
    let exec = execute_function(result.code.as_ref().unwrap(), "too_many", &[1, 0, 0, 4]).unwrap();
    assert_eq!(exec.return_word(), 5);
    assert_ix_iy_sp(&exec);
}

#[test]
fn indexed_frame_boundary_is_rejected() {
    let mut src = String::from("u8 id(u8 x) { return x; }\nu8 f() {\n");
    for i in 0..70 {
        src.push_str(&format!("  u16 v{i} = {i};\n"));
    }
    src.push_str("  u8 t = id(1);\n");
    for i in 0..70 {
        src.push_str(&format!("  t = t + u8(v{i});\n"));
    }
    src.push_str("  return t;\n}\n");
    let result = compile_source("test.c80", &src);
    assert!(result.has_errors(), "expected frame displacement error");
    assert!(
        codes(&result).contains(&"cg-unsupported"),
        "{:?}",
        codes(&result)
    );
    assert!(result.code.is_none());
}

#[test]
fn recursion_still_rejected_with_calls_enabled() {
    let result = compile_source("test.c80", "u8 rec(u8 x) { return rec(x); }");
    assert!(
        codes(&result).contains(&"ty-recursion"),
        "{:?}",
        codes(&result)
    );
    assert!(result.code.is_none());
}

#[test]
fn data_only_unit_keeps_declaration_order() {
    let result = compile_ok(
        r#"
u8 frame_counter;
u8 positions[16];
str enemy_name = "hello world";
"#,
    );
    let code = result.code.as_ref().unwrap();
    let origin = code.assembled.origin;
    let frame = code.global("frame_counter").unwrap();
    let positions = code.global("positions").unwrap();
    let name = code.global("enemy_name").unwrap();
    assert_eq!(frame.addr, origin);
    assert_eq!(frame.size, 1);
    assert_eq!(positions.addr, origin.wrapping_add(1));
    assert_eq!(positions.size, 16);
    assert_eq!(name.addr, origin.wrapping_add(17));
    assert_eq!(name.size, 12);
    let start = (name.addr.wrapping_sub(origin)) as usize;
    assert_eq!(&code.assembled.bytes[start..start + 12], b"\x0Bhello world");
}

#[test]
fn pointer_deref_and_index_execute() {
    let result = compile_ok(
        r#"
u8 cell;
u8 get(ptr<u8> p) { return *p; }
void set(ptr<u8> p, u8 v) { *p = v; }
u8 at(ptr<u8> p, u8 i) { return p[i]; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let cell = code.global("cell").unwrap().addr;
    let got = execute_function_with(
        code,
        "get",
        &[cell],
        &ExecConfig {
            initial_mem: vec![(cell, 9)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(got.return_byte(), 9);
    let set = execute_function(code, "set", &[cell, 4]).unwrap();
    assert_eq!(set.ix, 0x1111);
    let at = execute_function_with(
        code,
        "at",
        &[cell, 0],
        &ExecConfig {
            initial_mem: vec![(cell, 7)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(at.return_byte(), 7);
}

#[test]
fn global_array_index_and_store() {
    let result = compile_ok(
        r#"
u8 positions[4];
u8 get(u8 i) { return positions[i]; }
void set(u8 i, u8 v) { positions[i] = v; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let base = code.global("positions").unwrap().addr;
    execute_function(code, "set", &[2, 9]).unwrap();
    let got = execute_function_with(
        code,
        "get",
        &[2],
        &ExecConfig {
            initial_mem: vec![(base.wrapping_add(2), 9)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(got.return_byte(), 9);
}

#[test]
fn pointer_aliases_local_and_global() {
    let result = compile_ok(
        r#"
u8 g;
u8 alias() {
    u8 x = 1;
    ptr<u8> p = &x;
    ptr<u8> q = &g;
    *p = 4;
    *q = 5;
    return x;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    let g = code.global("g").unwrap().addr;
    let out = execute_function(code, "alias", &[]).unwrap();
    assert_eq!(out.return_byte(), 4);
    assert!(
        out.data_accesses()
            .any(|a| a.kind == AccessKind::DataWrite && a.addr == g && a.value == 5),
        "{:?}",
        out.data_accesses().collect::<Vec<_>>()
    );
}

#[test]
fn ordered_word_then_byte_pointer_accesses() {
    let result = compile_ok(
        r#"
u16 cell;
void store(ptr<u16> p, u16 v) { *p = v; }
u8 low(ptr<u8> p) { return *p; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let cell = code.global("cell").unwrap().addr;
    let store = execute_function(code, "store", &[cell, 0x0201]).unwrap();
    let data_writes: Vec<_> = store
        .data_accesses()
        .filter(|a| {
            a.kind == AccessKind::DataWrite && (a.addr == cell || a.addr == cell.wrapping_add(1))
        })
        .collect();
    assert_eq!(data_writes.len(), 2, "{data_writes:?}");
    assert_eq!(data_writes[0].addr, cell);
    assert_eq!(data_writes[0].value, 0x01);
    assert_eq!(data_writes[1].addr, cell.wrapping_add(1));
    assert_eq!(data_writes[1].value, 0x02);
}

#[test]
fn pointer_index_wraps_in_16bit_space() {
    let result = compile_ok("u8 at(ptr<u8> p) { return p[1]; }");
    let code = result.code.as_ref().unwrap();
    let got = execute_function_with(
        code,
        "at",
        &[0xFFFF],
        &ExecConfig {
            initial_mem: vec![(0x0000, 42)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(got.return_byte(), 42);
}

#[test]
fn prefixed_strings_and_len() {
    let empty = compile_ok(r#"str empty = ""; u8 n() { return empty.len; }"#);
    let code = empty.code.as_ref().unwrap();
    let g = code.global("empty").unwrap();
    assert_eq!(g.size, 1);
    let start = (g.addr.wrapping_sub(code.assembled.origin)) as usize;
    assert_eq!(code.assembled.bytes[start], 0);
    assert_eq!(execute_function(code, "n", &[]).unwrap().return_byte(), 0);

    let hello = compile_ok(
        r#"
str s = "hi";
u8 n() { return s.len; }
u8 ch() { return s[1]; }
u8 lit(str t) { return t.len; }
u8 pass() { return lit("xy"); }
"#,
    );
    let code = hello.code.as_ref().unwrap();
    assert_eq!(execute_function(code, "n", &[]).unwrap().return_byte(), 2);
    assert_eq!(
        execute_function(code, "ch", &[]).unwrap().return_byte(),
        b'i'
    );
    assert_eq!(
        execute_function(code, "pass", &[]).unwrap().return_byte(),
        2
    );
    let text = assembly_of(&hello).to_ascii_uppercase();
    assert!(
        text.contains("RET"),
        "string payload must follow a returning function, got {text}"
    );

    let payload = "A".repeat(255);
    let src = format!(r#"str s = "{payload}"; u8 n() {{ return s.len; }}"#);
    let long = compile_ok(&src);
    assert_eq!(
        execute_function(long.code.as_ref().unwrap(), "n", &[])
            .unwrap()
            .return_byte(),
        255
    );
}

#[test]
fn buffer_fill_pointer_walk_stays_in_registers() {
    let result = compile_ok(
        r#"
u8 buf[8];
void fill(ptr<u8> p, u8 n, u8 v) {
    while (n != 0) {
        *p = v;
        p = p + 1;
        n = n - 1;
    }
}
u8 get(u8 i) { return buf[i]; }
"#,
    );
    let text = assembly_of(&result).to_ascii_uppercase();
    assert!(!text.contains("IX"), "{text}");
    assert!(!text.contains("PUSH"), "{text}");
    let code = result.code.as_ref().unwrap();
    let base = code.global("buf").unwrap().addr;
    let filled = execute_function(code, "fill", &[base, 8, 7]).unwrap();
    for i in 0u16..8 {
        assert!(
            filled
                .data_accesses()
                .any(|a| a.kind == AccessKind::DataWrite
                    && a.addr == base.wrapping_add(i)
                    && a.value == 7),
            "missing write at {i}: {:?}",
            filled.data_accesses().collect::<Vec<_>>()
        );
    }
}

#[test]
fn array_name_does_not_decay() {
    let result = compile_source("test.c80", "u8 a[2]; ptr<u8> f() { return a; }");
    assert!(
        codes(&result).contains(&"ty-mismatch"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn inline_asm_inc_and_flag_capture() {
    let result = compile_ok(
        r#"
u8 inc8(u8 value) {
    u8 result;
    asm(in: a = value, out: a = result, clobber: flags) {
        inc a
    }
    return result;
}
u8 add_carry(u16 left, u16 right) {
    u16 sum;
    bool carry_set;
    asm(in: hl = left, in: de = right, out: hl = sum,
        out: carry = carry_set, clobber: flags) {
        add hl,de
    }
    return u8(carry_set);
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "inc8", &[41]).unwrap().return_byte(),
        42
    );
    assert_eq!(
        execute_function(code, "add_carry", &[1, 2])
            .unwrap()
            .return_byte(),
        0
    );
    assert_eq!(
        execute_function(code, "add_carry", &[0xFFFF, 1])
            .unwrap()
            .return_byte(),
        1
    );
}

#[test]
fn ix_frame_preserves_hl_argument() {
    let result = compile_ok(
        r#"
u16 echo(u16 x) {
    u8 slot;
    ptr<u8> p;
    p = &slot;
    *p = 1;
    return x;
}
"#,
    );
    assert_eq!(
        execute_function(result.code.as_ref().unwrap(), "echo", &[0x1234])
            .unwrap()
            .return_word(),
        0x1234
    );
}

#[test]
fn inline_asm_register_permutation_and_live_preserve() {
    let result = compile_ok(
        r#"
u8 perm(u8 x, u8 y) {
    u8 xo;
    u8 yo;
    asm(in: d = x, in: e = y, out: d = yo, out: e = xo, clobber: a) {
        ld a, d
        ld d, e
        ld e, a
    }
    return xo + yo;
}
u8 keep(u8 x, u8 y) {
    u8 r;
    asm(in: b = 1, out: b = r, clobber: flags) {
        inc b
    }
    return x + y + r;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "perm", &[3, 7])
            .unwrap()
            .return_byte(),
        10
    );
    assert_eq!(
        execute_function(code, "keep", &[10, 20])
            .unwrap()
            .return_byte(),
        32
    );
}

#[test]
fn inline_asm_indexed_output_evaluated_once() {
    let result = compile_ok(
        r#"
u8 buf[4];
u8 idx;
u8 bump() {
    idx = idx + 1;
    return idx;
}
u8 once() {
    idx = 0;
    buf[0] = 0;
    buf[1] = 0;
    buf[2] = 0;
    asm(in: a = 9, out: a = buf[bump()], clobber: flags) {
        nop
    }
    return buf[1];
}
"#,
    );
    assert_eq!(
        execute_function(result.code.as_ref().unwrap(), "once", &[])
            .unwrap()
            .return_byte(),
        9
    );
}

#[test]
fn inline_asm_ldir_counts_and_raw_zero() {
    let result = compile_ok(
        r#"
u8 src[4];
u8 dst[4];
void copy(u16 n) {
    src[0] = 1;
    src[1] = 2;
    src[2] = 3;
    src[3] = 4;
    asm(in: hl = &src[0], in: de = &dst[0], in: bc = n,
        clobber: hl, de, bc, flags, memory) {
        ldir
    }
}
u8 get(u8 i) { return dst[i]; }
u8 copy_get(u16 n, u8 i) {
    copy(n);
    return get(i);
}
"#,
    );
    let text = assembly_of(&result).to_ascii_uppercase();
    assert!(text.contains("LDIR"), "{text}");
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "copy_get", &[1, 0])
            .unwrap()
            .return_byte(),
        1
    );
    assert_eq!(
        execute_function(code, "copy_get", &[3, 2])
            .unwrap()
            .return_byte(),
        3
    );
    let zero = execute_function_with(
        code,
        "copy",
        &[0],
        &ExecConfig {
            insn_limit: 200,
            ..ExecConfig::default()
        },
    )
    .expect("BC=0 LDIR may smash code and still return");
    let extra_writes = zero
        .accesses
        .iter()
        .filter(|a| a.kind == AccessKind::DataWrite)
        .count();
    assert!(
        extra_writes > 8,
        "BC=0 LDIR must copy many bytes, not skip: writes={extra_writes}"
    );
}

#[test]
fn inline_asm_port_io_and_external_stub() {
    let result = compile_ok(
        r#"
u8 ports() {
    u8 r;
    asm(in: a = 7, in: c = 0x12, clobber: flags) {
        out (c), a
    }
    asm(in: c = 0x12, out: a = r, clobber: flags) {
        in a, (c)
    }
    return r;
}
u8 ext() {
    u8 r;
    asm(out: a = r, clobber: flags, stack: 2) {
        call 0x9000
    }
    return r;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    let ports = execute_function_with(
        code,
        "ports",
        &[],
        &ExecConfig {
            scripted_ports: vec![(0x12, vec![42])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(ports.return_byte(), 42);
    assert!(
        ports
            .accesses
            .iter()
            .any(|a| a.kind == AccessKind::PortOut && a.addr == 0x12 && a.value == 7),
        "{:?}",
        ports.accesses
    );
    let ext = execute_function_with(
        code,
        "ext",
        &[],
        &ExecConfig {
            initial_mem: vec![(0x9000, 0x3E), (0x9001, 42), (0x9002, 0xC9)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(ext.return_byte(), 42);
}

#[test]
fn listing_maps_bytes_spans_timing_and_identity() {
    let src = r#"
u8 g;
u8 add1(u8 x) { return x + 1; }
u8 dead_local() {
    u8 unused;
    return 1;
}
"#;
    let result = compile_ok(src);
    let code = result.code.as_ref().unwrap();
    let map = result.map().expect("successful compile has a listing map");
    assert!(map.every_byte_mapped(&code.assembled), "unmapped bytes");
    assert_eq!(map.identity.origin, DEFAULT_CODE_ORIGIN);
    assert_eq!(map.identity.crate_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(
        map.symbol("add1").map(|s| s.kind),
        Some(SymbolKind::Function)
    );
    assert_eq!(map.symbol("g").map(|s| s.kind), Some(SymbolKind::Global));
    assert_eq!(
        map.symbol("g").map(|s| s.addr),
        Some(code.global("g").unwrap().addr)
    );

    let add1 = code.function("add1").unwrap();
    assert!(
        add1.mapped
            .iter()
            .any(|m| m.timing == StaticTiming::Exact(7) || m.text.contains("ADD")),
        "{:?}",
        add1.mapped
    );
    assert!(
        add1.mapped
            .iter()
            .any(|m| m.timing == StaticTiming::Exact(10)),
        "RET timing {:?}",
        add1.mapped.iter().map(|m| m.timing).collect::<Vec<_>>()
    );
    let plus = src.find("x + 1").unwrap() as u32;
    let hits = map.covering(result.sources.files()[0].id, plus);
    assert!(
        !hits.is_empty(),
        "source offset {plus} should map to generated ops"
    );
    let selected = map.select(result.sources.files()[0].id, plus).unwrap();
    assert_eq!(
        map.at_address(selected.address).map(|m| m.id),
        Some(selected.id)
    );
    assert_eq!(
        map.by_id(selected.id).map(|m| m.address),
        Some(selected.address)
    );

    assert!(
        map.no_code
            .iter()
            .any(|n| n.reason == NoCodeReason::Eliminated),
        "unused local should be no-code: {:?}",
        map.no_code
    );

    let other = compile(CompileInput {
        files: vec![SourceInput {
            name: "test.c80",
            text: src,
        }],
        origin: 0x9000,
        optimize: true,
        target: C80Target::GenericZ80,
    });
    assert!(!other.has_errors(), "{:?}", other.diagnostics);
    let other_map = other.map().unwrap();
    assert_eq!(other_map.identity.origin, 0x9000);
    let delta = other_map
        .symbol("add1")
        .unwrap()
        .addr
        .wrapping_sub(map.symbol("add1").unwrap().addr);
    assert_eq!(delta, 0x1000);
    assert_ne!(other_map.identity.source_hash, 0);
}

#[test]
fn synthetic_frame_and_inline_asm_spans() {
    let src = r#"
u8 frame(u16 x) {
    u8 slot;
    ptr<u8> p;
    p = &slot;
    *p = 1;
    return u8(x);
}
u8 inc_asm(u8 value) {
    u8 result;
    asm(in: a = value, out: a = result, clobber: flags) {
        inc a
    }
    return result;
}
"#;
    let result = compile_ok(src);
    let map = result.map().unwrap();
    let frame_fn = result.code.as_ref().unwrap().function("frame").unwrap();
    assert!(
        frame_fn
            .mapped
            .iter()
            .any(|m| m.synthetic && m.text.contains("IX")),
        "{:?}",
        frame_fn.mapped
    );
    let asm_off = src.find("inc a").unwrap() as u32;
    let file = result.sources.files()[0].id;
    let hits = map.covering(file, asm_off);
    assert!(
        hits.iter()
            .any(|m| m.text.to_ascii_lowercase().contains("inc")),
        "{hits:?}"
    );
}

#[test]
fn call_and_repeat_timing_are_not_false_totals() {
    let result = compile_ok(
        r#"
u8 id(u8 x) { return x; }
u8 wrap(u8 x) { return id(x); }
void copy(ptr<u8> d, ptr<u8> s, u16 n) {
    asm(in: hl = s, in: de = d, in: bc = n, clobber: hl, de, bc, flags, memory) {
        ldir
    }
}
"#,
    );
    let wrap = result.code.as_ref().unwrap().function("wrap").unwrap();
    let call = wrap
        .mapped
        .iter()
        .find(|m| m.text.contains("CALL"))
        .expect("call");
    assert_eq!(call.timing, StaticTiming::Exact(17));
    let map = result.map().unwrap();
    let cost = map.span_cost(&wrap.mapped.iter().collect::<Vec<_>>());
    assert!(cost.has_call, "{cost:?}");
    assert!(!cost.complete, "{cost:?}");

    let copy = result.code.as_ref().unwrap().function("copy").unwrap();
    assert!(
        copy.mapped
            .iter()
            .any(|m| matches!(m.timing, StaticTiming::Repeating { .. })),
        "{:?}",
        copy.mapped
            .iter()
            .map(|m| (m.text.clone(), m.timing))
            .collect::<Vec<_>>()
    );
    assert_eq!(copy.stack_provenance, StackProvenance::Proven);
}

#[test]
fn failed_compile_has_no_loadable_program() {
    let result = compile_source("bad.c80", "u8 f() { return; }");
    assert!(result.has_errors());
    assert!(result.code.is_none());
    assert!(result.map().is_none());
}

#[test]
fn unknown_asm_call_stack_is_not_proven() {
    let result = compile_ok(
        r#"
u8 ext() {
    u8 r;
    asm(out: a = r, clobber: flags, stack: 2) {
        call 0x9000
    }
    return r;
}
"#,
    );
    assert_eq!(
        result
            .code
            .as_ref()
            .unwrap()
            .function("ext")
            .unwrap()
            .stack_provenance,
        StackProvenance::Declared
    );
    let unknown = compile_ok(
        r#"
u8 ext() {
    u8 r;
    asm(out: a = r, clobber: flags) {
        call 0x9000
    }
    return r;
}
"#,
    );
    assert_eq!(
        unknown
            .code
            .as_ref()
            .unwrap()
            .function("ext")
            .unwrap()
            .stack_provenance,
        StackProvenance::Unknown
    );
}

fn global_bytes<'a>(result: &'a CompilationResult, name: &str) -> &'a [u8] {
    let code = result.code.as_ref().unwrap();
    let g = code.global(name).unwrap();
    super::z80::bytes_in_segments(&code.assembled, g.addr, g.size as usize)
        .unwrap_or_else(|| panic!("missing bytes for global {name}"))
}

#[test]
fn packed_struct_layout_static_data_and_stride() {
    let result = compile_ok(
        r#"
struct Sprite { u8 x; u8 y; u16 bitmap; bool visible; };
Sprite one = { 10, 20, 0x1234, true };
Sprite enemies[2] = { { 1 }, { 2, 3 } };
void step(ptr<Sprite> p, u8 n) {
    u8 i = 0;
    while (i < n) {
        p += 1;
        i = i + 1;
    }
    p->x = 99;
}
void set_second_x(u8 v) { enemies[1].x = v; }
u8 get_second_x() { return enemies[1].x; }
u16 size() { return sizeof(Sprite); }
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(global_bytes(&result, "one"), [10, 20, 0x34, 0x12, 1]);
    assert_eq!(
        global_bytes(&result, "enemies"),
        [1, 0, 0, 0, 0, 2, 3, 0, 0, 0]
    );
    assert_eq!(code.global("one").unwrap().size, 5);
    assert_eq!(code.global("enemies").unwrap().size, 10);
    let size = execute_function(code, "size", &[]).unwrap();
    assert_eq!(size.return_word(), 5);

    let base = code.global("enemies").unwrap().addr;
    let second = execute_function(code, "set_second_x", &[7]).unwrap();
    let writes: Vec<_> = second
        .data_accesses()
        .filter(|a| a.kind == AccessKind::DataWrite)
        .collect();
    assert!(
        writes
            .iter()
            .any(|a| a.addr == base.wrapping_add(5) && a.value == 7),
        "{writes:?}"
    );
    let got = execute_function_with(
        code,
        "get_second_x",
        &[],
        &ExecConfig {
            initial_mem: vec![(base.wrapping_add(5), 7)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(got.return_byte(), 7);
    let stepped = execute_function(code, "step", &[base, 1]).unwrap();
    let step_writes: Vec<_> = stepped
        .data_accesses()
        .filter(|a| a.kind == AccessKind::DataWrite)
        .collect();
    assert!(
        step_writes
            .iter()
            .any(|a| a.addr == base.wrapping_add(5) && a.value == 99),
        "{step_writes:?}"
    );
}

#[test]
fn pointer_struct_fields_and_move() {
    let result = compile_ok(
        r#"
struct Sprite { u8 x; u8 y; u16 bitmap; bool visible; };
Sprite enemies[8];
void move(ptr<Sprite> sprite, i8 dx) {
    sprite->x = sprite->x + u8(dx);
}
ptr<Sprite> player = ptr<Sprite>(0x9000);
u8 read_player() { return player->x; }
"#,
    );
    let code = result.code.as_ref().unwrap();
    let enemies = code.global("enemies").unwrap().addr;
    execute_function(code, "move", &[enemies, 3]).unwrap();
    let out = execute_function_with(
        code,
        "move",
        &[enemies, 3],
        &ExecConfig {
            initial_mem: vec![(enemies, 10)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    let writes: Vec<_> = out
        .data_accesses()
        .filter(|a| a.kind == AccessKind::DataWrite && a.addr == enemies)
        .collect();
    assert_eq!(writes.last().map(|a| a.value), Some(13), "{writes:?}");

    let peek = execute_function_with(
        code,
        "read_player",
        &[],
        &ExecConfig {
            initial_mem: vec![(0x9000, 42)],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(peek.return_byte(), 42);
}

#[test]
fn compound_and_inc_evaluate_dest_once() {
    let result = compile_ok(
        r#"
u8 i;
u8 a[4];
u8 bump() {
    i = 0;
    a[i++] += 5;
    return i;
}
u8 pre() {
    i = 1;
    return ++i;
}
u8 post() {
    i = 1;
    return i++;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    let a = code.global("a").unwrap().addr;
    let bump = execute_function(code, "bump", &[]).unwrap();
    assert_eq!(bump.return_byte(), 1);
    let writes: Vec<_> = bump
        .data_accesses()
        .filter(|w| w.kind == AccessKind::DataWrite && w.addr == a)
        .collect();
    assert_eq!(writes.last().map(|w| w.value), Some(5), "{writes:?}");
    assert_eq!(execute_function(code, "pre", &[]).unwrap().return_byte(), 2);
    assert_eq!(
        execute_function(code, "post", &[]).unwrap().return_byte(),
        1
    );
}

#[test]
fn for_continue_runs_update_and_do_while_reaches_condition() {
    let result = compile_ok(
        r#"
u8 n;
u8 for_cont() {
    n = 0;
    for (u8 i = 0; i < 3; i += 1) {
        if (i == 1) continue;
        n = n + 1;
    }
    return n;
}
u8 do_cont() {
    u8 i = 0;
    n = 0;
    do {
        i = i + 1;
        if (i == 1) continue;
        n = n + 1;
    } while (i < 3);
    return n;
}
u8 nested_break() {
    n = 0;
    u8 i = 0;
    while (i < 3) {
        u8 j = 0;
        while (j < 3) {
            if (j == 1) break;
            n = n + 1;
            j = j + 1;
        }
        i = i + 1;
    }
    return n;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "for_cont", &[])
            .unwrap()
            .return_byte(),
        2
    );
    assert_eq!(
        execute_function(code, "do_cont", &[])
            .unwrap()
            .return_byte(),
        2
    );
    assert_eq!(
        execute_function(code, "nested_break", &[])
            .unwrap()
            .return_byte(),
        3
    );
}

const OPT_COMPARE_SRC: &str = r#"
u8 g;
u8 count(u8 n) {
    u8 s = 0;
    while (n != 0) {
        s = s + 1;
        n = n - 1;
    }
    return s;
}
void writes() { g = 1; g = 2; g = 3; }
u8 poll() { u8 a = g; u8 b = g; return a + b; }
u8 nest(bool a, bool b) {
    if (a) {
        if (b) { return 1; }
        return 2;
    }
    return 3;
}
"#;

fn compile_opt(src: &str, optimize: bool) -> CompilationResult {
    compile(CompileInput {
        files: vec![SourceInput {
            name: "test.c80",
            text: src,
        }],
        origin: DEFAULT_CODE_ORIGIN,
        optimize,
        target: C80Target::GenericZ80,
    })
}

fn exec_ok(
    code: &crate::z80::GeneratedProgram,
    name: &str,
    args: &[u16],
) -> crate::harness::ExecResult {
    execute_function(code, name, args).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn source_effects(exec: &crate::harness::ExecResult) -> Vec<(AccessKind, u16, u8)> {
    exec.data_accesses()
        .map(|a| (a.kind, a.addr, a.value))
        .collect()
}

#[test]
fn short_branches_become_jr_long_stay_jp() {
    let short = compile_ok("u8 f(u8 x) { if (x == 0) { return 1; } return 2; }");
    let short_text = assembly_of(&short).to_ascii_uppercase();
    assert!(
        short_text.contains("JR"),
        "in-range branch should shorten:\n{short_text}"
    );
    assert!(
        !short_text.contains("DJNZ"),
        "DJNZ is not proven for v1:\n{short_text}"
    );
    let code = short.code.as_ref().unwrap();
    assert_eq!(exec_ok(code, "f", &[0]).return_byte(), 1);
    assert_eq!(exec_ok(code, "f", &[1]).return_byte(), 2);
    for mapped in &code.function("f").unwrap().mapped {
        if mapped.text.to_ascii_uppercase().starts_with("JR") {
            assert_eq!(
                mapped.bytes.len(),
                2,
                "JR must be 2 bytes: {:?}",
                mapped.bytes
            );
            let disp = mapped.bytes[1] as i8;
            assert!(
                (-128..=127).contains(&disp),
                "JR displacement out of range: {disp}"
            );
        }
    }

    let mut body = String::from("u8 g;\nu8 far(bool c) {\n    if (c) {\n");
    for _ in 0..80 {
        body.push_str("        g = g + 1;\n");
    }
    body.push_str("    }\n    return g;\n}\n");
    let far = compile_ok(&body);
    let far_text = assembly_of(&far).to_ascii_uppercase();
    assert!(
        far_text.contains("JP"),
        "far skip must remain JP:\n{far_text}"
    );
}

#[test]
fn baseline_and_optimized_match_effects_and_improve_cost() {
    let base = compile_opt(OPT_COMPARE_SRC, false);
    let opt = compile_opt(OPT_COMPARE_SRC, true);
    assert!(!base.has_errors(), "{:?}", base.diagnostics);
    assert!(!opt.has_errors(), "{:?}", opt.diagnostics);
    let base_code = base.code.as_ref().unwrap();
    let opt_code = opt.code.as_ref().unwrap();
    let g = global_addr(base_code, "g");
    assert_eq!(g, global_addr(opt_code, "g"));

    for n in [0u16, 1, 5, 255] {
        let b = exec_ok(base_code, "count", &[n]);
        let o = exec_ok(opt_code, "count", &[n]);
        assert_eq!(b.return_byte(), o.return_byte(), "count({n})");
        assert_eq!(b.sp, o.sp, "count({n}) SP");
        assert_eq!(source_effects(&b), source_effects(&o), "count({n}) effects");
        assert!(
            o.tstates <= b.tstates,
            "count({n}) T-states grew: {} -> {}",
            b.tstates,
            o.tstates
        );
    }
    let zero = exec_ok(opt_code, "count", &[0]);
    let one = exec_ok(opt_code, "count", &[1]);
    assert_eq!(zero.return_byte(), 0);
    assert!(
        zero.insns < 30,
        "n=0 must not wrap into 256 iterations: insns={}",
        zero.insns
    );
    assert!(zero.tstates < one.tstates);

    let bw = exec_ok(base_code, "writes", &[]);
    let ow = exec_ok(opt_code, "writes", &[]);
    assert_eq!(source_effects(&bw), source_effects(&ow));
    let g_writes: Vec<u8> = ow
        .data_accesses()
        .filter(|a| a.kind == AccessKind::DataWrite && a.addr == g)
        .map(|a| a.value)
        .collect();
    assert_eq!(g_writes, [1, 2, 3]);

    let cfg = ExecConfig {
        scripted_reads: vec![(g, vec![3, 5])],
        ..ExecConfig::default()
    };
    let bp = execute_function_with(base_code, "poll", &[], &cfg).unwrap();
    let op = execute_function_with(opt_code, "poll", &[], &cfg).unwrap();
    assert_eq!(bp.return_byte(), 8);
    assert_eq!(op.return_byte(), 8);
    assert_eq!(source_effects(&bp), source_effects(&op));

    for args in [[1u16, 1], [1, 0], [0, 1]] {
        let b = exec_ok(base_code, "nest", &args);
        let o = exec_ok(opt_code, "nest", &args);
        assert_eq!(b.return_byte(), o.return_byte(), "nest{args:?}");
        assert_eq!(b.sp, o.sp);
        assert_eq!(source_effects(&b), source_effects(&o));
    }

    let base_count = base_code.function("count").unwrap().size;
    let opt_count = opt_code.function("count").unwrap().size;
    let b5 = exec_ok(base_code, "count", &[5]);
    let o5 = exec_ok(opt_code, "count", &[5]);
    eprintln!(
        "count quality: {} -> {} bytes, 5-iter {} -> {} T-states",
        base_count, opt_count, b5.tstates, o5.tstates
    );
    assert!(
        opt_count <= base_count,
        "count grew: {base_count} -> {opt_count}"
    );
    assert!(
        !assembly_of(&opt).to_ascii_uppercase().contains("DJNZ"),
        "{}",
        assembly_of(&opt)
    );

    let base_map = base.map().unwrap();
    let opt_map = opt.map().unwrap();
    assert!(base_map.symbol("count").is_some());
    assert!(opt_map.symbol("count").is_some());
    assert_eq!(base_map.identity.origin, opt_map.identity.origin);
    for mapped in &opt_code.function("count").unwrap().mapped {
        assert!(
            mapped.statement_span.is_some(),
            "optimized insn lost span: {}",
            mapped.text
        );
    }
}

#[test]
fn compile_latency_fixture_completes() {
    let src = r#"
struct Sprite { u8 x; u8 y; };
Sprite cells[8];
u8 acc;
void fill(ptr<u8> p, u8 n, u8 v) {
    while (n != 0) {
        *p = v;
        p = p + 1;
        n = n - 1;
    }
}
void step(ptr<Sprite> p, u8 n) {
    while (n != 0) {
        p->x = p->x + 1;
        p = p + 1;
        n = n - 1;
    }
}
u8 walk(u8 n) {
    u8 s = 0;
    for (u8 i = 0; i < n; i = i + 1) {
        s = s + 1;
    }
    return s;
}
u8 choose(u8 x) {
    if (x == 0) { return 1; }
    if (x == 1) { return 2; }
    return 3;
}
"#;
    let start = std::time::Instant::now();
    const RUNS: u32 = 20;
    for _ in 0..RUNS {
        let result = compile_ok(src);
        let code = result.code.as_ref().unwrap();
        assert_eq!(exec_ok(code, "choose", &[0]).return_byte(), 1);
        assert_eq!(exec_ok(code, "choose", &[2]).return_byte(), 3);
        assert_eq!(exec_ok(code, "walk", &[0]).return_byte(), 0);
        assert_eq!(exec_ok(code, "walk", &[1]).return_byte(), 1);
    }
    let elapsed = start.elapsed();
    eprintln!(
        "compile_latency_fixture: {RUNS} compiles in {} ms on {}",
        elapsed.as_millis(),
        std::env::consts::OS
    );
    assert!(
        elapsed.as_secs() < 30,
        "compile latency fixture exceeded 30s: {elapsed:?}"
    );
}

fn compile_target(src: &str, target: C80Target) -> CompilationResult {
    let result = compile(CompileInput {
        files: vec![SourceInput {
            name: "test.c80",
            text: src,
        }],
        origin: DEFAULT_CODE_ORIGIN,
        optimize: true,
        target,
    });
    assert!(!result.has_errors(), "{:?}\n{src}", result.diagnostics);
    result
}

fn port_trace(exec: &crate::harness::ExecResult) -> Vec<(AccessKind, u16, u8, u8)> {
    exec.data_accesses()
        .filter(|a| matches!(a.kind, AccessKind::PortIn | AccessKind::PortOut))
        .map(|a| (a.kind, a.addr, a.value, a.high))
        .collect()
}

#[test]
fn port_io_tvc_immediate_and_dynamic() {
    let src = r#"
void imm_out() { cpu::out(0x06, 0x80); }
u8 imm_in() { return cpu::in(0x58); }
u8 dyn(u8 port, u8 value) {
    cpu::out(port, value);
    return cpu::in(port);
}
u8 nested(u8 p, u8 q) {
    cpu::out(p, cpu::in(q));
    return 0;
}
void disc() { cpu::in(0x12); }
void twice() { cpu::out(1, 1); cpu::out(1, 1); }
u8 keep(u8 x) { cpu::out(6, 1); return x; }
bool skip(bool cond) { return cond && (cpu::in(1) != 0); }
u8 poll() {
    u8 v;
    v = cpu::in(0x59);
    while ((v & 0x10) != 0) {
        v = cpu::in(0x59);
    }
    return v;
}
"#;
    let tvc = compile_target(src, C80Target::Tvc);
    let asm = assembly_of(&tvc).to_ascii_uppercase();
    assert!(asm.contains("OUT (6),A"), "{asm}");
    assert!(asm.contains("IN A,(88)"), "{asm}");
    assert!(
        !tvc.code
            .as_ref()
            .unwrap()
            .function("imm_out")
            .unwrap()
            .mapped
            .iter()
            .any(|m| m.text.to_ascii_uppercase().contains("OUT (C)")),
        "{asm}"
    );
    assert_eq!(func_bytes(&tvc, "imm_out"), [0x3E, 0x80, 0xD3, 0x06, 0xC9]);
    assert_eq!(func_bytes(&tvc, "imm_in"), [0xDB, 0x58, 0xC9]);
    let mapped = tvc.code.as_ref().unwrap().function("imm_out").unwrap();
    assert!(
        mapped
            .mapped
            .iter()
            .any(|m| m.text.to_ascii_uppercase().contains("OUT (6),A")
                && m.t_states.is_some()
                && m.statement_span.is_some()),
        "{:?}",
        mapped.mapped
    );

    let code = tvc.code.as_ref().unwrap();
    let imm = execute_function(code, "imm_out", &[]).unwrap();
    assert!(
        port_trace(&imm)
            .iter()
            .any(|a| a.0 == AccessKind::PortOut && a.1 == 6 && a.2 == 0x80),
        "{:?}",
        imm.accesses
    );

    let din = execute_function_with(
        code,
        "imm_in",
        &[],
        &ExecConfig {
            scripted_ports: vec![(0x58, vec![0xA5])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(din.return_byte(), 0xA5);

    let dynamic = execute_function_with(
        code,
        "dyn",
        &[0x12, 7],
        &ExecConfig {
            scripted_ports: vec![(0x12, vec![42])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(dynamic.return_byte(), 42);
    let ports = port_trace(&dynamic);
    assert!(
        ports
            .iter()
            .any(|a| a.0 == AccessKind::PortOut && a.1 == 0x12 && a.2 == 7),
        "{ports:?}"
    );
    assert!(
        ports
            .iter()
            .any(|a| a.0 == AccessKind::PortIn && a.1 == 0x12 && a.2 == 42),
        "{ports:?}"
    );

    let nested = execute_function_with(
        code,
        "nested",
        &[0x20, 0x21],
        &ExecConfig {
            scripted_ports: vec![(0x21, vec![9])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    let nports = port_trace(&nested);
    assert_eq!(nports[0], (AccessKind::PortIn, 0x21, 9, nports[0].3));
    assert_eq!(nports[1], (AccessKind::PortOut, 0x20, 9, nports[1].3));

    let disc = execute_function_with(
        code,
        "disc",
        &[],
        &ExecConfig {
            scripted_ports: vec![(0x12, vec![1])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert!(
        port_trace(&disc)
            .iter()
            .any(|a| a.0 == AccessKind::PortIn && a.1 == 0x12),
        "{:?}",
        disc.accesses
    );

    let twice = execute_function(code, "twice", &[]).unwrap();
    let tports: Vec<_> = port_trace(&twice)
        .into_iter()
        .filter(|a| a.0 == AccessKind::PortOut)
        .collect();
    assert_eq!(tports.len(), 2, "{tports:?}");

    let keep = execute_function(code, "keep", &[42]).unwrap();
    assert_eq!(keep.return_byte(), 42);

    let skipped = execute_function(code, "skip", &[0]).unwrap();
    assert!(port_trace(&skipped).is_empty(), "{:?}", skipped.accesses);
    let taken = execute_function_with(
        code,
        "skip",
        &[1],
        &ExecConfig {
            scripted_ports: vec![(1, vec![0])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert!(
        port_trace(&taken)
            .iter()
            .any(|a| a.0 == AccessKind::PortIn && a.1 == 1),
        "{:?}",
        taken.accesses
    );

    let polled = execute_function_with(
        code,
        "poll",
        &[],
        &ExecConfig {
            scripted_ports: vec![(0x59, vec![0x10, 0x10, 0x00])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(polled.return_byte(), 0);
    assert_eq!(
        port_trace(&polled)
            .iter()
            .filter(|a| a.0 == AccessKind::PortIn)
            .count(),
        3
    );
}

#[test]
fn port_io_generic_z80_zero_extends_bc() {
    let src = "void f() { cpu::out(0x12, 7); }\nu8 g(u8 p) { return cpu::in(p); }\n";
    let result = compile_target(src, C80Target::GenericZ80);
    let asm = assembly_of(&result).to_ascii_uppercase();
    assert!(asm.contains("OUT (C),A"), "{asm}");
    assert!(asm.contains("IN A,(C)"), "{asm}");
    assert!(!asm.contains("OUT (18),A"), "{asm}");
    let code = result.code.as_ref().unwrap();
    let out = execute_function(code, "f", &[]).unwrap();
    let ports = port_trace(&out);
    assert_eq!(ports.len(), 1, "{ports:?}");
    assert_eq!(ports[0].0, AccessKind::PortOut);
    assert_eq!(ports[0].1, 0x12);
    assert_eq!(ports[0].2, 7);
    assert_eq!(ports[0].3, 0, "high address byte must be zero: {ports:?}");

    let inn = execute_function_with(
        code,
        "g",
        &[0x34],
        &ExecConfig {
            scripted_ports: vec![(0x34, vec![0x55])],
            ..ExecConfig::default()
        },
    )
    .unwrap();
    assert_eq!(inn.return_byte(), 0x55);
    let ip = port_trace(&inn);
    assert_eq!(ip[0].3, 0, "IN A,(C) high byte is B: {ip:?}");
}

#[test]
fn port_io_optimized_matches_baseline_trace() {
    let src = r#"
u8 g;
void writes() {
    cpu::out(1, 1);
    g = 2;
    cpu::out(1, 3);
}
u8 poll() {
    u8 a = cpu::in(2);
    u8 b = cpu::in(2);
    return a + b;
}
"#;
    let base = compile(CompileInput {
        files: vec![SourceInput {
            name: "test.c80",
            text: src,
        }],
        origin: DEFAULT_CODE_ORIGIN,
        optimize: false,
        target: C80Target::Tvc,
    });
    let opt = compile(CompileInput {
        files: vec![SourceInput {
            name: "test.c80",
            text: src,
        }],
        origin: DEFAULT_CODE_ORIGIN,
        optimize: true,
        target: C80Target::Tvc,
    });
    assert!(!base.has_errors(), "{:?}", base.diagnostics);
    assert!(!opt.has_errors(), "{:?}", opt.diagnostics);
    let cfg = ExecConfig {
        scripted_ports: vec![(2, vec![3, 5])],
        ..ExecConfig::default()
    };
    let bw = execute_function(base.code.as_ref().unwrap(), "writes", &[]).unwrap();
    let ow = execute_function(opt.code.as_ref().unwrap(), "writes", &[]).unwrap();
    assert_eq!(port_trace(&bw), port_trace(&ow));
    assert_eq!(source_effects(&bw), source_effects(&ow));
    let bp = execute_function_with(base.code.as_ref().unwrap(), "poll", &[], &cfg).unwrap();
    let op = execute_function_with(opt.code.as_ref().unwrap(), "poll", &[], &cfg).unwrap();
    assert_eq!(bp.return_byte(), 8);
    assert_eq!(op.return_byte(), 8);
    assert_eq!(port_trace(&bp), port_trace(&op));
}

#[test]
fn cpu_di_ei_ldir_emit_and_copy() {
    let ints = compile_ok("void ints() { cpu::di(); cpu::ei(); }\n");
    assert_eq!(func_bytes(&ints, "ints"), [0xF3, 0xFB, 0xC9]);

    let result = compile_ok(
        r#"
u8 src[4];
u8 dst[4];
u8 buf[4];
void copy(u16 n) {
    src[0] = 1;
    src[1] = 2;
    src[2] = 3;
    src[3] = 4;
    cpu::ldir(&src[0], &dst[0], n);
}
u8 get(u8 i) { return dst[i]; }
u8 copy_get(u16 n, u8 i) {
    copy(n);
    return get(i);
}
void fill() {
    buf[0] = 9;
    buf[1] = 0;
    buf[2] = 0;
    buf[3] = 0;
    cpu::ldir(&buf[0], &buf[1], 3);
}
u8 fill_get(u8 i) {
    fill();
    return buf[i];
}
"#,
    );
    let text = assembly_of(&result).to_ascii_uppercase();
    assert!(text.contains("LDIR"), "{text}");
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "copy_get", &[1, 0])
            .unwrap()
            .return_byte(),
        1
    );
    assert_eq!(
        execute_function(code, "copy_get", &[3, 2])
            .unwrap()
            .return_byte(),
        3
    );
    assert_eq!(
        execute_function(code, "fill_get", &[3])
            .unwrap()
            .return_byte(),
        9,
        "HL is source and DE is dest (Z80 LDIR order)"
    );
    let zero = execute_function_with(
        code,
        "copy",
        &[0],
        &ExecConfig {
            insn_limit: 200,
            ..ExecConfig::default()
        },
    )
    .expect("BC=0 LDIR may smash code and still return");
    let extra_writes = zero
        .accesses
        .iter()
        .filter(|a| a.kind == AccessKind::DataWrite)
        .count();
    assert!(
        extra_writes > 8,
        "BC=0 LDIR must copy many bytes, not skip: writes={extra_writes}"
    );
}

#[test]
fn call_second_byte_arg_can_depend_on_the_first() {
    let result = compile_ok(
        r#"
u8 gy;
u8 gn;
void take(u8 y, u8 n) {
    gy = y;
    gn = n;
}
u8 by;
u8 oy;
u8 n_up(u8 b, u8 o) {
    by = b;
    oy = o;
    take(by, oy - by);
    return gn;
}
u8 y_up(u8 b, u8 o) {
    by = b;
    oy = o;
    take(by, oy - by);
    return gy;
}
u8 n_up4(u8 b, u8 o) {
    by = b;
    oy = o;
    take(by + 4, oy - by);
    return gn;
}
u8 n_down(u8 b, u8 o) {
    by = b;
    oy = o;
    take(oy, by - oy);
    return gn;
}
"#,
    );
    let code = result.code.as_ref().unwrap();
    assert_eq!(
        execute_function(code, "n_up", &[117, 118])
            .unwrap()
            .return_byte(),
        1,
        "take(by, oy - by) must pass the difference, not by"
    );
    assert_eq!(
        execute_function(code, "y_up", &[117, 118])
            .unwrap()
            .return_byte(),
        117
    );
    assert_eq!(
        execute_function(code, "n_up4", &[117, 118])
            .unwrap()
            .return_byte(),
        1,
        "take(by + 4, oy - by) must not reuse by + 4 as the count"
    );
    assert_eq!(
        execute_function(code, "n_down", &[118, 117])
            .unwrap()
            .return_byte(),
        1
    );
}
