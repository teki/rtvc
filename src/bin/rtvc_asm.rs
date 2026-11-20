use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rtvc_core::asm::{AsmFileKind, AsmFiles, assemble_program_with};
use rtvc_core::cas::{TVC_CAS_TYPE_BASIC, encode_tvc_cas};

const FORMAT_VERSION: &str = "rtvc-asm-v1";
const TVC_BASIC_LOAD_ADDR: u16 = 0x19EF;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    input: Input,
    output: Option<PathBuf>,
    origin: u16,
    format: OutputFormat,
    defines: BTreeMap<String, String>,
    include_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Input {
    Stdin,
    Path(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Toml,
    Cas,
    Bin,
}

/// A CLI failure. Source errors print as bare `file:line: message` so
/// editors can match them; other failures get the `rtvc-asm:` prefix.
enum CliError {
    Source(String),
    Other(String),
}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        CliError::Other(message)
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(CliError::Source(err)) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
        Err(CliError::Other(err)) => {
            eprintln!("rtvc-asm: {err}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), CliError> {
    let args = env::args().collect::<Vec<_>>();
    let program = args.first().map(String::as_str).unwrap_or("rtvc-asm");
    if args[1..]
        .iter()
        .any(|arg| arg.as_str() == "-h" || arg.as_str() == "--help")
    {
        println!("{}", usage(program));
        return Ok(());
    }
    let options = parse_args(program, &args[1..])?;
    let (source_name, source) = read_source(&options.input)?;
    let source = apply_defines(&source, &options.defines)
        .map_err(|(line, err)| CliError::Source(format!("{source_name}:{line}: {err}")))?;
    let files = CliFiles {
        include_dirs: &options.include_dirs,
        defines: &options.defines,
    };
    let assembled = assemble_program_with(&source, options.origin, &source_name, Some(&files))
        .map_err(|err| CliError::Source(err.display_in(&source_name)))?;

    let output = render_output(&source_name, &options, &assembled)?;
    Ok(write_output(options.output.as_deref(), &output)?)
}

fn parse_args(program: &str, args: &[String]) -> Result<Options, String> {
    let mut input = None;
    let mut output = None;
    let mut origin = 0u16;
    let mut format = OutputFormat::Toml;
    let mut defines = BTreeMap::new();
    let mut include_dirs = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "-o" | "--output" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--output requires a path".to_string())?;
                output = Some(PathBuf::from(value));
            }
            "--origin" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--origin requires an address".to_string())?;
                origin = parse_number(value)?;
            }
            "--format" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--format requires toml, cas, or bin".to_string())?;
                format = parse_format(value)?;
            }
            "-d" | "--define" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--define requires NAME=VALUE".to_string())?;
                let (name, number) = parse_define(value)?;
                if defines.insert(name.clone(), number).is_some() {
                    return Err(format!("duplicate definition '{name}'"));
                }
            }
            "-I" | "--include-dir" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--include-dir requires a directory".to_string())?;
                include_dirs.push(PathBuf::from(value));
            }
            value if value.starts_with("-I") && value.len() > 2 => {
                include_dirs.push(PathBuf::from(&value[2..]));
            }
            value if value.starts_with("--include-dir=") => {
                include_dirs.push(PathBuf::from(&value["--include-dir=".len()..]));
            }
            value if value.starts_with("--origin=") => {
                origin = parse_number(&value["--origin=".len()..])?;
            }
            value if value.starts_with("--format=") => {
                format = parse_format(&value["--format=".len()..])?;
            }
            value if value.starts_with("--output=") => {
                output = Some(PathBuf::from(&value["--output=".len()..]));
            }
            value if value.starts_with("--define=") => {
                let (name, number) = parse_define(&value["--define=".len()..])?;
                if defines.insert(name.clone(), number).is_some() {
                    return Err(format!("duplicate definition '{name}'"));
                }
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown option '{value}'\n\n{}", usage(program)));
            }
            value => {
                if input.is_some() {
                    return Err(format!(
                        "unexpected argument '{value}'\n\n{}",
                        usage(program)
                    ));
                }
                input = Some(if value == "-" {
                    Input::Stdin
                } else {
                    Input::Path(PathBuf::from(value))
                });
            }
        }
        index += 1;
    }

    Ok(Options {
        input: input.ok_or_else(|| usage(program))?,
        output,
        origin,
        format,
        defines,
        include_dirs,
    })
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} [--origin <addr>] [--format toml|cas|bin] [-d NAME=VALUE] [-I <dir>] [-o <output>] <input.asm>\n\
         use '-' as input to read source from stdin; omit -o to write output to stdout"
    )
}

/// Reads `INCLUDE`, `INCBIN` and `IMPORT` files. A relative path is looked
/// up next to the file that names it, then in each `-I` directory.
struct CliFiles<'a> {
    include_dirs: &'a [PathBuf],
    defines: &'a BTreeMap<String, String>,
}

impl AsmFiles for CliFiles<'_> {
    fn read(&self, path: &str, from: &str, kind: AsmFileKind) -> Result<(String, Vec<u8>), String> {
        let wanted = Path::new(path);
        let candidates: Vec<PathBuf> = if wanted.is_absolute() {
            vec![wanted.to_path_buf()]
        } else {
            let base = if from == "-" {
                Path::new("")
            } else {
                Path::new(from).parent().unwrap_or(Path::new(""))
            };
            std::iter::once(base.join(wanted))
                .chain(self.include_dirs.iter().map(|dir| dir.join(wanted)))
                .collect()
        };
        let Some(found) = candidates.iter().find(|candidate| candidate.is_file()) else {
            let searched: Vec<_> = candidates
                .iter()
                .map(|candidate| candidate.display().to_string())
                .collect();
            return Err(format!(
                "cannot find '{path}' (looked for {})",
                searched.join(", ")
            ));
        };
        let name = found.display().to_string();
        let bytes = fs::read(found).map_err(|err| format!("failed to read {name}: {err}"))?;
        if kind != AsmFileKind::Source {
            return Ok((name, bytes));
        }
        // %NAME% placeholders apply to included source as to the main file.
        let text = String::from_utf8(bytes).map_err(|_| format!("'{name}' is not UTF-8 text"))?;
        let text = apply_defines(&text, self.defines)
            .map_err(|(line, err)| format!("{name}:{line}: {err}"))?;
        Ok((name, text.into_bytes()))
    }
}

fn read_source(input: &Input) -> Result<(String, String), String> {
    match input {
        Input::Stdin => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|err| format!("failed to read stdin: {err}"))?;
            Ok(("-".to_string(), source))
        }
        Input::Path(path) => {
            let source = fs::read_to_string(path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            Ok((path.display().to_string(), source))
        }
    }
}

fn write_output(path: Option<&Path>, value: &[u8]) -> Result<(), String> {
    match path {
        Some(path) => {
            let mut file = fs::File::create(path)
                .map_err(|err| format!("failed to create {}: {err}", path.display()))?;
            file.write_all(value)
                .map_err(|err| format!("failed to write {}: {err}", path.display()))
        }
        None => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout
                .write_all(value)
                .map_err(|err| format!("failed to write stdout: {err}"))
        }
    }
}

fn render_output(
    source_name: &str,
    options: &Options,
    assembled: &rtvc_core::asm::AssembledProgram,
) -> Result<Vec<u8>, String> {
    match options.format {
        OutputFormat::Toml => Ok(render_toml(source_name, options.origin, assembled).into_bytes()),
        OutputFormat::Cas => render_cas(assembled),
        OutputFormat::Bin => render_bin(assembled),
    }
}

fn render_bin(assembled: &rtvc_core::asm::AssembledProgram) -> Result<Vec<u8>, String> {
    if assembled.segments.len() != 1 {
        return Err("--format bin requires exactly one contiguous output segment".to_string());
    }
    Ok(assembled.segments[0].bytes.clone())
}

fn render_cas(assembled: &rtvc_core::asm::AssembledProgram) -> Result<Vec<u8>, String> {
    if assembled.segments.len() != 1 {
        return Err("--format cas requires exactly one contiguous BASIC_START segment".to_string());
    }
    let segment = &assembled.segments[0];
    if segment.addr != TVC_BASIC_LOAD_ADDR {
        return Err(format!(
            "--format cas requires a BASIC_START segment at {TVC_BASIC_LOAD_ADDR:04X}H, got {:04X}H",
            segment.addr
        ));
    }
    Ok(encode_tvc_cas(
        &segment.bytes,
        TVC_CAS_TYPE_BASIC,
        0xFF,
        TVC_BASIC_LOAD_ADDR,
    ))
}

fn render_toml(
    source_name: &str,
    requested_origin: u16,
    assembled: &rtvc_core::asm::AssembledProgram,
) -> String {
    let mut out = Vec::new();
    out.push(format!("format = {}", toml_string(FORMAT_VERSION)));
    out.push(format!("source = {}", toml_string(source_name)));
    out.push(format!("requested_origin = {}", hex16(requested_origin)));
    out.push(format!("origin = {}", hex16(assembled.origin)));
    out.push(format!("next_addr = {}", hex16(assembled.next_addr)));
    if assembled.files.len() > 1 {
        let files: Vec<_> = assembled
            .files
            .iter()
            .map(|file| toml_string(file))
            .collect();
        out.push(format!("files = [{}]", files.join(", ")));
    }

    if !assembled.symbols.is_empty() {
        out.push(String::new());
        out.push("[symbols]".to_string());
        for (name, value) in &assembled.symbols {
            out.push(format!("{name} = {}", hex16(*value)));
        }
    }

    for mapping in &assembled.mappings {
        out.push(String::new());
        out.push("[[mappings]]".to_string());
        out.push(format!("name = {}", toml_string(&mapping.name)));
        out.push(format!("source_base = {}", hex16(mapping.source_base)));
        out.push(format!("mapped_base = {}", hex16(mapping.mapped_base)));
    }

    for line in &assembled.lines {
        out.push(String::new());
        out.push("[[lines]]".to_string());
        if line.file != 0 {
            out.push(format!("file = {}", line.file));
        }
        out.push(format!("line = {}", line.line));
        out.push(format!("addr = {}", hex16(line.addr)));
        if line.run_addr != line.addr {
            out.push(format!("run_addr = {}", hex16(line.run_addr)));
        }
        out.push(format!("len = {}", line.len));
        out.push(format!("source = {}", toml_string(&line.source)));
    }

    for segment in &assembled.segments {
        out.push(String::new());
        out.push("[[segments]]".to_string());
        out.push(format!("addr = {}", hex16(segment.addr)));
        out.push(format!("len = {}", segment.bytes.len()));
        emit_bytes_toml(&mut out, "bytes", &segment.bytes);
    }

    out.join("\n") + "\n"
}

fn emit_bytes_toml(out: &mut Vec<String>, key: &str, bytes: &[u8]) {
    out.push(format!("{key} = ["));
    for row in bytes.chunks(16) {
        let items = row
            .iter()
            .map(|byte| hex8(*byte))
            .collect::<Vec<_>>()
            .join(", ");
        let ascii = row
            .iter()
            .map(|byte| {
                if (0x20..=0x7E).contains(byte) {
                    *byte as char
                } else {
                    '.'
                }
            })
            .collect::<String>();
        out.push(format!(
            "  {items},{} # |{ascii}|",
            " ".repeat(95usize.saturating_sub(items.len() + 1))
        ));
    }
    out.push("]".to_string());
}

fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn hex8(value: u8) -> String {
    format!("0x{value:02X}")
}

fn hex16(value: u16) -> String {
    format!("0x{value:04X}")
}

fn parse_number(value: &str) -> Result<u16, String> {
    let value = value.trim().to_ascii_uppercase().replace('_', "");
    let (digits, radix) = if let Some(rest) = value.strip_prefix('$') {
        (rest, 16)
    } else if let Some(rest) = value.strip_prefix("0X") {
        (rest, 16)
    } else if let Some(rest) = value.strip_suffix('H') {
        (rest, 16)
    } else {
        (value.as_str(), 10)
    };
    u16::from_str_radix(digits, radix).map_err(|_| format!("invalid 16-bit address '{value}'"))
}

fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value.to_ascii_lowercase().as_str() {
        "toml" => Ok(OutputFormat::Toml),
        "cas" => Ok(OutputFormat::Cas),
        "bin" => Ok(OutputFormat::Bin),
        _ => Err(format!(
            "invalid output format '{value}' (expected toml, cas, or bin)"
        )),
    }
}

fn parse_define(value: &str) -> Result<(String, String), String> {
    let (name, value) = value
        .split_once('=')
        .ok_or_else(|| format!("invalid definition '{value}' (expected NAME=VALUE)"))?;
    let name = name.trim().to_ascii_uppercase();
    let mut chars = name.chars();
    let valid = chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '.')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.');
    if !valid {
        return Err(format!("invalid definition name '{name}'"));
    }
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("definition '{name}' has an empty value"));
    }
    Ok((name, value.to_string()))
}

/// Substitute `%NAME%` placeholders. An error carries its 1-based line.
fn apply_defines(
    source: &str,
    defines: &BTreeMap<String, String>,
) -> Result<String, (usize, String)> {
    let mut output = String::with_capacity(source.len());
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let comment_start = find_comment_start(line);
        let (code, comment) = line.split_at(comment_start);
        let code = apply_defines_to_code(code, defines).map_err(|err| (index + 1, err))?;
        output.push_str(&code);
        output.push_str(comment);
    }
    Ok(output)
}

fn apply_defines_to_code(
    source: &str,
    defines: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut output = String::with_capacity(source.len());
    let mut quote = None;
    let mut escape = false;
    let mut index = 0usize;
    let mut text_start = 0usize;
    while index < source.len() {
        let ch = source[index..]
            .chars()
            .next()
            .expect("index must point at a character boundary");
        let next_index = index + ch.len_utf8();

        if escape {
            escape = false;
            index = next_index;
            continue;
        }
        if quote.is_some() && ch == '\\' {
            escape = true;
            index = next_index;
            continue;
        }
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
            }
            index = next_index;
            continue;
        }

        match ch {
            '"' => quote = Some(ch),
            '\'' if single_quote_starts_operand(source, index)
                || single_quote_starts_statement_string(source, index) =>
            {
                quote = Some(ch)
            }
            '%' => {
                output.push_str(&source[text_start..index]);
                let rest = &source[index + 1..];
                let end = rest
                    .find('%')
                    .ok_or_else(|| "unterminated definition placeholder '%'".to_string())?;
                let name = rest[..end].to_ascii_uppercase();
                let value = defines
                    .get(&name)
                    .ok_or_else(|| format!("missing definition for %{name}%"))?;
                output.push_str(value);
                index += end + 2;
                text_start = index;
                continue;
            }
            _ => {}
        }
        index = next_index;
    }
    output.push_str(&source[text_start..]);
    Ok(output)
}

/// Find the first assembler comment delimiter outside a quoted string so
/// definition substitution can leave comments and string data untouched.
fn find_comment_start(source: &str) -> usize {
    let mut quote = None;
    let mut escape = false;
    for (index, ch) in source.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if quote.is_some() && ch == '\\' {
            escape = true;
            continue;
        }
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        match ch {
            ';' => return index,
            '"' => quote = Some(ch),
            '\'' if single_quote_starts_operand(source, index)
                || single_quote_starts_statement_string(source, index) =>
            {
                quote = Some(ch)
            }
            _ => {}
        }
    }
    source.len()
}

fn single_quote_starts_operand(source: &str, index: usize) -> bool {
    source[..index]
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_none_or(|ch| ch == ',')
}

fn single_quote_starts_statement_string(source: &str, index: usize) -> bool {
    source[..index]
        .chars()
        .last()
        .is_some_and(|ch| ch.is_whitespace())
        && source[index + 1..].contains('\'')
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtvc_core::asm::assemble_program;

    #[test]
    fn parses_cli_options() {
        let args = vec![
            "--origin".to_string(),
            "8000H".to_string(),
            "--format".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "out.bin".to_string(),
            "helper.asm".to_string(),
        ];
        let options = parse_args("rtvc-asm", &args).unwrap();
        assert_eq!(options.origin, 0x8000);
        assert_eq!(options.format, OutputFormat::Bin);
        assert_eq!(options.output, Some(PathBuf::from("out.bin")));
        assert_eq!(options.input, Input::Path(PathBuf::from("helper.asm")));
        assert!(options.defines.is_empty());
    }

    #[test]
    fn parses_include_directories() {
        let args = vec![
            "-I".to_string(),
            "lib".to_string(),
            "-Iinc".to_string(),
            "--include-dir=share/asm".to_string(),
            "main.asm".to_string(),
        ];
        let options = parse_args("rtvc-asm", &args).unwrap();
        assert_eq!(
            options.include_dirs,
            [
                PathBuf::from("lib"),
                PathBuf::from("inc"),
                PathBuf::from("share/asm")
            ]
        );
    }

    #[test]
    fn reads_files_next_to_the_includer_then_from_include_dirs() {
        let root = std::env::temp_dir().join(format!("rtvc-asm-files-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src/parts")).unwrap();
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(root.join("src/parts/local.inc"), "LD A,%VALUE%\n").unwrap();
        fs::write(root.join("lib/shared.inc"), "NOP\n").unwrap();
        fs::write(root.join("lib/font.bin"), [1, 2, 3]).unwrap();

        let include_dirs = vec![root.join("lib")];
        let defines = BTreeMap::from([("VALUE".to_string(), "42".to_string())]);
        let files = CliFiles {
            include_dirs: &include_dirs,
            defines: &defines,
        };
        let main = root.join("src/main.asm").display().to_string();

        let (name, bytes) = files
            .read("parts/local.inc", &main, AsmFileKind::Source)
            .unwrap();
        assert_eq!(name, root.join("src/parts/local.inc").display().to_string());
        assert_eq!(bytes, b"LD A,42\n");

        // Relative to the file that names it, not to the main source.
        let (name, _) = files
            .read("../../lib/shared.inc", &name, AsmFileKind::Source)
            .unwrap();
        assert!(name.ends_with("shared.inc"));

        let (_, bytes) = files.read("font.bin", &main, AsmFileKind::Binary).unwrap();
        assert_eq!(bytes, [1, 2, 3]);

        let err = files
            .read("none.inc", &main, AsmFileKind::Source)
            .unwrap_err();
        assert!(err.starts_with("cannot find 'none.inc' (looked for "));

        let no_defines = BTreeMap::new();
        let files = CliFiles {
            include_dirs: &include_dirs,
            defines: &no_defines,
        };
        let err = files
            .read("parts/local.inc", &main, AsmFileKind::Source)
            .unwrap_err();
        assert!(err.ends_with("local.inc:1: missing definition for %VALUE%"));

        // Binary files are not touched by placeholder substitution.
        fs::write(root.join("lib/data.bin"), b"%X%").unwrap();
        let (_, bytes) = files.read("data.bin", &main, AsmFileKind::Binary).unwrap();
        assert_eq!(bytes, b"%X%");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn parses_command_line_definitions() {
        let args = vec![
            "-d".to_string(),
            "block_size=0748H".to_string(),
            "--define=TARGET=5E28H".to_string(),
            "input.asm".to_string(),
        ];
        let options = parse_args("rtvc-asm", &args).unwrap();
        assert_eq!(options.defines["BLOCK_SIZE"], "0748H");
        assert_eq!(options.defines["TARGET"], "5E28H");
    }

    #[test]
    fn substitutes_definition_placeholders() {
        let defines = BTreeMap::from([
            ("SIZE".to_string(), "0748H".to_string()),
            ("TARGET".to_string(), "7530H".to_string()),
        ]);
        assert_eq!(
            apply_defines("LD BC,%size%\nLD DE,%TARGET%\n", &defines).unwrap(),
            "LD BC,0748H\nLD DE,7530H\n"
        );
        assert_eq!(
            apply_defines("NOP\nLD BC,%MISSING%", &defines).unwrap_err(),
            (2, "missing definition for %MISSING%".to_string())
        );
    }

    #[test]
    fn leaves_definition_like_text_in_comments_untouched() {
        let defines = BTreeMap::from([("VALUE".to_string(), "42".to_string())]);
        assert_eq!(
            apply_defines(
                "; %COMMENT%\nLD A,%VALUE% ; %INLINE%\nDB 'A;%;B'\n",
                &defines
            )
            .unwrap(),
            "; %COMMENT%\nLD A,42 ; %INLINE%\nDB 'A;%;B'\n"
        );
    }

    #[test]
    fn parses_address_formats() {
        assert_eq!(parse_number("32768").unwrap(), 0x8000);
        assert_eq!(parse_number("0x8000").unwrap(), 0x8000);
        assert_eq!(parse_number("$8000").unwrap(), 0x8000);
        assert_eq!(parse_number("8000H").unwrap(), 0x8000);
    }

    #[test]
    fn renders_cas_output() {
        let assembled = assemble_program("BASIC_START\nRET\n", 0).unwrap();
        let cas = render_cas(&assembled).unwrap();
        assert_eq!(cas[0], 0x11);
        assert_eq!(&cas[0x80..0x85], &[0x00, 0x01, 0x42, 0x00, 0xFF]);
        assert_eq!(&cas[0x87..0x89], &[0xEF, 0x19]);
        assert_eq!(&cas[0x90..0xA0], &assembled.segments[0].bytes[..16]);
        assert_eq!(cas[0x90 + 0x41], 0xC9);
    }

    #[test]
    fn renders_named_mapping_metadata() {
        let assembled = assemble_program("ORG C000H, SYS0, 0000H\nNOP\n", 0).unwrap();
        let toml = render_toml("mapping.asm", 0, &assembled);
        assert!(toml.contains("[[mappings]]"));
        assert!(toml.contains("name = \"SYS0\""));
        assert!(toml.contains("source_base = 0xC000"));
        assert!(toml.contains("mapped_base = 0x0000"));
        assert!(!toml.contains("physical_offset"));
    }

    #[test]
    fn rejects_cas_without_basic_start() {
        let assembled = assemble_program("ORG 8000H\nRET\n", 0).unwrap();
        let error = render_cas(&assembled).unwrap_err();
        assert!(error.contains("BASIC_START"));
    }
}
