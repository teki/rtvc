//! H5 code-quality table: the compiler against hand-written Z80.
//!
//! Each fixture is a C80 routine and a hand-written reference for it in
//! assembler source. Both run through the same harness
//! ([`execute_function_with`]) with the same arguments, globals and port
//! input; their results and their ordered source-visible effects (reads and
//! writes of globals, `static` locals and port I/O) must agree. The
//! reference uses the same calling convention and honours the same
//! language guarantee as the compiler: every source-level read and write of
//! a global happens, once, in source order.
//!
//! `cargo test -p rtvc-c80 --lib quality_table -- --nocapture` prints the
//! table; `C80_QUALITY_LISTINGS=1` adds each fixture's listing and
//! `C80_QUALITY_DUMP=1` the measured rows in [`BUDGET`] syntax. The test fails when a compiler row gets worse than its budget in
//! [`BUDGET`]; lower a budget deliberately when the compiler improves.

use super::harness::{AccessKind, ExecConfig, ExecResult, execute_function_with};
use super::z80::{GeneratedProgram, RegHome, Rr};
use super::*;
use rtvc_core::asm::assemble_program;

/// Fixture code and data sit below the TVC video RAM window (8000H-BBFFH)
/// that put_row and fill_rows write.
const FIXTURE_ORIGIN: u16 = 0x4000;

/// Where references are assembled: above every fixture's code and data,
/// below the harness stack.
const REF_ORIGIN: u16 = 0xC000;

struct Run {
    args: Vec<u16>,
    /// (global, byte offset, bytes) written before the call.
    mem: Vec<(&'static str, u16, Vec<u8>)>,
    ports: Vec<(u8, Vec<u8>)>,
}

fn run(args: &[u16]) -> Run {
    Run {
        args: args.to_vec(),
        mem: Vec::new(),
        ports: Vec::new(),
    }
}

impl Run {
    fn with(mut self, global: &'static str, offset: u16, bytes: &[u8]) -> Run {
        self.mem.push((global, offset, bytes.to_vec()));
        self
    }

    fn port(mut self, port: u8, bytes: &[u8]) -> Run {
        self.ports.push((port, bytes.to_vec()));
        self
    }
}

struct Fixture {
    name: &'static str,
    src: &'static str,
    func: &'static str,
    /// Hand-written reference; `None` for a compiler-only row.
    reference: Option<&'static str>,
    runs: Vec<Run>,
}

/// Compiler bytes and summed T-states per fixture, recorded deliberately.
/// A row may only move down.
const BUDGET: &[(&str, u16, u64)] = &[
    // BUDGET_BEGIN
    ("count", 14, 1595),
    ("walk", 16, 1564),
    ("fill", 14, 2601),
    ("put_row", 34, 302),
    ("fill_rows h=2", 45, 353),
    ("fill_rows h=24", 45, 3829),
    ("hits", 28, 393),
    ("step", 49, 573),
    ("bubble sort", 159, 8616),
    ("shift-add multiply", 80, 9178),
    ("sum (registers.c80)", 35, 1464),
    ("t[i] u8", 11, 114),
    ("t[i] u16", 15, 170),
    ("&&/|| condition", 21, 265),
    ("pong move_player", 123, 2190),
    // BUDGET_END
];

const COUNT: &str = "u8 count(u8 n) {\n\
    u8 s = 0;\n\
    while (n != 0) { s = s + 1; n = n - 1; }\n\
    return s;\n\
}\n";

const WALK: &str = "u8 walk(u8 n) {\n\
    u8 s = 0;\n\
    for (u8 i = 0; i < n; i = i + 1) { s = s + 1; }\n\
    return s;\n\
}\n";

const FILL: &str = "u8 buf[64];\n\
void fill(ptr<u8> p, u8 n, u8 v) {\n\
    while (n != 0) { *p = v; p = p + 1; n = n - 1; }\n\
}\n";

const PUT_ROW: &str = "u8 draw_col;\nu8 draw_val;\n\
void put_row(u8 y) {\n\
    u16 a;\n\
    ptr<u8> p;\n\
    a = u16(y) << 6;\n\
    p = ptr<u8>(a + 0x8000) + draw_col;\n\
    *p = draw_val;\n\
}\n";

const FILL_ROWS: &str = "u8 draw_col;\nu8 draw_val;\nu8 draw_h;\n\
@inline void put_row(u8 y) {\n\
    u16 a;\n\
    ptr<u8> p;\n\
    a = u16(y) << 6;\n\
    p = ptr<u8>(a + 0x8000) + draw_col;\n\
    *p = draw_val;\n\
}\n\
void fill_rows(u8 y) {\n\
    u8 n;\n\
    n = draw_h;\n\
    while (n != 0) { put_row(y); y = y + 1; n = n - 1; }\n\
}\n";

const HITS: &str = "u8 by;\n\
bool hits(u8 pad_y) {\n\
    if (by < pad_y + 24 && pad_y < by + 4) { return true; }\n\
    return false;\n\
}\n";

const STEP: &str = "u8 bx;\nu8 by;\ni8 vx;\ni8 vy;\n\
void step() {\n\
    bx = bx + u8(vx);\n\
    by = by + u8(vy);\n\
    if (by < 2 || by >= 234) { vy = -vy; }\n\
}\n";

const BUBBLE: &str = "u8 a[6] = {5, 3, 9, 1, 7, 2};\n\
void sort() {\n\
    for (u8 i = 0; i < 5; i++) {\n\
        for (u8 j = 0; j < 5 - i; j++) {\n\
            if (a[j] > a[j + 1]) { u8 t = a[j]; a[j] = a[j + 1]; a[j + 1] = t; }\n\
        }\n\
    }\n\
}\n";

const MUL: &str = "u16 mul(u16 a, u16 b) {\n\
    u16 r = 0;\n\
    while (b != 0) { if ((b & 1) != 0) { r += a; } a <<= 1; b >>= 1; }\n\
    return r;\n\
}\n";

const SUM: &str = "u8 data[8] = {0x80, 0x01, 0x80, 0x80, 1, 2, 3, 4};\n\
@fastcall u16 sum(ptr<u8> p, u8 n) {\n\
    static u8 left;\n\
    left = n;\n\
    u16 s @ de = 0;\n\
    while (left != 0) { s = s + u16(*p); p++; left--; }\n\
    return s;\n\
}\n";

const INDEX8: &str = "u8 t8[8] = {1, 2, 3, 4, 5, 6, 7, 8};\n\
u8 get8(u8 i) { return t8[i]; }\n";

const INDEX16: &str = "u16 t16[8] = {100, 200, 300, 400, 500, 600, 700, 800};\n\
u16 get16(u8 i) { return t16[i]; }\n";

const COND: &str = "u8 cond(u8 a, u8 b) {\n\
    if ((a < 10 && b != 0) || a == 99) { return 1; }\n\
    return 2;\n\
}\n";

const PONG: &str = include_str!("../../info/c80/pong/pong.c80");

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "count",
            src: COUNT,
            func: "count",
            reference: Some(REF_COUNT),
            runs: vec![run(&[0]), run(&[5]), run(&[50])],
        },
        Fixture {
            name: "walk",
            src: WALK,
            func: "walk",
            reference: Some(REF_WALK),
            runs: vec![run(&[0]), run(&[3]), run(&[50])],
        },
        Fixture {
            name: "fill",
            src: FILL,
            func: "fill",
            reference: Some(REF_FILL),
            runs: vec![
                run(&[g_placeholder(), 0, 7]),
                run(&[g_placeholder(), 3, 7]),
                run(&[g_placeholder(), 60, 0xAA]),
            ],
        },
        Fixture {
            name: "put_row",
            src: PUT_ROW,
            func: "put_row",
            reference: Some(REF_PUT_ROW),
            runs: vec![
                run(&[3])
                    .with("draw_col", 0, &[2])
                    .with("draw_val", 0, &[0xFF]),
                run(&[239])
                    .with("draw_col", 0, &[61])
                    .with("draw_val", 0, &[0x55]),
            ],
        },
        Fixture {
            name: "fill_rows h=2",
            src: FILL_ROWS,
            func: "fill_rows",
            reference: Some(REF_FILL_ROWS),
            runs: vec![
                run(&[10])
                    .with("draw_h", 0, &[2])
                    .with("draw_col", 0, &[2])
                    .with("draw_val", 0, &[0xFF]),
            ],
        },
        Fixture {
            name: "fill_rows h=24",
            src: FILL_ROWS,
            func: "fill_rows",
            reference: Some(REF_FILL_ROWS),
            runs: vec![
                run(&[100])
                    .with("draw_h", 0, &[24])
                    .with("draw_col", 0, &[61])
                    .with("draw_val", 0, &[0xFF]),
            ],
        },
        Fixture {
            name: "hits",
            src: HITS,
            func: "hits",
            reference: Some(REF_HITS),
            runs: vec![
                run(&[100]).with("by", 0, &[110]),
                run(&[100]).with("by", 0, &[124]),
                run(&[100]).with("by", 0, &[96]),
                run(&[100]).with("by", 0, &[97]),
            ],
        },
        Fixture {
            name: "step",
            src: STEP,
            func: "step",
            reference: Some(REF_STEP),
            runs: vec![
                ball(10, 100, 1, 1),
                ball(10, 2, 0xFF, 0xFF),
                ball(10, 233, 1, 1),
            ],
        },
        Fixture {
            name: "bubble sort",
            src: BUBBLE,
            func: "sort",
            reference: Some(REF_BUBBLE),
            runs: vec![run(&[])],
        },
        Fixture {
            name: "shift-add multiply",
            src: MUL,
            func: "mul",
            reference: Some(REF_MUL),
            runs: vec![
                run(&[0, 37]),
                run(&[123, 37]),
                run(&[0x1234, 37]),
                run(&[7, 0x8001]),
            ],
        },
        Fixture {
            name: "sum (registers.c80)",
            src: SUM,
            func: "sum",
            reference: Some(REF_SUM),
            runs: vec![
                run(&[g_placeholder(), 4]),
                run(&[g_placeholder(), 0]),
                run(&[g_placeholder(), 8]),
            ],
        },
        Fixture {
            name: "t[i] u8",
            src: INDEX8,
            func: "get8",
            reference: Some(REF_INDEX8),
            runs: vec![run(&[0]), run(&[5])],
        },
        Fixture {
            name: "t[i] u16",
            src: INDEX16,
            func: "get16",
            reference: Some(REF_INDEX16),
            runs: vec![run(&[0]), run(&[5])],
        },
        Fixture {
            name: "&&/|| condition",
            src: COND,
            func: "cond",
            reference: Some(REF_COND),
            runs: vec![run(&[5, 1]), run(&[5, 0]), run(&[20, 1]), run(&[99, 0])],
        },
        Fixture {
            name: "pong move_player",
            src: PONG,
            func: "move_player",
            reference: None,
            runs: vec![
                run(&[]).with("py", 0, &[10]).port(0x58, &[0xFD]),
                run(&[]).with("py", 0, &[10]).port(0x58, &[0xFB]),
                run(&[]).with("py", 0, &[10]).port(0x58, &[0xFF]),
            ],
        },
    ]
}

fn ball(bx: u8, by: u8, vx: u8, vy: u8) -> Run {
    run(&[])
        .with("bx", 0, &[bx])
        .with("by", 0, &[by])
        .with("vx", 0, &[vx])
        .with("vy", 0, &[vy])
}

/// Pointer arguments name the fixture's first array: replaced at run time.
const fn g_placeholder() -> u16 {
    0xFFFF
}

fn compile_fixture(src: &str) -> GeneratedProgram {
    compile_fixture_opt(src, true)
}

fn compile_fixture_opt(src: &str, optimize: bool) -> GeneratedProgram {
    let result = compile(CompileInput {
        files: vec![SourceInput {
            name: "quality.c80",
            text: src,
        }],
        origin: FIXTURE_ORIGIN,
        optimize,
        target: C80Target::GenericZ80,
    });
    assert!(!result.has_errors(), "{:#?}", result.diagnostics);
    result.code.unwrap()
}

/// The program with `func` replaced by the reference, which names every
/// global `G_<name>` (`EQU` lines are prepended; `.` and other
/// punctuation in a `static` local's name become `_`).
fn with_reference(code: &GeneratedProgram, func: &str, asm: &str) -> (GeneratedProgram, u16) {
    let mut src = String::new();
    for g in &code.globals {
        let name: String = g
            .name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        src.push_str(&format!("G_{name} EQU {}\n", g.addr));
    }
    src.push_str(&format!("ORG {REF_ORIGIN}\n"));
    src.push_str(asm);
    let assembled =
        assemble_program(&src, REF_ORIGIN).unwrap_or_else(|e| panic!("reference {func}: {e}"));
    let end = code
        .assembled
        .segments
        .iter()
        .map(|s| u32::from(s.addr) + s.bytes.len() as u32)
        .max()
        .unwrap_or(0);
    assert!(
        end <= u32::from(REF_ORIGIN),
        "fixture code reaches {end:04X}"
    );
    let mut out = code.clone();
    let mut bytes = 0u16;
    for seg in assembled.segments {
        bytes += seg.bytes.len() as u16;
        out.assembled.segments.push(seg);
    }
    let f = out.functions.iter_mut().find(|f| f.name == func).unwrap();
    f.addr = REF_ORIGIN;
    f.preserves.clear();
    (out, bytes)
}

/// Source-visible accesses in order, globals named so that builds with
/// different layouts compare: (kind, `name+offset` or address, value).
fn source_effects(code: &GeneratedProgram, exec: &ExecResult) -> Vec<(AccessKind, String, u8)> {
    exec.data_accesses()
        .filter_map(|a| {
            if matches!(a.kind, AccessKind::PortIn | AccessKind::PortOut) {
                return Some((a.kind, format!("port {:02X}", a.addr as u8), a.value));
            }
            if let Some(g) = code
                .globals
                .iter()
                .find(|g| a.addr >= g.addr && a.addr < g.addr.wrapping_add(g.size))
            {
                return Some((a.kind, format!("{}+{}", g.name, a.addr - g.addr), a.value));
            }
            (a.addr >= 0x8000 && a.addr < 0xBC00)
                .then(|| (a.kind, format!("{:04X}", a.addr), a.value))
        })
        .collect()
}

fn result_of(code: &GeneratedProgram, func: &str, exec: &ExecResult) -> Option<u16> {
    match code.function(func).unwrap().ret {
        Some(RegHome::Byte(_)) => Some(u16::from(exec.a)),
        Some(RegHome::Word(Rr::Hl)) => Some(exec.hl),
        Some(RegHome::Word(Rr::De)) => Some(exec.de),
        Some(RegHome::Word(Rr::Bc)) => Some(exec.bc),
        None => None,
    }
}

fn exec(code: &GeneratedProgram, func: &str, r: &Run) -> ExecResult {
    let first_array = code.globals.first().map(|g| g.addr).unwrap_or(0);
    let args: Vec<u16> = r
        .args
        .iter()
        .map(|&a| if a == g_placeholder() { first_array } else { a })
        .collect();
    let mut initial_mem = Vec::new();
    for (name, off, bytes) in &r.mem {
        let g = code
            .global(name)
            .unwrap_or_else(|| panic!("no global {name}"));
        for (k, b) in bytes.iter().enumerate() {
            initial_mem.push((g.addr + off + k as u16, *b));
        }
    }
    let config = ExecConfig {
        initial_mem,
        scripted_ports: r.ports.clone(),
        insn_limit: 200_000,
        tstate_limit: 2_000_000,
        ..ExecConfig::default()
    };
    execute_function_with(code, func, &args, &config).unwrap_or_else(|e| panic!("{func}: {e}"))
}

struct Row {
    name: &'static str,
    bytes: u16,
    tstates: u64,
    ref_bytes: Option<u16>,
    ref_tstates: Option<u64>,
}

fn measure(f: &Fixture) -> Row {
    let code = compile_fixture(f.src);
    let gf = code.function(f.func).unwrap();
    let bytes = gf.size;
    let mut tstates = 0;
    let mut results = Vec::new();
    for r in &f.runs {
        let e = exec(&code, f.func, r);
        tstates += e.tstates;
        results.push((
            result_of(&code, f.func, &e),
            source_effects(&code, &e),
            e.sp,
        ));
    }
    // The --no-optimize build: same results, same ordered effects.
    let base = compile_fixture_opt(f.src, false);
    for (r, want) in f.runs.iter().zip(&results) {
        let e = exec(&base, f.func, r);
        let got = (
            result_of(&base, f.func, &e),
            source_effects(&base, &e),
            e.sp,
        );
        assert_eq!(
            &got, want,
            "{}: --no-optimize differs for {:?}",
            f.name, r.args
        );
    }
    let (ref_bytes, ref_tstates) = match f.reference {
        Some(asm) => {
            let (rc, rb) = with_reference(&code, f.func, asm);
            let mut rt = 0;
            for (r, want) in f.runs.iter().zip(&results) {
                let e = exec(&rc, f.func, r);
                rt += e.tstates;
                let got = (
                    result_of(&code, f.func, &e),
                    source_effects(&code, &e),
                    e.sp,
                );
                assert_eq!(
                    got.0, want.0,
                    "{}: reference result differs for {:?}",
                    f.name, r.args
                );
                assert_eq!(
                    got.1, want.1,
                    "{}: reference effects differ for {:?}",
                    f.name, r.args
                );
                assert_eq!(got.2, want.2, "{}: reference SP differs", f.name);
            }
            (Some(rb), Some(rt))
        }
        None => (None, None),
    };
    Row {
        name: f.name,
        bytes,
        tstates,
        ref_bytes,
        ref_tstates,
    }
}

fn ratio(a: u64, b: Option<u64>) -> String {
    match b {
        Some(b) if b > 0 => format!("{:.2}", a as f64 / b as f64),
        _ => "-".to_string(),
    }
}

#[test]
fn quality_table() {
    let rows: Vec<Row> = fixtures().iter().map(measure).collect();
    println!(
        "{:<22} {:>6} {:>6} {:>5} {:>8} {:>8} {:>5}",
        "fixture", "bytes", "ref", "x", "T", "ref T", "x"
    );
    for r in &rows {
        println!(
            "{:<22} {:>6} {:>6} {:>5} {:>8} {:>8} {:>5}",
            r.name,
            r.bytes,
            r.ref_bytes.map_or("-".to_string(), |b| b.to_string()),
            ratio(u64::from(r.bytes), r.ref_bytes.map(u64::from)),
            r.tstates,
            r.ref_tstates.map_or("-".to_string(), |t| t.to_string()),
            ratio(r.tstates, r.ref_tstates),
        );
    }
    if std::env::var_os("C80_QUALITY_LISTINGS").is_some() {
        for f in fixtures() {
            let code = compile_fixture(f.src);
            let gf = code.function(f.func).unwrap();
            println!(
                "==== {} homes {:?} ret {:?}",
                f.name, gf.param_homes, gf.ret
            );
            for m in &gf.mapped {
                println!("    {}", m.text);
            }
        }
    }
    if std::env::var_os("C80_QUALITY_DUMP").is_some() {
        for r in &rows {
            println!("    (\"{}\", {}, {}),", r.name, r.bytes, r.tstates);
        }
    }
    let mut worse = Vec::new();
    for r in &rows {
        match BUDGET.iter().find(|(n, _, _)| *n == r.name) {
            Some(&(_, b, t)) => {
                if r.bytes > b || r.tstates > t {
                    worse.push(format!(
                        "{}: {} bytes / {} T, budget {b} / {t}",
                        r.name, r.bytes, r.tstates
                    ));
                }
            }
            None => worse.push(format!("{}: no budget row", r.name)),
        }
    }
    assert!(worse.is_empty(), "{}", worse.join("\n"));
}

// ---------------------------------------------------------------------------
// Hand-written references
// ---------------------------------------------------------------------------

// n in A, result in A. The loop is written as a loop: DJNZ with the
// zero-trip case handled first.
const REF_COUNT: &str = "
        OR A
        RET Z
        LD B,A
        XOR A
CL:     INC A
        DJNZ CL
        RET
";

const REF_WALK: &str = REF_COUNT;

// p in HL, n in A, v in C.
const REF_FILL: &str = "
        OR A
        RET Z
        LD B,A
FL:     LD (HL),C
        INC HL
        DJNZ FL
        RET
";

// y in A. y * 64 by two right rotations: H gets y >> 2, L gets (y & 3) << 6;
// + 8000H is an OR because y * 64 < 4000H. draw_col is added as a full byte.
const REF_PUT_ROW: &str = "
        RRCA
        RRCA
        LD H,A
        AND 0C0H
        LD L,A
        XOR H
        OR 80H
        LD H,A
        LD A,(G_draw_col)
        ADD A,L
        LD L,A
        ADC A,H
        SUB L
        LD H,A
        LD A,(G_draw_val)
        LD (HL),A
        RET
";

// y in A. draw_h is read once; draw_col and draw_val on every row, as the
// source does.
const REF_FILL_ROWS: &str = "
        LD C,A
        LD A,(G_draw_h)
        OR A
        RET Z
        LD B,A
RL:     LD A,C
        RRCA
        RRCA
        LD H,A
        AND 0C0H
        LD L,A
        XOR H
        OR 80H
        LD H,A
        LD A,(G_draw_col)
        ADD A,L
        LD L,A
        ADC A,H
        SUB L
        LD H,A
        LD A,(G_draw_val)
        LD (HL),A
        INC C
        DJNZ RL
        RET
";

// pad_y in A, bool in A. `by` is read once per comparison, as in the source.
const REF_HITS: &str = "
        LD C,A
        ADD A,24
        LD B,A
        LD A,(G_by)
        CP B
        JR NC,HF
        LD A,(G_by)
        ADD A,4
        LD B,A
        LD A,C
        CP B
        JR NC,HF
        LD A,1
        RET
HF:     XOR A
        RET
";

const REF_STEP: &str = "
        LD A,(G_bx)
        LD B,A
        LD A,(G_vx)
        ADD A,B
        LD (G_bx),A
        LD A,(G_by)
        LD B,A
        LD A,(G_vy)
        ADD A,B
        LD (G_by),A
        LD A,(G_by)
        CP 2
        JR C,SN
        LD A,(G_by)
        CP 234
        RET C
SN:     LD A,(G_vy)
        NEG
        LD (G_vy),A
        RET
";

// The inner loop walks HL along the array; the swap reads a[j] and
// a[j+1] again and writes both, in source order.
const REF_BUBBLE: &str = "
        LD C,5
BO:     LD HL,G_a
        LD B,C
BI:     LD A,(HL)
        INC HL
        CP (HL)
        JR Z,BN
        JR C,BN
        DEC HL
        LD E,(HL)
        INC HL
        LD A,(HL)
        DEC HL
        LD (HL),A
        INC HL
        LD (HL),E
BN:     DJNZ BI
        DEC C
        JR NZ,BO
        RET
";

// a in HL, b in DE, result in HL.
const REF_MUL: &str = "
        LD B,H
        LD C,L
        LD HL,0
ML:     LD A,D
        OR E
        RET Z
        SRL D
        RR E
        JR NC,MS
        ADD HL,BC
MS:     SLA C
        RL B
        JR ML
";

// p in HL, n in A, result in HL. `left` is a static: written once, then
// read for the test and read and written for `left--` on every pass.
const REF_SUM: &str = "
        LD (G_sum_left),A
        LD DE,0
SL:     LD A,(G_sum_left)
        OR A
        JR Z,SX
        LD A,(HL)
        ADD A,E
        LD E,A
        JR NC,SN2
        INC D
SN2:    INC HL
        LD A,(G_sum_left)
        DEC A
        LD (G_sum_left),A
        JR SL
SX:     EX DE,HL
        RET
";

// i in A, result in A.
const REF_INDEX8: &str = "
        LD HL,G_t8
        ADD A,L
        LD L,A
        ADC A,H
        SUB L
        LD H,A
        LD A,(HL)
        RET
";

// i in A, result in HL.
const REF_INDEX16: &str = "
        LD L,A
        LD H,0
        ADD HL,HL
        LD DE,G_t16
        ADD HL,DE
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        RET
";

// a in A, b in C, result in A.
const REF_COND: &str = "
        CP 10
        JR NC,C2
        INC C
        DEC C
        JR NZ,C1
C2:     CP 99
        LD A,2
        RET NZ
C1:     LD A,1
        RET
";
