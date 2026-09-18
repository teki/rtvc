//! CLI integration tests for rtvc-c80 (T07 / T11).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rtvc_core::bus::CpuBus;

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

#[test]
fn pong_cas_inject_starts_the_playfield() {
    use rtvc_core::cas::{
        TVC_CAS_HEADER_LEN, TVC_CAS_LOAD_ADDR, TVC_CAS_TYPE_BASIC, encode_tvc_cas,
        flatten_cas_image,
    };
    use rtvc_c80::project::{
        ProjectUnitInput, UnitKind, compile_project, parse_manifest, render_rtvc_asm_v1,
    };

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../info/c80/pong");
    let manifest_text = fs::read_to_string(dir.join("rtvc-c80.toml")).unwrap();
    let src = fs::read_to_string(dir.join("pong.c80")).unwrap();
    let bas = fs::read_to_string(dir.join("main.bas")).unwrap();
    let manifest = parse_manifest(&manifest_text).unwrap();
    let result = compile_project(
        &manifest,
        &[ProjectUnitInput {
            name: "pong",
            kind: UnitKind::C80,
            origin: Some(0x3000),
            text: &src,
            source_name: "pong.c80",
            exports: &[],
            stack_extra: None,
        }],
        Some(&bas),
    );
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let code = result.code.as_ref().unwrap();
    let basic = result.basic.as_ref().unwrap();
    assert_eq!(basic.origin, 0x19EF);
    let play = code.function("play").expect("play").addr;
    assert!(play >= 0x3000, "play at {play:04X}");

    let toml = render_rtvc_asm_v1("pong.toml", 0x3000, &code.assembled);
    let assembled = rtvc_core::asm::parse_rtvc_asm_v1(&toml).unwrap();
    let owned: Vec<(u16, Vec<u8>)> = assembled
        .segments
        .iter()
        .map(|s| (s.addr, s.bytes.clone()))
        .collect();
    let refs: Vec<(u16, &[u8])> = owned.iter().map(|(a, b)| (*a, b.as_slice())).collect();
    let image = flatten_cas_image(&refs).unwrap();
    let cas = encode_tvc_cas(&image.bytes, TVC_CAS_TYPE_BASIC, 0xFF, TVC_CAS_LOAD_ADDR);
    let linear = &cas[TVC_CAS_HEADER_LEN..];

    let mut tvc = boot_tvc12();
    wait_until(&mut tvc, 400, |tvc| peek16(tvc, 0x1722) == 0x19EF);
    for _ in 0..120 {
        tvc.run_for_a_frame();
    }
    assert!(tvc.load_cas(&cas));
    for (i, byte) in linear.iter().enumerate() {
        assert_eq!(
            tvc.bus.r8(TVC_CAS_LOAD_ADDR.wrapping_add(i as u16)),
            *byte,
            "injected RAM at {:04X}",
            TVC_CAS_LOAD_ADDR.wrapping_add(i as u16)
        );
    }
    assert_eq!(
        tvc.bus.r8(play),
        linear[(play - TVC_CAS_LOAD_ADDR) as usize],
        "play bytes injected"
    );
    let extra = (basic.payload.len() as u16).saturating_sub(1);
    let last = TVC_CAS_LOAD_ADDR.wrapping_add(extra);
    assert_eq!(
        peek16(&mut tvc, 0x1722),
        TVC_CAS_LOAD_ADDR,
        "TEXT after inject"
    );
    assert_eq!(peek16(&mut tvc, 0x1724), last, "CHAIN after inject");
    assert_eq!(peek16(&mut tvc, 0x1726), last, "TOP after inject");
    type_line(&mut tvc, "RUN");
    let paddle_row = 0x8000u16.wrapping_add(108 * 64);
    let ball = 0x8000u16.wrapping_add(118 * 64).wrapping_add(32);
    wait_until(&mut tvc, 600, |tvc| {
        tvc.bus.mmu.get_map_val() == 0x50
            && tvc.bus.r8(paddle_row.wrapping_add(2)) == 0xFF
            && tvc.bus.r8(paddle_row.wrapping_add(61)) == 0xFF
            && tvc.bus.r8(ball) == 0xFF
    });
    assert_eq!(tvc.bus.mmu.get_map_val(), 0x50, "VID0 mapped at 8000H");
    let pc = tvc.z80.state.pc;
    assert!(
        (0x3000..0x4000).contains(&pc),
        "PC {pc:04X} should be in pong after RUN"
    );
    let bx_addr = code
        .globals
        .iter()
        .find(|g| g.name == "bx")
        .expect("bx")
        .addr;
    let py_addr = code
        .globals
        .iter()
        .find(|g| g.name == "py")
        .expect("py")
        .addr;
    assert_eq!(tvc.bus.r8(bx_addr), 32, "ball starts centred");
    assert_eq!(tvc.bus.r8(py_addr), 108, "player paddle start");
    assert_eq!(
        tvc.bus.vid.cursor_interrupt_setup(),
        (0x0EFF, Some(239)),
        "playfield IRQ must be last paper line, not VT-DOS 0x0AFF"
    );

    tvc.key_down(32);
    wait_until(&mut tvc, 90, |tvc| tvc.bus.r8(bx_addr) != 32);
    tvc.key_up(32);
    assert_ne!(tvc.bus.r8(bx_addr), 32, "Space should serve");

    let py_before = tvc.bus.r8(py_addr);
    tvc.key_down(38);
    wait_until(&mut tvc, 90, |tvc| tvc.bus.r8(py_addr) < py_before);
    tvc.key_up(38);
    assert!(
        tvc.bus.r8(py_addr) < py_before,
        "joystick up should move the left paddle"
    );
}

fn boot_tvc12() -> rtvc_core::tvc::Tvc {
    let mut tvc =
        rtvc_core::tvc::Tvc::new_with_vid_model(false, rtvc_core::vid::VidModel::Interleaved);
    tvc.add_rom("TVC12_D4.64K", include_bytes!("../../roms/TVC12_D4.64K"));
    tvc.add_rom("TVC12_D3.64K", include_bytes!("../../roms/TVC12_D3.64K"));
    tvc.add_rom("TVC12_D7.64K", include_bytes!("../../roms/TVC12_D7.64K"));
    tvc.set_fast_boot(true);
    tvc.reset();
    tvc
}

fn peek16(tvc: &mut rtvc_core::tvc::Tvc, addr: u16) -> u16 {
    u16::from_le_bytes([tvc.bus.r8(addr), tvc.bus.r8(addr.wrapping_add(1))])
}

fn wait_until(
    tvc: &mut rtvc_core::tvc::Tvc,
    frames: u32,
    mut pred: impl FnMut(&mut rtvc_core::tvc::Tvc) -> bool,
) {
    for _ in 0..frames {
        tvc.run_for_a_frame();
        if pred(tvc) {
            return;
        }
    }
    let pc = tvc.z80.state.pc;
    let text = peek16(tvc, 0x1722);
    let marker = tvc.bus.r8(0x3FFF);
    panic!("timeout pc={pc:04X} TEXT={text:04X} m3fff={marker:02X}");
}

fn type_line(tvc: &mut rtvc_core::tvc::Tvc, text: &str) {
    for ch in text.chars().chain(std::iter::once('\r')) {
        let code = ch as u32;
        tvc.key_down(code);
        if ch != '\r' {
            tvc.key_press(ch);
        }
        for _ in 0..8 {
            tvc.run_for_a_frame();
        }
        tvc.key_up(code);
        for _ in 0..3 {
            tvc.run_for_a_frame();
        }
    }
    for _ in 0..40 {
        tvc.run_for_a_frame();
    }
}
