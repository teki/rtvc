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

fn assert_ix_iy_sp(exec: &crate::compiler::harness::ExecResult) {
    assert_eq!(exec.sp, crate::compiler::harness::DEFAULT_SP);
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
