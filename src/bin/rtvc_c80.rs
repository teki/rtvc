use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rtvc_core::compiler::project::{
    ProjectUnitInput, compile_project, contiguous_bytes, is_ascii_ident, parse_manifest,
    render_rtvc_asm_v1, resolve_manifest_path, write_atomic,
};
use rtvc_core::compiler::{
    CompilationResult, CompileInput, DEFAULT_CODE_ORIGIN, SourceInput, compile,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("rtvc-c80: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let program = args.first().map(String::as_str).unwrap_or("rtvc-c80");
    if args[1..].iter().any(|a| a == "-h" || a == "--help") {
        println!("{}", usage(program));
        return Ok(());
    }
    let options = parse_args(program, &args[1..])?;
    let result = compile_input(&options)?;
    if result.has_errors() {
        for diag in &result.diagnostics {
            eprintln!("{}: {}", diag.code.as_str(), diag.message);
        }
        return Err("compilation failed".to_string());
    }
    for diag in &result.diagnostics {
        eprintln!("{}: {}", diag.code.as_str(), diag.message);
    }
    let code = result
        .code
        .as_ref()
        .ok_or_else(|| "no code generated".to_string())?;
    let mut pending: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    if let Some(path) = &options.emit_asm {
        pending.push((path.clone(), code.assembly.as_bytes().to_vec()));
    }
    if let Some(path) = &options.emit_segments {
        let text = render_rtvc_asm_v1(
            &options.input_name,
            options.origin.unwrap_or(code.origin),
            &code.assembled,
        );
        pending.push((path.clone(), text.into_bytes()));
    }
    if let Some(path) = &options.emit_bin {
        let bytes = contiguous_bytes(code)?;
        pending.push((path.clone(), bytes));
    }
    write_atomic(&pending)?;
    Ok(())
}

struct Options {
    input: PathBuf,
    input_name: String,
    origin: Option<u16>,
    target: Option<String>,
    emit_asm: Option<PathBuf>,
    emit_segments: Option<PathBuf>,
    emit_bin: Option<PathBuf>,
}

fn parse_args(program: &str, args: &[String]) -> Result<Options, String> {
    if args.first().map(String::as_str) != Some("build") {
        return Err(format!("expected 'build'\n\n{}", usage(program)));
    }
    let mut input = None;
    let mut origin = None;
    let mut target = None;
    let mut emit_asm = None;
    let mut emit_segments = None;
    let mut emit_bin = None;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--origin" => {
                i += 1;
                let value = args.get(i).ok_or("--origin requires an address")?;
                origin = Some(parse_number(value)?);
            }
            "--target" => {
                i += 1;
                target = Some(args.get(i).ok_or("--target requires a name")?.clone());
            }
            "--emit-asm" => {
                i += 1;
                emit_asm = Some(PathBuf::from(
                    args.get(i).ok_or("--emit-asm requires a path")?,
                ));
            }
            "--emit-segments" => {
                i += 1;
                emit_segments = Some(PathBuf::from(
                    args.get(i).ok_or("--emit-segments requires a path")?,
                ));
            }
            "--emit-bin" => {
                i += 1;
                emit_bin = Some(PathBuf::from(
                    args.get(i).ok_or("--emit-bin requires a path")?,
                ));
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown option '{value}'\n\n{}", usage(program)));
            }
            value => {
                if input.is_some() {
                    return Err("multiple inputs".to_string());
                }
                input = Some(PathBuf::from(value));
            }
        }
        i += 1;
    }
    let input = input.ok_or_else(|| format!("missing input\n\n{}", usage(program)))?;
    if emit_asm.is_none() && emit_segments.is_none() && emit_bin.is_none() {
        return Err("at least one of --emit-asm, --emit-segments, --emit-bin is required".into());
    }
    let input_name = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("input")
        .to_string();
    Ok(Options {
        input,
        input_name,
        origin,
        target,
        emit_asm,
        emit_segments,
        emit_bin,
    })
}

fn compile_input(options: &Options) -> Result<CompilationResult, String> {
    let ext = options
        .input
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "toml" {
        if options.origin.is_some() {
            return Err("--origin is invalid with a manifest".to_string());
        }
        let text = fs::read_to_string(&options.input).map_err(|e| e.to_string())?;
        let manifest = parse_manifest(&text)?;
        if let Some(target) = &options.target {
            if manifest.target.as_str() != target {
                return Err(format!(
                    "--target {target} does not match manifest {}",
                    manifest.target.as_str()
                ));
            }
        }
        let dir = options.input.parent().unwrap_or_else(|| Path::new("."));
        let mut owned = Vec::new();
        for unit in &manifest.units {
            let path = resolve_manifest_path(dir, &unit.path);
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("cannot read unit '{}': {e}", path.display()))?;
            owned.push((
                unit.name.clone(),
                unit.kind,
                unit.origin,
                unit.exports.clone(),
                unit.stack_extra,
                path,
                text,
            ));
        }
        let inputs: Vec<ProjectUnitInput<'_>> = owned
            .iter()
            .map(
                |(name, kind, origin, exports, stack_extra, path, text)| ProjectUnitInput {
                    name,
                    kind: *kind,
                    origin: *origin,
                    text,
                    source_name: path.to_str().unwrap_or(name),
                    exports,
                    stack_extra: *stack_extra,
                },
            )
            .collect();
        return Ok(compile_project(&manifest, &inputs));
    }
    if ext != "c80" {
        return Err("input must be a .c80 file or a .toml manifest".to_string());
    }
    let stem = options
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("input file stem is not an ASCII identifier")?;
    if !is_ascii_ident(stem) {
        return Err("unit name from file stem must be an ASCII identifier".to_string());
    }
    let text = fs::read_to_string(&options.input).map_err(|e| e.to_string())?;
    let result = compile(CompileInput {
        files: vec![SourceInput {
            name: &options.input_name,
            text: &text,
        }],
        origin: options.origin.unwrap_or(DEFAULT_CODE_ORIGIN),
    });
    if options.origin.is_none() {
        let emits = result
            .code
            .as_ref()
            .is_some_and(|code| code.assembled.segments.iter().any(|s| !s.bytes.is_empty()));
        if emits {
            return Err("--origin is required when the unit emits bytes".to_string());
        }
    }
    Ok(result)
}

fn parse_number(value: &str) -> Result<u16, String> {
    let value = value.trim();
    let (radix, digits) = if let Some(rest) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        (16, rest)
    } else if let Some(rest) = value.strip_suffix('H').or_else(|| value.strip_suffix('h')) {
        (16, rest)
    } else {
        (10, value)
    };
    u16::from_str_radix(digits, radix).map_err(|_| format!("invalid address '{value}'"))
}

fn usage(program: &str) -> String {
    format!(
        "Usage: {program} build INPUT [--target TARGET] [--origin ADDRESS]\n\
         [--emit-asm PATH] [--emit-segments PATH] [--emit-bin PATH]"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_rejects_missing_emit_flag() {
        let err = parse_args("rtvc-c80", &["build".into(), "a.c80".into()])
            .err()
            .unwrap();
        assert!(err.contains("emit"));
    }
}
