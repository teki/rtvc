//! The tutorial examples under `info/c80/` as tests (plan HX1).
//!
//! Every example is compiled in both optimize modes and checked three ways:
//!
//! - **Directives** (`//!` comment lines in the example sources) state
//!   results next to the code they describe. The grammar is documented in
//!   `info/c80/tutorial.md` ("Examples check themselves"). A malformed or
//!   unknown directive fails the test; nothing is skipped silently.
//! - **Coverage**: every function that returns a value has at least one
//!   `expect` (or an `expect-skip` with a reason).
//! - **Golden listings** in `info/c80/expected/`: the optimized `--emit-asm`
//!   text of every single-file example and every project. Set
//!   `C80_UPDATE_EXPECTED=1` to rewrite them after a deliberate change.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rtvc_c80::harness::{AccessKind, ExecConfig, ExecResult, execute_function_with};
use rtvc_c80::project::{
    ProjectUnitInput, compile_project_keeping_bodies, compile_project_opt, parse_manifest,
    resolve_manifest_path,
};
use rtvc_c80::{
    C80Target, CType, CompilationResult, CompileInput, GeneratedGlobal, GeneratedProgram, RegHome,
    SourceInput, compile, compile_keeping_bodies,
};

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../info/c80")
}

fn expected_dir() -> PathBuf {
    examples_dir().join("expected")
}

// ---------------------------------------------------------------------------
// Examples
// ---------------------------------------------------------------------------

/// One C80 source file that carries directives.
struct SourceText {
    /// Path relative to `info/c80/`, for messages.
    rel: String,
    text: String,
}

/// A buildable example: a single file or a project manifest.
struct Example {
    /// Golden file stem: the file stem or the project directory name.
    name: String,
    sources: Vec<SourceText>,
    kind: ExampleKind,
}

enum ExampleKind {
    Single { file: String },
    Project { manifest: PathBuf },
}

impl Example {
    fn compile(&self, optimize: bool) -> CompilationResult {
        self.compile_with(optimize, false)
    }

    /// `keep_bodies`: also emit functions whose every call was inlined, so a
    /// directive can still call them by name (H3i).
    fn compile_with(&self, optimize: bool, keep_bodies: bool) -> CompilationResult {
        match &self.kind {
            ExampleKind::Single { file } => {
                let input = CompileInput {
                    files: vec![SourceInput {
                        name: file,
                        text: &self.sources[0].text,
                    }],
                    origin: 0x8000,
                    optimize,
                    target: C80Target::GenericZ80,
                };
                if keep_bodies {
                    compile_keeping_bodies(input)
                } else {
                    compile(input)
                }
            }
            ExampleKind::Project { manifest } => compile_manifest(manifest, optimize, keep_bodies),
        }
    }
}

fn rel_name(path: &Path) -> String {
    path.strip_prefix(examples_dir())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Every example, sorted by name: `*.c80` (not `*.fail.c80`) and every
/// directory with an `rtvc-c80.toml`.
fn examples() -> Vec<Example> {
    let mut entries: Vec<PathBuf> = fs::read_dir(examples_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    let mut out = Vec::new();
    for path in entries {
        let file = path.file_name().unwrap().to_str().unwrap().to_string();
        if path.is_file() && file.ends_with(".c80") && !file.ends_with(".fail.c80") {
            out.push(Example {
                name: file.trim_end_matches(".c80").to_string(),
                sources: vec![SourceText {
                    rel: rel_name(&path),
                    text: fs::read_to_string(&path).unwrap(),
                }],
                kind: ExampleKind::Single { file },
            });
        } else if path.is_dir() && path.join("rtvc-c80.toml").is_file() {
            let manifest = path.join("rtvc-c80.toml");
            let parsed = parse_manifest(&fs::read_to_string(&manifest).unwrap()).unwrap();
            let sources = parsed
                .units
                .iter()
                .map(|u| resolve_manifest_path(&path, &u.path))
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("c80"))
                .map(|p| SourceText {
                    rel: rel_name(&p),
                    text: fs::read_to_string(&p).unwrap(),
                })
                .collect();
            out.push(Example {
                name: file,
                sources,
                kind: ExampleKind::Project { manifest },
            });
        }
    }
    out
}

/// Compiles a manifest as the CLI does: paths resolve from its directory.
fn compile_manifest(manifest_path: &Path, optimize: bool, keep_bodies: bool) -> CompilationResult {
    let manifest = parse_manifest(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let dir = manifest_path.parent().unwrap();
    let owned: Vec<(String, String)> = manifest
        .units
        .iter()
        .map(|unit| {
            let path = resolve_manifest_path(dir, &unit.path);
            (rel_name(&path), fs::read_to_string(&path).unwrap())
        })
        .collect();
    let inputs: Vec<ProjectUnitInput<'_>> = manifest
        .units
        .iter()
        .zip(owned.iter())
        .map(|(unit, (name, text))| ProjectUnitInput {
            name: &unit.name,
            kind: unit.kind,
            origin: unit.origin,
            text,
            source_name: name,
            exports: &unit.exports,
            stack_extra: unit.stack_extra,
        })
        .collect();
    let basic = manifest
        .basic
        .as_ref()
        .map(|spec| fs::read_to_string(resolve_manifest_path(dir, &spec.path)).unwrap());
    if keep_bodies {
        compile_project_keeping_bodies(&manifest, &inputs, basic.as_deref(), optimize)
    } else {
        compile_project_opt(&manifest, &inputs, basic.as_deref(), optimize)
    }
}

fn code_of<'a>(result: &'a CompilationResult, what: &str) -> &'a GeneratedProgram {
    let errors: Vec<String> = result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| format!("{}: {}", d.code.as_str(), d.message))
        .collect();
    assert!(errors.is_empty(), "{what} does not compile: {errors:#?}");
    result
        .code
        .as_ref()
        .unwrap_or_else(|| panic!("{what}: no code"))
}

// ---------------------------------------------------------------------------
// Directive grammar
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Arg {
    Int(i64),
    /// `&name` or `&name[k]`: a global's address, element `k`.
    Addr(String, u16),
}

#[derive(Debug, Clone)]
struct Call {
    name: String,
    args: Vec<Arg>,
}

/// A memory or port location in `expect-mem` and `with`.
#[derive(Debug, Clone)]
enum Place {
    /// `name`, `name[i]`, or `name[i..j]` (element indices, end exclusive).
    Global(String, Option<(u16, u16)>),
    /// `[addr]` or `[addr..end]`: raw bytes.
    Raw(u16, u16),
    /// `out(p)` in `expect-mem`: the bytes written to port `p`, in order.
    /// `in(p)` in `with`: the bytes `IN` from port `p` returns, in order.
    Port(u8),
}

#[derive(Debug, Clone)]
enum Cmp {
    Eq,
    Le,
}

#[derive(Debug, Clone)]
enum Directive {
    Expect {
        call: Call,
        value: Arg,
        with: Vec<(Place, Vec<i64>)>,
    },
    ExpectMem {
        call: Call,
        place: Place,
        values: Vec<i64>,
        with: Vec<(Place, Vec<i64>)>,
    },
    Bytes {
        name: String,
        cmp: Cmp,
        n: u32,
    },
    Tstates {
        name: String,
        n: u64,
    },
    Skip {
        name: String,
    },
    Error {
        code: String,
    },
}

#[derive(Debug)]
struct Located {
    file: String,
    line: usize,
    text: String,
    directive: Directive,
}

impl Located {
    fn at(&self) -> String {
        format!("{}:{}: `{}`", self.file, self.line, self.text)
    }
}

fn parse_int(text: &str) -> Result<i64, String> {
    let t = text.trim();
    match t {
        "true" => return Ok(1),
        "false" => return Ok(0),
        _ => {}
    }
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t),
    };
    let value = if let Some(hex) = digits.strip_prefix("0x") {
        i64::from_str_radix(hex, 16)
    } else {
        digits.parse::<i64>()
    }
    .map_err(|_| format!("bad number `{t}`"))?;
    let value = if neg { -value } else { value };
    if !(-0x8000..=0xFFFF).contains(&value) {
        return Err(format!("number `{t}` does not fit 16 bits"));
    }
    Ok(value)
}

fn parse_ident(text: &str) -> Result<String, String> {
    let t = text.trim();
    let ok = !t.is_empty()
        && t.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if ok {
        Ok(t.to_string())
    } else {
        Err(format!("bad name `{t}`"))
    }
}

/// `[i]` or `[i..j]` after a name, as element indices (end exclusive).
fn parse_index(text: &str) -> Result<(u16, u16), String> {
    let inner = text
        .strip_prefix('[')
        .and_then(|t| t.strip_suffix(']'))
        .ok_or_else(|| format!("bad index `{text}`"))?;
    let to_u16 = |t: &str| -> Result<u16, String> {
        u16::try_from(parse_int(t)?).map_err(|_| format!("bad index `{t}`"))
    };
    match inner.split_once("..") {
        Some((a, b)) => {
            let (a, b) = (to_u16(a)?, to_u16(b)?);
            if b <= a {
                return Err(format!("empty range `{text}`"));
            }
            Ok((a, b))
        }
        None => {
            let a = to_u16(inner)?;
            Ok((a, a + 1))
        }
    }
}

fn parse_place(text: &str) -> Result<Place, String> {
    let t = text.trim();
    for (prefix, is_port) in [("out(", true), ("in(", true)] {
        if let Some(rest) = t.strip_prefix(prefix) {
            let port = rest
                .strip_suffix(')')
                .ok_or_else(|| format!("bad port `{t}`"))?;
            let port = u8::try_from(parse_int(port)?).map_err(|_| format!("bad port `{t}`"))?;
            let _ = is_port;
            return Ok(Place::Port(port));
        }
    }
    if t.starts_with('[') {
        let (a, b) = parse_index(t)?;
        return Ok(Place::Raw(a, b));
    }
    match t.find('[') {
        Some(i) => Ok(Place::Global(
            parse_ident(&t[..i])?,
            Some(parse_index(&t[i..])?),
        )),
        None => Ok(Place::Global(parse_ident(t)?, None)),
    }
}

fn parse_call(text: &str) -> Result<Call, String> {
    let t = text.trim();
    let open = t
        .find('(')
        .ok_or_else(|| format!("expected a call, got `{t}`"))?;
    let inner = t[open + 1..]
        .strip_suffix(')')
        .ok_or_else(|| format!("unclosed call `{t}`"))?;
    let name = parse_ident(&t[..open])?;
    let mut args = Vec::new();
    if !inner.trim().is_empty() {
        for a in inner.split(',') {
            args.push(parse_arg(a)?);
        }
    }
    Ok(Call { name, args })
}

/// A number, or `&name` / `&name[k]` for a global's address.
fn parse_arg(text: &str) -> Result<Arg, String> {
    let a = text.trim();
    match a.strip_prefix('&') {
        Some(sym) => {
            let (name, index) = match sym.find('[') {
                Some(i) => (parse_ident(&sym[..i])?, parse_index(&sym[i..])?.0),
                None => (parse_ident(sym)?, 0),
            };
            Ok(Arg::Addr(name, index))
        }
        None => Ok(Arg::Int(parse_int(a)?)),
    }
}

fn parse_values(text: &str) -> Result<Vec<i64>, String> {
    text.split(',').map(parse_int).collect()
}

/// `with p = v, q = v1 v2` — several bytes for a port are space separated.
fn parse_with(text: &str) -> Result<Vec<(Place, Vec<i64>)>, String> {
    text.split(',')
        .map(|item| {
            let (place, value) = item
                .split_once('=')
                .ok_or_else(|| format!("expected `place = value` in `{item}`"))?;
            let values = value
                .split_whitespace()
                .map(parse_int)
                .collect::<Result<Vec<_>, _>>()?;
            if values.is_empty() {
                return Err(format!("missing value in `{item}`"));
            }
            Ok((parse_place(place)?, values))
        })
        .collect()
}

fn split_with(text: &str) -> Result<(&str, Vec<(Place, Vec<i64>)>), String> {
    match text.split_once(" with ") {
        Some((head, tail)) => Ok((head, parse_with(tail)?)),
        None => Ok((text, Vec::new())),
    }
}

fn parse_directive(body: &str) -> Result<Directive, String> {
    let (kind, rest) = body
        .split_once(':')
        .ok_or_else(|| "expected `kind: ...`".to_string())?;
    let rest = rest.trim();
    match kind.trim() {
        "expect" => {
            let (head, with) = split_with(rest)?;
            let (call, value) = head
                .split_once("==")
                .ok_or_else(|| "expected `call == value`".to_string())?;
            Ok(Directive::Expect {
                call: parse_call(call)?,
                value: parse_arg(value)?,
                with,
            })
        }
        "expect-mem" => {
            let (head, with) = split_with(rest)?;
            let (call, effect) = head
                .split_once("=>")
                .ok_or_else(|| "expected `call => place == values`".to_string())?;
            let (place, values) = effect
                .split_once("==")
                .ok_or_else(|| "expected `place == values`".to_string())?;
            Ok(Directive::ExpectMem {
                call: parse_call(call)?,
                place: parse_place(place)?,
                values: parse_values(values)?,
                with,
            })
        }
        "expect-bytes" => {
            let (name, cmp, n) = if let Some((a, b)) = rest.split_once("<=") {
                (a, Cmp::Le, b)
            } else if let Some((a, b)) = rest.split_once("==") {
                (a, Cmp::Eq, b)
            } else {
                return Err("expected `name <= N` or `name == N`".to_string());
            };
            let n = u32::try_from(parse_int(n)?).map_err(|_| "negative size".to_string())?;
            Ok(Directive::Bytes {
                name: parse_ident(name)?,
                cmp,
                n,
            })
        }
        "expect-tstates" => {
            let (name, n) = rest
                .split_once("==")
                .ok_or_else(|| "expected `name == N`".to_string())?;
            let n = u64::try_from(parse_int(n)?).map_err(|_| "negative count".to_string())?;
            Ok(Directive::Tstates {
                name: parse_ident(name)?,
                n,
            })
        }
        "expect-skip" => {
            let (name, reason) = rest
                .split_once(" - ")
                .ok_or_else(|| "expected `name - reason`".to_string())?;
            if reason.trim().is_empty() {
                return Err("an expect-skip needs a reason".to_string());
            }
            Ok(Directive::Skip {
                name: parse_ident(name)?,
            })
        }
        "expect-error" => {
            let code = rest.trim();
            let ok = !code.is_empty() && code.chars().all(|c| c.is_ascii_lowercase() || c == '-');
            if !ok {
                return Err(format!("bad diagnostic code `{code}`"));
            }
            Ok(Directive::Error {
                code: code.to_string(),
            })
        }
        other => Err(format!("unknown directive `{other}`")),
    }
}

/// All directives of one source. Panics on the first malformed line.
fn directives(src: &SourceText) -> Vec<Located> {
    let mut out = Vec::new();
    for (i, line) in src.text.lines().enumerate() {
        let Some(body) = line.trim_start().strip_prefix("//!") else {
            continue;
        };
        let directive = parse_directive(body.trim()).unwrap_or_else(|err| {
            panic!(
                "{}:{}: malformed directive `{}`: {err}",
                src.rel,
                i + 1,
                line.trim()
            )
        });
        out.push(Located {
            file: src.rel.clone(),
            line: i + 1,
            text: body.trim().to_string(),
            directive,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Running directives
// ---------------------------------------------------------------------------

fn exec_config() -> ExecConfig {
    ExecConfig {
        insn_limit: 2_000_000,
        tstate_limit: 50_000_000,
        ..ExecConfig::default()
    }
}

/// Bytes per element: an array's stride (struct elements included), or the
/// whole object for a scalar or struct global.
fn elem_width(g: &GeneratedGlobal) -> u16 {
    match g.ty {
        CType::Array(a) if a.len > 0 => g.size / a.len,
        _ => g.size.max(1),
    }
}

/// The unit of a place's index and values: the element for byte and word
/// elements, a single byte for anything wider (structs).
fn place_width(g: &GeneratedGlobal) -> u16 {
    match elem_width(g) {
        w @ (1 | 2) => w,
        _ => 1,
    }
}

fn arg_value(code: &GeneratedProgram, arg: &Arg) -> Result<u16, String> {
    match arg {
        Arg::Int(v) => Ok(*v as u16),
        Arg::Addr(name, k) => {
            let g = code
                .global(name)
                .ok_or_else(|| format!("no global `{name}`"))?;
            Ok(g.addr + k * elem_width(g))
        }
    }
}

/// Byte range `[lo, hi)` of a place in memory.
fn byte_range(code: &GeneratedProgram, place: &Place) -> Result<(u16, u16, u16), String> {
    match place {
        Place::Global(name, index) => {
            let g = code
                .global(name)
                .ok_or_else(|| format!("no global `{name}`"))?;
            let w = place_width(g);
            let (lo, hi) = match index {
                Some((i, j)) => (g.addr + i * w, g.addr + j * w),
                None => (g.addr, g.addr + w),
            };
            if hi > g.addr + g.size {
                return Err(format!("`{name}` index past its {} bytes", g.size));
            }
            Ok((lo, hi, w))
        }
        Place::Raw(a, b) => Ok((*a, *b, 1)),
        Place::Port(_) => Err("a port is not memory".to_string()),
    }
}

/// Little-endian bytes of `values`, each `width` (1 or 2) bytes wide. A
/// value that does not fit a byte place is an error, not a silent truncation.
fn place_bytes(values: &[i64], width: u16) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for &v in values {
        if width == 1 && !(-0x80..=0xFF).contains(&v) {
            return Err(format!("{v} does not fit a byte"));
        }
        let bits = v as u16;
        out.extend((0..width).map(|k| (bits >> (8 * k)) as u8));
    }
    Ok(out)
}

fn run_call(
    code: &GeneratedProgram,
    call: &Call,
    with: &[(Place, Vec<i64>)],
) -> Result<ExecResult, String> {
    let mut config = exec_config();
    for (place, values) in with {
        match place {
            Place::Port(p) => config.scripted_ports.push((*p, place_bytes(values, 1)?)),
            _ => {
                let (lo, hi, w) = byte_range(code, place)?;
                let bytes = place_bytes(values, w)?;
                if bytes.len() != usize::from(hi - lo) {
                    return Err(format!("{} values for {} bytes", values.len(), hi - lo));
                }
                for (k, b) in bytes.into_iter().enumerate() {
                    config.initial_mem.push((lo + k as u16, b));
                }
            }
        }
    }
    let args = call
        .args
        .iter()
        .map(|a| arg_value(code, a))
        .collect::<Result<Vec<_>, _>>()?;
    let exec =
        execute_function_with(code, &call.name, &args, &config).map_err(|e| e.to_string())?;
    // A proven stack bound must cover what the call really used.
    if let Some(f) = code.function(&call.name) {
        if f.stack_provenance == rtvc_c80::StackProvenance::Proven && exec.sp_used > f.stack_bound {
            return Err(format!(
                "`{}` used {} stack bytes, more than its proven bound {}",
                call.name, exec.sp_used, f.stack_bound
            ));
        }
    }
    Ok(exec)
}

/// Memory after the call: the image, the `with` presets, then every write.
fn final_bytes(
    code: &GeneratedProgram,
    config_mem: &[(u16, u8)],
    exec: &ExecResult,
    lo: u16,
    hi: u16,
) -> Vec<u8> {
    let mut mem: HashMap<u16, u8> = HashMap::new();
    for seg in &code.assembled.segments {
        for (i, b) in seg.bytes.iter().enumerate() {
            mem.insert(seg.addr.wrapping_add(i as u16), *b);
        }
    }
    for &(a, b) in config_mem {
        mem.insert(a, b);
    }
    for acc in &exec.accesses {
        if acc.kind == AccessKind::DataWrite {
            mem.insert(acc.addr, acc.value);
        }
    }
    (lo..hi)
        .map(|a| mem.get(&a).copied().unwrap_or(0))
        .collect()
}

/// The listing lines of one function: from its label to the next label
/// that starts another function or global.
fn function_listing<'a>(code: &'a GeneratedProgram, name: &str) -> Result<Vec<&'a str>, String> {
    let func = code
        .function(name)
        .ok_or_else(|| format!("no function `{name}`"))?;
    let starts: Vec<String> = code
        .functions
        .iter()
        .map(|f| format!("{}:", f.label))
        .chain(code.globals.iter().map(|g| format!("{}:", g.label)))
        .collect();
    let mine = format!("{}:", func.label);
    let mut lines = code.assembly.lines().skip_while(|l| l.trim() != mine);
    lines
        .next()
        .ok_or_else(|| format!("`{name}` not in the listing"))?;
    Ok(lines
        .take_while(|l| !starts.iter().any(|s| l.trim() == s) && !l.trim().starts_with("ORG"))
        .collect())
}

/// `expect-tstates` only makes sense when every run takes the same path.
/// An unconditional `CALL` of a generated function whose own listing is
/// straight-line is allowed (H3i: a call and its inlined form side by side).
fn straight_line(code: &GeneratedProgram, lines: &[&str]) -> Result<(), String> {
    for line in lines {
        let t = line.trim().to_ascii_uppercase();
        let mnemonic = t.split_whitespace().next().unwrap_or("");
        if mnemonic == "CALL" {
            let target = t[4..].trim();
            if let Some(callee) = code
                .functions
                .iter()
                .find(|f| f.label.eq_ignore_ascii_case(target))
            {
                straight_line(code, &function_listing(code, &callee.name)?)?;
                continue;
            }
        }
        let conditional_ret = mnemonic == "RET" && t.len() > 3;
        if matches!(
            mnemonic,
            "JP" | "JR" | "DJNZ" | "CALL" | "RST" | "RETI" | "RETN"
        ) || conditional_ret
            || t.contains("LDIR")
            || t.contains("LDDR")
        {
            return Err(format!(
                "not straight-line (`{}`); expect-tstates needs a listing without branches or calls",
                line.trim()
            ));
        }
    }
    Ok(())
}

fn ret_value(code: &GeneratedProgram, name: &str, exec: &ExecResult) -> Result<(u16, u16), String> {
    let func = code.function(name).unwrap();
    match func.ret {
        Some(RegHome::Byte(_)) => Ok((u16::from(exec.a), 0xFF)),
        Some(RegHome::Word(_)) => Ok((exec.hl, 0xFFFF)),
        None => Err(format!("`{name}` returns nothing; use expect-mem")),
    }
}

/// The function a directive calls or measures.
fn directive_target(d: &Directive) -> Option<&str> {
    match d {
        Directive::Expect { call, .. } | Directive::ExpectMem { call, .. } => Some(&call.name),
        Directive::Bytes { name, .. } | Directive::Tstates { name, .. } => Some(name),
        _ => None,
    }
}

/// Runs one directive against one build. `Err` is a failure message.
fn check(code: &GeneratedProgram, d: &Directive, optimized: bool) -> Result<(), String> {
    match d {
        Directive::Expect { call, value, with } => {
            let exec = run_call(code, call, with)?;
            let (got, mask) = ret_value(code, &call.name, &exec)?;
            let want = arg_value(code, value)? & mask;
            if got != want {
                return Err(format!("returned {got:#06X}, expected {want:#06X}"));
            }
            Ok(())
        }
        Directive::ExpectMem {
            call,
            place,
            values,
            with,
        } => {
            let exec = run_call(code, call, with)?;
            if let Place::Port(p) = place {
                let got: Vec<u8> = exec
                    .accesses
                    .iter()
                    .filter(|a| a.kind == AccessKind::PortOut && a.addr == u16::from(*p))
                    .map(|a| a.value)
                    .collect();
                let want = place_bytes(values, 1)?;
                if got != want {
                    return Err(format!(
                        "port {p:#04X} got {got:02X?}, expected {want:02X?}"
                    ));
                }
                return Ok(());
            }
            let (lo, hi, w) = byte_range(code, place)?;
            let want = place_bytes(values, w)?;
            if want.len() != usize::from(hi - lo) {
                return Err(format!("{} values for {} bytes", values.len(), hi - lo));
            }
            let mut preset = Vec::new();
            for (pl, vals) in with {
                if !matches!(pl, Place::Port(_)) {
                    let (a, b, w2) = byte_range(code, pl)?;
                    let bytes = place_bytes(vals, w2)?;
                    preset.extend((a..b).zip(bytes));
                }
            }
            let got = final_bytes(code, &preset, &exec, lo, hi);
            if got != want {
                return Err(format!(
                    "memory {lo:#06X}.. is {got:02X?}, expected {want:02X?}"
                ));
            }
            Ok(())
        }
        // Budgets describe the default (optimized) build.
        Directive::Bytes { name, cmp, n } => {
            if !optimized {
                return Ok(());
            }
            let func = code
                .function(name)
                .ok_or_else(|| format!("no function `{name}`"))?;
            let size = u32::from(func.size);
            let ok = match cmp {
                Cmp::Eq => size == *n,
                Cmp::Le => size <= *n,
            };
            if !ok {
                return Err(format!("`{name}` is {size} bytes"));
            }
            Ok(())
        }
        Directive::Tstates { name, n } => {
            if !optimized {
                return Ok(());
            }
            straight_line(code, &function_listing(code, name)?)?;
            let func = code.function(name).unwrap();
            let args = vec![0u16; func.param_homes.len().max(func.param_types.len())];
            let args = &args[..if func.conv == rtvc_c80::CallConv::Stack {
                func.param_types.len()
            } else {
                func.param_homes.len()
            }];
            let exec = execute_function_with(code, name, args, &exec_config())
                .map_err(|e| e.to_string())?;
            if exec.tstates != *n {
                return Err(format!(
                    "`{name}` takes {} T-states (RET included)",
                    exec.tstates
                ));
            }
            Ok(())
        }
        Directive::Skip { .. } => Ok(()),
        Directive::Error { .. } => Err("expect-error belongs in a *.fail.c80 file".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Directive results and the coverage gate, for every example and mode.
#[test]
fn examples_check_themselves() {
    let mut failures = Vec::new();
    let mut count = 0usize;
    for ex in examples() {
        let ds: Vec<Located> = ex.sources.iter().flat_map(directives).collect();
        for optimize in [true, false] {
            let result = ex.compile(optimize);
            let mode = if optimize { "opt" } else { "base" };
            let code = code_of(&result, &format!("{} ({mode})", ex.name));
            // A private function whose every call was inlined has no body
            // in this build; its directives run against the same build with
            // bodies kept (the other functions' code is identical).
            let kept = ex.compile_with(optimize, true);
            let kept_code = code_of(&kept, &format!("{} ({mode}, bodies kept)", ex.name));
            for d in &ds {
                count += 1;
                let target = match directive_target(&d.directive) {
                    Some(name) if code.function(name).is_none() => kept_code,
                    _ => code,
                };
                if let Err(msg) = check(target, &d.directive, optimize) {
                    failures.push(format!("{} [{mode}]: {msg}", d.at()));
                }
            }
            // Coverage: every value-returning function in these sources.
            for func in &code.functions {
                if func.ret.is_none() {
                    continue;
                }
                let file = result
                    .sources
                    .files()
                    .iter()
                    .find(|f| f.id == func.span.file);
                let in_sources =
                    file.is_some_and(|f| ex.sources.iter().any(|s| f.name.ends_with(&s.rel)));
                if !in_sources {
                    continue;
                }
                let covered = ds.iter().any(|d| match &d.directive {
                    Directive::Expect { call, .. } => call.name == func.name,
                    Directive::Skip { name } => *name == func.name,
                    _ => false,
                });
                if !covered && optimize {
                    failures.push(format!(
                        "{}: `{}` returns a value but has no `//! expect:` line",
                        ex.name, func.name
                    ));
                }
            }
        }
    }
    assert!(count > 0, "no directives found");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `*.fail.c80`: each must be rejected with exactly the listed diagnostics,
/// each pointing into the file.
#[test]
fn fail_examples_are_rejected_as_documented() {
    let mut entries: Vec<PathBuf> = fs::read_dir(examples_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".fail.c80"))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no *.fail.c80 examples");
    let mut failures = Vec::new();
    for path in entries {
        let src = SourceText {
            rel: rel_name(&path),
            text: fs::read_to_string(&path).unwrap(),
        };
        let mut want: Vec<String> = Vec::new();
        for d in directives(&src) {
            match d.directive {
                Directive::Error { code } => want.push(code),
                _ => failures.push(format!(
                    "{}: only expect-error belongs in a fail file",
                    d.at()
                )),
            }
        }
        if want.is_empty() {
            failures.push(format!("{}: no `//! expect-error:` line", src.rel));
        }
        want.sort();
        // The unit name is the stem before `.fail`.
        let file = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".fail", "_fail");
        for optimize in [true, false] {
            let result = compile(CompileInput {
                files: vec![SourceInput {
                    name: &file,
                    text: &src.text,
                }],
                origin: 0x8000,
                optimize,
                target: C80Target::GenericZ80,
            });
            let errors: Vec<_> = result.diagnostics.iter().filter(|d| d.is_error()).collect();
            let mut got: Vec<String> = errors.iter().map(|d| d.code.as_str().to_string()).collect();
            got.sort();
            if got != want {
                failures.push(format!(
                    "{} (optimize={optimize}): errors {got:?}, expected {want:?}: {:#?}",
                    src.rel,
                    errors.iter().map(|d| &d.message).collect::<Vec<_>>()
                ));
            }
            for d in errors {
                if d.span.end <= d.span.start {
                    failures.push(format!("{}: `{}` has no location", src.rel, d.message));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Golden listings: the optimized assembly of every example, plus the
/// baseline listing of `optimize.c80` that the tutorial compares against.
#[test]
fn golden_listings_match() {
    let update = std::env::var_os("C80_UPDATE_EXPECTED").is_some();
    let dir = expected_dir();
    if update {
        fs::create_dir_all(&dir).unwrap();
    }
    let mut failures = Vec::new();
    let mut wanted: Vec<String> = Vec::new();
    for ex in examples() {
        let mut builds = vec![(format!("{}.asm", ex.name), true)];
        if ex.name == "optimize" {
            builds.push(("optimize.unopt.asm".to_string(), false));
        }
        for (file, optimize) in builds {
            let result = ex.compile(optimize);
            let text = code_of(&result, &ex.name).assembly.clone();
            assert!(
                !text.contains("/Users") && !text.contains(":\\"),
                "{file}: listing contains a host path"
            );
            let path = dir.join(&file);
            wanted.push(file.clone());
            if update {
                fs::write(&path, &text).unwrap();
                continue;
            }
            match fs::read_to_string(&path) {
                Ok(golden) if golden == text => {}
                Ok(golden) => failures.push(format!(
                    "{file}: listing differs from info/c80/expected/{file}{}",
                    first_difference(&golden, &text)
                )),
                Err(_) => failures.push(format!("{file}: missing info/c80/expected/{file}")),
            }
        }
    }
    // No stale golden files.
    if let Ok(read) = fs::read_dir(&dir) {
        for e in read {
            let name = e.unwrap().file_name().to_string_lossy().to_string();
            if name.ends_with(".asm") && !wanted.contains(&name) {
                failures.push(format!("info/c80/expected/{name} has no example"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{}\nRun with C80_UPDATE_EXPECTED=1 after a deliberate codegen change and review the diff.",
        failures.join("\n")
    );
}

fn first_difference(golden: &str, text: &str) -> String {
    for (i, (a, b)) in golden.lines().zip(text.lines()).enumerate() {
        if a != b {
            return format!(
                "\n  line {}: expected `{a}`\n  line {}: got      `{b}`",
                i + 1,
                i + 1
            );
        }
    }
    format!(
        "\n  lengths differ: {} vs {} lines",
        golden.lines().count(),
        text.lines().count()
    )
}

/// The grammar itself: malformed lines are errors, never ignored.
#[test]
fn malformed_directives_are_rejected() {
    for bad in [
        "expect: f(1) = 2",
        "expect: f 1 == 2",
        "expect: f(1) == 0x1FFFF",
        "expect-mem: f() == 1",
        "expect-mem: f() => g[2..2] == 1",
        "expect-bytes: f < 3",
        "expect-tstates: f <= 3",
        "expect-skip: f",
        "expect-skip: f - ",
        "expect-error: Not A Code",
        "expected: f() == 1",
        "expect: f() == 1 with g",
    ] {
        assert!(parse_directive(bad).is_err(), "accepted `{bad}`");
    }
    for good in [
        "expect: f(1, -2, 0x10, true, &buf, &buf[3]) == 255",
        "expect: f() == 1 with g = 2, in(0x58) = 1 2, t[1..3] = 4 5",
        "expect: start() == &buf[0]",
        "expect-mem: f() => buf[0..2] == 1, 2",
        "expect-mem: f() => [0x8000] == 0xFF",
        "expect-mem: f() => out(0x10) == 7",
        "expect-bytes: f <= 24",
        "expect-bytes: f == 1",
        "expect-tstates: f == 10",
        "expect-skip: play - never returns",
        "expect-error: ty-mismatch",
    ] {
        parse_directive(good).unwrap_or_else(|e| panic!("rejected `{good}`: {e}"));
    }
}
