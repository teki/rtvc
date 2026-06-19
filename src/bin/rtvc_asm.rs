use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rtvc_core::asm::assemble_program;

const FORMAT_VERSION: &str = "rtvc-asm-v1";
const TVC_BASIC_LOAD_ADDR: u16 = 0x19EF;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    input: Input,
    output: Option<PathBuf>,
    origin: u16,
    format: OutputFormat,
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

fn main() {
    if let Err(err) = run() {
        eprintln!("rtvc-asm: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
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
    let assembled = assemble_program(&source, options.origin).map_err(|err| err.to_string())?;

    let output = render_output(&source_name, &options, &assembled)?;
    write_output(options.output.as_deref(), &output)
}

fn parse_args(program: &str, args: &[String]) -> Result<Options, String> {
    let mut input = None;
    let mut output = None;
    let mut origin = 0u16;
    let mut format = OutputFormat::Toml;
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
            value if value.starts_with("--origin=") => {
                origin = parse_number(&value["--origin=".len()..])?;
            }
            value if value.starts_with("--format=") => {
                format = parse_format(&value["--format=".len()..])?;
            }
            value if value.starts_with("--output=") => {
                output = Some(PathBuf::from(&value["--output=".len()..]));
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
    })
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} [--origin <addr>] [--format toml|cas|bin] [-o <output>] <input.asm>\n\
         use '-' as input to read source from stdin; omit -o to write output to stdout"
    )
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
    Ok(tvc_program_cas(&segment.bytes))
}

fn tvc_program_cas(payload: &[u8]) -> Vec<u8> {
    let dfsize = 144 + payload.len();
    let blocks = dfsize / 128;
    let remainder = dfsize % 128;
    let mut cas = vec![0; 144 + payload.len()];
    cas[0] = 0x11;
    cas[2] = (blocks & 0xFF) as u8;
    cas[3] = (blocks >> 8) as u8;
    cas[4] = (remainder & 0xFF) as u8;
    cas[5] = (remainder >> 8) as u8;
    cas[0x80] = 0x00;
    cas[0x81] = 0x01;
    cas[0x82] = (payload.len() & 0xFF) as u8;
    cas[0x83] = (payload.len() >> 8) as u8;
    cas[0x84] = 0xFF;
    cas[0x87] = (TVC_BASIC_LOAD_ADDR & 0xFF) as u8;
    cas[0x88] = (TVC_BASIC_LOAD_ADDR >> 8) as u8;
    cas[0x90..].copy_from_slice(payload);
    cas
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

    if !assembled.symbols.is_empty() {
        out.push(String::new());
        out.push("[symbols]".to_string());
        for (name, value) in &assembled.symbols {
            out.push(format!("{name} = {}", hex16(*value)));
        }
    }

    for line in &assembled.lines {
        out.push(String::new());
        out.push("[[lines]]".to_string());
        out.push(format!("line = {}", line.line));
        out.push(format!("addr = {}", hex16(line.addr)));
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn rejects_cas_without_basic_start() {
        let assembled = assemble_program("ORG 8000H\nRET\n", 0).unwrap();
        let error = render_cas(&assembled).unwrap_err();
        assert!(error.contains("BASIC_START"));
    }
}
