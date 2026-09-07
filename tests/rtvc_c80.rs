//! CLI integration tests for rtvc-c80 (T07).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rtvc-c80"))
}

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rtvc-c80-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
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
    let _ = fs::remove_dir_all(&dir);
}
