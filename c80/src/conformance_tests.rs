//! H0 differential conformance suite (see `.agents/plans/c80-hardening.md`).
//!
//! Every case is compiled with `optimize` on and off, `f` is executed in the
//! bounded harness, and the result is compared with a Rust reference. Cases
//! that fail today are listed in [`KNOWN_GAPS`] (and, for the seeded random
//! expressions, in [`RANDOM_GAPS`]). A group test fails when an unlisted case
//! fails, when a listed case passes, and when it fails with another kind.
//!
//! Set `C80_CONFORMANCE_DUMP=1` to print the observed ledger lines for the
//! groups that run (for example after a compiler fix shrinks the list).

use super::*;
use crate::harness::{ExecConfig, execute_function_with};

// ---------------------------------------------------------------------------
// Ledger
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Pass,
    /// The compiler emits an error (no code).
    Rejected,
    /// Code is produced but returns a wrong value, clobbers IX/IY, or times out.
    WrongValue,
}

/// Why a case is in the ledger. For `Rejected` entries the first error
/// message must match the cause (checked); for `WrongValue` entries the cause
/// is the diagnosed root cause (documentation, see the findings file).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cause {
    // D4 / F-021 (`ShiftCodegen`, `ShiftType`): every shift case passes since
    // H3 and the causes are gone. Adding a `Rejected` gap back needs the
    // user's sign-off (2026-11-13).
    /// D5 / F-022: "register pressure". Every H0 pressure entry passes
    /// since H2a (spills to static slots, PUSH/POP around calls). The same
    /// sources under strict `@fastcall` are not gaps but rejections by
    /// design; they live in [`STRICT_REJECTIONS`], not in this ledger.
    #[allow(dead_code)]
    Pressure,
    // F-023 (`StaticAddr`): a global's address in a static initializer
    // passes since H3a (`DW label+offset`); the cause is gone. The ledger is
    // empty from here; adding a `Rejected` gap back needs the user's
    // sign-off (2026-11-14).
    // H0 also recorded `WrongValue` root causes, all fixed in H1 and removed
    // from the ledger (see c80-compiler-findings.md and the H1 ledger in
    // c80-hardening-progress.md):
    // - D1 / F-018: in-place ALU op overwrote a live local sharing `A`/`HL`;
    // - D2 / F-019: call spill slots collided;
    // - D3 / F-020: a loop-carried register local was not saved across a call;
    // - F-024: a signed byte compare read a stale scratch register after the
    //   right operand load relocated the flipped left operand;
    // - F-025: an in-place word op on `HL` destroyed a live byte view of `L`.
    // A new `WrongValue` entry needs a new root-cause variant here.
}

impl Cause {
    fn matches_rejection(self, message: &str) -> bool {
        match self {
            Self::Pressure => message.contains("register pressure"),
        }
    }
}

// Ledger entries are written as `("id", Rejected, Cause)`; kept for when
// one is added (with the user's sign-off) although the ledger is empty.
#[allow(unused_imports)]
use Cause::*;
#[allow(unused_imports)]
use Kind::Rejected;

/// Expected failures. An id without suffix applies to both optimize modes; an
/// `@opt` / `@base` suffix applies to one mode only and takes precedence.
const KNOWN_GAPS: &[(&str, Kind, Cause)] = &[
    // KNOWN_GAPS_BEGIN
    // KNOWN_GAPS_END
];

/// Seeded random-expression outcomes, one character per program index:
/// `.` pass, `R` rejected, `W` wrong value. Rows are 50 programs long.
const RANDOM_GAPS: &[(&str, &[&str])] = &[
    // RANDOM_GAPS_BEGIN
    // RANDOM_GAPS_END
];

fn expected_kind(id: &str, mode: &str) -> Option<(Kind, Cause)> {
    let suffixed = format!("{id}@{mode}");
    KNOWN_GAPS
        .iter()
        .find(|(gap, _, _)| *gap == suffixed)
        .or_else(|| KNOWN_GAPS.iter().find(|(gap, _, _)| *gap == id))
        .map(|&(_, kind, cause)| (kind, cause))
}

fn ledger_base(id: &str) -> &str {
    id.split('@').next().unwrap()
}

fn dump_enabled() -> bool {
    std::env::var_os("C80_CONFORMANCE_DUMP").is_some()
}

// ---------------------------------------------------------------------------
// Types and reference semantics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum T {
    Bool,
    U8,
    I8,
    U16,
    I16,
}

const INT_TYPES: [T; 4] = [T::U8, T::I8, T::U16, T::I16];
const ALL_TYPES: [T; 5] = [T::Bool, T::U8, T::I8, T::U16, T::I16];

const BYTE_VALUES: [u16; 12] = [0, 1, 2, 3, 7, 8, 0x55, 0x7F, 0x80, 0x81, 0xFE, 0xFF];
const WORD_VALUES: [u16; 14] = [
    0, 1, 2, 0xFF, 0x100, 0x101, 0x1234, 0x7FFF, 0x8000, 0x8001, 0xFF00, 0x00FF, 0xFFFE, 0xFFFF,
];
const SHAPE_BYTE_VALUES: [u16; 6] = [0, 1, 0x55, 0x7F, 0x80, 0xFF];
const SHAPE_WORD_VALUES: [u16; 6] = [0, 1, 0xFF, 0x1234, 0x8000, 0xFFFF];
const SHIFT_CONSTS: [u16; 11] = [0, 1, 3, 7, 8, 9, 15, 16, 17, 100, 255];

impl T {
    fn name(self) -> &'static str {
        match self {
            T::Bool => "bool",
            T::U8 => "u8",
            T::I8 => "i8",
            T::U16 => "u16",
            T::I16 => "i16",
        }
    }

    fn bits(self) -> u32 {
        match self {
            T::Bool | T::U8 | T::I8 => 8,
            T::U16 | T::I16 => 16,
        }
    }

    fn signed(self) -> bool {
        matches!(self, T::I8 | T::I16)
    }

    fn is_word(self) -> bool {
        self.bits() == 16
    }

    fn values(self) -> &'static [u16] {
        match self {
            T::Bool => &[0, 1],
            T::U8 | T::I8 => &BYTE_VALUES,
            T::U16 | T::I16 => &WORD_VALUES,
        }
    }

    fn shape_values(self) -> &'static [u16] {
        match self {
            T::Bool => &[0, 1],
            T::U8 | T::I8 => &SHAPE_BYTE_VALUES,
            T::U16 | T::I16 => &SHAPE_WORD_VALUES,
        }
    }

    fn mask(self) -> u16 {
        if self.is_word() { 0xFFFF } else { 0x00FF }
    }

    fn norm(self, v: u16) -> u16 {
        match self {
            T::Bool => u16::from(v != 0),
            _ => v & self.mask(),
        }
    }

    /// Interpreted value of normalized bits.
    fn sval(self, v: u16) -> i32 {
        match self {
            T::I8 => i32::from(v as u8 as i8),
            T::I16 => i32::from(v as i16),
            _ => i32::from(v),
        }
    }

    /// Source-text literal for normalized bits.
    fn lit(self, v: u16) -> String {
        match self {
            T::Bool => (if v != 0 { "true" } else { "false" }).to_string(),
            _ if self.signed() => {
                let s = self.sval(v);
                if s < 0 {
                    format!("({s})")
                } else {
                    s.to_string()
                }
            }
            _ => format!("0x{v:X}"),
        }
    }
}

fn cast(from: T, to: T, v: u16) -> u16 {
    if to == T::Bool {
        return u16::from(v != 0);
    }
    to.norm(from.sval(v) as u16)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

const ALU_OPS: [Op; 5] = [Op::Add, Op::Sub, Op::And, Op::Or, Op::Xor];
const CMP_OPS: [Op; 6] = [Op::Eq, Op::Ne, Op::Lt, Op::Le, Op::Gt, Op::Ge];

impl Op {
    fn sym(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::And => "&",
            Op::Or => "|",
            Op::Xor => "^",
            Op::Shl => "<<",
            Op::Shr => ">>",
            Op::Eq => "==",
            Op::Ne => "!=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Op::Add => "add",
            Op::Sub => "sub",
            Op::And => "and",
            Op::Or => "or",
            Op::Xor => "xor",
            Op::Shl => "shl",
            Op::Shr => "shr",
            Op::Eq => "eq",
            Op::Ne => "ne",
            Op::Lt => "lt",
            Op::Le => "le",
            Op::Gt => "gt",
            Op::Ge => "ge",
        }
    }

    fn is_cmp(self) -> bool {
        CMP_OPS.contains(&self)
    }

    /// Reference result. For shifts `b` is the u8 count; for comparisons the
    /// result is a bool (0/1).
    fn eval(self, t: T, a: u16, b: u16) -> u16 {
        let (sa, sb) = (t.sval(a), t.sval(b));
        let width = t.bits();
        match self {
            Op::Add => t.norm(a.wrapping_add(b)),
            Op::Sub => t.norm(a.wrapping_sub(b)),
            Op::And => t.norm(a & b),
            Op::Or => t.norm(a | b),
            Op::Xor => t.norm(a ^ b),
            Op::Shl => {
                let n = u32::from(b as u8);
                if n >= width { 0 } else { t.norm(a << n) }
            }
            Op::Shr => {
                let n = u32::from(b as u8);
                if t.signed() {
                    t.norm((sa >> n.min(width - 1)) as u16)
                } else if n >= width {
                    0
                } else {
                    t.norm(a >> n)
                }
            }
            Op::Eq => u16::from(sa == sb),
            Op::Ne => u16::from(sa != sb),
            Op::Lt => u16::from(sa < sb),
            Op::Le => u16::from(sa <= sb),
            Op::Gt => u16::from(sa > sb),
            Op::Ge => u16::from(sa >= sb),
        }
    }
}

// ---------------------------------------------------------------------------
// Case execution
// ---------------------------------------------------------------------------

struct Case {
    id: String,
    src: String,
    ret: T,
    /// (arguments in declaration order, expected normalized result)
    checks: Vec<(Vec<u16>, u16)>,
}

struct Outcome {
    kind: Kind,
    detail: String,
}

const MODES: [(&str, bool); 2] = [("opt", true), ("base", false)];

fn exec_config() -> ExecConfig {
    ExecConfig {
        insn_limit: 2_000_000,
        tstate_limit: 50_000_000,
        ..ExecConfig::default()
    }
}

fn run_mode(case: &Case, optimize: bool, config: &ExecConfig) -> Outcome {
    let result = compile(CompileInput {
        files: vec![SourceInput {
            name: "conformance.c80",
            text: &case.src,
        }],
        origin: 0x8000,
        optimize,
        target: C80Target::GenericZ80,
    });
    let Some(code) = result.code.as_ref().filter(|_| !result.has_errors()) else {
        let message = result
            .diagnostics
            .iter()
            .find(|d| d.is_error())
            .map(|d| format!("{}: {}", d.code.as_str(), d.message))
            .unwrap_or_else(|| "no code generated".to_string());
        return Outcome {
            kind: Kind::Rejected,
            detail: message,
        };
    };
    // H2a: IX is the `@stackcall` parameter frame and nothing else.
    for func in &code.functions {
        if func.conv != CallConv::Stack {
            assert!(
                func.mapped.iter().all(|m| !m
                    .text
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .any(|t| t == "IX")),
                "{}@{optimize}: unannotated '{}' uses IX\n{}",
                case.id,
                func.name,
                case.src
            );
        }
    }
    let entry = code.function("f");
    for (args, expected) in &case.checks {
        let exec = match execute_function_with(code, "f", args, config) {
            Ok(exec) => exec,
            Err(err) if err.0.contains("timed out") => {
                return Outcome {
                    kind: Kind::WrongValue,
                    detail: format!("f{args:04X?}: {err}"),
                };
            }
            Err(err) => panic!("{}: harness error {err}\n{}", case.id, case.src),
        };
        let got = if case.ret.is_word() {
            exec.return_word()
        } else {
            u16::from(exec.return_byte())
        };
        let mut problems = Vec::new();
        if exec.timed_out {
            problems.push("timed out".to_string());
        }
        if exec.ix != 0x1111 || exec.iy != 0x2222 {
            problems.push(format!("IX={:04X} IY={:04X}", exec.ix, exec.iy));
        }
        // A proven bound must cover the real high-water mark (pushes count,
        // static slots do not).
        if let Some(f) = entry {
            if f.stack_provenance == StackProvenance::Proven && exec.sp_used > f.stack_bound {
                problems.push(format!(
                    "stack used {} > proven bound {}",
                    exec.sp_used, f.stack_bound
                ));
            }
        }
        if got != *expected {
            problems.push(format!("got {got:#06X}, expected {expected:#06X}"));
        }
        if !problems.is_empty() {
            return Outcome {
                kind: Kind::WrongValue,
                detail: format!("f{args:04X?}: {}", problems.join(", ")),
            };
        }
    }
    Outcome {
        kind: Kind::Pass,
        detail: String::new(),
    }
}

fn run_case(case: &Case, config: &ExecConfig) -> [Outcome; 2] {
    MODES.map(|(_, optimize)| run_mode(case, optimize, config))
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Pass => "Pass",
        Kind::Rejected => "Rejected",
        Kind::WrongValue => "WrongValue",
    }
}

/// Run a group of cases whose ids all start with `prefix.` and check them
/// against `KNOWN_GAPS` exactly.
fn check_group(prefix: &str, cases: Vec<Case>) {
    let config = exec_config();
    let group = format!("{prefix}.");
    let mut errors = Vec::new();
    let mut dump = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for case in &cases {
        assert!(case.id.starts_with(&group), "{} not in {prefix}", case.id);
        assert!(
            seen.insert(case.id.clone()),
            "duplicate case id {}",
            case.id
        );
        let outcomes = run_case(case, &config);
        for ((mode, _), outcome) in MODES.iter().zip(outcomes.iter()) {
            let expected = expected_kind(&case.id, mode);
            let expected_kind = expected.map_or(Kind::Pass, |(k, _)| k);
            if outcome.kind != expected_kind {
                errors.push(format!(
                    "{}@{mode}: expected {}, got {} {}\n    {}",
                    case.id,
                    kind_name(expected_kind),
                    kind_name(outcome.kind),
                    outcome.detail,
                    case.src.replace('\n', "\n    ")
                ));
            } else if let Some((Kind::Rejected, cause)) = expected {
                if !cause.matches_rejection(&outcome.detail) {
                    errors.push(format!(
                        "{}@{mode}: rejected for another cause than {cause:?}: {}",
                        case.id, outcome.detail
                    ));
                }
            }
        }
        if dump_enabled() {
            let [opt, base] = &outcomes;
            if opt.kind == base.kind {
                if opt.kind != Kind::Pass {
                    dump.push(format!(
                        "(\"{}\", {}, ?), // {}",
                        case.id,
                        kind_name(opt.kind),
                        opt.detail
                    ));
                }
            } else {
                for ((mode, _), o) in MODES.iter().zip(outcomes.iter()) {
                    if o.kind != Kind::Pass {
                        dump.push(format!(
                            "(\"{}@{mode}\", {}, ?), // {}",
                            case.id,
                            kind_name(o.kind),
                            o.detail
                        ));
                    }
                }
            }
        }
    }
    for (gap, _, _) in KNOWN_GAPS {
        if gap.starts_with(&group) && !seen.contains(ledger_base(gap)) {
            errors.push(format!("stale KNOWN_GAPS entry {gap}: no such case"));
        }
    }
    if dump_enabled() && !dump.is_empty() {
        println!("// ---- {prefix}\n{}", dump.join("\n"));
    }
    assert!(
        errors.is_empty(),
        "{} conformance mismatch(es) in {prefix}:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Operator matrix
// ---------------------------------------------------------------------------

fn pairs(xs: &[u16], ys: &[u16]) -> Vec<(u16, u16)> {
    xs.iter()
        .flat_map(|&x| ys.iter().map(move |&y| (x, y)))
        .collect()
}

fn shift_counts() -> Vec<u16> {
    let mut counts: Vec<u16> = BYTE_VALUES.iter().copied().chain([9, 15, 16, 17]).collect();
    counts.sort_unstable();
    counts.dedup();
    counts
}

fn op_cases(t: T) -> Vec<Case> {
    let n = t.name();
    let vals = t.values();
    let mut cases = Vec::new();
    for op in ALU_OPS {
        cases.push(Case {
            id: format!("op.{n}.{}", op.name()),
            src: format!("{n} f({n} a, {n} b) {{ return a {} b; }}", op.sym()),
            ret: t,
            checks: pairs(vals, vals)
                .into_iter()
                .map(|(a, b)| (vec![a, b], op.eval(t, a, b)))
                .collect(),
        });
        cases.push(Case {
            id: format!("op.{n}.{}_assign", op.name()),
            src: format!("{n} f({n} a, {n} b) {{ a {}= b; return a; }}", op.sym()),
            ret: t,
            checks: pairs(vals, vals)
                .into_iter()
                .map(|(a, b)| (vec![a, b], op.eval(t, a, b)))
                .collect(),
        });
    }
    for op in [Op::Shl, Op::Shr] {
        cases.push(Case {
            id: format!("op.{n}.{}", op.name()),
            src: format!("{n} f({n} a, u8 b) {{ return a {} b; }}", op.sym()),
            ret: t,
            checks: pairs(vals, &shift_counts())
                .into_iter()
                .map(|(a, b)| (vec![a, b], op.eval(t, a, b)))
                .collect(),
        });
        cases.push(Case {
            id: format!("op.{n}.{}_assign", op.name()),
            src: format!("{n} f({n} a, u8 b) {{ a {}= b; return a; }}", op.sym()),
            ret: t,
            checks: pairs(vals, &shift_counts())
                .into_iter()
                .map(|(a, b)| (vec![a, b], op.eval(t, a, b)))
                .collect(),
        });
        for k in SHIFT_CONSTS {
            cases.push(Case {
                id: format!("op.{n}.{}.k{k}", op.name()),
                src: format!("{n} f({n} a) {{ return a {} {k}; }}", op.sym()),
                ret: t,
                checks: vals.iter().map(|&a| (vec![a], op.eval(t, a, k))).collect(),
            });
        }
    }
    for op in CMP_OPS {
        cases.push(Case {
            id: format!("op.{n}.{}", op.name()),
            src: format!("bool f({n} a, {n} b) {{ return a {} b; }}", op.sym()),
            ret: T::Bool,
            checks: pairs(vals, vals)
                .into_iter()
                .map(|(a, b)| (vec![a, b], op.eval(t, a, b)))
                .collect(),
        });
    }
    cases.push(Case {
        id: format!("op.{n}.neg"),
        src: format!("{n} f({n} a) {{ return -a; }}"),
        ret: t,
        checks: vals
            .iter()
            .map(|&a| (vec![a], t.norm(a.wrapping_neg())))
            .collect(),
    });
    cases.push(Case {
        id: format!("op.{n}.bitnot"),
        src: format!("{n} f({n} a) {{ return ~a; }}"),
        ret: t,
        checks: vals.iter().map(|&a| (vec![a], t.norm(!a))).collect(),
    });
    cases.push(Case {
        id: format!("op.{n}.not"),
        src: format!("bool f({n} a) {{ return !bool(a); }}"),
        ret: T::Bool,
        checks: vals.iter().map(|&a| (vec![a], u16::from(a == 0))).collect(),
    });
    cases.push(Case {
        id: format!("op.{n}.not_lt"),
        src: format!("bool f({n} a, {n} b) {{ return !(a < b); }}"),
        ret: T::Bool,
        checks: pairs(vals, vals)
            .into_iter()
            .map(|(a, b)| (vec![a, b], 1 - Op::Lt.eval(t, a, b)))
            .collect(),
    });
    cases
}

fn bool_cases() -> Vec<Case> {
    let vals = [0u16, 1];
    let two = |id: &str, src: &str, f: fn(u16, u16) -> u16| Case {
        id: format!("op.bool.{id}"),
        src: src.to_string(),
        ret: T::Bool,
        checks: pairs(&vals, &vals)
            .into_iter()
            .map(|(a, b)| (vec![a, b], f(a, b)))
            .collect(),
    };
    vec![
        Case {
            id: "op.bool.not".into(),
            src: "bool f(bool a) { return !a; }".into(),
            ret: T::Bool,
            checks: vec![(vec![0], 1), (vec![1], 0)],
        },
        two("eq", "bool f(bool a, bool b) { return a == b; }", |a, b| {
            u16::from(a == b)
        }),
        two("ne", "bool f(bool a, bool b) { return a != b; }", |a, b| {
            u16::from(a != b)
        }),
        two(
            "andand",
            "bool f(bool a, bool b) { return a && b; }",
            |a, b| a & b,
        ),
        two(
            "oror",
            "bool f(bool a, bool b) { return a || b; }",
            |a, b| a | b,
        ),
        two(
            "mixed",
            "bool f(bool a, bool b) { return (a || !b) && (a != b); }",
            |a, b| u16::from((a != 0 || b == 0) && a != b),
        ),
    ]
}

fn cast_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for from in ALL_TYPES {
        for to in ALL_TYPES {
            let (fname, tname) = (from.name(), to.name());
            cases.push(Case {
                id: format!("cast.{fname}.{tname}"),
                src: format!("{tname} f({fname} a) {{ return {tname}(a); }}"),
                ret: to,
                checks: from
                    .values()
                    .iter()
                    .map(|&a| (vec![a], cast(from, to, a)))
                    .collect(),
            });
        }
    }
    cases
}

#[test]
fn conformance_op_u8() {
    check_group("op.u8", op_cases(T::U8));
}

#[test]
fn conformance_op_i8() {
    check_group("op.i8", op_cases(T::I8));
}

#[test]
fn conformance_op_u16() {
    check_group("op.u16", op_cases(T::U16));
}

#[test]
fn conformance_op_i16() {
    check_group("op.i16", op_cases(T::I16));
}

#[test]
fn conformance_op_bool() {
    check_group("op.bool", bool_cases());
}

#[test]
fn conformance_casts() {
    check_group("cast", cast_cases());
}

// ---------------------------------------------------------------------------
// Shape matrix: operands stay live after in-place operations
// ---------------------------------------------------------------------------

fn shape_cases(t: T) -> Vec<Case> {
    let n = t.name();
    let vals = t.shape_values();
    let two = pairs(vals, vals);
    let (k1, k2) = (1u16, 2u16);
    let mut cases = Vec::new();
    for (i, op) in ALU_OPS.into_iter().enumerate() {
        for (j, op3) in ALU_OPS.into_iter().enumerate() {
            let op2 = ALU_OPS[(i + j) % ALU_OPS.len()];
            let tag = format!("{}.{}.{}", op.name(), op2.name(), op3.name());
            let (s1, s2, s3) = (op.sym(), op2.sym(), op3.sym());
            let combine = |a: u16, b: u16| op2.eval(t, op.eval(t, a, b), op3.eval(t, a, b));
            cases.push(Case {
                id: format!("shape.{n}.pair.{tag}"),
                src: format!("{n} f({n} a, {n} b) {{ return (a {s1} b) {s2} (a {s3} b); }}"),
                ret: t,
                checks: two
                    .iter()
                    .map(|&(a, b)| (vec![a, b], combine(a, b)))
                    .collect(),
            });
            cases.push(Case {
                id: format!("shape.{n}.locals.{tag}"),
                src: format!(
                    "{n} f({n} a, {n} b) {{ {n} s = a {s1} b; {n} t = a {s3} b; return s {s2} t; }}"
                ),
                ret: t,
                checks: two
                    .iter()
                    .map(|&(a, b)| (vec![a, b], combine(a, b)))
                    .collect(),
            });
            cases.push(Case {
                id: format!("shape.{n}.single.{tag}"),
                src: format!(
                    "{n} f({n} a) {{ {n} b = a {s1} {}; {n} c = a {s3} {}; return b {s2} c; }}",
                    t.lit(k1),
                    t.lit(k2)
                ),
                ret: t,
                checks: vals
                    .iter()
                    .map(|&a| (vec![a], op2.eval(t, op.eval(t, a, k1), op3.eval(t, a, k2))))
                    .collect(),
            });
        }
        for op2 in ALU_OPS {
            let (s1, s2) = (op.sym(), op2.sym());
            cases.push(Case {
                id: format!("shape.{n}.reuse.{}.{}", op.name(), op2.name()),
                src: format!(
                    "{n} f({n} a, {n} b) {{ {n} s = a {s1} b; return (s {s2} a) {s2} b; }}"
                ),
                ret: t,
                checks: two
                    .iter()
                    .map(|&(a, b)| {
                        let s = op.eval(t, a, b);
                        (vec![a, b], op2.eval(t, op2.eval(t, s, a), b))
                    })
                    .collect(),
            });
        }
        for cmp in CMP_OPS {
            cases.push(Case {
                id: format!("shape.{n}.cmp.{}.{}", op.name(), cmp.name()),
                src: format!(
                    "bool f({n} a, {n} b) {{ {n} s = a {} b; return s {} a; }}",
                    op.sym(),
                    cmp.sym()
                ),
                ret: T::Bool,
                checks: two
                    .iter()
                    .map(|&(a, b)| (vec![a, b], cmp.eval(t, op.eval(t, a, b), a)))
                    .collect(),
            });
        }
    }
    cases
}

#[test]
fn conformance_shape_u8() {
    check_group("shape.u8", shape_cases(T::U8));
}

#[test]
fn conformance_shape_i8() {
    check_group("shape.i8", shape_cases(T::I8));
}

#[test]
fn conformance_shape_u16() {
    check_group("shape.u16", shape_cases(T::U16));
}

#[test]
fn conformance_shape_i16() {
    check_group("shape.i16", shape_cases(T::I16));
}

// ---------------------------------------------------------------------------
// Program fixtures (plan Appendix A)
// ---------------------------------------------------------------------------

fn prog(id: &str, ret: T, src: &str, checks: Vec<(Vec<u16>, u16)>) -> Case {
    Case {
        id: format!("prog.{id}"),
        src: src.to_string(),
        ret,
        checks,
    }
}

fn nine_byte_locals(a: u8, b: u8) -> u8 {
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
}

fn seven_word_locals(a: u16, b: u16) -> u16 {
    let c = a.wrapping_add(b);
    let d = c ^ a;
    let e = d.wrapping_add(1);
    let g = e & 0x3fff;
    let h = g | b;
    let i = h.wrapping_sub(c);
    let j = i.wrapping_add(d).wrapping_add(e);
    [a, b, c, d, e, g, h, i, j]
        .into_iter()
        .fold(0u16, u16::wrapping_add)
}

fn fixture_cases() -> Vec<Case> {
    let byte_args: Vec<(u16, u16)> = pairs(&[0, 1, 0x37, 0x80, 0xFF], &[0, 2, 0x5A, 0xFF]);
    let word_args: Vec<(u16, u16)> =
        pairs(&[0, 1, 0x1234, 0x8000, 0xFFFF], &[0, 2, 0x5A5A, 0xFFFF]);
    let bubble = {
        let mut a = [5u8, 3, 9, 1, 7, 2];
        a.sort_unstable();
        u16::from(a[0].wrapping_add(a[5] << 4))
    };
    let walk_sum = |n: u16| (1..=n).sum::<u16>();
    vec![
        prog(
            "array_sum",
            T::U8,
            "u8 t[8] = {1,2,3,4,5,6,7,8};\n\
             u8 f() { u8 s = 0; for (u8 i = 0; i < 8; i++) { s += t[i]; } return s; }",
            vec![(vec![], 36)],
        ),
        prog(
            "u16_array_index",
            T::U16,
            "u16 t[5] = {100,200,300,400,500};\n\
             u16 f(u8 i) { return t[i] + t[i + 1]; }",
            vec![(vec![0], 300), (vec![2], 700), (vec![3], 900)],
        ),
        prog(
            "u16_array_store",
            T::U16,
            "u16 t[5];\n\
             u16 f(u8 i) { t[i] = 0x1234; t[i+1] = t[i] + 1; return t[i] + t[i+1]; }",
            vec![(vec![0], 0x2469), (vec![3], 0x2469)],
        ),
        prog(
            "struct_array",
            T::U16,
            "struct P { u8 x; u16 w; u8 y; };\n\
             P ps[4];\n\
             u16 f(u8 i) { ps[i].w = 1000; ps[i].x = 7; ps[i].y = 9; ps[1].w = 5;\n\
                           return ps[i].w + u16(ps[i].x) + u16(ps[i].y) + ps[1].w; }",
            vec![(vec![3], 1021), (vec![0], 1021), (vec![1], 26)],
        ),
        prog(
            "nine_byte_locals",
            T::U8,
            "u8 f(u8 a, u8 b) { u8 c = a + b; u8 d = c ^ a; u8 e = d + 1; u8 g = e & 0x3f;\n\
                               u8 h = g | b; u8 i = h - c; u8 j = i + d + e;\n\
                               return a + b + c + d + e + g + h + i + j; }",
            byte_args
                .iter()
                .map(|&(a, b)| (vec![a, b], u16::from(nine_byte_locals(a as u8, b as u8))))
                .collect(),
        ),
        prog(
            "seven_word_locals",
            T::U16,
            "u16 f(u16 a, u16 b) { u16 c = a + b; u16 d = c ^ a; u16 e = d + 1; u16 g = e & 0x3fff;\n\
                                 u16 h = g | b; u16 i = h - c; u16 j = i + d + e;\n\
                                 return a + b + c + d + e + g + h + i + j; }",
            word_args
                .iter()
                .map(|&(a, b)| (vec![a, b], seven_word_locals(a, b)))
                .collect(),
        ),
        prog(
            "nested_calls",
            T::U8,
            "u8 add(u8 a, u8 b) { return a + b; }\n\
             u8 sub(u8 a, u8 b) { return a - b; }\n\
             u8 f(u8 x, u8 y) { return add(sub(x, y), add(add(x, 1), sub(y, 2))); }",
            byte_args
                .iter()
                .map(|&(x, y)| {
                    let (x, y) = (x as u8, y as u8);
                    let v = x
                        .wrapping_sub(y)
                        .wrapping_add(x.wrapping_add(1).wrapping_add(y.wrapping_sub(2)));
                    (vec![u16::from(x), u16::from(y)], u16::from(v))
                })
                .chain([(vec![50, 8], 99)])
                .collect(),
        ),
        prog(
            "call_in_loop",
            T::U16,
            "u16 sq(u16 n) { u16 s = 0; u16 i = n; while (i != 0) { s += n; i -= 1; } return s; }\n\
             u16 f(u8 n) { u16 t = 0; for (u8 i = 1; i <= n; i++) { t += sq(u16(i)); } return t; }",
            vec![(vec![0], 0), (vec![1], 1), (vec![10], 385)],
        ),
        prog(
            "clobbering_callee_loop",
            T::U8,
            "u8 g(u8 x) { asm(inout: a = x, clobber: b, c, d, e, h, l, flags) { ld bc,0\n\
                          ld de,0\n\
                          ld hl,0\n\
                          inc a } return x; }\n\
             u8 f(u8 n) { u8 s = 0; while (n != 0) { n = n - 1; s = g(s); } return s; }",
            vec![(vec![0], 0), (vec![1], 1), (vec![5], 5)],
        ),
        prog(
            "clobbering_callee_loop_word",
            T::U16,
            "u8 g(u8 x) { asm(inout: a = x, clobber: b, c, d, e, h, l, flags) { ld bc,0\n\
                          ld de,0\n\
                          ld hl,0\n\
                          inc a } return x; }\n\
             u16 f(u16 n) { u16 s = 0; u8 k = 0; while (n != 0) { n = n - 1; k = g(k); s += 2; }\n\
                            return s + u16(k); }",
            vec![
                (vec![0], 0),
                (vec![1], 3),
                (vec![5], 15),
                // 300 iterations: s = 600, k = 300 mod 256 = 44.
                (vec![300], 600 + 44),
            ],
        ),
        prog(
            "stackcall5",
            T::U16,
            "@stackcall u16 m(u16 a, u16 b, u16 c, u16 d, u8 e) { return a + b - c + d + u16(e); }\n\
             u16 f(u16 x) { return m(x, 2, 3, x, 5); }",
            vec![(vec![100], 204), (vec![0], 4), (vec![0xFFFF], 2)],
        ),
        prog(
            "pointer_walk",
            T::U16,
            "u8 buf[16];\n\
             u16 f(u8 n) { ptr<u8> p = &buf[0]; u8 i = 0;\n\
                           while (i < n) { *p = i + 1; p += 1; i++; }\n\
                           u16 s = 0; for (u8 k = 0; k < n; k++) { s += u16(buf[k]); } return s; }",
            [0u16, 1, 5, 16]
                .into_iter()
                .map(|n| (vec![n], walk_sum(n)))
                .collect(),
        ),
        prog(
            "signed_loop",
            T::I16,
            "i16 f(i16 n) { i16 s = 0;\n\
                            for (i16 i = -n; i <= n; i += 1) { if (i < 0) { s -= i; } else { s += i; } }\n\
                            return s; }",
            vec![(vec![0], 0), (vec![1], 2), (vec![10], 110)],
        ),
        prog(
            "bubble_sort",
            T::U8,
            "u8 a[6] = {5,3,9,1,7,2};\n\
             u8 f() { for (u8 i = 0; i < 5; i++) { for (u8 j = 0; j < 5 - i; j++) {\n\
                        if (a[j] > a[j+1]) { u8 t = a[j]; a[j] = a[j+1]; a[j+1] = t; } } }\n\
                      return a[0] + (a[5] << 4); }",
            vec![(vec![], bubble)],
        ),
        prog(
            "bubble_sort_no_shift",
            T::U8,
            "u8 a[6] = {5,3,9,1,7,2};\n\
             u8 f() { for (u8 i = 0; i < 5; i++) { for (u8 j = 0; j < 5 - i; j++) {\n\
                        if (a[j] > a[j+1]) { u8 t = a[j]; a[j] = a[j+1]; a[j+1] = t; } } }\n\
                      return a[0] + a[1] + a[2] + a[3] + a[4] + a[5] + a[5] + a[0] + a[0]; }",
            vec![(vec![], 1 + 2 + 3 + 5 + 7 + 9 + 9 + 1 + 1)],
        ),
        prog(
            "shift_add_multiply",
            T::U16,
            "u16 mul(u16 a, u16 b) { u16 r = 0;\n\
                                     while (b != 0) { if ((b & 1) != 0) { r += a; } a <<= 1; b >>= 1; }\n\
                                     return r; }\n\
             u16 f(u16 x) { return mul(x, 37); }",
            [0u16, 1, 123, 0x1234]
                .into_iter()
                .map(|x| (vec![x], x.wrapping_mul(37)))
                .collect(),
        ),
        prog(
            "str_walk",
            T::U8,
            "str s = \"HELLO\";\n\
             u8 f() { u8 t = 0; for (u8 i = 0; i < s.len; i++) { t += s[i]; } return t; }",
            vec![(vec![], (72 + 69 + 76 + 76 + 79) & 0xFF)],
        ),
        // Minimal repros of the review defects and of findings made by this suite.
        prog(
            "d1_sum_and",
            T::U8,
            "u8 f(u8 a, u8 b) { u8 s = a + b; u8 t = a & b; return s - t; }",
            byte_args
                .iter()
                .map(|&(a, b)| (vec![a, b], (a + b - (a & b)) & 0xFF))
                .collect(),
        ),
        prog(
            "d1_inc_xor",
            T::U8,
            "u8 f(u8 a) { u8 b = a + 1; u8 c = a + 2; return b ^ c; }",
            [0u16, 1, 0x7F, 0xFF]
                .into_iter()
                .map(|a| (vec![a], ((a + 1) ^ (a + 2)) & 0xFF))
                .collect(),
        ),
        prog(
            "d1_signed_compare_local",
            T::U8,
            "u8 f(i8 a, i8 b) { bool x = a < b; return u8(a) + u8(x); }",
            vec![(vec![1, 2], 2), (vec![3, 0], 3), (vec![0xFF, 0], 0)],
        ),
        prog(
            "d3_loop_call",
            T::U8,
            "u8 g(u8 x) { asm(inout: a = x, clobber: b, c, d, e, h, l, flags) { ld bc,0 } return x; }\n\
             u8 f(u8 n) { u8 s = 0; while (n != 0) { n = n - 1; s = g(s) + 1; } return s; }",
            vec![(vec![0], 0), (vec![1], 1), (vec![3], 3)],
        ),
        prog(
            "d5_three_words",
            T::U16,
            "u16 f(u16 a, u16 b) { u16 c = a + b; return a + b + c; }",
            word_args
                .iter()
                .map(|&(a, b)| (vec![a, b], a.wrapping_add(b).wrapping_mul(2)))
                .collect(),
        ),
        prog(
            "f024_signed_compare_cast",
            T::Bool,
            "bool f(i8 a, i8 c) { return c >= i8(i16(a)); }",
            pairs(&BYTE_VALUES, &BYTE_VALUES)
                .into_iter()
                .map(|(a, c)| (vec![a, c], Op::Ge.eval(T::I8, c, a)))
                .collect(),
        ),
        prog(
            "f024_signed_compare_word_view",
            T::Bool,
            "bool f(i8 c, u16 w) { return c >= i8(w); }",
            pairs(&BYTE_VALUES, &WORD_VALUES)
                .into_iter()
                .map(|(c, w)| (vec![c, w], Op::Ge.eval(T::I8, c, w & 0xFF)))
                .collect(),
        ),
        prog(
            "d1_word_sum_and",
            T::U16,
            "u16 f(u16 a, u16 b) { u16 s = a + b; u16 t = a & b; return s - t; }",
            word_args
                .iter()
                .map(|&(a, b)| (vec![a, b], a.wrapping_add(b).wrapping_sub(a & b)))
                .collect(),
        ),
        prog(
            "d1_signed_word_compare_loop",
            T::I16,
            "i16 f(i16 a, i16 b) { i16 s = 0; while (a < b) { a += 1; s += 1; } return s; }",
            vec![
                (vec![1, 5], 4),
                (vec![0xFFFF, 1], 2),
                (vec![5, 1], 0),
                (vec![0x7FFE, 0x7FFF], 1),
            ],
        ),
        prog(
            "d1_store_over_copy",
            T::U8,
            "u8 f(u8 a) { u8 x = a; u8 y = x; x = x + 5; return y ^ x; }",
            [0u16, 1, 0x7F, 0xFB, 0xFF]
                .into_iter()
                .map(|a| (vec![a], (a ^ (a + 5)) & 0xFF))
                .collect(),
        ),
        prog(
            "d1_postfix_old_value",
            T::U8,
            "u8 f(u8 a) { u8 x = a; u8 y = x++; return y + x + x; }",
            [0u16, 1, 0x7F, 0xFF]
                .into_iter()
                .map(|a| (vec![a], (a + 2 * (a + 1)) & 0xFF))
                .collect(),
        ),
        prog(
            "d2_calls_keep_spills",
            T::U8,
            "u8 g(u8 x) { asm(inout: a = x, clobber: b, c, d, e, h, l, flags) { inc a } return x; }\n\
             u8 f(u8 a, u8 b, u8 c) { return g(a) + (g(b) + g(c)); }",
            vec![
                (vec![1, 2, 3], 9),
                (vec![0x10, 0x20, 0x40], 0x73),
                (vec![0xFF, 0, 0], 2),
            ],
        ),
        prog(
            "d3_loop_call_word",
            T::U16,
            "u8 g(u8 x) { asm(inout: a = x, clobber: b, c, d, e, h, l, flags) { ld bc,0 } return x; }\n\
             u16 f(u16 n) { u16 s = 0; while (n != 0) { n = n - 1; s = s + u16(g(1)); } return s; }",
            vec![(vec![0], 0), (vec![1], 1), (vec![3], 3), (vec![300], 300)],
        ),
        prog(
            "f026_overlapping_homes",
            T::U16,
            "u16 f(u16 a, u8 b) { u8 c = b; u16 d = a; u8 e = 0; while (e < b) { d += a; e += 1; } return d + u16(c); }",
            vec![(vec![3, 2], 11), (vec![1, 0], 1), (vec![0x100, 3], 0x403)],
        ),
        prog(
            "f026_tutorial_buf",
            T::I16,
            "pub u8 cells[16];\n\
             @fastcall pub i16 fill(i16 value) { u8 v; ptr<u8> p; u8 n; v = u8(value); p = &cells[0]; n = 16;\n\
               while (n != 0) { *p = v; p = p + 1; n = n - 1; } return value; }\n\
             pub i16 sum(i16 unused) { ptr<u8> p; u8 n; u16 s; p = &cells[0]; n = 16; s = 0;\n\
               while (n != 0) { s = s + u16(*p); p = p + 1; n = n - 1; } return i16(s); }\n\
             @fastcall pub i16 bump(i16 unused) { ptr<u8> p; u8 n; p = &cells[0]; n = 16;\n\
               while (n != 0) { *p = *p + 1; p = p + 1; n = n - 1; } return i16(cells[0]); }\n\
             i16 f(i16 v) { i16 r = fill(v); i16 b = bump(0); return sum(0) + (r - v) + (b - i16(u8(v + 1))); }",
            vec![
                (vec![3], 64),
                (vec![0xFF], 0),
                (vec![0x10], 272),
                (vec![0x1234], 0x35 * 16),
            ],
        ),
        prog(
            "f025_byte_view_then_inc",
            T::U8,
            "u8 f(u16 a) { u8 x = u8(a); u16 y = a + 1; return x + u8(y); }",
            WORD_VALUES
                .iter()
                .map(|&a| (vec![a], (a.wrapping_mul(2).wrapping_add(1)) & 0xFF))
                .collect(),
        ),
        prog(
            "f025_byte_view_compare",
            T::Bool,
            "bool f(u16 a) { return u8(a) < u8(a + 1); }",
            WORD_VALUES
                .iter()
                .map(|&a| (vec![a], u16::from((a & 0xFF) < (a.wrapping_add(1) & 0xFF))))
                .collect(),
        ),
        prog(
            "linked_struct_static",
            T::U8,
            "struct N { u8 v; ptr<N> next; };\n\
             N c = {3, ptr<N>(0)};\n\
             N b = {2, &c};\n\
             N a = {1, &b};\n\
             u8 f() { ptr<N> p = &a; u8 s = 0;\n\
                      while (u16(p) != 0) { s += p->v; p = p->next; } return s; }",
            vec![(vec![], 6)],
        ),
        prog(
            "linked_struct_runtime",
            T::U8,
            "struct N { u8 v; ptr<N> next; };\n\
             N a; N b; N c;\n\
             u8 f() { a.v = 1; b.v = 2; c.v = 3;\n\
                      a.next = &b; b.next = &c; c.next = ptr<N>(0);\n\
                      ptr<N> p = &a; u8 s = 0;\n\
                      while (u16(p) != 0) { s += p->v; p = p->next; } return s; }",
            vec![(vec![], 6)],
        ),
        // H2b: pins, `static` locals, strict `@fastcall`.
        prog(
            "h2b_pinned_loop",
            T::U16,
            "u16 f(u16 a, u8 n @ b) { u16 s @ hl = 0; while (n != 0) { s = s + a; n--; } return s; }",
            vec![
                (vec![3, 4], 12),
                (vec![0x1000, 0], 0),
                (vec![0x4000, 5], 0x4000),
            ],
        ),
        prog(
            "h2b_pin_across_call",
            T::U8,
            "u8 g(u8 v) { return v ^ 0x55; }\n\
             u8 f(u8 x) { u8 k @ e = x; u8 r = g(k); return r + k; }",
            vec![(vec![0], 0x55), (vec![0x10], 0x55), (vec![0xFF], 0xA9)],
        ),
        prog(
            "h2b_static_counter",
            T::U8,
            "u8 tick(u8 d) { static u8 n = 250; n += d; return n; }\n\
             u8 f(u8 d) { tick(d); tick(d); return tick(d); }",
            vec![(vec![1], 253), (vec![3], 3)],
        ),
        prog(
            "h2b_static_array",
            T::U16,
            "u16 f(u8 i) { static u16 t[4] = {1, 2, 3, 4}; t[i & 3] += 0x100;\n\
             ptr<u16> p = &t[0]; return *p + t[3]; }",
            vec![(vec![0], 0x105), (vec![3], 0x105), (vec![1], 5)],
        ),
        prog(
            "h2b_strict_leaf",
            T::U8,
            "@fastcall u8 mix(u8 a, u8 b) { u8 n @ d = 3; while (n != 0) { a = a + b; n--; } return a; }\n\
             u8 f(u8 a, u8 b) { return mix(a, b); }",
            vec![(vec![1, 2], 7), (vec![0xFF, 1], 2)],
        ),
        prog(
            "h2c_strict_counter_across_preserving_leaf",
            T::U8,
            "@preserves(bc) pub u8 inc(u8 v) { return v + 1; }\n\
             @fastcall u8 count(u8 n @ b) { u8 s = 0; while (n != 0) { s = inc(s); n--; } return s; }\n\
             u8 f(u8 n) { return count(n); }",
            vec![(vec![0], 0), (vec![7], 7), (vec![255], 255)],
        ),
        prog(
            "h2c_declared_saves_around_clobbering_call",
            T::U8,
            "pub u8 twice(u8 v) { return v + v; }\n\
             @preserves(bc, de) u8 wrap(u8 v) { return twice(v) + 1; }\n\
             u8 f(u8 x, u8 y) { u8 k @ c = y; u16 w @ de = u16(x); u8 r = wrap(x); return r + k + u8(w); }",
            vec![(vec![1, 2], 6), (vec![100, 3], 0x30)],
        ),
        prog(
            "h2c_inferred_private_chain",
            T::U16,
            "u16 dbl(u16 v) { return v + v; }\n\
             u16 quad(u16 v) { return dbl(dbl(v)); }\n\
             u16 f(u16 a, u16 b, u8 n) { u16 s = 0; while (n != 0) { s = s + quad(a) + b; n--; } return s; }",
            vec![
                (vec![1, 2, 3], 18),
                (vec![0x100, 1, 2], 0x802),
                (vec![5, 5, 0], 0),
            ],
        ),
        prog(
            "h2c_preserved_param_comes_back",
            T::U16,
            "@preserves(de) u16 add(u16 a, u16 b) { return a + b; }\n\
             u16 f(u16 a, u16 b) { u16 r = add(a, b); return r + b; }",
            vec![(vec![1, 2], 5), (vec![0xFFFF, 1], 1)],
        ),
        prog(
            "h2c_asm_clobber_in_inferred_leaf",
            T::U8,
            "u8 zap(u8 v) { asm (clobber: b) { LD B,0 } return v + 1; }\n\
             u8 f(u8 x, u8 y) { u8 k @ b = y; u8 c @ c = x; u8 r = zap(x); return r + k + c; }",
            vec![(vec![1, 2], 5), (vec![10, 20], 41)],
        ),
        // F-032: values computed before `&&`/`||` in the same expression.
        prog(
            "f032_call_arg_before_and",
            T::U8,
            "u8 g = 0;\n\
             void use(u8 x, bool t) { g = x + u8(t); }\n\
             u8 f(u8 a, u8 b) { use(a, b == 2 && a == 1); return g; }",
            vec![
                (vec![1, 2], 2),
                (vec![1, 3], 1),
                (vec![5, 2], 5),
                (vec![0xFF, 2], 0xFF),
            ],
        ),
        prog(
            "f032_binary_lhs_before_and",
            T::U8,
            "u8 f(u8 a, u8 b) { return a + u8(b == 2 && a == 1); }",
            vec![(vec![1, 2], 2), (vec![1, 3], 1), (vec![7, 2], 7)],
        ),
        prog(
            "f032_word_lhs_before_or",
            T::U16,
            "u16 f(u16 a, u16 b) { return (a ^ b) + u16(a == 0 || b > a); }",
            vec![
                (vec![0, 5], 6),
                (vec![3, 5], 7),
                (vec![5, 3], 6),
                (vec![0x8000, 0xFFFF], 0x7FFF + 1),
            ],
        ),
        prog(
            "f032_nested_logic_args",
            T::U8,
            "u8 g = 0;\n\
             void use3(u8 x, bool t, u8 y) { g = x + u8(t) + y; }\n\
             u8 f(u8 a, u8 b) { use3(a ^ b, (a < b && b != 0) || a == 9, b + 1); return g; }",
            vec![
                (vec![1, 2], 3 + 1 + 3),
                (vec![2, 1], 3 + 0 + 2),
                (vec![9, 1], 8 + 1 + 2),
                (vec![0, 0], 0 + 0 + 1),
            ],
        ),
    ]
}

#[test]
fn conformance_program_fixtures() {
    check_group("prog", fixture_cases());
}

fn assert_fixtures_pass(ids: &[&str]) {
    let config = exec_config();
    let cases = fixture_cases();
    for id in ids {
        let full = format!("prog.{id}");
        let case = cases
            .iter()
            .find(|c| c.id == full)
            .unwrap_or_else(|| panic!("no fixture {full}"));
        for ((mode, _), out) in MODES.iter().zip(run_case(case, &config)) {
            assert_eq!(
                out.kind,
                Kind::Pass,
                "{full}@{mode}: {}\n{}",
                out.detail,
                case.src
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Strict `@fastcall` rejections (H2b)
// ---------------------------------------------------------------------------

/// The fixtures that were rejected for register pressure until H2a. Each one
/// passes unannotated; with `@fastcall` on every function it must be
/// rejected (by design, not a gap) with the strict diagnostic, in both modes.
/// `u16_array_index` left the list in H3: index scaling became `ADD HL,HL`
/// (a shift) instead of an addition through a second pair, and it now fits.
const STRICT_REJECTIONS: &[&str] = &[
    "u16_array_store",
    "struct_array",
    "seven_word_locals",
    "call_in_loop",
    "signed_loop",
    "bubble_sort_no_shift",
    "linked_struct_runtime",
];

/// `src` with `@fastcall` before every function definition (a line that
/// starts with `[pub] type name(`) that has no attribute.
fn with_fastcall(src: &str) -> String {
    let is_def = |t: &str| {
        let t = t.strip_prefix("pub ").unwrap_or(t);
        let Some(rest) = ["u8 ", "i8 ", "u16 ", "i16 ", "bool ", "void "]
            .iter()
            .find_map(|ty| t.strip_prefix(ty))
            .or_else(|| {
                t.strip_prefix("ptr<")
                    .and_then(|r| r.split_once("> ").map(|x| x.1))
            })
        else {
            return false;
        };
        let name_len = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        name_len > 0 && rest[name_len..].starts_with('(')
    };
    let mut out = String::new();
    for line in src.lines() {
        let t = line.trim_start();
        if is_def(t) {
            out.push_str(&line[..line.len() - t.len()]);
            out.push_str("@fastcall ");
            out.push_str(t);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

#[test]
fn conformance_strict_fastcall_rejections() {
    let config = exec_config();
    let cases = fixture_cases();
    let mut errors = Vec::new();
    for id in STRICT_REJECTIONS {
        let full = format!("prog.{id}");
        let case = cases
            .iter()
            .find(|c| c.id == full)
            .unwrap_or_else(|| panic!("no fixture {full}"));
        let strict = Case {
            id: format!("strict.{id}"),
            src: with_fastcall(&case.src),
            ret: case.ret,
            checks: case.checks.clone(),
        };
        assert!(
            strict.src.contains("@fastcall"),
            "{full}: no function found"
        );
        for ((mode, _), (plain, strict_out)) in MODES.iter().zip(
            run_case(case, &config)
                .into_iter()
                .zip(run_case(&strict, &config)),
        ) {
            if plain.kind != Kind::Pass {
                errors.push(format!(
                    "{full}@{mode}: unannotated does not pass: {}",
                    plain.detail
                ));
            }
            if strict_out.kind != Kind::Rejected
                || !strict_out.detail.contains("must fit in registers")
            {
                errors.push(format!(
                    "strict.{id}@{mode}: expected the strict @fastcall rejection, got {} {}\n{}",
                    kind_name(strict_out.kind),
                    strict_out.detail,
                    strict.src
                ));
            }
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

// Named H1 regressions: each defect's repro must pass in both modes.

#[test]
fn h1_d1_byte_in_place_keeps_live_operand() {
    assert_fixtures_pass(&["d1_sum_and", "d1_inc_xor", "d1_signed_compare_local"]);
}

#[test]
fn h1_d1_word_in_place_keeps_live_operand() {
    assert_fixtures_pass(&[
        "d1_word_sum_and",
        "d5_three_words",
        "d1_signed_word_compare_loop",
    ]);
}

#[test]
fn h1_d1_store_keeps_old_copies() {
    assert_fixtures_pass(&["d1_store_over_copy", "d1_postfix_old_value"]);
}

#[test]
fn h1_d2_spill_slots_outlive_the_call() {
    assert_fixtures_pass(&["nested_calls", "d2_calls_keep_spills"]);
}

#[test]
fn h1_d3_byte_local_live_around_loop() {
    assert_fixtures_pass(&["d3_loop_call", "clobbering_callee_loop"]);
}

#[test]
fn h1_d3_word_counter_live_around_loop() {
    assert_fixtures_pass(&["d3_loop_call_word", "clobbering_callee_loop_word"]);
}

#[test]
fn h1_f024_signed_compare_biases_copies() {
    assert_fixtures_pass(&["f024_signed_compare_cast", "f024_signed_compare_word_view"]);
}

#[test]
fn h1_f025_byte_views_move_with_their_pair() {
    assert_fixtures_pass(&["f025_byte_view_then_inc", "f025_byte_view_compare"]);
}

#[test]
fn h1_f026_stable_homes_do_not_overlap() {
    assert_fixtures_pass(&["f026_overlapping_homes", "f026_tutorial_buf"]);
}

// ---------------------------------------------------------------------------
// Seeded random expressions
// ---------------------------------------------------------------------------

/// Numerical Recipes LCG; the high bits are used.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) as u32
    }

    fn below(&mut self, n: u32) -> u32 {
        self.next() % n
    }

    fn pick<X: Copy>(&mut self, xs: &[X]) -> X {
        xs[self.below(xs.len() as u32) as usize]
    }
}

#[derive(Debug, Clone)]
enum E {
    Var(usize),
    Lit(u16),
    Neg(Box<E>),
    Not(Box<E>),
    Bin(Op, Box<E>, Box<E>),
    /// `T(l cmp r)`
    Cmp(Op, Box<E>, Box<E>),
    /// `T(U(l) op U(r))`; `op` may be a comparison.
    Wide(T, Op, Box<E>, Box<E>),
    /// `l << count` / `l >> count`.
    Shift(Op, Box<E>, Box<Count>),
    /// `T((l != 0) && (r != 0))` (`||` when the flag is set): a
    /// short-circuit inside a larger expression, so values computed before
    /// it are live across its blocks (F-032).
    Logic(bool, Box<E>, Box<E>),
}

#[derive(Debug, Clone)]
enum Count {
    Lit(u16),
    /// `(u8(e) & 15)`
    Masked(E),
}

const VAR_NAMES: [&str; 4] = ["a", "b", "c", "d"];

impl E {
    fn has_var(&self) -> bool {
        match self {
            E::Var(_) => true,
            E::Lit(_) => false,
            E::Neg(x) | E::Not(x) => x.has_var(),
            E::Bin(_, l, r) | E::Cmp(_, l, r) | E::Wide(_, _, l, r) | E::Logic(_, l, r) => {
                l.has_var() || r.has_var()
            }
            E::Shift(_, l, _) => l.has_var(),
        }
    }

    fn render(&self, t: T) -> String {
        let n = t.name();
        match self {
            E::Var(i) => VAR_NAMES[*i].to_string(),
            E::Lit(v) => t.lit(*v),
            E::Neg(x) => format!("-({})", x.render(t)),
            E::Not(x) => format!("~({})", x.render(t)),
            E::Bin(op, l, r) => format!("({} {} {})", l.render(t), op.sym(), r.render(t)),
            E::Cmp(op, l, r) => format!("{n}({} {} {})", l.render(t), op.sym(), r.render(t)),
            E::Wide(u, op, l, r) => {
                let un = u.name();
                format!(
                    "{n}({un}({}) {} {un}({}))",
                    l.render(t),
                    op.sym(),
                    r.render(t)
                )
            }
            E::Shift(op, l, count) => {
                let c = match count.as_ref() {
                    Count::Lit(k) => k.to_string(),
                    Count::Masked(e) => format!("(u8({}) & 15)", e.render(t)),
                };
                format!("({} {} {c})", l.render(t), op.sym())
            }
            E::Logic(or, l, r) => {
                let z = t.lit(0);
                format!(
                    "{n}(({} != {z}) {} ({} != {z}))",
                    l.render(t),
                    if *or { "||" } else { "&&" },
                    r.render(t)
                )
            }
        }
    }

    fn eval(&self, t: T, env: &[u16]) -> u16 {
        match self {
            E::Var(i) => env[*i],
            E::Lit(v) => *v,
            E::Neg(x) => t.norm(x.eval(t, env).wrapping_neg()),
            E::Not(x) => t.norm(!x.eval(t, env)),
            E::Bin(op, l, r) => op.eval(t, l.eval(t, env), r.eval(t, env)),
            E::Cmp(op, l, r) => cast(T::Bool, t, op.eval(t, l.eval(t, env), r.eval(t, env))),
            E::Wide(u, op, l, r) => {
                let a = cast(t, *u, l.eval(t, env));
                let b = cast(t, *u, r.eval(t, env));
                let v = op.eval(*u, a, b);
                if op.is_cmp() {
                    cast(T::Bool, t, v)
                } else {
                    cast(*u, t, v)
                }
            }
            E::Shift(op, l, count) => {
                let k = match count.as_ref() {
                    Count::Lit(k) => *k,
                    Count::Masked(e) => cast(t, T::U8, e.eval(t, env)) & 15,
                };
                op.eval(t, l.eval(t, env), k)
            }
            E::Logic(or, l, r) => {
                let (a, b) = (l.eval(t, env) != 0, r.eval(t, env) != 0);
                u16::from(if *or { a || b } else { a && b })
            }
        }
    }
}

struct Gen<'a> {
    rng: &'a mut Lcg,
    depth: u32,
    t: T,
    vars: usize,
    shifts: bool,
    logic: bool,
}

impl Gen<'_> {
    fn var(&mut self) -> E {
        E::Var(self.rng.below(self.vars as u32) as usize)
    }

    fn lit(&mut self) -> E {
        let v = if self.rng.below(2) == 0 {
            self.rng.pick(self.t.values())
        } else {
            u16::from(self.rng.pick(&[1u8, 2, 3, 4, 5, 15, 16, 0x3F, 0x40]))
        };
        E::Lit(self.t.norm(v))
    }

    fn with_var(&mut self, e: E) -> E {
        if e.has_var() { e } else { self.var() }
    }

    fn expr(&mut self, depth: u32) -> E {
        if depth == 0 || self.rng.below(5) == 0 {
            return if self.rng.below(4) == 0 {
                self.lit()
            } else {
                self.var()
            };
        }
        let roll = self.rng.below(if self.shifts { 20 } else { 16 });
        if self.logic && self.rng.below(3) == 0 {
            let l = self.expr(depth - 1);
            let r = self.expr(depth - 1);
            let r = if l.has_var() { r } else { self.with_var(r) };
            let logic = E::Logic(self.rng.below(2) == 0, Box::new(l), Box::new(r));
            // Usually as the right operand, with a value computed before it.
            return if self.rng.below(4) == 0 {
                logic
            } else {
                let op = self.rng.pick(&ALU_OPS);
                let first = self.expr(depth - 1);
                let first = self.with_var(first);
                E::Bin(op, Box::new(first), Box::new(logic))
            };
        }
        match roll {
            0 => {
                let x = self.expr(depth - 1);
                E::Neg(Box::new(self.with_var(x)))
            }
            1 => {
                let x = self.expr(depth - 1);
                E::Not(Box::new(self.with_var(x)))
            }
            2..=9 => {
                let op = self.rng.pick(&ALU_OPS);
                let l = self.expr(depth - 1);
                let r = self.expr(depth - 1);
                let r = if l.has_var() { r } else { self.with_var(r) };
                E::Bin(op, Box::new(l), Box::new(r))
            }
            10 | 11 => {
                let op = self.rng.pick(&CMP_OPS);
                let l = self.expr(depth - 1);
                let r = self.expr(depth - 1);
                let r = if l.has_var() { r } else { self.with_var(r) };
                E::Cmp(op, Box::new(l), Box::new(r))
            }
            12..=15 => {
                let u = self.rng.pick(&INT_TYPES);
                let ops: &[Op] = if self.shifts {
                    &[Op::Add, Op::Sub, Op::And, Op::Xor, Op::Lt, Op::Ge, Op::Ne]
                } else {
                    &[
                        Op::Add,
                        Op::Sub,
                        Op::And,
                        Op::Or,
                        Op::Xor,
                        Op::Lt,
                        Op::Ge,
                        Op::Ne,
                    ]
                };
                let op = self.rng.pick(ops);
                let l = self.expr(depth - 1);
                let l = self.with_var(l);
                let r = self.expr(depth - 1);
                let r = self.with_var(r);
                E::Wide(u, op, Box::new(l), Box::new(r))
            }
            _ => {
                let op = self.rng.pick(&[Op::Shl, Op::Shr]);
                let l = self.expr(depth - 1);
                let l = self.with_var(l);
                let count = if self.rng.below(3) == 0 {
                    let e = self.expr(depth - 1);
                    Count::Masked(self.with_var(e))
                } else {
                    Count::Lit(u16::from(
                        self.rng.pick(&[0u8, 1, 2, 3, 4, 7, 8, 9, 15, 16, 17]),
                    ))
                };
                E::Shift(op, Box::new(l), Box::new(count))
            }
        }
    }
}

struct RandomProgram {
    src: String,
    exprs: [E; 3],
}

fn random_program(rng: &mut Lcg, t: T, family: &Family) -> RandomProgram {
    let (shifts, depth, logic) = (family.shifts, family.depth, family.logic);
    let gen_expr = |vars: usize, rng: &mut Lcg| {
        let mut g = Gen {
            rng,
            depth,
            t,
            vars,
            shifts,
            logic,
        };
        let e = g.expr(g.depth);
        g.with_var(e)
    };
    let e1 = gen_expr(2, rng);
    let e2 = gen_expr(3, rng);
    let e3 = gen_expr(4, rng);
    let n = t.name();
    let src = format!(
        "{n} f({n} a, {n} b) {{\n    {n} c = {};\n    {n} d = {};\n    return {};\n}}",
        e1.render(t),
        e2.render(t),
        e3.render(t)
    );
    RandomProgram {
        src,
        exprs: [e1, e2, e3],
    }
}

fn random_case(id: String, rng: &mut Lcg, t: T, family: &Family) -> Case {
    let p = random_program(rng, t, family);
    let vals = t.values();
    let mut args: Vec<(u16, u16)> = vec![
        (vals[1], vals[2]),
        (vals[vals.len() - 1], vals[0]),
        (vals[vals.len() / 2], vals[vals.len() - 1]),
    ];
    for _ in 0..3 {
        args.push((rng.pick(vals), rng.pick(vals)));
    }
    let checks = args
        .into_iter()
        .map(|(a, b)| {
            let mut env = vec![a, b];
            let c = p.exprs[0].eval(t, &env);
            env.push(c);
            let d = p.exprs[1].eval(t, &env);
            env.push(d);
            (vec![a, b], p.exprs[2].eval(t, &env))
        })
        .collect();
    Case {
        id,
        src: p.src,
        ret: t,
        checks,
    }
}

const RANDOM_ROW: usize = 50;

fn check_random(family: &Family, t: T) {
    let config = exec_config();
    let group = format!("{}.{}", family.name, t.name());
    let count = family.count;
    let type_index = INT_TYPES.iter().position(|&x| x == t).unwrap() as u64;
    let mut rng = Lcg(family.seed + type_index);
    let mut observed = [String::new(), String::new()];
    let mut reasons: std::collections::BTreeMap<String, usize> = Default::default();
    let mut details = Vec::new();
    for index in 0..count {
        let case = random_case(format!("{group}.{index}"), &mut rng, t, family);
        let outcomes = run_case(&case, &config);
        for (slot, outcome) in outcomes.iter().enumerate() {
            observed[slot].push(match outcome.kind {
                Kind::Pass => '.',
                Kind::Rejected => 'R',
                Kind::WrongValue => 'W',
            });
            if outcome.kind != Kind::Pass {
                let reason = match outcome.kind {
                    Kind::Rejected => outcome.detail.clone(),
                    _ => "wrong value".to_string(),
                };
                *reasons
                    .entry(format!("{}: {reason}", MODES[slot].0))
                    .or_default() += 1;
                details.push(format!(
                    "{}@{}: {} {}\n{}",
                    case.id,
                    MODES[slot].0,
                    kind_name(outcome.kind),
                    outcome.detail,
                    case.src
                ));
            }
        }
    }
    let mut errors = Vec::new();
    for (slot, (mode, _)) in MODES.iter().enumerate() {
        let key = format!("{group}@{mode}");
        let expected: String = RANDOM_GAPS
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, rows)| rows.concat())
            .unwrap_or_else(|| ".".repeat(count));
        if expected != observed[slot] {
            let diffs: Vec<String> = expected
                .chars()
                .zip(observed[slot].chars())
                .enumerate()
                .filter(|(_, (e, o))| e != o)
                .map(|(i, (e, o))| format!("{i}: expected {e} got {o}"))
                .collect();
            errors.push(format!(
                "{key}: ledger mismatch (len {} vs {}): {}",
                expected.len(),
                observed[slot].len(),
                diffs.join(", ")
            ));
        }
    }
    if dump_enabled() {
        let mut out = format!("// ---- {group}\n");
        for (slot, (mode, _)) in MODES.iter().enumerate() {
            out.push_str(&format!("(\"{group}@{mode}\", &[\n"));
            for row in observed[slot].as_bytes().chunks(RANDOM_ROW) {
                out.push_str(&format!("    \"{}\",\n", std::str::from_utf8(row).unwrap()));
            }
            out.push_str("]),\n");
        }
        for (reason, n) in &reasons {
            out.push_str(&format!("// {n:4} x {reason}\n"));
        }
        if std::env::var_os("C80_CONFORMANCE_DUMP_DETAILS").is_some() {
            out.push_str(&details.join("\n"));
        }
        println!("{out}");
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

/// A random-expression family: `count` programs per integer type, expression
/// trees of at most `depth` levels, seeded with `seed + type index`.
struct Family {
    name: &'static str,
    depth: u32,
    shifts: bool,
    /// Short-circuit nodes with live outer values (F-032).
    logic: bool,
    count: usize,
    seed: u64,
}

/// `rand` is the plan's depth-4 family. Most of it is rejected for register
/// pressure until H2, so `rsmall` (depth 2) keeps miscompile coverage today.
/// `rshift` adds shift nodes (all rejected until H3).
const FAMILIES: [Family; 4] = [
    Family {
        name: "rand",
        depth: 4,
        shifts: false,
        logic: false,
        count: 300,
        seed: 0xC80_0100,
    },
    Family {
        name: "rsmall",
        depth: 2,
        shifts: false,
        logic: false,
        count: 300,
        seed: 0xC80_0200,
    },
    Family {
        name: "rshift",
        depth: 3,
        shifts: true,
        logic: false,
        count: 100,
        seed: 0xC80_0300,
    },
    Family {
        name: "rlogic",
        depth: 3,
        shifts: false,
        logic: true,
        count: 150,
        seed: 0xC80_0400,
    },
];

fn family(name: &str) -> &'static Family {
    FAMILIES.iter().find(|f| f.name == name).unwrap()
}

#[test]
fn conformance_random_u8() {
    check_random(family("rand"), T::U8);
}

#[test]
fn conformance_random_i8() {
    check_random(family("rand"), T::I8);
}

#[test]
fn conformance_random_u16() {
    check_random(family("rand"), T::U16);
}

#[test]
fn conformance_random_i16() {
    check_random(family("rand"), T::I16);
}

#[test]
fn conformance_random_small_u8() {
    check_random(family("rsmall"), T::U8);
}

#[test]
fn conformance_random_small_i8() {
    check_random(family("rsmall"), T::I8);
}

#[test]
fn conformance_random_small_u16() {
    check_random(family("rsmall"), T::U16);
}

#[test]
fn conformance_random_small_i16() {
    check_random(family("rsmall"), T::I16);
}

#[test]
fn conformance_random_shift_u8() {
    check_random(family("rshift"), T::U8);
}

#[test]
fn conformance_random_shift_i8() {
    check_random(family("rshift"), T::I8);
}

#[test]
fn conformance_random_shift_u16() {
    check_random(family("rshift"), T::U16);
}

#[test]
fn conformance_random_shift_i16() {
    check_random(family("rshift"), T::I16);
}

#[test]
fn conformance_random_logic_u8() {
    check_random(family("rlogic"), T::U8);
}

#[test]
fn conformance_random_logic_i8() {
    check_random(family("rlogic"), T::I8);
}

#[test]
fn conformance_random_logic_u16() {
    check_random(family("rlogic"), T::U16);
}

#[test]
fn conformance_random_logic_i16() {
    check_random(family("rlogic"), T::I16);
}

/// The ledger itself must be well formed: unique ids, known groups, and
/// random rows of the right length.
#[test]
fn conformance_ledger_is_well_formed() {
    let mut seen = std::collections::HashSet::new();
    for (id, kind, cause) in KNOWN_GAPS {
        assert!(seen.insert(*id), "duplicate KNOWN_GAPS id {id}");
        assert_ne!(*kind, Kind::Pass, "{id}");
        let base = ledger_base(id);
        assert!(
            id.len() == base.len() || id.ends_with("@opt") || id.ends_with("@base"),
            "bad mode suffix in {id}"
        );
        assert!(
            ["op.", "cast.", "shape.", "prog."]
                .iter()
                .any(|p| base.starts_with(p)),
            "{id} is not in a known group"
        );
        // Every current cause is a rejection; a `WrongValue` entry needs a
        // root-cause variant first (see `Cause`).
        assert!(
            *kind != Kind::WrongValue,
            "{id}: WrongValue entries need a root-cause variant, not {cause:?}"
        );
    }
    for (key, rows) in RANDOM_GAPS {
        let len: usize = rows.iter().map(|r| r.len()).sum();
        let name = key.split('.').next().unwrap();
        let want = family(name).count;
        assert_eq!(len, want, "{key}");
        assert!(
            rows.iter()
                .all(|r| r.chars().all(|c| matches!(c, '.' | 'R' | 'W'))),
            "{key}"
        );
    }
}

/// H1 gate: the suite holds no known miscompile. A case either passes or is
/// rejected with a located diagnostic.
#[test]
fn conformance_ledger_has_no_wrong_values() {
    let wrong: Vec<&str> = KNOWN_GAPS
        .iter()
        .filter(|(_, kind, _)| *kind == Kind::WrongValue)
        .map(|(id, _, _)| *id)
        .collect();
    assert!(wrong.is_empty(), "WrongValue entries: {wrong:?}");
    for (key, rows) in RANDOM_GAPS {
        assert!(
            rows.iter().all(|r| !r.contains('W')),
            "{key} has a WrongValue random program"
        );
    }
}

fn compile_errors(src: &str) -> Vec<Diagnostic> {
    let result = compile(CompileInput {
        files: vec![SourceInput {
            name: "guard.c80",
            text: src,
        }],
        origin: 0x8000,
        optimize: true,
        target: C80Target::GenericZ80,
    });
    result
        .diagnostics
        .into_iter()
        .filter(|d| d.is_error())
        .collect()
}

/// The structural clobber guard fires when an in-place write would destroy a
/// live value: switching occupant parking off brings back D1 (byte and word),
/// and the guard turns it into a located `cg-internal` error instead of
/// wrong code.
#[test]
fn clobber_guard_reports_unparked_live_values() {
    for src in [
        "u8 f(u8 a, u8 b) { u8 s = a + b; u8 t = a & b; return s - t; }",
        "u16 f(u16 a, u16 b) { u16 s = a + b; u16 t = a & b; return s - t; }",
    ] {
        assert!(compile_errors(src).is_empty(), "{src}");
        let errors = crate::lower::test_hooks::without_parking(|| compile_errors(src));
        let guard = errors
            .iter()
            .find(|d| d.code == DiagCode::CgInternal)
            .unwrap_or_else(|| panic!("guard did not fire for {src}: {errors:?}"));
        assert!(
            guard.message.contains("would be overwritten"),
            "{}",
            guard.message
        );
        assert!(
            guard.span.end > guard.span.start,
            "guard error is not located"
        );
    }
}

// ---------------------------------------------------------------------------
// H3i: the program fixtures with every helper marked `@inline`
// ---------------------------------------------------------------------------

/// The source with `@inline ` in front of every function except `f` and
/// `@stackcall` ones (an error by design), and the names it marked.
fn mark_helpers_inline(src: &str) -> (String, Vec<String>) {
    let mut sources = SourceMap::new();
    let mut ids = IdGen::new();
    let mut diags = Vec::new();
    let id = sources.add("fixture.c80", src);
    let unit = parse_file(sources.get(id), &mut ids, &mut diags);
    let mut starts = Vec::new();
    for item in &unit.items {
        if let Item::Function(func) = item {
            if func.name.name != "f" && func.conv != CallConv::Stack {
                starts.push((func.span.start as usize, func.name.name.clone()));
            }
        }
    }
    let mut out = src.to_string();
    for (at, _) in starts.iter().rev() {
        out.insert_str(*at, "@inline ");
    }
    (out, starts.into_iter().map(|(_, n)| n).collect())
}

#[test]
fn conformance_program_fixtures_inlined() {
    let config = exec_config();
    let mut marked = 0usize;
    let mut errors = Vec::new();
    for case in fixture_cases() {
        let (src, names) = mark_helpers_inline(&case.src);
        if names.is_empty() {
            continue;
        }
        marked += 1;
        let inlined = Case {
            id: case.id.clone(),
            src: src.clone(),
            ret: case.ret,
            checks: case.checks.clone(),
        };
        for (mode, optimize) in MODES {
            let out = run_mode(&inlined, optimize, &config);
            if out.kind != Kind::Pass {
                errors.push(format!(
                    "{}@{mode}: {} {}\n    {}",
                    case.id,
                    kind_name(out.kind),
                    out.detail,
                    src.replace('\n', "\n    ")
                ));
                continue;
            }
            let result = compile(CompileInput {
                files: vec![SourceInput {
                    name: "conformance.c80",
                    text: &src,
                }],
                origin: 0x8000,
                optimize,
                target: C80Target::GenericZ80,
            });
            let code = result.code.as_ref().unwrap();
            let f = code.function("f").unwrap();
            for m in &f.mapped {
                let t = m.text.to_ascii_uppercase();
                if t.starts_with("CALL ")
                    && names
                        .iter()
                        .any(|n| t.ends_with(&format!("_{}", n.to_ascii_uppercase())))
                {
                    errors.push(format!("{}@{mode}: f still has {t}", case.id));
                }
            }
        }
    }
    assert!(marked >= 17, "only {marked} fixtures have helpers");
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}
