use super::*;
use crate::compiler::ir::function_by_name;

fn compile_src(src: &str) -> CompilationResult {
    compile_source("test.c80", src)
}

fn codes(result: &CompilationResult) -> Vec<&'static str> {
    result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code.as_str())
        .collect()
}

fn ok(src: &str) -> CompilationResult {
    let result = compile_src(src);
    assert!(!result.has_errors(), "{:?}\n{src}", result.diagnostics);
    assert!(result.program.is_some(), "expected typed IR");
    result
}

#[test]
fn signed_unsigned_limits_and_wrapping() {
    let result = ok(r#"
u8 wrap() { return 255 + 1; }
i8 min() { return -128; }
u16 wide() { return 255 + 1; }
"#);
    let wrap = function_by_name(result.program.as_ref().unwrap(), "wrap").unwrap();
    let has_u8_add = wrap.blocks.iter().any(|b| {
        b.ops.iter().any(|op| {
            matches!(
                op,
                IrOp::Binary {
                    ty: CType::U8,
                    op: IrBinary::Add,
                    ..
                }
            )
        })
    });
    assert!(has_u8_add, "{:?}", wrap.blocks);

    let result = compile_src("i8 bad() { return 128; }");
    assert!(
        codes(&result).contains(&"ty-literal-range"),
        "{:?}",
        codes(&result)
    );
    assert!(result.program.is_none());

    let result = compile_src("i8 okcast() { return i8(128); }");
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

#[test]
fn narrowing_widening_and_bool_conversion() {
    ok(r#"
u16 widen(u8 x) { return u16(x); }
u8 narrow(u16 x) { return u8(x); }
bool flag(u8 x) { return bool(x); }
u8 from_bool(bool b) { return u8(b); }
"#);
    let result = compile_src("bool b() { return 1; }");
    assert!(
        codes(&result).contains(&"ty-mismatch"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn mixed_typed_operands_rejected() {
    let result = compile_src(
        r#"
u8 mix(u8 a, u16 b) { return a + b; }
"#,
    );
    assert!(
        codes(&result).contains(&"ty-mismatch"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn contextual_literals() {
    ok("u8 x() { u8 a = 255; return a; }");
    let result = compile_src("u8 x() { u8 a = 256; return a; }");
    assert!(
        codes(&result).contains(&"ty-literal-range"),
        "{:?}",
        codes(&result)
    );
    let result = compile_src("i8 x() { return -128; }");
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

#[test]
fn local_shadowing_and_duplicates() {
    ok(r#"
u8 f() {
    u8 x = 1;
    {
        u8 x = 2;
        return x;
    }
}
"#);
    let result = compile_src("u8 f() { u8 x = 1; u8 x = 2; return x; }");
    assert!(
        codes(&result).contains(&"ty-duplicate-name"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn branches_definite_assignment() {
    let result = compile_src(
        r#"
u8 one(bool c) {
    u8 x;
    if (c) { x = 1; }
    return x;
}
"#,
    );
    assert!(
        codes(&result).contains(&"ty-use-before-assign"),
        "{:?}",
        codes(&result)
    );

    ok(r#"
u8 all(bool c) {
    u8 x;
    if (c) { x = 1; } else { x = 2; }
    return x;
}
"#);
}

#[test]
fn zero_trip_loop_does_not_assign() {
    let result = compile_src(
        r#"
u8 z(bool c) {
    u8 x;
    while (c) { x = 1; break; }
    return x;
}
"#,
    );
    assert!(
        codes(&result).contains(&"ty-use-before-assign"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn break_continue_and_returns() {
    ok(r#"
void loop(bool c) {
    while (c) {
        if (c) { continue; }
        break;
    }
    return;
}
u8 val() { return 1; }
"#);
    let result = compile_src("u8 missing() { }");
    assert!(
        codes(&result).contains(&"ty-missing-return"),
        "{:?}",
        codes(&result)
    );
    let result = compile_src("void v() { return 1; }");
    assert!(result.has_errors());
}

#[test]
fn use_before_assignment() {
    let result = compile_src("u8 f() { u8 x; return x; }");
    assert!(
        codes(&result).contains(&"ty-use-before-assign"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn recursive_calls_rejected() {
    let result = compile_src("void f() { f(); }");
    assert!(
        codes(&result).contains(&"ty-recursion"),
        "{:?}",
        codes(&result)
    );
    let result = compile_src(
        r#"
void a() { b(); }
void b() { a(); }
"#,
    );
    assert!(
        codes(&result).contains(&"ty-recursion"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn rejected_c_syntax() {
    let result = compile_src("int f() { return 0; }");
    assert!(result.has_errors());
    let result = compile_src("void f() { a++; }");
    assert!(result.has_errors());
}

#[test]
fn sizeof_scalar() {
    let result = ok("u16 f() { return sizeof(u8) + sizeof(u16); }");
    let f = function_by_name(result.program.as_ref().unwrap(), "f").unwrap();
    let consts: Vec<u16> = f
        .blocks
        .iter()
        .flat_map(|b| b.ops.iter())
        .filter_map(|op| match op {
            IrOp::Const { bits, .. } => Some(*bits),
            _ => None,
        })
        .collect();
    assert!(consts.contains(&1), "{consts:?}");
    assert!(consts.contains(&2), "{consts:?}");
}

#[test]
fn sizeof_ptr_and_str() {
    let result = ok("u16 f() { return sizeof(ptr<u8>) + sizeof(str); }");
    let f = function_by_name(result.program.as_ref().unwrap(), "f").unwrap();
    let consts: Vec<u16> = f
        .blocks
        .iter()
        .flat_map(|b| b.ops.iter())
        .filter_map(|op| match op {
            IrOp::Const { bits, .. } => Some(*bits),
            _ => None,
        })
        .collect();
    assert_eq!(consts.iter().filter(|&&c| c == 2).count(), 2, "{consts:?}");
}

#[test]
fn array_and_string_constant_bounds() {
    let result = compile_src("u8 a[4]; u8 f() { return a[4]; }");
    assert!(
        codes(&result).contains(&"ty-literal-range"),
        "{:?}",
        codes(&result)
    );
    let result = compile_src("u8 a[4]; u8 f() { return a[-1]; }");
    assert!(
        codes(&result).contains(&"ty-literal-range"),
        "{:?}",
        codes(&result)
    );
    let result = compile_src(r#"str s = "hi"; u8 f() { return s[2]; }"#);
    assert!(
        codes(&result).contains(&"ty-literal-range"),
        "{:?}",
        codes(&result)
    );
    let result = compile_src(r#"u8 f(str s) { return s[200]; }"#);
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

#[test]
fn str_payload_length_limits() {
    let empty = ok(r#"str s = ""; u8 f() { return s.len; }"#);
    assert!(empty.program.is_some());
    let too_long = "x".repeat(256);
    let src = format!(r#"str s = "{too_long}";"#);
    let result = compile_src(&src);
    assert!(
        codes(&result).contains(&"ty-literal-range"),
        "{:?}",
        codes(&result)
    );
}

#[test]
fn returning_local_address_is_a_warning() {
    let result = compile_src("ptr<u8> f() { u8 x = 1; return &x; }");
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "ty-return-local-addr" && !d.is_error()),
        "{:?}",
        result.diagnostics
    );
    assert!(result.code.is_some());
}

#[test]
fn asm_overlapping_inputs_rejected() {
    let result = compile_src(
        r#"
u8 f(u8 x, u8 y) {
    asm(in: a = x, in: a = y, clobber: flags) { nop }
    return x;
}
"#,
    );
    assert!(
        codes(&result).contains(&"ty-mismatch"),
        "{:?}",
        codes(&result)
    );
    let pair = compile_src(
        r#"
u8 f(u16 p, u8 q) {
    asm(in: hl = p, in: l = q, clobber: flags) { nop }
    return q;
}
"#,
    );
    assert!(codes(&pair).contains(&"ty-mismatch"), "{:?}", codes(&pair));
}

#[test]
fn asm_inout_requires_assignment_and_out_assigns() {
    let unassigned = compile_src(
        r#"
u8 f() {
    u8 x;
    asm(inout: a = x, clobber: flags) { inc a }
    return x;
}
"#,
    );
    assert!(
        codes(&unassigned).contains(&"ty-use-before-assign"),
        "{:?}",
        codes(&unassigned)
    );
    let assigned = ok(r#"
u8 f() {
    u8 r;
    asm(out: a = r, clobber: flags) { ld a, 3 }
    return r;
}
"#);
    assert!(assigned.code.is_some());
}

#[test]
fn asm_rejects_org_and_ret() {
    let org = compile_src("void f() { asm { org 0x1000 } }");
    assert!(codes(&org).contains(&"cg-unsupported"), "{:?}", codes(&org));
    let ret = compile_src("void f() { asm { ret } }");
    assert!(codes(&ret).contains(&"cg-unsupported"), "{:?}", codes(&ret));
}

#[test]
fn asm_call_without_stack_is_a_warning() {
    let result = compile_src(
        r#"
u8 f() {
    u8 r;
    asm(out: a = r, clobber: flags) { call 0x9000 }
    return r;
}
"#,
    );
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "ln-stack" && !d.is_error()),
        "{:?}",
        result.diagnostics
    );
}
