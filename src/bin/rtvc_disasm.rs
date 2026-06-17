use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rtvc_core::asm::assemble_line;
use rtvc_core::bus::{CpuBus, FakeBus};
use rtvc_core::disasm::disassemble_at;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    input: Input,
    output: Option<PathBuf>,
    origin: u16,
    title: Option<String>,
    symbols: Option<PathBuf>,
    bank: Option<String>,
    bank_offset: u16,
    data_ranges: Vec<Range>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Input {
    Stdin,
    Path(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    start: u16,
    end: u16,
}

#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    summary: String,
    usage: Vec<String>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("rtvc-disasm: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    let program = args.first().map(String::as_str).unwrap_or("rtvc-disasm");
    if args[1..]
        .iter()
        .any(|arg| arg.as_str() == "-h" || arg.as_str() == "--help")
    {
        println!("{}", usage(program));
        return Ok(());
    }
    let options = parse_args(program, &args[1..])?;
    let (source_name, bytes) = read_input(&options.input)?;
    let symbols = load_symbols(&options)?;
    let asm = render_listing(&source_name, &bytes, &options, &symbols)?;
    write_output(options.output.as_deref(), &asm)
}

fn parse_args(program: &str, args: &[String]) -> Result<Options, String> {
    let mut input = None;
    let mut output = None;
    let mut origin = 0u16;
    let mut title = None;
    let mut symbols = None;
    let mut bank = None;
    let mut bank_offset = 0u16;
    let mut data_ranges = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "-o" | "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--output requires a path".to_string())?,
                ));
            }
            "--origin" => {
                index += 1;
                origin = parse_number(
                    args.get(index)
                        .ok_or_else(|| "--origin requires an address".to_string())?,
                )?;
            }
            "--title" => {
                index += 1;
                title = Some(
                    args.get(index)
                        .ok_or_else(|| "--title requires text".to_string())?
                        .clone(),
                );
            }
            "--symbols" => {
                index += 1;
                symbols = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--symbols requires a path".to_string())?,
                ));
            }
            "--bank" => {
                index += 1;
                bank = Some(
                    args.get(index)
                        .ok_or_else(|| "--bank requires a bank name".to_string())?
                        .to_ascii_lowercase(),
                );
            }
            "--bank-offset" => {
                index += 1;
                bank_offset = parse_number(
                    args.get(index)
                        .ok_or_else(|| "--bank-offset requires an address".to_string())?,
                )?;
            }
            "--data-range" => {
                index += 1;
                data_ranges
                    .push(parse_range(args.get(index).ok_or_else(|| {
                        "--data-range requires start-end".to_string()
                    })?)?);
            }
            value if value.starts_with("--origin=") => {
                origin = parse_number(&value["--origin=".len()..])?;
            }
            value if value.starts_with("--output=") => {
                output = Some(PathBuf::from(&value["--output=".len()..]));
            }
            value if value.starts_with("--title=") => {
                title = Some(value["--title=".len()..].to_string());
            }
            value if value.starts_with("--symbols=") => {
                symbols = Some(PathBuf::from(&value["--symbols=".len()..]));
            }
            value if value.starts_with("--bank=") => {
                bank = Some(value["--bank=".len()..].to_ascii_lowercase());
            }
            value if value.starts_with("--bank-offset=") => {
                bank_offset = parse_number(&value["--bank-offset=".len()..])?;
            }
            value if value.starts_with("--data-range=") => {
                data_ranges.push(parse_range(&value["--data-range=".len()..])?);
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
        title,
        symbols,
        bank,
        bank_offset,
        data_ranges,
    })
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} [--origin <addr>] [-o <output.asm>] [options] <input.bin>\n\
         options:\n\
           --title <text>             listing title comment\n\
           --symbols <rom_symbols.json>\n\
           --bank <sys|exth>          select symbols from a ROM symbol file\n\
           --bank-offset <addr>       physical bank offset of input bytes\n\
           --data-range <start-end>   CPU-address range to emit as DB; may repeat\n\
         use '-' as input to read bytes from stdin; omit -o to write assembly to stdout"
    )
}

fn read_input(input: &Input) -> Result<(String, Vec<u8>), String> {
    match input {
        Input::Stdin => {
            let mut bytes = Vec::new();
            io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|err| format!("failed to read stdin: {err}"))?;
            Ok(("-".to_string(), bytes))
        }
        Input::Path(path) => fs::read(path)
            .map(|bytes| (path.display().to_string(), bytes))
            .map_err(|err| format!("failed to read {}: {err}", path.display())),
    }
}

fn write_output(path: Option<&Path>, asm: &str) -> Result<(), String> {
    match path {
        Some(path) => {
            let mut file = fs::File::create(path)
                .map_err(|err| format!("failed to create {}: {err}", path.display()))?;
            file.write_all(asm.as_bytes())
                .map_err(|err| format!("failed to write {}: {err}", path.display()))
        }
        None => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout
                .write_all(asm.as_bytes())
                .map_err(|err| format!("failed to write stdout: {err}"))
        }
    }
}

fn load_symbols(options: &Options) -> Result<BTreeMap<u16, Vec<Symbol>>, String> {
    let Some(path) = &options.symbols else {
        return Ok(BTreeMap::new());
    };
    let bank = options
        .bank
        .as_deref()
        .ok_or_else(|| "--symbols requires --bank".to_string())?;
    let document: Value = serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    let raw_symbols = document
        .get("symbols")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} has no symbols array", path.display()))?;
    let mut out: BTreeMap<u16, Vec<Symbol>> = BTreeMap::new();
    for symbol in raw_symbols {
        if symbol.get("bank").and_then(Value::as_str) != Some(bank) {
            continue;
        }
        let offset = symbol
            .get("offset")
            .and_then(Value::as_str)
            .and_then(|value| parse_number(value).ok())
            .ok_or_else(|| "ROM symbol has invalid offset".to_string())?;
        let Some(local_offset) = offset.checked_sub(options.bank_offset) else {
            continue;
        };
        let addr = options.origin.wrapping_add(local_offset);
        let name = symbol
            .get("name")
            .and_then(Value::as_str)
            .map(label_name)
            .ok_or_else(|| "ROM symbol has no name".to_string())?;
        let summary = symbol
            .get("summary")
            .and_then(Value::as_str)
            .map(ascii_comment)
            .unwrap_or_default();
        let usage = symbol
            .get("usage")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        out.entry(addr).or_default().push(Symbol {
            name,
            summary,
            usage,
        });
    }
    Ok(out)
}

fn render_listing(
    source_name: &str,
    bytes: &[u8],
    options: &Options,
    symbols: &BTreeMap<u16, Vec<Symbol>>,
) -> Result<String, String> {
    let mut bus = FakeBus::new();
    for (i, byte) in bytes.iter().enumerate() {
        bus.w8(options.origin.wrapping_add(i as u16), *byte);
    }

    let mut out = Vec::new();
    out.push(
        "; -----------------------------------------------------------------------------"
            .to_string(),
    );
    if let Some(title) = &options.title {
        out.push(format!("; {}", ascii_comment(title)));
    } else {
        out.push("; Z80 disassembly".to_string());
    }
    out.push(format!("; Source: {source_name}"));
    out.push(format!("; ORG: {}", hex16(options.origin)));
    out.push(format!("; Size: {} bytes", bytes.len()));
    if let Some(symbols) = &options.symbols {
        out.push(format!("; Symbols: {}", symbols.display()));
    }
    if !options.data_ranges.is_empty() {
        let ranges = options
            .data_ranges
            .iter()
            .map(|range| format!("{}-{}", hex16(range.start), hex16(range.end)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push(format!("; Data ranges: {ranges}"));
    }
    out.push(
        "; -----------------------------------------------------------------------------"
            .to_string(),
    );
    out.push(String::new());
    out.push(format!("ORG {}", hex16(options.origin)));
    out.push(String::new());

    let mut offset = 0usize;
    while offset < bytes.len() {
        let addr = options.origin.wrapping_add(offset as u16);
        emit_labels(&mut out, addr, symbols);
        if in_data_range(addr, &options.data_ranges) {
            let chunk_len = data_chunk_len(addr, bytes.len() - offset, options, symbols);
            emit_db(&mut out, bytes, options.origin, addr, chunk_len);
            offset += chunk_len;
            continue;
        }

        let inst = disassemble_at(&mut bus, addr);
        let inst_len = (inst.len as usize).min(bytes.len() - offset);
        let boundary_len = boundary_len(addr, inst_len, options, symbols);
        if boundary_len != inst.len as usize {
            emit_db(&mut out, bytes, options.origin, addr, boundary_len);
            offset += boundary_len;
            continue;
        }
        let can_assemble = assemble_line(&inst.text, addr)
            .map(|assembled| assembled == inst.bytes)
            .unwrap_or(false);
        if can_assemble {
            out.push(format!("    {}", inst.text));
            offset += inst.len as usize;
        } else {
            emit_db(&mut out, bytes, options.origin, addr, inst.len as usize);
            offset += inst.len as usize;
        }
    }
    out.push(String::new());
    Ok(out.join("\n"))
}

fn emit_labels(out: &mut Vec<String>, addr: u16, symbols: &BTreeMap<u16, Vec<Symbol>>) {
    let Some(symbols) = symbols.get(&addr) else {
        return;
    };
    out.push(String::new());
    for symbol in symbols {
        if !symbol.summary.is_empty() {
            out.push(format!("; {} - {}", symbol.name, symbol.summary));
        }
        if !symbol.usage.is_empty() {
            out.push(format!("; usage: {}", symbol.usage.join(",")));
        }
        out.push(format!("{}:", symbol.name));
    }
}

fn emit_db(out: &mut Vec<String>, bytes: &[u8], origin: u16, addr: u16, len: usize) {
    let start = addr.wrapping_sub(origin) as usize;
    let items = bytes[start..start + len]
        .iter()
        .map(|byte| hex8(*byte))
        .collect::<Vec<_>>()
        .join(", ");
    out.push(format!("    DB {items}"));
}

fn data_chunk_len(
    addr: u16,
    remaining: usize,
    options: &Options,
    symbols: &BTreeMap<u16, Vec<Symbol>>,
) -> usize {
    let mut len = 16usize.min(remaining);
    if let Some(range) = options
        .data_ranges
        .iter()
        .find(|range| range.start <= addr && addr <= range.end)
    {
        len = len.min(range.end.wrapping_sub(addr) as usize + 1);
    }
    boundary_len(addr, len, options, symbols)
}

fn boundary_len(
    addr: u16,
    len: usize,
    options: &Options,
    symbols: &BTreeMap<u16, Vec<Symbol>>,
) -> usize {
    let mut len = len.max(1);
    for next in 1..len {
        let next_addr = addr.wrapping_add(next as u16);
        if symbols.contains_key(&next_addr) || data_boundary(next_addr, &options.data_ranges) {
            len = next;
            break;
        }
    }
    len
}

fn in_data_range(addr: u16, ranges: &[Range]) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= addr && addr <= range.end)
}

fn data_boundary(addr: u16, ranges: &[Range]) -> bool {
    ranges.iter().any(|range| range.start == addr)
}

fn parse_range(value: &str) -> Result<Range, String> {
    let (start, end) = value
        .split_once('-')
        .ok_or_else(|| format!("invalid data range '{value}'"))?;
    let start = parse_number(start)?;
    let end = parse_number(end)?;
    if end < start {
        return Err(format!("invalid descending data range '{value}'"));
    }
    Ok(Range { start, end })
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

fn hex8(value: u8) -> String {
    format!("{value:02X}H")
}

fn hex16(value: u16) -> String {
    format!("{value:04X}H")
}

fn label_name(value: &str) -> String {
    let mut name = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    if !name
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '.')
    {
        name.insert(0, '_');
    }
    name
}

fn ascii_comment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_graphic() || ch == ' ' {
                ch
            } else {
                '?'
            }
        })
        .collect()
}
