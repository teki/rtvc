use super::*;
use crate::abi::{assign_params, return_home};
use crate::harness::{AccessKind, ExecConfig, execute_function, execute_function_with};
use rtvc_core::asm::assemble_line;
use rtvc_core::bus::FakeBus;
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
    // H5: the loop test is `OR A`, not `CP 0` (48 -> 45), and the loop is
    // rotated: entry `JR` to the test, one `JP NZ` per pass. That costs
    // 10 T for zero passes and 3 T for one, and saves 7 T on every further
    // pass (45 -> 55, 78 -> 81, 210 -> 185).
    assert_eq!(zero.tstates, 55);
    // H3i: `s + 1` / `n - 1` stored back are `INC`/`DEC` in place (was
    // 106 / 338 through A).
    assert_eq!(one.tstates, 81); // H5: 84 before `OR A`, 78 before rotation
    assert_eq!(many.tstates, 185); // H5: 228 before `OR A`, 210 before rotation
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
@noinline u8 id(u8 x) { return x; }
@noinline u8 add8(u8 a, u8 b) { return a + b; }
@noinline u16 add(u16 a, u16 b) { return a + b; }
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
    // Parameter 62 of a `@stackcall` function sits at IX+128.
    let params: Vec<String> = (0..63).map(|i| format!("u16 p{i}")).collect();
    let src = format!(
        "@stackcall u16 f({}) {{ return p62; }}\n",
        params.join(", ")
    );
    let result = compile_source("test.c80", &src);
    assert!(result.has_errors(), "expected frame displacement error");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("does not fit a signed displacement")),
        "{:?}",
        result.diagnostics
    );
    assert!(result.code.is_none());
}

#[test]
fn many_word_locals_use_static_slots_not_ix() {
    // H1 rejected this (70 word locals overflowed the IX frame); since H2a
    // the locals that do not fit registers live in static slots.
    let mut src = String::from("u8 id(u8 x) { return x; }\nu8 f() {\n");
    for i in 0..70 {
        src.push_str(&format!("  u16 v{i} = {i};\n"));
    }
    src.push_str("  u8 t = id(1);\n");
    for i in 0..70 {
        src.push_str(&format!("  t = t + u8(v{i});\n"));
    }
    src.push_str("  return t;\n}\n");
    let result = compile_ok(&src);
    let code = result.code.as_ref().unwrap();
    let f = code.function("f").unwrap();
    assert!(f.spill.slot_bytes > 0, "{:?}", f.spill);
    assert!(
        f.mapped.iter().all(|m| !m.text.contains("IX")),
        "unannotated function uses IX"
    );
    let exec = execute_function(code, "f", &[]).unwrap();
    // 1 + (0 + 1 + ... + 69) = 2416, truncated to u8.
    assert_eq!(exec.return_byte(), (2416u16 & 0xFF) as u8);
    assert_ix_iy_sp(&exec);
}

#[test]
fn spills_reserve_exactly_their_slots() {
    // Nine bytes of live values in a loop: the slot area holds exactly the
    // values without a register, and the size report includes it.
    let src = "u16 mix(u16 a, u16 b, u8 n) { u16 s = 0; u16 t = 1;
        while (n != 0) { s = s + a; t = t + b; a = a ^ t; n--; } return s + t; }";
    let result = compile_ok(src);
    let code = result.code.as_ref().unwrap();
    let f = code.function("mix").unwrap();
    assert_eq!(f.spill.slot_bytes, 5, "{:?}", f.spill);
    let data: Vec<_> = f
        .mapped
        .iter()
        .filter(|m| m.kind == MappedKind::Data)
        .collect();
    assert_eq!(data.len(), 1, "{data:?}");
    assert!(data[0].synthetic);
    assert_eq!(data[0].bytes, vec![0; 5]);
    let code_bytes: usize = f.mapped.iter().map(|m| m.bytes.len()).sum();
    assert_eq!(usize::from(f.size), code_bytes);
    assert!(result.code.as_ref().unwrap().assembly.contains("; spill"));
    let exec = execute_function(code, "mix", &[1, 2, 3]).unwrap();
    assert_eq!(exec.return_word(), 0x11);
    assert_ix_iy_sp(&exec);

    // A value live across a call is pushed, not stored: no slot area.
    let result = compile_ok(
        "pub u8 twice(u8 v) { return v + v; }
         u8 keep(u8 x, u8 y) { u8 a = twice(x); return a + y; }",
    );
    let code = result.code.as_ref().unwrap();
    let keep = code.function("keep").unwrap();
    assert_eq!(keep.spill.slot_bytes, 0);
    assert_eq!(keep.spill.saves, vec![2]);
    let exec = execute_function(code, "keep", &[5, 1]).unwrap();
    assert_eq!(exec.return_byte(), 11);
    assert!(exec.sp_used <= keep.stack_bound);
    assert_eq!(keep.stack_provenance, StackProvenance::Proven);
}

#[test]
fn stackcall_arguments_spill_when_pairs_run_out() {
    let result = compile_ok(
        "@stackcall u16 g(u16 a, u16 b, u16 c, u16 d) { return a - b + c - d; }
         u16 f(u16 x, u16 y) { u16 z = x + y; u16 w = x ^ y;
             return g(x + 1, y + 2, z + 3, w + 4) + z + w; }",
    );
    let code = result.code.as_ref().unwrap();
    let f = code.function("f").unwrap();
    assert!(f.mapped.iter().all(|m| !m.text.contains("IX")));
    let exec = execute_function(code, "f", &[10, 3]).unwrap();
    // z = 13, w = 9: g(11, 5, 16, 13) = 9, plus z + w.
    assert_eq!(exec.return_word(), 31);
    assert!(
        exec.sp_used <= f.stack_bound,
        "{} > {}",
        exec.sp_used,
        f.stack_bound
    );
    assert_ix_iy_sp(&exec);
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
@noinline void copy(u16 n) {
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
    );
    // BC=0 copies 64 KiB over the code, so whether the smashed function
    // still reaches its RET depends on where the code happens to sit (it
    // stopped returning when H5 shortened the listing). Either outcome
    // shows the LDIR ran: a skipped one returns after the four stores.
    match zero {
        Ok(zero) => {
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
        Err(e) => assert!(e.0.contains("timed out"), "{e}"),
    }
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
            // H5: `x + 1` before RET is `INC A` (carry is dead there).
            .any(|m| m.timing == StaticTiming::Exact(4) && m.text == "INC A"),
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
    // `&slot` is a link-time address: the local lives in the function's
    // static slot area, which the map shows as synthetic data.
    assert!(
        frame_fn
            .mapped
            .iter()
            .any(|m| m.synthetic && m.text.starts_with("DB ")),
        "{:?}",
        frame_fn.mapped
    );
    assert!(
        frame_fn.mapped.iter().all(|m| !m.text.contains("IX")),
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
pub u8 id(u8 x) { return x; }
u8 wrap(u8 x) { return id(x) + 1; }
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
@noinline void copy(u16 n) {
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
    );
    // BC=0 copies 64 KiB over the code, so whether the smashed function
    // still reaches its RET depends on where the code happens to sit (it
    // stopped returning when H5 shortened the listing). Either outcome
    // shows the LDIR ran: a skipped one returns after the four stores.
    match zero {
        Ok(zero) => {
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
        Err(e) => assert!(e.0.contains("timed out"), "{e}"),
    }
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

// ---------------------------------------------------------------------------
// H2b: strict `@fastcall`, pinned registers, `static` locals
// ---------------------------------------------------------------------------

fn errors_of(src: &str) -> Vec<String> {
    let result = compile_source("test.c80", src);
    result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| format!("{}: {}", d.code.as_str(), d.message))
        .collect()
}

fn run_both(src: &str, name: &str, args: &[u16]) -> Vec<crate::harness::ExecResult> {
    [true, false]
        .into_iter()
        .map(|optimize| {
            let result = compile_opt(src, optimize);
            assert!(!result.has_errors(), "{:?}\n{src}", result.diagnostics);
            let code = result.code.as_ref().unwrap();
            let exec = execute_function(code, name, args).unwrap();
            assert_ix_iy_sp(&exec);
            exec
        })
        .collect()
}

fn listing_of(src: &str, name: &str) -> Vec<String> {
    let result = compile_ok(src);
    result
        .code
        .as_ref()
        .unwrap()
        .function(name)
        .unwrap()
        .mapped
        .iter()
        .map(|m| m.text.clone())
        .collect()
}

#[test]
fn pinned_counter_stays_in_b_through_the_loop() {
    let src = "u8 f(u8 x) { u8 n @ b = 4; u8 s = 0; while (n != 0) { s = s + x; n--; } return s; }";
    for exec in run_both(src, "f", &[3]) {
        assert_eq!(exec.return_byte(), 12);
    }
    let text = listing_of(src, "f");
    // H5: the source's `n != 0` guard runs once at the entry (`OR A`, was
    // `CP 0`), and `n--` closing the loop is `DJNZ` (A and flags are dead
    // there).
    let want = [
        "LD C,A",
        "LD B,4",
        "LD E,0",
        "LD A,B",
        "OR A",
        "JR Z,FN23_F_B4",
        "LD A,E",
        "ADD A,C",
        "LD E,A",
        "DJNZ FN23_F_B3",
        "LD A,E",
        "RET",
    ];
    assert_eq!(text, want, "{text:#?}");
    // Nothing but the counter's own updates writes B.
    let writes_b: Vec<&String> = text.iter().filter(|t| t.starts_with("LD B,")).collect();
    assert_eq!(writes_b, ["LD B,4"], "{text:#?}");
    assert!(text.iter().any(|t| t.starts_with("DJNZ ")), "{text:#?}");
}

#[test]
fn pinned_parameter_gets_one_entry_move() {
    let src = "u8 f(u8 x @ d, u8 y) { return x + y; }";
    for exec in run_both(src, "f", &[40, 2]) {
        assert_eq!(exec.return_byte(), 42);
    }
    let text = listing_of(src, "f");
    // H5: `x` is only read once, straight from A, so the optimizer drops the
    // `LD A,D` reload and then the dead entry move itself. Never more than one.
    assert_eq!(text, ["ADD A,C", "RET"], "{text:#?}");
    let unopt = compile_opt(src, false);
    let moves = unopt
        .code
        .as_ref()
        .unwrap()
        .function("f")
        .unwrap()
        .mapped
        .iter()
        .filter(|m| m.text == "LD D,A")
        .count();
    assert_eq!(moves, 1);
    let src = "@stackcall u8 g(u8 a, u8 b @ c) { return a + b; }";
    for exec in run_both(src, "g", &[40, 2]) {
        assert_eq!(exec.return_byte(), 42);
    }
}

#[test]
fn pin_conflicts_are_located_errors() {
    let alu = errors_of("u8 f(u8 x, u8 y) { u8 n @ a = x; u8 r = y + 1; return r + n; }");
    assert!(
        alu.iter()
            .any(|e| e.contains("'n' is pinned to a") && e.contains("the u8 `+`")),
        "{alu:?}"
    );
    let overlap = errors_of("u8 f() { u16 w @ bc = 1; u8 n @ c = 2; return n + u8(w); }");
    assert!(
        overlap.iter().any(|e| e.contains("overlaps 'w @ bc'")),
        "{overlap:?}"
    );
    let addr = errors_of("u8 f() { u8 n @ b = 1; ptr<u8> p = &n; return *p; }");
    assert!(
        addr.iter().any(|e| e.contains("pinned to register b")),
        "{addr:?}"
    );
    let width = errors_of("u16 f() { u16 w @ b = 1; return w; }");
    assert!(
        width.iter().any(|e| e.contains("a byte register")),
        "{width:?}"
    );
    let unknown = errors_of("u8 f() { u8 n @ x = 1; return n; }");
    assert!(
        unknown.iter().any(|e| e.contains("unknown register 'x'")),
        "{unknown:?}"
    );
    let global = errors_of("u8 g @ b;\nu8 f() { return g; }");
    assert!(
        global
            .iter()
            .any(|e| e.contains("only to a local or a parameter")),
        "{global:?}"
    );
}

#[test]
fn pins_in_disjoint_scopes_may_share_a_register() {
    let src = "u8 f(u8 x) { u8 s = 0; { u8 n @ b = x; s = s + n; } { u8 m @ b = 2; s = s + m; } return s; }";
    for exec in run_both(src, "f", &[5]) {
        assert_eq!(exec.return_byte(), 7);
    }
}

#[test]
fn pinned_value_across_a_call_is_pushed_or_rejected() {
    let plain =
        "pub u8 g(u8 v) { return v + 1; }\nu8 f(u8 x) { u8 n @ b = x; u8 r = g(1); return r + n; }";
    for exec in run_both(plain, "f", &[40]) {
        assert_eq!(exec.return_byte(), 42);
    }
    let text = listing_of(plain, "f");
    assert!(text.contains(&"PUSH BC".to_string()), "{text:#?}");
    let strict = format!(
        "@fastcall {}",
        plain
            .replace("u8 f(", "@fastcall u8 f(")
            .trim_start_matches("@fastcall ")
    );
    let errs = errors_of(&strict);
    assert!(
        errs.iter()
            .any(|e| e.contains("'n' is live in b across the call to 'g'")
                && e.contains("no register contract")
                && e.contains("declare @preserves(b) on 'g'")),
        "{errs:?}"
    );
}

#[test]
fn strict_fastcall_accepts_registers_and_rejects_fallbacks() {
    let ok =
        compile_ok("@fastcall u8 add(u8 a, u8 b) { return a + b; }\nu8 plain(u8 a) { return a; }");
    let code = ok.code.as_ref().unwrap();
    assert!(code.function("add").unwrap().strict_registers);
    assert!(!code.function("plain").unwrap().strict_registers);
    assert_eq!(code.function("add").unwrap().conv, CallConv::Register);

    let addr = errors_of("@fastcall u8 f(u8 v) { u8 c = 0; ptr<u8> p = &c; *p = v; return c; }");
    assert!(
        addr.iter().any(
            |e| e.starts_with("cg-unsupported: @fastcall 'f' must fit in registers")
                && e.contains("'c' has its address taken")
                && e.contains("'static'")
        ),
        "{addr:?}"
    );
    let stack =
        errors_of("@stackcall u8 s(u8 a) { return a; }\n@fastcall u8 f(u8 v) { return s(v); }");
    assert!(
        stack.iter().any(|e| e.contains("must fit in registers")),
        "{stack:?}"
    );
    // A static local is the strict way to get memory.
    let src = "@fastcall u8 f(u8 v) { static u8 c; ptr<u8> p = &c; *p = v; return c + 1; }";
    for exec in run_both(src, "f", &[41]) {
        assert_eq!(exec.return_byte(), 42);
    }
}

#[test]
fn static_locals_persist_initialize_index_and_address() {
    let src =
        "u8 tick() { static u8 n = 5; n++; return n; }\nu8 f() { tick(); tick(); return tick(); }";
    for exec in run_both(src, "f", &[]) {
        assert_eq!(exec.return_byte(), 8);
    }
    let src = "u8 f(u8 i) { static u8 t[4] = {10, 20, 30}; ptr<u8> p = &t[3]; *p = i; return t[i & 3] + t[0]; }";
    for exec in run_both(src, "f", &[1]) {
        assert_eq!(exec.return_byte(), 30);
    }
    for exec in run_both(src, "f", &[3]) {
        assert_eq!(exec.return_byte(), 13);
    }
    let result = compile_ok(src);
    let code = result.code.as_ref().unwrap();
    let g = code.global("f.t").expect("static array is reported as f.t");
    assert_eq!(g.size, 4);
    // Placed directly after the function.
    let f = code.function("f").unwrap();
    assert_eq!(g.addr, f.addr + f.size);
}

#[test]
fn static_local_rules_are_checked() {
    let auto = errors_of("u8 f() { u8 t[4]; t[0] = 1; return t[0]; }");
    assert!(
        auto.iter().any(|e| e.contains("declare them 'static'")),
        "{auto:?}"
    );
    let init = errors_of("u8 f(u8 x) { static u8 n = x; return n; }");
    assert!(
        init.iter().any(|e| e.contains("compile-time value")),
        "{init:?}"
    );
    let top = errors_of("static u8 g;\nu8 f() { return g; }");
    assert!(
        top.iter().any(|e| e.contains("'static' is for locals")),
        "{top:?}"
    );
    let for_init =
        errors_of("u8 f() { u8 s = 0; for (static u8 i = 0; i < 3; i++) { s++; } return s; }");
    assert!(
        for_init.iter().any(|e| e.contains("cannot be static")),
        "{for_init:?}"
    );
    let konst = errors_of("u8 f() { static const u8 n = 1; return n; }");
    assert!(
        konst.iter().any(|e| e.contains("'static const'")),
        "{konst:?}"
    );
    let pinned = errors_of("u8 f() { static u8 n @ b; return n; }");
    assert!(
        pinned.iter().any(|e| e.contains("cannot be pinned")),
        "{pinned:?}"
    );
}

// ---------------------------------------------------------------------------
// H2c: register contracts (`@preserves`, limited inference)
// ---------------------------------------------------------------------------

#[test]
fn strict_caller_keeps_its_counter_in_b_across_a_preserving_leaf() {
    let src = "@preserves(bc) pub u8 inc(u8 v) { return v + 1; }
@fastcall u8 count(u8 n @ b) { u8 s = 0; while (n != 0) { s = inc(s); n--; } return s; }";
    for exec in run_both(src, "count", &[5]) {
        assert_eq!(exec.return_byte(), 5);
    }
    let text = listing_of(src, "count");
    // H5: entry guard (`OR A`: A already holds n), then `DJNZ`.
    let want = [
        "LD B,A", "LD C,0", "OR A", "JR Z,F", "LD A,C", "CALL F", "LD C,A", "DJNZ F", "LD A,C",
        "RET",
    ];
    assert_eq!(text.len(), want.len(), "{text:#?}");
    for (got, want) in text.iter().zip(want) {
        assert!(got.starts_with(want), "{text:#?}");
    }
    let code = compile_ok(src);
    let inc = code.code.as_ref().unwrap().function("inc").unwrap();
    assert_eq!(inc.contract, crate::Contract::Declared);
    assert_eq!(inc.preserves, vec![R8::B, R8::C]);
}

#[test]
fn preserves_violations_are_located_errors() {
    // Directly: the word `+` needs HL.
    let direct = errors_of("u16 t;\n@preserves(hl) void acc(u16 a, u16 b) { t = a + b; }");
    assert!(
        direct.iter().any(|e| e.starts_with(
            "cg-unsupported: 'acc' declares @preserves(hl), but the u16 `+` needs hl"
        )),
        "{direct:?}"
    );
    // Through a callee without a contract, in a strict function.
    let callee = errors_of(
        "pub u8 twice(u8 v) { return v + v; }\n@fastcall @preserves(bc) u8 wrap(u8 v) { return twice(v) + 1; }",
    );
    assert!(
        callee.iter().any(|e| e.contains("the caller's bc (kept by @preserves) is live in bc across the call to 'twice', which has no register contract")),
        "{callee:?}"
    );
    // Through an inline asm clobber, in a strict function.
    let asm =
        errors_of("@fastcall @preserves(c) u8 f(u8 v) { asm (clobber: c) { LD C,0 } return v; }");
    assert!(
        asm.iter().any(|e| e.contains("asm block changes")),
        "{asm:?}"
    );
    // Declaration errors.
    let decl = errors_of(
        "@preserves(a) u8 f(u8 x) { return x; }\n@preserves(h) u16 g(u16 x) { return x; }\n@preserves(bc) u8 h(u8 x @ b) { return x; }",
    );
    assert!(
        decl.iter()
            .any(|e| e.contains("'f' returns u8 in a, so it cannot preserve a")),
        "{decl:?}"
    );
    assert!(
        decl.iter()
            .any(|e| e.contains("'g' returns u16 in hl, so it cannot preserve h")),
        "{decl:?}"
    );
    assert!(
        decl.iter()
            .any(|e| e.contains("'x' is pinned to b, but 'h' declares it in @preserves")),
        "{decl:?}"
    );
    let parse = errors_of(
        "@preserves(bc) @preserves(de) void k() { }\n@preserves() void m() { }\n@preserves(ix) void n() { }\n@preserves(bc) u8 g;",
    );
    assert_eq!(parse.len(), 4, "{parse:?}");
}

#[test]
fn default_functions_save_a_preserved_register_around_a_clobbering_call() {
    let src = "pub u8 twice(u8 v) { return v + v; }\n@preserves(bc) u8 wrap(u8 v) { return twice(v) + 1; }";
    for exec in run_both(src, "wrap", &[20]) {
        assert_eq!(exec.return_byte(), 41);
    }
    let text = listing_of(src, "wrap");
    assert_eq!(
        text,
        // H5: `+ 1` before RET is `INC A`.
        ["PUSH BC", "CALL FN7_TWICE", "POP BC", "INC A", "RET"],
        "{text:#?}"
    );
    // A parameter in a preserved register is read in place and comes back.
    let src = "@preserves(de) u16 add(u16 a, u16 b) { return a + b; }";
    assert_eq!(listing_of(src, "add"), ["ADD HL,DE", "RET"]);
    for exec in run_both(src, "add", &[0x1234, 0x0101]) {
        assert_eq!(exec.return_word(), 0x1335);
        assert_eq!(exec.de, 0x0101);
    }
}

#[test]
fn inference_is_used_for_private_callees_only() {
    let private = "@noinline u8 inc(u8 v) { return v + 1; }\nu8 f(u8 x, u8 y) { u8 a = inc(x); return a + y; }";
    let public = private.replace("u8 inc(", "pub u8 inc(");
    let p = compile_ok(private);
    let inc = p.code.as_ref().unwrap().function("inc").unwrap();
    assert_eq!(inc.contract, crate::Contract::Inferred);
    assert_eq!(
        inc.preserves,
        vec![R8::B, R8::C, R8::D, R8::E, R8::H, R8::L]
    );
    assert!(
        !listing_of(private, "f")
            .iter()
            .any(|l| l.starts_with("PUSH"))
    );
    let q = compile_ok(&public);
    let inc = q.code.as_ref().unwrap().function("inc").unwrap();
    assert_eq!(inc.contract, crate::Contract::None);
    assert!(inc.preserves.is_empty());
    assert!(listing_of(&public, "f").iter().any(|l| l == "PUSH BC"));
    for src in [private.to_string(), public] {
        for exec in run_both(&src, "f", &[5, 1]) {
            assert_eq!(exec.return_byte(), 7);
        }
    }
}

#[test]
fn inferred_contracts_do_not_depend_on_source_order() {
    let a = "@noinline u8 inc(u8 v) { return v + 1; }\n@noinline u8 two(u8 v) { return inc(inc(v)); }\nu8 f(u8 x, u8 y) { u8 a = two(x); return a + y; }";
    let b = "u8 f(u8 x, u8 y) { u8 a = two(x); return a + y; }\n@noinline u8 two(u8 v) { return inc(inc(v)); }\n@noinline u8 inc(u8 v) { return v + 1; }";
    let strip = |l: Vec<String>| -> Vec<String> {
        l.into_iter()
            .map(|t| {
                t.split('_')
                    .next()
                    .unwrap_or("")
                    .chars()
                    .filter(|c| !c.is_ascii_digit())
                    .collect()
            })
            .collect()
    };
    for name in ["inc", "two", "f"] {
        assert_eq!(
            strip(listing_of(a, name)),
            strip(listing_of(b, name)),
            "{name}"
        );
    }
    let ra = compile_ok(a);
    let rb = compile_ok(b);
    for name in ["inc", "two"] {
        let fa = ra.code.as_ref().unwrap().function(name).unwrap();
        let fb = rb.code.as_ref().unwrap().function(name).unwrap();
        assert_eq!((fa.contract, &fa.preserves), (fb.contract, &fb.preserves));
    }
}

#[test]
fn the_harness_rejects_a_broken_contract() {
    // The permanent guard: claim a register the function writes and the
    // harness must notice, both for the entry function and a nested call.
    let src = "u8 inc(u8 v) { u8 t = v; return t + 1; }\nu8 f(u8 x) { return inc(x); }";
    // Unoptimized, so `inc` stays a call (optimized builds inline it, H3i).
    let mut result = compile_opt(src, false);
    let code = result.code.as_mut().unwrap();
    let written: Vec<R8> = R8::ALL
        .into_iter()
        .filter(|r| !code.function("inc").unwrap().preserves.contains(r) && *r != R8::A)
        .collect();
    let inc = code.functions.iter_mut().find(|f| f.name == "inc").unwrap();
    if let Some(&r) = written.first() {
        inc.preserves.push(r);
    } else {
        // `inc` writes only A here; claim A (a result) to exercise the check.
        inc.preserves.push(R8::A);
    }
    let direct = execute_function(code, "inc", &[1]).unwrap_err();
    assert!(
        direct.0.contains("broke its inferred register contract"),
        "{direct:?}"
    );
    let nested = execute_function(code, "f", &[1]).unwrap_err();
    assert!(nested.0.contains("'inc' broke"), "{nested:?}");
}

#[test]
fn contracts_cross_units_only_when_declared() {
    use crate::project::{ProjectUnitInput, UnitKind, compile_project, parse_manifest};
    let manifest = parse_manifest(
        "[[unit]]\nname = \"lib\"\npath = \"lib.c80\"\norigin = 0x8000\n\n[[unit]]\nname = \"main\"\npath = \"main.c80\"\norigin = 0x8100\n",
    )
    .unwrap();
    let lib_plain = "pub u8 inc(u8 v) { return v + 1; }\n";
    let lib_decl = "@preserves(bc, de, hl) pub u8 inc(u8 v) { return v + 1; }\n";
    let main = "import lib;\nu8 f(u8 x, u8 y) { u8 a = lib::inc(x); return a + y; }\n";
    for (lib, pushes) in [(lib_plain, true), (lib_decl, false)] {
        let inputs = [
            ProjectUnitInput {
                name: "lib",
                kind: UnitKind::C80,
                origin: Some(0x8000),
                text: lib,
                source_name: "lib.c80",
                exports: &[],
                stack_extra: None,
            },
            ProjectUnitInput {
                name: "main",
                kind: UnitKind::C80,
                origin: Some(0x8100),
                text: main,
                source_name: "main.c80",
                exports: &[],
                stack_extra: None,
            },
        ];
        let result = compile_project(&manifest, &inputs, None);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let code = result.code.as_ref().unwrap();
        let f = code.function("f").unwrap();
        let has_push = f.mapped.iter().any(|m| m.text.starts_with("PUSH"));
        assert_eq!(
            has_push,
            pushes,
            "{:?}",
            f.mapped.iter().map(|m| &m.text).collect::<Vec<_>>()
        );
        let exec = execute_function(code, "f", &[5, 1]).unwrap();
        assert_eq!(exec.return_byte(), 7);
    }
}

#[test]
fn inline_asm_marker_is_a_located_user_error() {
    let src = "void f() {\n    asm (clobber: flags) {\n        call @{main::g} ; @{not::this}\n    }\n}\n";
    let result = compile_source("test.c80", src);
    let errs: Vec<_> = result.diagnostics.iter().filter(|d| d.is_error()).collect();
    assert_eq!(errs.len(), 1, "{:?}", result.diagnostics);
    let d = errs[0];
    assert_eq!(d.code.as_str(), "cg-unsupported");
    assert!(
        d.message.contains("not substituted in inline asm"),
        "{}",
        d.message
    );
    let at = src.find("@{main::g}").unwrap() as u32;
    assert_eq!(
        (d.span.start, d.span.end),
        (at, at + "@{main::g}".len() as u32)
    );
}

// ---------------------------------------------------------------------------
// H3: shifts
// ---------------------------------------------------------------------------

#[test]
fn shift_ops_assemble_with_disassembler_timing() {
    let r = |r| r;
    let ops = [
        (Z80Op::Sla(r(R8::B)), "8"),
        (Z80Op::Srl(R8::A), "8"),
        (Z80Op::Sra(R8::H), "8"),
        (Z80Op::Rl8(R8::D), "8"),
        (Z80Op::Rr8(R8::L), "8"),
        (Z80Op::Rlca, "4"),
        (Z80Op::Rrca, "4"),
        (Z80Op::Rra, "4"),
        (Z80Op::Inc8(R8::E), "4"),
        (Z80Op::Dec8(R8::C), "4"),
    ];
    for (op, t) in ops {
        let text = op.render();
        let bytes = assemble_line(&text, 0x8000).expect(&text);
        assert_eq!(bytes.len() as u32, op.size_bytes(), "{text}");
        let mut bus = FakeBus::new();
        bus.mem[0x8000..0x8000 + bytes.len()].copy_from_slice(&bytes);
        let d = disassemble_at(&mut bus, 0x8000);
        assert_eq!(
            d.text.to_ascii_uppercase().replace(' ', ""),
            text.replace(' ', "")
        );
        assert_eq!(d.t_states, Some(t), "{text}");
        assert_eq!(op.approx_tstates().to_string(), t, "{text}");
    }
    // DJNZ back to itself: 13 taken / 8 not taken.
    let bytes = assemble_line("DJNZ 8000H", 0x8000).unwrap();
    assert_eq!(bytes, [0x10, 0xFE]);
    let mut bus = FakeBus::new();
    bus.mem[0x8000..0x8002].copy_from_slice(&bytes);
    assert_eq!(disassemble_at(&mut bus, 0x8000).t_states, Some("13/8"));
    // Every shift-family op writes exactly its operand (H1 guard).
    assert_eq!(Z80Op::Sla(R8::B).writes(), [R8::B]);
    assert_eq!(Z80Op::Rr8(R8::L).writes(), [R8::L]);
    assert_eq!(Z80Op::Rra.writes(), [R8::A]);
    assert_eq!(Z80Op::Djnz { target: "x".into() }.writes(), [R8::B]);
}

#[test]
fn shift_by_one_is_one_instruction() {
    let result = compile_ok("u16 w(u16 a) { return a << 1; }\nu8 b(u8 a) { return a >> 1; }");
    assert_eq!(func_bytes(&result, "w"), &[0x29, 0xC9]); // ADD HL,HL / RET
    assert_eq!(func_bytes(&result, "b"), &[0xCB, 0x3F, 0xC9]); // SRL A / RET
    assert_eq!(func_tstates(&result, "w"), ["11", "10"]);
    assert_eq!(func_tstates(&result, "b"), ["8", "10"]);
}

#[test]
fn pinned_shift_runs_in_its_register() {
    let src = "u8 f(u8 v, u8 x @ b) { x <<= 1; return x + v; }";
    let text = listing_of(src, "f");
    assert!(text.iter().any(|t| t == "SLA B"), "{text:#?}");
    assert!(!text.iter().any(|t| t.starts_with("ADD A,A")), "{text:#?}");
    for exec in run_both(src, "f", &[1, 0x81]) {
        assert_eq!(exec.return_byte(), 3);
    }
    let src = "u8 f(u8 v, u8 n, u8 x @ b) { x >>= n; return x + v; }";
    let text = listing_of(src, "f");
    assert!(text.iter().any(|t| t == "SRL B"), "{text:#?}");
    for (n, want) in [(0u16, 0x91u8), (3, 0x13), (8, 1), (255, 1)] {
        for exec in run_both(src, "f", &[1, n, 0x90]) {
            assert_eq!(exec.return_byte(), want, "n={n}");
        }
    }
}

#[test]
fn variable_shift_guards_zero_and_runs_past_the_width() {
    let src = "u16 f(u16 a, u8 n) { return a << n; }\n\
               i16 g(i16 a, u8 n) { return a >> n; }\n\
               u8 h(u8 a, u8 n) { return a >> n; }";
    let result = compile_ok(src);
    let code = result.code.as_ref().unwrap();
    let text = assembly_of(&result);
    assert!(text.contains("DJNZ"), "{text}");
    for (n, want) in [
        (0u16, 0x1234u16),
        (1, 0x2468),
        (15, 0),
        (16, 0),
        (17, 0),
        (255, 0),
    ] {
        let want = if n == 15 { 0 } else { want };
        let exec = execute_function(code, "f", &[0x1234, n]).unwrap();
        assert_eq!(exec.return_word(), want, "f n={n}");
    }
    for (n, want) in [
        (0u16, 0x8001u16),
        (1, 0xC000),
        (15, 0xFFFF),
        (16, 0xFFFF),
        (255, 0xFFFF),
    ] {
        let exec = execute_function(code, "g", &[0x8001, n]).unwrap();
        assert_eq!(exec.return_word(), want, "g n={n}");
    }
    for (n, want) in [(0u16, 0x80u8), (7, 1), (8, 0), (255, 0)] {
        let exec = execute_function(code, "h", &[0x80, n]).unwrap();
        assert_eq!(exec.return_byte(), want, "h n={n}");
    }
    // A loop: the span cost is incomplete, never a single exact number.
    let f = code.function("f").unwrap();
    let mapped: Vec<_> = f.mapped.iter().collect();
    let map = result.map().unwrap();
    let cost = map.span_cost(&mapped);
    assert!(!cost.complete && cost.has_loop, "{cost:?}");
}

#[test]
fn strict_variable_shift_needs_a_free_counter() {
    let ok = "@fastcall u8 f(u8 v, u8 n) { return v >> n; }";
    for exec in run_both(ok, "f", &[0x80, 3]) {
        assert_eq!(exec.return_byte(), 0x10);
    }
    // Every byte register holds a live parameter and the count stays live:
    // the counter copy needs a spill, which strict code rejects.
    let full = "@fastcall u8 f(u8 v, u8 n, u8 b2, u8 e, u8 d, u8 l, u8 h) {\n\
                return (v << n) + b2 + e + d + l + h + n; }";
    let errs = errors_of(full);
    assert!(
        errs.iter()
            .any(|e| e.contains("@fastcall 'f' must fit in registers")),
        "{errs:?}"
    );
    let plain = full.replace("@fastcall ", "");
    for exec in run_both(&plain, "f", &[1, 2, 3, 4, 5, 6, 7]) {
        assert_eq!(exec.return_byte(), 4 + 3 + 4 + 5 + 6 + 7 + 2);
    }
}

#[test]
fn shifts_on_globals_fields_statics_and_pointers() {
    let src = r#"
struct P { u8 a; u16 w; };
u8 g8;
u16 g16;
P gp;
u8 f(u8 n) {
    static u16 acc = 3;
    g8 = 0x81;
    g16 = 0x8421;
    g8 <<= n;
    g16 >>= 3;
    ptr<P> p = &gp;
    p->w = 0x0101;
    p->w <<= n;
    p->a = 0xF0;
    p->a >>= 1;
    acc = acc << n;
    return g8 + u8(g16) + u8(p->w >> 8) + p->a + u8(acc);
}
"#;
    // g8 = 0x02, g16 = 0x1084, p->w = 0x0202, p->a = 0x78, acc = 6.
    for exec in run_both(src, "f", &[1]) {
        assert_eq!(
            exec.return_byte(),
            0x02u8
                .wrapping_add(0x84)
                .wrapping_add(2)
                .wrapping_add(0x78)
                .wrapping_add(6)
        );
    }
}

/// H3a relies on the helper assembler resolving `DW label+n` for labels that
/// are defined later and in another `ORG` block (another unit's segment).
#[test]
fn helper_assembler_resolves_forward_and_cross_org_word_references() {
    let src = "    ORG 0x1000\nTAB:\n    DW LATER+3, OTHER+2, OTHER\nLATER:\n    DB 1,2,3,4\n    ORG 0x2000\nOTHER:\n    DB 9\n";
    let asm = rtvc_core::asm::assemble_program(src, 0x1000).expect("assembles");
    assert_eq!(asm.symbols["LATER"], 0x1006);
    let seg = asm
        .segments
        .iter()
        .find(|s| s.addr == 0x1000)
        .expect("first segment");
    assert_eq!(&seg.bytes[..6], &[0x09, 0x10, 0x02, 0x20, 0x00, 0x20]);
}

fn global_word(code: &GeneratedProgram, name: &str) -> u16 {
    let g = code
        .global(name)
        .unwrap_or_else(|| panic!("no global {name}"));
    let bytes = crate::z80::bytes_in_segments(&code.assembled, g.addr, 2).unwrap();
    u16::from_le_bytes([bytes[0], bytes[1]])
}

/// H3a / F-023: `&array[K]`, `&array[K].field`, `&global.field`, `&global`
/// and the `ptr<T>(...)` / `u16(...)` wrappers are link-time addresses: the
/// image holds the target's address plus the constant byte offset.
#[test]
fn static_initializer_addresses_match_the_symbol_table() {
    let src = "struct Pt { u8 x; u16 w; };\n\
               u8 arr[8];\nPt ps[4];\nPt one = {1, 2};\n\
               ptr<u8> p3 = &arr[3];\nptr<u16> pw = &ps[2].w;\nptr<u16> ow = &one.w;\n\
               ptr<Pt> po = &one;\nu16 raw = u16(&arr[1]);\nptr<u8> pb = ptr<u8>(&ps[1]);\n\
               const u8 K = 5;\nptr<u8> pk = &arr[K];\n\
               u8 f() { return *p3; }\n";
    let result = compile_ok(src);
    let code = result.code.as_ref().unwrap();
    let arr = global_addr(code, "arr");
    let ps = global_addr(code, "ps");
    let one = global_addr(code, "one");
    assert_eq!(global_word(code, "p3"), arr + 3);
    assert_eq!(global_word(code, "pw"), ps + 2 * 3 + 1);
    assert_eq!(global_word(code, "ow"), one + 1);
    assert_eq!(global_word(code, "po"), one);
    assert_eq!(global_word(code, "raw"), arr + 1);
    assert_eq!(global_word(code, "pb"), ps + 3);
    assert_eq!(global_word(code, "pk"), arr + 5);
    let label = |name: &str| code.global(name).unwrap().label.clone();
    assert!(
        code.assembly.contains(&format!("DW {}+7", label("ps"))),
        "{}",
        code.assembly
    );
    assert!(code.assembly.contains(&format!("DW {}\n", label("one"))));
    // The map names the target of each address word.
    let map = result.map().unwrap();
    let pw = global_addr(code, "pw");
    let r = map.data_ref_at(pw + 1).expect("data ref");
    assert_eq!(
        (r.global.as_str(), r.target.as_str(), r.addend),
        ("pw", "ps", 7)
    );
    assert!(map.data_ref_at(arr).is_none());
}

/// Forward references, a self-referencing node, an address inside a struct
/// array initializer, and a function-local `static` holding the address of
/// another static and of a global: the data walks correctly at run time.
#[test]
fn static_linked_data_walks_at_run_time() {
    let src = "struct N { u8 v; ptr<N> next; };\n\
               N a = {1, &b};\nN b = {2, &c};\nN c = {4, ptr<N>(0)};\n\
               N ring[2] = { {8, &ring[1]}, {16, &ring[0]} };\n\
               u8 table[4] = {10, 20, 30, 40};\n\
               u8 sum() { ptr<N> p = &a; u8 s = 0;\n\
               while (u16(p) != 0) { s += p->v; p = p->next; } return s; }\n\
               u8 hop() { ptr<N> p = ring[0].next; return p->next->v; }\n\
               u8 local() { static u8 buf[3] = {5, 6, 7}; static ptr<u8> q = &buf[2];\n\
               static ptr<u8> t = &table[1]; static ptr<u16> self_ref = ptr<u16>(&self_ref);\n\
               return *q + *t + u8(u16(self_ref) - u16(&self_ref)); }\n";
    for optimize in [true, false] {
        let result = compile(CompileInput {
            files: vec![SourceInput {
                name: "t.c80",
                text: src,
            }],
            origin: 0x8000,
            optimize,
            target: C80Target::GenericZ80,
        });
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let code = result.code.as_ref().unwrap();
        assert_eq!(execute_function(code, "sum", &[]).unwrap().return_byte(), 7);
        assert_eq!(execute_function(code, "hop", &[]).unwrap().return_byte(), 8);
        assert_eq!(
            execute_function(code, "local", &[]).unwrap().return_byte(),
            27
        );
        assert_eq!(
            global_word(code, "local.q"),
            global_addr(code, "local.buf") + 2
        );
    }
}

/// Every rejected form has a located diagnostic; nothing is silently zero.
#[test]
fn static_initializer_address_rejections_are_located() {
    let cases: &[(&str, &str, &str)] = &[
        ("u8 arr[4];\nptr<u8> p = &arr;\n", "use &arr[0]", "&arr"),
        (
            "const u8 K = 1;\nptr<u8> p = &K;\n",
            "address of a constant",
            "&K",
        ),
        (
            "str s = \"hi\";\nptr<u8> p = &s[0];\n",
            "string payload",
            "&s[0]",
        ),
        (
            "u8 arr[4];\nptr<u8> p = &arr[1] + 1;\n",
            "arithmetic on an address",
            "&arr[1] + 1",
        ),
        (
            "u8 arr[4];\nptr<u8> p = &arr[4];\n",
            "outside the array",
            "4",
        ),
        (
            "ptr<u8> q;\nptr<u8> p = &q[1];\n",
            "index only an array",
            "q",
        ),
        (
            "u8 arr[4];\nu8 i = 1;\nptr<u8> p = &arr[i];\n",
            "constant index",
            "i",
        ),
        ("u8 arr[4];\nu16 p = &arr[0];\n", "expected u16", "&arr[0]"),
        (
            "u8 arr[4];\nconst ptr<u8> p = &arr[0];\n",
            "const cannot hold an address",
            "&arr[0]",
        ),
        (
            "u8 f() { u8 x = 1; static ptr<u8> q = &x; return x; }\n",
            "automatic local 'x'",
            "x",
        ),
        (
            "u8 f() { u8 y @ b = 2; static ptr<u8> q = &y; return y; }\n",
            "pinned to register b",
            "&y",
        ),
        (
            "struct N { u8 v; ptr<N> next; };\nN a = {1, &a + 1};\n",
            "arithmetic on an address",
            "&a + 1",
        ),
    ];
    for (src, needle, at) in cases {
        let result = compile_source("t.c80", src);
        let errs: Vec<_> = result.diagnostics.iter().filter(|d| d.is_error()).collect();
        assert_eq!(errs.len(), 1, "{src}: {:?}", result.diagnostics);
        assert!(
            errs[0].message.contains(needle),
            "{src}: {}",
            errs[0].message
        );
        let span = errs[0].span;
        let text = &src[span.start as usize..span.end as usize];
        assert_eq!(text, *at, "{src}");
        assert!(result.code.is_none(), "{src}");
    }
}

#[test]
fn byte_pointers_in_bc_and_de_are_used_in_place() {
    // H5 item 4: LD A,(BC), LD (DE),A and INC rr; HL is never touched.
    let src = "u8 from[4] = {1, 2, 3, 250}; u8 to[4];
        u8 f(u8 n) {
            ptr<u8> p @ bc = &from[0];
            ptr<u8> q @ de = &to[0];
            while (n != 0) { *q = *p; p++; q++; n--; }
            return to[3] + to[0];
        }";
    for exec in run_both(src, "f", &[4]) {
        assert_eq!(exec.return_byte(), 251);
    }
    for exec in run_both(src, "f", &[3]) {
        assert_eq!(exec.return_byte(), 1);
    }
    for optimize in [true, false] {
        let result = compile_opt(src, optimize);
        let text: Vec<String> = result
            .code
            .as_ref()
            .unwrap()
            .function("f")
            .unwrap()
            .mapped
            .iter()
            .map(|m| m.text.clone())
            .collect();
        for want in ["LD A,(BC)", "LD (DE),A", "INC BC", "INC DE"] {
            assert!(text.iter().any(|t| t == want), "{want}: {text:#?}");
        }
        // The loop (from the load to the closing jump) never uses HL as a
        // pointer and never steps it.
        let start = text.iter().position(|t| t == "LD A,(BC)").unwrap();
        let end = start
            + text[start..]
                .iter()
                .position(|t| t.starts_with("JP "))
                .unwrap();
        assert!(
            !text[start..end].iter().any(|t| t.contains("HL")),
            "{text:#?}"
        );
    }
}

#[test]
fn a_call_in_return_position_is_a_tail_jump_with_a_tighter_stack_bound() {
    // H5 item 5: `CALL g / RET` -> `JP g` (optimized builds only). `g`
    // starts at f's entry SP, so f's bound is g's, not g's + 2.
    let src = "@noinline u8 g(u8 x) { return x + x; }
        @noinline u8 h(u8 x) { return g(x) + 1; }
        u8 f(u8 x) { return h(x); }";
    for optimize in [true, false] {
        let result = compile_opt(src, optimize);
        let code = result.code.as_ref().unwrap();
        let exec = execute_function(code, "f", &[20]).unwrap();
        assert_eq!(exec.return_byte(), 41);
        assert_ix_iy_sp(&exec);
        let f = code.function("f").unwrap();
        let h = code.function("h").unwrap();
        assert!(
            exec.sp_used <= f.stack_bound,
            "{} > {}",
            exec.sp_used,
            f.stack_bound
        );
        let text: Vec<&str> = f.mapped.iter().map(|m| m.text.as_str()).collect();
        if optimize {
            assert_eq!(text, [format!("JP {}", h.label).as_str()]);
            assert_eq!(f.stack_bound, h.stack_bound);
            assert_eq!(exec.sp_used, f.stack_bound);
        } else {
            assert!(text.iter().any(|t| t.starts_with("CALL")), "{text:?}");
            assert_eq!(f.stack_bound, h.stack_bound + 2);
        }
    }
}

/// Source names that a careless label scheme could turn into something the
/// helper assembler reads as a register, a condition code, or a number
/// (`h`, `nz`, `dead`, `beef`, `ffh`, `b1`...). F-035.
const RISKY_NAMES: &[&str] = &[
    "a", "b", "c", "d", "e", "h", "l", "i", "r", "af", "bc", "de", "hl", "sp", "ix", "iy", "ixh",
    "ixl", "iyh", "iyl", "nz", "z", "nc", "po", "pe", "p", "m", "dead", "beef", "cafe", "x0f",
    "h0", "fh", "ffh", "deadh", "abh", "eh", "ch", "dh", "b1", "ab", "bb", "b0", "fb", "fn", "fnh",
    "f0", "s", "b3", "c1", "n", "g", "u", "ia",
];

#[test]
fn functions_named_like_registers_conditions_and_numbers_are_called_correctly() {
    // F-035: `F16_H` (a function `h`) assembled as the number 0xF16. Every
    // generated label now starts with a non-hex letter prefix.
    let mut src = String::new();
    let mut expected = 0u8;
    for (i, name) in RISKY_NAMES.iter().enumerate() {
        let k = i as u8 + 1;
        src.push_str(&format!(
            "@noinline u8 {name}(u8 x) {{ return x + {k}; }}\n"
        ));
        expected = expected.wrapping_add(10u8.wrapping_add(k));
    }
    // A tail call (`JP label`) and a block label of each kind.
    src.push_str("@noinline u8 tail(u8 x) { return h(x); }\n");
    expected = expected.wrapping_add(10 + 6);
    src.push_str("u8 entry(u8 x) { u8 s = 0;\n");
    for name in RISKY_NAMES {
        src.push_str(&format!("s += {name}(x);\n"));
    }
    src.push_str("s += tail(x); while (x != 0) { x -= 1; } return s; }\n");
    for exec in run_both(&src, "entry", &[10]) {
        assert_eq!(exec.return_byte(), expected, "{src}");
    }
}

#[test]
fn globals_and_statics_named_like_registers_conditions_and_numbers_are_distinct() {
    let mut src = String::new();
    for name in RISKY_NAMES {
        src.push_str(&format!("u8 {name};\n"));
    }
    src.push_str("@noinline u8 bump() { static u8 h; static u8 dead; h += 1; dead += 2; return h + dead; }\n");
    src.push_str("u8 entry(u8 x) {\n");
    for (i, name) in RISKY_NAMES.iter().enumerate() {
        src.push_str(&format!("{name} = x + {};\n", i + 1));
    }
    src.push_str("u8 acc = bump(); acc = bump() + acc;\n");
    let mut expected = 3u8 + 6;
    for (i, name) in RISKY_NAMES.iter().enumerate() {
        src.push_str(&format!("acc += {name};\n"));
        expected = expected.wrapping_add(10 + i as u8 + 1);
    }
    src.push_str("return acc; }\n");
    for exec in run_both(&src, "entry", &[10]) {
        assert_eq!(exec.return_byte(), expected, "{src}");
    }
}

#[test]
fn a_parallel_move_scratch_never_holds_a_live_carried_value() {
    // F-036: restoring homes before a branch on a constant parked a byte in
    // D, where a value carried across the second `||` still lived (the H1
    // clobber guard rejected it). Both modes, no folding needed.
    let src = "i8 f(i8 a, i8 b) { i8 c = a;
        i8 d = (i8(i16(b) + i16(i8((c != 0) || true))) & i8(i16(b) - i16((b + i8(true || (c != 0))))));
        return a & i8(c != b); }";
    for exec in run_both(src, "f", &[5, 3]) {
        assert_eq!(exec.return_byte(), 1);
    }
    for exec in run_both(src, "f", &[4, 4]) {
        assert_eq!(exec.return_byte(), 0);
    }
}
