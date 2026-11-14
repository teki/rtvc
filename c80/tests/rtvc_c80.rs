//! CLI integration tests for rtvc-c80 (T07 / T11).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rtvc_core::bus::CpuBus;

mod support;
use support::{boot_tvc12, peek16, type_line, wait_until};

fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rtvc-c80"))
}

/// A fresh directory per call. Tests run on parallel threads of one process,
/// and the macOS wall clock ticks in microseconds, so pid + time alone gave
/// two tests the same directory (one then deleted it under the other). A
/// process-wide counter makes every name unique.
fn scratch() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "rtvc-c80-cli-{}-{}-{n}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn origin_is_invalid_on_a_manifest() {
    let dir = scratch();
    fs::write(dir.join("main.c80"), "void f() { return; }\n").unwrap();
    fs::write(
        dir.join("rtvc-c80.toml"),
        "[[unit]]\nname = \"main\"\npath = \"main.c80\"\norigin = 0x8000\n",
    )
    .unwrap();
    let out = Command::new(exe())
        .args([
            "build",
            dir.join("rtvc-c80.toml").to_str().unwrap(),
            "--origin",
            "0x8000",
            "--emit-asm",
            dir.join("out.asm").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--origin"), "{err}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compile_error_does_not_replace_existing_output() {
    let dir = scratch();
    let dest = dir.join("out.asm");
    fs::write(&dest, "keep").unwrap();
    fs::write(dir.join("bad.c80"), "void f() { return 1; }\n").unwrap();
    let out = Command::new(exe())
        .args([
            "build",
            dir.join("bad.c80").to_str().unwrap(),
            "--origin",
            "0x8000",
            "--emit-asm",
            dest.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "keep");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn manifest_paths_resolve_from_manifest_dir_not_cwd() {
    let dir = scratch();
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.c80"), "void f() { return; }\n").unwrap();
    fs::write(
        dir.join("proj.toml"),
        "[[unit]]\nname = \"main\"\npath = \"src/main.c80\"\norigin = 0x8000\n",
    )
    .unwrap();
    let cwd = dir.join("cwd");
    fs::create_dir_all(&cwd).unwrap();
    let out_asm = dir.join("out.asm");
    let out = Command::new(exe())
        .current_dir(&cwd)
        .args([
            "build",
            dir.join("proj.toml").to_str().unwrap(),
            "--emit-asm",
            out_asm.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = fs::read_to_string(&out_asm).unwrap();
    assert!(text.to_ascii_uppercase().contains("RET"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn multi_unit_cli_build_emits_segments() {
    let dir = scratch();
    fs::write(dir.join("main.c80"), "pub u8 inc(u8 x) { return x + 1; }\n").unwrap();
    fs::write(
        dir.join("boot.asm"),
        "start:\n    LD A, 7\n    CALL @{main::inc}\ncont:\n    RET\n",
    )
    .unwrap();
    fs::write(
        dir.join("proj.toml"),
        r#"
[[unit]]
name = "boot"
kind = "asm"
path = "boot.asm"
origin = 0x2000
exports = ["start"]

[[unit]]
name = "main"
path = "main.c80"
origin = 0x2200
"#,
    )
    .unwrap();
    let segs = dir.join("out.toml");
    let out = Command::new(exe())
        .args([
            "build",
            dir.join("proj.toml").to_str().unwrap(),
            "--emit-segments",
            segs.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = fs::read_to_string(&segs).unwrap();
    assert!(text.contains("rtvc-asm-v1"));
    assert!(text.contains("[[segments]]"));
    let image = rtvc_core::asm::parse_rtvc_asm_v1(&text).unwrap();
    assert!(image.segments.iter().any(|s| s.addr == 0x2000));
    assert!(image.segments.iter().any(|s| s.addr == 0x2200));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tvc_manifest_loads_basic_from_the_manifest_directory() {
    let dir = scratch();
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("echo.c80"),
        "@fastcall pub i16 echo(i16 value) { return value; }\n",
    )
    .unwrap();
    fs::write(src_dir.join("main.bas"), "10 LET R=USR(@{math::echo},42)\n").unwrap();
    fs::write(
        dir.join("proj.toml"),
        "target = \"tvc\"\n[basic]\npath = \"src/main.bas\"\n[[unit]]\nname = \"math\"\npath = \"src/echo.c80\"\norigin = 0x3000\n",
    )
    .unwrap();
    let cwd = std::env::temp_dir();
    let out = Command::new(exe())
        .current_dir(&cwd)
        .args([
            "build",
            dir.join("proj.toml").to_str().unwrap(),
            "--emit-asm",
            dir.join("out.asm").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mixed_cli_emit_segments_contains_basic_and_runs_usr() {
    let dir = scratch();
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("echo.c80"),
        "@fastcall pub i16 echo(i16 value) { return value; }\n",
    )
    .unwrap();
    fs::write(
        src_dir.join("main.bas"),
        "\
10 LET R=USR(@{math::echo},42)\n\
20 IF R=42 THEN POKE 16383,1\n\
30 LET R=USR(@{math::echo},-1)\n\
40 IF R=-1 THEN POKE 16383,PEEK(16383)+1\n\
50 LET R=USR(@{math::echo},32767)\n\
60 IF R=32767 THEN POKE 16383,PEEK(16383)+1\n\
70 LET R=USR(@{math::echo},-32767)\n\
80 IF R=-32767 THEN POKE 16383,PEEK(16383)+1\n\
90 POKE 16382,1\n",
    )
    .unwrap();
    fs::write(
        dir.join("proj.toml"),
        "target = \"tvc\"\n[basic]\npath = \"src/main.bas\"\n[[unit]]\nname = \"math\"\npath = \"src/echo.c80\"\norigin = 0x3000\n",
    )
    .unwrap();
    let segs = dir.join("out.toml");
    let asm = dir.join("out.asm");
    let out = Command::new(exe())
        .args([
            "build",
            dir.join("proj.toml").to_str().unwrap(),
            "--emit-segments",
            segs.to_str().unwrap(),
            "--emit-asm",
            asm.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = fs::read_to_string(&segs).unwrap();
    let image = rtvc_core::asm::parse_rtvc_asm_v1(&text).unwrap();
    let base = *image.symbols.get("project::basic_base").unwrap();
    let size = *image.symbols.get("project::basic_program_size").unwrap() as usize;
    assert_eq!(base, 0x19EF);
    let basic = image
        .segments
        .iter()
        .find(|s| s.addr == base)
        .expect("BASIC segment");
    assert_eq!(basic.bytes.len(), size);
    assert_eq!(*basic.bytes.last().unwrap(), 0);
    let asm_text = fs::read_to_string(&asm).unwrap();
    assert!(asm_text.contains("ORG 0x19EF"));

    let bin = Command::new(exe())
        .args([
            "build",
            dir.join("proj.toml").to_str().unwrap(),
            "--emit-bin",
            dir.join("out.bin").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!bin.status.success());
    let err = String::from_utf8_lossy(&bin.stderr);
    assert!(err.contains("contiguous"), "{err}");

    let owned: Vec<(u16, Vec<u8>)> = image
        .segments
        .iter()
        .map(|s| (s.addr, s.bytes.clone()))
        .collect();
    let refs: Vec<(u16, &[u8])> = owned.iter().map(|(a, b)| (*a, b.as_slice())).collect();
    let linear = rtvc_core::cas::flatten_cas_image(&refs).unwrap();
    let cas = rtvc_core::cas::encode_tvc_cas(
        &linear.bytes,
        rtvc_core::cas::TVC_CAS_TYPE_BASIC,
        0xFF,
        rtvc_core::cas::TVC_CAS_LOAD_ADDR,
    );

    let mut tvc = boot_tvc12();
    wait_until(&mut tvc, 400, |tvc| peek16(tvc, 0x1722) == 0x19EF);
    for _ in 0..120 {
        tvc.run_for_a_frame();
    }
    assert!(tvc.load_cas(&cas));
    type_line(&mut tvc, "RUN");
    wait_until(&mut tvc, 1200, |tvc| {
        tvc.bus.r8(0x3FFF) == 4 && tvc.bus.r8(0x3FFE) == 1
    });
    assert_eq!(tvc.bus.r8(0x3FFF), 4);
    assert_eq!(tvc.bus.r8(0x3FFE), 1);
    let _ = fs::remove_dir_all(&dir);
}

/// Compiles a manifest exactly as the CLI's `compile_input` does: unit and
/// BASIC paths resolve from the manifest directory.
fn compile_manifest_like_cli(
    manifest_path: &std::path::Path,
    optimize: bool,
) -> rtvc_c80::CompilationResult {
    use rtvc_c80::project::{
        ProjectUnitInput, compile_project_opt, parse_manifest, resolve_manifest_path,
    };

    let text = fs::read_to_string(manifest_path).unwrap();
    let manifest =
        parse_manifest(&text).unwrap_or_else(|err| panic!("{}: {err}", manifest_path.display()));
    let dir = manifest_path.parent().unwrap();
    let owned: Vec<(PathBuf, String)> = manifest
        .units
        .iter()
        .map(|unit| {
            let path = resolve_manifest_path(dir, &unit.path);
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("cannot read unit '{}': {err}", path.display()));
            (path, text)
        })
        .collect();
    let inputs: Vec<ProjectUnitInput<'_>> = manifest
        .units
        .iter()
        .zip(owned.iter())
        .map(|(unit, (path, text))| ProjectUnitInput {
            name: &unit.name,
            kind: unit.kind,
            origin: unit.origin,
            text,
            source_name: path.to_str().unwrap_or(&unit.name),
            exports: &unit.exports,
            stack_extra: unit.stack_extra,
        })
        .collect();
    let basic_text = manifest.basic.as_ref().map(|spec| {
        let path = resolve_manifest_path(dir, &spec.path);
        fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read BASIC '{}': {err}", path.display()))
    });
    compile_project_opt(&manifest, &inputs, basic_text.as_deref(), optimize)
}

#[test]
fn tutorial_examples_build_without_errors() {
    use rtvc_c80::{C80Target, CompileInput, SourceInput, compile};

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../info/c80");
    let mut entries: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    let mut sources = 0;
    let mut manifests = 0;
    let mut failures = Vec::new();
    for path in &entries {
        let name = path.file_name().unwrap().to_str().unwrap();
        // *.fail.c80 are rejected on purpose; tests/examples.rs checks them.
        if path.extension().and_then(|s| s.to_str()) == Some("c80") && !name.ends_with(".fail.c80")
        {
            sources += 1;
            let text = fs::read_to_string(path).unwrap();
            for optimize in [true, false] {
                let result = compile(CompileInput {
                    files: vec![SourceInput { name, text: &text }],
                    origin: 0x8000,
                    optimize,
                    target: C80Target::GenericZ80,
                });
                if result.has_errors() || result.code.is_none() {
                    failures.push(format!(
                        "{name} (optimize={optimize}): {:?}",
                        result.diagnostics
                    ));
                }
            }
        }
        let manifest = path.join("rtvc-c80.toml");
        if path.is_dir() && manifest.is_file() {
            manifests += 1;
            for optimize in [true, false] {
                let result = compile_manifest_like_cli(&manifest, optimize);
                if result.has_errors() || result.code.is_none() {
                    failures.push(format!(
                        "{} (optimize={optimize}): {:?}",
                        manifest.display(),
                        result.diagnostics
                    ));
                }
            }
        }
    }
    assert!(sources >= 10, "found only {sources} .c80 examples");
    assert!(manifests >= 4, "found only {manifests} manifests");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn diagnostics_print_path_line_column_and_source() {
    let dir = scratch();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("src/bad.c80"),
        "u8 ok() { return 1; }\nu8 f(u8 a) {\n    return a + 300;\n}\n",
    )
    .unwrap();
    // Run from `dir` with a relative path: the path is printed as given.
    let out = Command::new(exe())
        .current_dir(&dir)
        .args([
            "build",
            "src/bad.c80",
            "--origin",
            "0x8000",
            "--emit-asm",
            "out.asm",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    let first = err.lines().next().unwrap_or("");
    assert!(
        first.starts_with("src/bad.c80:3:16: error ty-literal-range: "),
        "{err}"
    );
    assert!(err.contains("3 |     return a + 300;\n"), "{err}");
    assert!(err.contains("  |                ^~~\n"), "{err}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn manifest_diagnostics_use_the_manifest_relative_path() {
    let dir = scratch();
    fs::create_dir_all(dir.join("proj")).unwrap();
    fs::write(dir.join("proj/main.c80"), "void f() {\n    return 1;\n}\n").unwrap();
    fs::write(
        dir.join("proj/rtvc-c80.toml"),
        "[[unit]]\nname = \"main\"\npath = \"main.c80\"\norigin = 0x8000\n",
    )
    .unwrap();
    let out = Command::new(exe())
        .current_dir(&dir)
        .args(["build", "proj/rtvc-c80.toml", "--emit-asm", "out.asm"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.starts_with("proj/main.c80:2:"), "{err}");
    assert!(err.contains("2 |     return 1;\n"), "{err}");
    let _ = fs::remove_dir_all(&dir);
}
