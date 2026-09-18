use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rtvc_core::asm::assemble_program;
use rtvc_core::basic::tokenize_program;
use rtvc_core::cas::{TVC_CAS_LOAD_ADDR, TVC_CAS_TYPE_BASIC, encode_tvc_cas, flatten_cas_image};

#[cfg(feature = "asm-toml")]
use rtvc_core::asm::parse_rtvc_asm_v1;

const TVC_BASIC_LOAD_ADDR: u16 = TVC_CAS_LOAD_ADDR;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceKind {
    Asm,
    Bas,
    Toml,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("rtvc-tocas: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    let program = args.first().map(String::as_str).unwrap_or("rtvc-tocas");
    if args[1..]
        .iter()
        .any(|arg| arg.as_str() == "-h" || arg.as_str() == "--help")
    {
        println!("{}", usage(program));
        return Ok(());
    }
    let inputs = parse_args(program, &args[1..])?;
    for input in inputs {
        match source_kind(&input) {
            Some(kind) => {
                let (output, note) = convert_file(&input, kind)?;
                if let Some(note) = note {
                    println!("{} -> {} ({note})", input.display(), output.display());
                } else {
                    println!("{} -> {}", input.display(), output.display());
                }
            }
            None => println!("skip {} (unrecognised extension)", input.display()),
        }
    }
    Ok(())
}

fn parse_args(program: &str, args: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut inputs = Vec::new();
    for arg in args {
        if arg.starts_with('-') {
            return Err(format!("unknown option '{arg}'\n\n{}", usage(program)));
        }
        inputs.push(PathBuf::from(arg));
    }
    if inputs.is_empty() {
        return Err(usage(program));
    }
    Ok(inputs)
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} <file.(bas|asm|toml)> [file.(bas|asm|toml)...]\n\
         compile .bas with rtvc-basic, assemble .asm with rtvc-asm --format cas,\n\
         and flatten rtvc-asm-v1 TOML from 19EFH with zero-filled gaps;\n\
         write each output beside the source, replacing the extension with .cas;\n\
         skip files with any other extension"
    )
}

fn convert_file(input: &Path, kind: SourceKind) -> Result<(PathBuf, Option<String>), String> {
    let output = cas_output_path(input);
    let source = fs::read_to_string(input)
        .map_err(|err| format!("failed to read {}: {err}", input.display()))?;
    let (bytes, note) = convert_source(kind, &source)?;
    fs::write(&output, bytes)
        .map_err(|err| format!("failed to write {}: {err}", output.display()))?;
    Ok((output, note))
}

fn source_kind(path: &Path) -> Option<SourceKind> {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("asm") => Some(SourceKind::Asm),
        Some("bas") => Some(SourceKind::Bas),
        Some("toml") => Some(SourceKind::Toml),
        _ => None,
    }
}

fn cas_output_path(input: &Path) -> PathBuf {
    input.with_extension("cas")
}

fn convert_source(kind: SourceKind, source: &str) -> Result<(Vec<u8>, Option<String>), String> {
    match kind {
        SourceKind::Asm => Ok((convert_asm(source)?, None)),
        SourceKind::Bas => Ok((convert_bas(source)?, None)),
        SourceKind::Toml => convert_toml(source),
    }
}

fn convert_asm(source: &str) -> Result<Vec<u8>, String> {
    let assembled = assemble_program(source, 0).map_err(|err| err.to_string())?;
    if assembled.segments.len() != 1 {
        return Err(
            "asm CAS output requires exactly one contiguous BASIC_START segment".to_string(),
        );
    }
    let segment = &assembled.segments[0];
    if segment.addr != TVC_BASIC_LOAD_ADDR {
        return Err(format!(
            "asm CAS output requires a BASIC_START segment at {TVC_BASIC_LOAD_ADDR:04X}H, got {:04X}H",
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

fn convert_bas(source: &str) -> Result<Vec<u8>, String> {
    let payload = tokenize_program(source).map_err(|err| err.to_string())?;
    Ok(encode_tvc_cas(&payload, TVC_CAS_TYPE_BASIC, 0x00, 0))
}

fn convert_toml(source: &str) -> Result<(Vec<u8>, Option<String>), String> {
    #[cfg(not(feature = "asm-toml"))]
    {
        let _ = source;
        return Err("TOML CAS conversion requires the asm-toml feature".to_string());
    }
    #[cfg(feature = "asm-toml")]
    {
        let assembled = parse_rtvc_asm_v1(source)?;
        let owned: Vec<(u16, Vec<u8>)> = assembled
            .segments
            .iter()
            .map(|s| (s.addr, s.bytes.clone()))
            .collect();
        let refs: Vec<(u16, &[u8])> = owned.iter().map(|(a, b)| (*a, b.as_slice())).collect();
        let image = flatten_cas_image(&refs)?;
        let cas = encode_tvc_cas(&image.bytes, TVC_CAS_TYPE_BASIC, 0xFF, TVC_CAS_LOAD_ADDR);
        let note = format!(
            "{} payload bytes, {} padding, {} CAS bytes",
            image.payload_bytes,
            image.padding_bytes,
            cas.len()
        );
        Ok((cas, Some(note)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtvc_core::cas::TVC_CAS_HEADER_LEN;

    #[test]
    fn parses_multiple_inputs() {
        let args = vec![
            "hello.bas".to_string(),
            "demo.asm".to_string(),
            "other.BAS".to_string(),
        ];
        let inputs = parse_args("rtvc-tocas", &args).unwrap();
        assert_eq!(
            inputs,
            vec![
                PathBuf::from("hello.bas"),
                PathBuf::from("demo.asm"),
                PathBuf::from("other.BAS"),
            ]
        );
    }

    #[test]
    fn rejects_unknown_options() {
        let error = parse_args("rtvc-tocas", &["-o".to_string(), "a.bas".to_string()]).unwrap_err();
        assert!(error.contains("unknown option '-o'"));
    }

    #[test]
    fn classifies_extensions_case_insensitively() {
        assert_eq!(
            source_kind(Path::new("coding/demo.asm")),
            Some(SourceKind::Asm)
        );
        assert_eq!(
            source_kind(Path::new("coding/demo.BAS")),
            Some(SourceKind::Bas)
        );
        assert_eq!(
            source_kind(Path::new("out/program.toml")),
            Some(SourceKind::Toml)
        );
        assert_eq!(source_kind(Path::new("coding/README")), None);
    }

    #[test]
    fn writes_cas_beside_the_source() {
        assert_eq!(
            cas_output_path(Path::new("coding/demo.bas")),
            PathBuf::from("coding/demo.cas")
        );
        assert_eq!(
            cas_output_path(Path::new("/tmp/helper.ASM")),
            PathBuf::from("/tmp/helper.cas")
        );
        assert_eq!(
            cas_output_path(Path::new("out/pong.toml")),
            PathBuf::from("out/pong.cas")
        );
    }

    #[test]
    fn converts_basic_like_rtvc_basic() {
        let (cas, note) = convert_source(SourceKind::Bas, "10 PRINT 1\n").unwrap();
        assert!(note.is_none());
        let payload = tokenize_program("10 PRINT 1\n").unwrap();
        assert_eq!(cas[0], 0x11);
        assert_eq!(cas[0x80], 0x00);
        assert_eq!(cas[0x81], 0x01);
        assert_eq!(
            u16::from_le_bytes([cas[0x82], cas[0x83]]) as usize,
            payload.len()
        );
        assert_eq!(cas[0x84], 0x00);
        assert_eq!(&cas[0x90..], payload.as_slice());
    }

    #[test]
    fn converts_asm_like_rtvc_asm_format_cas() {
        let assembled = assemble_program("BASIC_START\nRET\n", 0).unwrap();
        let (cas, _) = convert_source(SourceKind::Asm, "BASIC_START\nRET\n").unwrap();
        assert_eq!(cas[0], 0x11);
        assert_eq!(&cas[0x80..0x85], &[0x00, 0x01, 0x42, 0x00, 0xFF]);
        assert_eq!(&cas[0x87..0x89], &[0xEF, 0x19]);
        assert_eq!(&cas[0x90..0xA0], &assembled.segments[0].bytes[..16]);
        assert_eq!(cas[0x90 + 0x41], 0xC9);
    }

    #[test]
    fn rejects_asm_without_basic_start() {
        let error = convert_source(SourceKind::Asm, "ORG 8000H\nRET\n")
            .err()
            .unwrap();
        assert!(error.contains("BASIC_START"));
    }

    #[cfg(feature = "asm-toml")]
    #[test]
    fn flattens_toml_with_gap_and_asm_autostart() {
        let stub = tokenize_program("10 LET R=USR(7168)\n").unwrap();
        let toml = format!(
            "format = \"rtvc-asm-v1\"\norigin = 0x19EF\nnext_addr = 0x1C01\n\n[[segments]]\naddr = 0x1C00\nlen = 1\nbytes = [0xC9]\n\n[[segments]]\naddr = 0x19EF\nlen = {}\nbytes = [{}]\n",
            stub.len(),
            stub.iter()
                .map(|b| format!("0x{b:02X}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let (cas, note) = convert_source(SourceKind::Toml, &toml).unwrap();
        let note = note.unwrap();
        assert!(note.contains("padding"), "{note}");
        assert_eq!(cas[0x84], 0xFF);
        assert_eq!(&cas[0x87..0x89], &[0xEF, 0x19]);
        let payload_len = u16::from_le_bytes([cas[0x82], cas[0x83]]) as usize;
        assert_eq!(payload_len, (0x1C00 - 0x19EF) + 1);
        assert_eq!(cas.len(), TVC_CAS_HEADER_LEN + payload_len);
        assert_eq!(
            &cas[TVC_CAS_HEADER_LEN..TVC_CAS_HEADER_LEN + stub.len()],
            stub
        );
        let code_off = TVC_CAS_HEADER_LEN + (0x1C00 - 0x19EF) as usize;
        assert_eq!(cas[code_off], 0xC9);
        let relocated = convert_source(
            SourceKind::Toml,
            "format = \"rtvc-asm-v1\"\norigin = 0x4000\nnext_addr = 0x4001\n\n[[segments]]\naddr = 0x4000\nlen = 1\nbytes = [0x00]\n",
        )
        .unwrap_err();
        assert!(relocated.contains("4000"), "{relocated}");
    }
}
