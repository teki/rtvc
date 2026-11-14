//! H3i: `@inline`, `@noinline`, and automatic inlining (F-031).

use super::*;
use crate::harness::{AccessKind, ExecConfig, execute_function, execute_function_with};

fn build(src: &str, optimize: bool) -> CompilationResult {
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

fn ok(src: &str, optimize: bool) -> CompilationResult {
    let result = build(src, optimize);
    assert!(!result.has_errors(), "{:#?}\n{src}", result.diagnostics);
    result
}

fn errors(src: &str) -> Vec<String> {
    build(src, true)
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.message.clone())
        .collect()
}

fn texts(result: &CompilationResult, name: &str) -> Vec<String> {
    result
        .code
        .as_ref()
        .unwrap()
        .function(name)
        .unwrap_or_else(|| panic!("no function {name}"))
        .mapped
        .iter()
        .map(|m| m.text.clone())
        .collect()
}

fn has_call(result: &CompilationResult, name: &str) -> bool {
    // A tail call (H5) is `JP <function label>`.
    let labels: Vec<String> = result
        .code
        .as_ref()
        .map(|c| c.functions.iter().map(|f| f.label.clone()).collect())
        .unwrap_or_default();
    texts(result, name).iter().any(|t| {
        t.starts_with("CALL")
            || t.strip_prefix("JP ")
                .is_some_and(|l| labels.iter().any(|x| x == l))
    })
}

/// Both optimize modes: `f(args)` returns `want` as a byte, with no CALL.
fn byte_both(src: &str, args: &[u16], want: u8) {
    for optimize in [true, false] {
        let result = ok(src, optimize);
        assert!(
            !has_call(&result, "f"),
            "optimize={optimize}: {:#?}",
            texts(&result, "f")
        );
        let exec = execute_function(result.code.as_ref().unwrap(), "f", args).unwrap();
        assert_eq!(exec.return_byte(), want, "optimize={optimize} f{args:?}");
    }
}

#[test]
fn surface_errors() {
    let e = errors("@inline @stackcall u8 g(u8 x) { return x; }\nu8 f() { return g(1); }");
    assert!(
        e.iter()
            .any(|m| m.starts_with("@inline cannot be combined with @stackcall")),
        "{e:?}"
    );
    let e = errors("@stackcall @inline u8 g(u8 x) { return x; }");
    assert!(
        e.iter()
            .any(|m| m.starts_with("@inline cannot be combined with @stackcall")),
        "{e:?}"
    );
    let e = errors("@inline @noinline u8 g(u8 x) { return x; }");
    assert!(
        e.iter()
            .any(|m| m.contains("@noinline contradicts @inline")),
        "{e:?}"
    );
    let e = errors("@inline @inline u8 g(u8 x) { return x; }");
    assert!(e.iter().any(|m| m == "duplicate @inline"), "{e:?}");
    let e = errors("@inline u8 g;");
    assert!(
        e.iter().any(|m| m == "@inline applies only to functions"),
        "{e:?}"
    );
    let e = errors("@noinline u8 g;");
    assert!(
        e.iter().any(|m| m == "@noinline applies only to functions"),
        "{e:?}"
    );
    // Combinable with @fastcall and @preserves, in any order.
    for src in [
        "@inline @fastcall @preserves(bc) u8 g(u8 x) { return x + 1; }\nu8 f(u8 x) { return g(x); }",
        "@preserves(bc) @fastcall @inline u8 g(u8 x) { return x + 1; }\nu8 f(u8 x) { return g(x); }",
        "@fastcall @inline pub u8 g(u8 x) { return x + 1; }\nu8 f(u8 x) { return g(x); }",
    ] {
        byte_both(src, &[4], 5);
    }
}

/// `u8 inc(u8 x) { return x + 1; }` inlined into a loop is one instruction.
#[test]
fn inc_in_a_loop_is_one_instruction() {
    let src = "@inline u8 inc(u8 x) { return x + 1; }\n\
               u8 f(u8 n @ b) { u8 s = 0; while (n != 0) { s = inc(s); n--; } return s; }";
    for optimize in [true, false] {
        let result = ok(src, optimize);
        let f = result.code.as_ref().unwrap().function("f").unwrap();
        let expanded: Vec<&str> = f
            .mapped
            .iter()
            .filter(|m| !m.inlined.is_empty())
            .map(|m| m.text.as_str())
            .collect();
        assert_eq!(expanded.len(), 1, "optimize={optimize}: {expanded:?}");
        assert!(expanded[0].starts_with("INC "), "{expanded:?}");
        assert_eq!(
            f.mapped
                .iter()
                .find(|m| !m.inlined.is_empty())
                .unwrap()
                .bytes
                .len(),
            1
        );
        assert_eq!(f.inlined.len(), 1);
        assert_eq!(f.inlined[0].callee, "inc");
        assert_eq!(f.inlined[0].bytes, 1);
        assert!(!f.inlined[0].auto);
        let code = result.code.as_ref().unwrap();
        // Private @inline: no out-of-line body.
        assert!(code.function("inc").is_none());
        for n in [0u16, 1, 7] {
            let exec = execute_function(code, "f", &[n]).unwrap();
            assert_eq!(u16::from(exec.return_byte()), n);
        }
    }
    let result = ok(src, true);
    assert!(
        result
            .code
            .as_ref()
            .unwrap()
            .assembly
            .contains("; inline inc\n")
    );
}

#[test]
fn argument_is_evaluated_once() {
    let src = "@inline u8 twice(u8 x) { return x + x; }\n\
               u8 f() { return twice(cpu::in(0x58)); }";
    for optimize in [true, false] {
        let result = ok(src, optimize);
        assert!(!has_call(&result, "f"));
        let cfg = ExecConfig {
            scripted_ports: vec![(0x58, vec![3, 100])],
            ..ExecConfig::default()
        };
        let exec = execute_function_with(result.code.as_ref().unwrap(), "f", &[], &cfg).unwrap();
        assert_eq!(exec.return_byte(), 6);
        let reads = exec
            .accesses
            .iter()
            .filter(|a| a.kind == AccessKind::PortIn && a.addr & 0xFF == 0x58)
            .count();
        assert_eq!(reads, 1, "optimize={optimize}");
    }
    // Several arguments: left to right, each once.
    let src = "@inline u8 sub(u8 a, u8 b) { return a - b; }\n\
               u8 f() { return sub(cpu::in(0x58), cpu::in(0x59)); }";
    for optimize in [true, false] {
        let result = ok(src, optimize);
        let cfg = ExecConfig {
            scripted_ports: vec![(0x58, vec![10]), (0x59, vec![3])],
            ..ExecConfig::default()
        };
        let exec = execute_function_with(result.code.as_ref().unwrap(), "f", &[], &cfg).unwrap();
        assert_eq!(exec.return_byte(), 7);
        let order: Vec<u16> = exec
            .accesses
            .iter()
            .filter(|a| a.kind == AccessKind::PortIn)
            .map(|a| a.addr & 0xFF)
            .collect();
        assert_eq!(order, [0x58, 0x59], "optimize={optimize}");
    }
}

#[test]
fn several_returns() {
    let src = "@inline u8 clamp(u8 x) { if (x > 10) { return 10; } if (x < 2) { return 2; } return x; }\n\
               u8 f(u8 a) { return clamp(a) + 1; }";
    for (a, want) in [
        (0u16, 3u8),
        (1, 3),
        (2, 3),
        (5, 6),
        (10, 11),
        (11, 11),
        (255, 11),
    ] {
        byte_both(src, &[a], want);
    }
}

#[test]
fn loop_in_the_callee() {
    let src = "@inline u8 sum_to(u8 n) { u8 s = 0; while (n != 0) { s += n; n--; } return s; }\n\
               u8 f(u8 a, u8 b) { return sum_to(a) + sum_to(b); }";
    for (a, b) in [(0u16, 0u16), (1, 4), (10, 3)] {
        let want = ((1..=a).sum::<u16>() + (1..=b).sum::<u16>()) as u8;
        byte_both(src, &[a, b], want);
    }
}

#[test]
fn three_level_nesting() {
    let src = "@inline u8 a1(u8 x) { return x + 1; }\n\
               @inline u8 a2(u8 x) { return a1(x) + a1(x); }\n\
               @inline u8 a3(u8 x) { return a2(x) + 1; }\n\
               u8 f(u8 x) { return a3(x); }";
    for x in [0u16, 5, 200] {
        byte_both(
            src,
            &[x],
            ((x as u8).wrapping_add(1)).wrapping_mul(2).wrapping_add(1),
        );
    }
    let result = ok(src, true);
    let f = result.code.as_ref().unwrap().function("f").unwrap();
    let depth = f.mapped.iter().map(|m| m.inlined.len()).max().unwrap();
    assert_eq!(depth, 3, "a3 > a2 > a1");
    let deepest = f.mapped.iter().find(|m| m.inlined.len() == 3).unwrap();
    let names: Vec<&str> = deepest.inlined.iter().map(|fr| fr.name.as_str()).collect();
    assert_eq!(names, ["a3", "a2", "a1"]);
    // The innermost instruction maps to a1's source line, its outermost
    // frame to the call in f.
    let line = &src[deepest.span.start as usize..deepest.span.end as usize];
    assert!(line.contains("x + 1"), "{line}");
    let call = &src[deepest.inlined[0].call.start as usize..deepest.inlined[0].call.end as usize];
    assert_eq!(call, "a3(x)");
    // Every instruction of f lies in f's address range, and the map covers
    // every byte.
    for m in &f.mapped {
        assert!(m.address >= f.addr && m.address < f.addr + f.size);
    }
    let map = result.map().expect("listing map");
    assert!(map.every_byte_mapped(&result.code.as_ref().unwrap().assembled));
    let at = map.at_address(deepest.address).unwrap();
    assert_eq!(at.inlined.len(), 3);
}

#[test]
fn inline_asm_label_inlined_twice() {
    let src = "@inline u8 count_down(u8 n) {\n\
                   u8 r;\n\
                   asm (in: b = n, out: a = r, clobber: b, clobber: flags) {\n\
                       xor a\n\
                   loop:\n\
                       inc a\n\
                       djnz loop\n\
                   }\n\
                   return r;\n\
               }\n\
               u8 f(u8 x, u8 y) { return count_down(x) + count_down(y); }";
    byte_both(src, &[3, 4], 7);
    let result = ok(src, true);
    let asm = &result.code.as_ref().unwrap().assembly;
    let labels: Vec<&str> = asm
        .lines()
        .filter(|l| l.ends_with(':') && l.to_ascii_uppercase().contains("LOOP"))
        .collect();
    assert_eq!(labels.len(), 2, "{asm}");
    assert_ne!(labels[0], labels[1]);
}

#[test]
fn pub_inline_is_still_emitted() {
    let src = "@inline pub u8 g(u8 x) { return x + 3; }\nu8 f(u8 x) { return g(x) + 1; }";
    byte_both(src, &[1], 5);
    let result = ok(src, true);
    let code = result.code.as_ref().unwrap();
    assert!(code.function("g").is_some(), "pub @inline keeps its body");
    assert_eq!(execute_function(code, "g", &[4]).unwrap().return_byte(), 7);
}

#[test]
fn constant_arguments_fold() {
    let src = "@inline u8 pick(u8 x, bool hi) { if (hi) { return x + 10; } return x; }\n\
               u8 f(u8 a) { return pick(a, true) + pick(a, false); }";
    for a in [0u16, 7] {
        byte_both(src, &[a], (2 * a + 10) as u8);
    }
    let t = texts(&ok(src, true), "f");
    assert!(
        !t.iter()
            .any(|l| l.starts_with("JR") || l.starts_with("JP") || l.starts_with("CP")),
        "the branch on a constant is decided at compile time: {t:#?}"
    );
    // A constant parameter is an immediate operand.
    let src = "@inline u8 add(u8 a, u8 b) { return a + b; }\nu8 f(u8 a) { return add(a, 5); }";
    assert_eq!(texts(&ok(src, true), "f"), ["ADD A,5", "RET"]);
}

/// A `static` local is the function's own: one location for every expansion
/// and the out-of-line copy.
#[test]
fn static_local_is_shared() {
    let src = "@inline pub u8 tick() { static u8 n = 0; n++; return n; }\n\
               u8 f() { tick(); tick(); return tick(); }";
    for optimize in [true, false] {
        let result = ok(src, optimize);
        let code = result.code.as_ref().unwrap();
        let statics: Vec<&str> = code
            .globals
            .iter()
            .filter(|g| g.name.contains("tick") && g.name.contains('n'))
            .map(|g| g.label.as_str())
            .collect();
        assert_eq!(statics.len(), 1, "{:#?}", code.globals);
        let label = statics[0];
        let refs = |name: &str| {
            texts(&result, name)
                .iter()
                .filter(|t| t.contains(label))
                .count()
        };
        assert!(refs("f") >= 3, "every expansion uses {label}");
        assert!(refs("tick") >= 1, "the out-of-line copy uses {label}");
        assert!(!has_call(&result, "f"));
        assert_eq!(execute_function(code, "f", &[]).unwrap().return_byte(), 3);
    }
}

/// Strictness belongs to the function being lowered: an expansion inside a
/// strict caller is held to the caller's rule, and the message names both.
#[test]
fn strict_caller_names_the_inlined_callee() {
    let body = "u8 c = a + b; u8 d = c ^ a; u8 e = d + 1; u8 g = e & 0x3f; u8 h = g | b; \
                u8 i = h - c; u8 j = i + d + e; return a + b + c + d + e + g + h + i + j;";
    let src = format!(
        "@inline u8 busy(u8 a, u8 b) {{ {body} }}\n@fastcall u8 f(u8 a, u8 b) {{ return busy(a, b); }}"
    );
    let result = build(&src, true);
    let err = result
        .diagnostics
        .iter()
        .find(|d| d.is_error())
        .expect("strict caller rejects the spill");
    assert!(
        err.message.contains("(in 'busy', inlined into 'f')"),
        "{}",
        err.message
    );
    let call = src.find("busy(a, b);").unwrap() as u32;
    assert!(
        err.related
            .iter()
            .any(|r| r.span.start == call && r.message.contains("'busy' is inlined here (@inline)")),
        "{:#?}",
        err.related
    );
    // The same expansion in a default caller is fine.
    let src = format!(
        "@inline u8 busy(u8 a, u8 b) {{ {body} }}\nu8 f(u8 a, u8 b) {{ return busy(a, b); }}"
    );
    for optimize in [true, false] {
        let result = ok(&src, optimize);
        assert!(!has_call(&result, "f"));
    }
    // A strict @inline callee is checked on its own (out-of-line) lowering
    // even when every caller is default.
    let src = format!(
        "@inline @fastcall u8 busy(u8 a, u8 b) {{ {body} }}\nu8 f(u8 a, u8 b) {{ return busy(a, b); }}"
    );
    assert!(build(&src, true).has_errors());
}

#[test]
fn pin_conflicts_name_both() {
    let src = "@inline u8 g(u8 x) { u8 t @ c = x; return t + 1; }\n\
               u8 f(u8 a, u8 b) { u8 k @ c = b; u8 r = g(a); return r + k; }";
    let result = build(src, true);
    let err = result
        .diagnostics
        .iter()
        .find(|d| d.is_error())
        .expect("pin conflict");
    assert!(
        err.message.contains("cannot inline 'g' here"),
        "{}",
        err.message
    );
    assert!(err.message.contains("t @ c"), "{}", err.message);
    let call = src.find("g(a)").unwrap() as u32;
    assert_eq!(err.span.start, call);
    assert!(err.related.iter().any(|r| r.message.contains("pin in 'g'")));
    // The other side: the caller's pin, named in the message and located.
    assert!(err.message.contains("'k @ c' of 'f'"), "{}", err.message);
    let k_pin = src.find("@ c = b").unwrap() as u32;
    assert!(
        err.related.iter().any(|r| r.span.start == k_pin),
        "{:#?}",
        err.related
    );
    // Against the caller's @preserves.
    let src = "@inline u8 g(u8 x) { u8 t @ c = x; return t + 1; }\n\
               @preserves(c) u8 f(u8 a) { return g(a); }";
    let msgs = errors(src);
    assert!(
        msgs.iter().any(|m| m.contains("cannot inline 'g' here")),
        "{msgs:?}"
    );
    // No conflict: the pin applies inside the expansion only.
    let src = "@inline u8 g(u8 x) { u8 t @ c = x; return t + 1; }\n\
               u8 f(u8 a, u8 b) { u8 r = g(a); return r + b; }";
    byte_both(src, &[4, 10], 15);
}

/// A function that contains an expansion keeps its own contract: checked at
/// compile time, and by the harness guard at run time.
#[test]
fn contracts_of_functions_with_expansions() {
    let src = "@inline u8 g(u8 x) { return x + 1; }\n\
               @preserves(bc, de) u8 f(u8 x) { return g(g(x)); }";
    for optimize in [true, false] {
        let result = ok(src, optimize);
        let code = result.code.as_ref().unwrap();
        let f = code.function("f").unwrap();
        assert_eq!(f.contract, Contract::Declared);
        assert!(!has_call(&result, "f"));
        // The harness guard compares B C D E on entry and return.
        let exec = execute_function(code, "f", &[5]).unwrap();
        assert_eq!(exec.return_byte(), 7);
    }
    // An expansion that writes B breaks f's @preserves(b).
    let src = "@inline u8 g(u8 x) { u8 n @ b = x; u8 s = 0; while (n != 0) { s += 2; n--; } return s; }\n\
               @preserves(b) u8 f(u8 x) { return g(x); }";
    let msgs = errors(src);
    assert!(!msgs.is_empty(), "f's contract must still be checked");
    // A caller of f relies on f's inferred contract, which includes the
    // expansion's writes.
    let src = "@inline u8 g(u8 x) { u8 n @ b = x; u8 s = 0; while (n != 0) { s += 2; n--; } return s; }\n\
               @noinline u8 f(u8 x) { return g(x); }\n\
               u8 h(u8 x, u8 y) { u8 k @ b = y; u8 r = f(x); return r + k; }";
    for optimize in [true, false] {
        let result = ok(src, optimize);
        let code = result.code.as_ref().unwrap();
        assert!(!code.function("f").unwrap().preserves.contains(&R8::B));
        assert_eq!(
            execute_function(code, "h", &[3, 10]).unwrap().return_byte(),
            16
        );
    }
}

#[test]
fn stack_bound_drops_the_call() {
    let leaf = "u8 g(u8 x) { return x + 1; }\nu8 f(u8 x) { return g(x) + g(x); }";
    let called = ok(&leaf.replace("u8 g(", "@noinline u8 g("), true);
    let inlined = ok(&leaf.replace("u8 g(", "@inline u8 g("), true);
    let bound = |r: &CompilationResult| r.code.as_ref().unwrap().function("f").unwrap().stack_bound;
    assert_eq!(bound(&inlined) + 2, bound(&called));
    assert_eq!(bound(&inlined), 0);
}

#[test]
fn no_optimize_still_expands_every_call() {
    let src = "@inline u8 g(u8 x) { return x + 1; }\nu8 f(u8 x) { return g(x) + g(1); }";
    let result = ok(src, false);
    assert!(!has_call(&result, "f"));
    assert_eq!(
        execute_function(result.code.as_ref().unwrap(), "f", &[4])
            .unwrap()
            .return_byte(),
        7
    );
}

// --- Automatic inlining -------------------------------------------------

#[test]
fn auto_inlining_rules() {
    // Rule A: one call site; the body has no other caller and goes away.
    let src = "u8 g(u8 x) { return x + x + 1; }\nu8 f(u8 x) { return g(x); }";
    let opt = ok(src, true);
    let code = opt.code.as_ref().unwrap();
    assert!(!has_call(&opt, "f"));
    assert!(code.function("g").is_none(), "absorbed body is not emitted");
    let f = code.function("f").unwrap();
    assert_eq!(f.inlined.len(), 1);
    assert!(f.inlined[0].auto);
    assert!(code.assembly.contains("; inline g (auto)"));
    assert_eq!(execute_function(code, "f", &[4]).unwrap().return_byte(), 9);
    // The build that keeps bodies differs only by the extra body.
    let kept = compile_keeping_bodies(CompileInput {
        files: vec![SourceInput {
            name: "test.c80",
            text: src,
        }],
        origin: DEFAULT_CODE_ORIGIN,
        optimize: true,
        target: C80Target::GenericZ80,
    });
    let kept_code = kept.code.as_ref().unwrap();
    assert_eq!(
        execute_function(kept_code, "g", &[4])
            .unwrap()
            .return_byte(),
        9
    );
    assert_eq!(texts(&kept, "f"), texts(&opt, "f"));
    // Never in --no-optimize builds.
    let base = ok(src, false);
    assert!(has_call(&base, "f"));
    assert!(base.code.as_ref().unwrap().function("g").is_some());
    // Never `pub`, `@noinline`, or `@stackcall`.
    for src in [
        "pub u8 g(u8 x) { return x + x + 1; }\nu8 f(u8 x) { return g(x); }",
        "@noinline u8 g(u8 x) { return x + x + 1; }\nu8 f(u8 x) { return g(x); }",
        "@stackcall u8 g(u8 x) { return x + x + 1; }\nu8 f(u8 x) { return g(x); }",
    ] {
        let result = ok(src, true);
        assert!(has_call(&result, "f"), "{src}");
        assert!(result.code.as_ref().unwrap().function("g").is_some());
    }
    // Never a body with inline asm that calls.
    let src = "u8 g(u8 x) { u8 r; asm (in: a = x, out: a = r, clobber: bc, clobber: de, clobber: hl, clobber: flags, stack: 2) { call 0x1234 } return r; }\n\
               u8 f(u8 x) { return g(x); }";
    let result = ok(src, true);
    assert!(has_call(&result, "f"), "{:#?}", texts(&result, "f"));
    // Rule B: several sites, inlined where the caller does not grow.
    let src = "u8 id(u8 x) { return x; }\n\
               u8 f(u8 x) { return id(x); }\nu8 h(u8 x) { return id(x) + 1; }";
    let result = ok(src, true);
    assert!(!has_call(&result, "f") && !has_call(&result, "h"));
    assert!(result.code.as_ref().unwrap().function("id").is_none());
    // Rule B rejects a site that would grow: three uses of a 40-byte body.
    let big = "u8 big(u8 x) { u8 a = x + 1; u8 b = a ^ x; u8 c = b + a; u8 d = c & 0x3f; \
               return a + b + c + d; }\n";
    let src = format!(
        "{big}u8 f(u8 x) {{ return big(x); }}\nu8 h(u8 x) {{ return big(x) + 1; }}\n\
         u8 k(u8 x) {{ return big(x) + 2; }}"
    );
    let result = ok(&src, true);
    for name in ["f", "h", "k"] {
        assert!(
            has_call(&result, name),
            "{name}: {:#?}",
            texts(&result, name)
        );
    }
}

/// An automatic expansion that would make the caller spill is not taken.
#[test]
fn auto_inlining_never_adds_a_spill() {
    let body = "u8 c = a + b; u8 d = c ^ a; u8 e = d + 1; u8 g = e & 0x3f; u8 h = g | b; \
                u8 i = h - c; u8 j = i + d + e; return a + b + c + d + e + g + h + i + j;";
    let src = format!(
        "u8 busy(u8 a, u8 b) {{ {body} }}\n\
         u8 f(u8 a, u8 b, u8 x, u8 y) {{ u8 r = busy(a, b); return r + x + y; }}"
    );
    let result = ok(&src, true);
    let code = result.code.as_ref().unwrap();
    let f = code.function("f").unwrap();
    let noinline = ok(&src.replace("u8 busy(", "@noinline u8 busy("), true);
    let f0 = noinline.code.as_ref().unwrap().function("f").unwrap();
    let spill = |s: &crate::SpillReport| s.slot_bytes + s.saves.iter().sum::<u16>();
    assert!(
        spill(&f.spill) <= spill(&f0.spill),
        "auto-inlining added a spill: {:?} vs {:?}",
        f.spill,
        f0.spill
    );
    let want = {
        let (a, b, x, y) = (3u8, 9u8, 5u8, 7u8);
        let c = a.wrapping_add(b);
        let d = c ^ a;
        let e = d.wrapping_add(1);
        let g = e & 0x3f;
        let h = g | b;
        let i = h.wrapping_sub(c);
        let j = i.wrapping_add(d).wrapping_add(e);
        [a, b, c, d, e, g, h, i, j]
            .into_iter()
            .fold(0u8, u8::wrapping_add)
            .wrapping_add(x)
            .wrapping_add(y)
    };
    assert_eq!(
        execute_function(code, "f", &[3, 9, 5, 7])
            .unwrap()
            .return_byte(),
        want
    );
}

#[test]
fn auto_inlining_is_deterministic() {
    let src = "u8 a1(u8 x) { return x + 1; }\nu8 a2(u8 x) { return a1(x) + a1(x); }\n\
               u8 f(u8 x) { return a2(x) + a1(x); }\nu8 h(u8 x) { return a2(x); }";
    let one = ok(src, true);
    let two = ok(src, true);
    assert_eq!(
        one.code.as_ref().unwrap().assembly,
        two.code.as_ref().unwrap().assembly
    );
}
