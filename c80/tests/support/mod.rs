//! Headless TVC helpers shared by the integration tests that load C80
//! programs into the emulator (`rtvc_c80.rs`, `tvc.rs`).
#![allow(dead_code)]

use rtvc_core::bus::CpuBus;

pub fn boot_tvc12() -> rtvc_core::tvc::Tvc {
    let mut tvc =
        rtvc_core::tvc::Tvc::new_with_vid_model(false, rtvc_core::machine::VidModel::Interleaved);
    tvc.add_rom("TVC12_D4.64K", include_bytes!("../../../roms/TVC12_D4.64K"));
    tvc.add_rom("TVC12_D3.64K", include_bytes!("../../../roms/TVC12_D3.64K"));
    tvc.add_rom("TVC12_D7.64K", include_bytes!("../../../roms/TVC12_D7.64K"));
    tvc.set_fast_boot(true);
    tvc.reset();
    tvc
}

pub fn peek16(tvc: &mut rtvc_core::tvc::Tvc, addr: u16) -> u16 {
    u16::from_le_bytes([tvc.bus.r8(addr), tvc.bus.r8(addr.wrapping_add(1))])
}

pub fn wait_until(
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

pub fn type_line(tvc: &mut rtvc_core::tvc::Tvc, text: &str) {
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

/// Builds a TVC project manifest in process (paths resolve from the
/// manifest directory, as in the CLI) and returns the result with the
/// cassette `rtvc-tocas` would write: the segments flattened from 19EFH,
/// BASIC type, autostart.
pub fn build_cas(
    manifest_path: &std::path::Path,
    optimize: bool,
) -> (rtvc_c80::CompilationResult, Vec<u8>) {
    use rtvc_c80::project::{
        ProjectUnitInput, compile_project_opt, parse_manifest, render_rtvc_asm_v1,
        resolve_manifest_path,
    };
    use rtvc_core::cas::{
        TVC_CAS_LOAD_ADDR, TVC_CAS_TYPE_BASIC, encode_tvc_cas, flatten_cas_image,
    };

    let text = std::fs::read_to_string(manifest_path).unwrap();
    let manifest = parse_manifest(&text).unwrap();
    let dir = manifest_path.parent().unwrap();
    let owned: Vec<(String, String)> = manifest
        .units
        .iter()
        .map(|unit| {
            let path = resolve_manifest_path(dir, &unit.path);
            (
                path.to_string_lossy().into_owned(),
                std::fs::read_to_string(&path).unwrap(),
            )
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
        .map(|spec| std::fs::read_to_string(resolve_manifest_path(dir, &spec.path)).unwrap());
    let result = compile_project_opt(&manifest, &inputs, basic.as_deref(), optimize);
    assert!(
        !result.has_errors(),
        "{} (optimize={optimize}): {:#?}",
        manifest_path.display(),
        result.diagnostics
    );
    let code = result.code.as_ref().unwrap();
    let toml = render_rtvc_asm_v1("project.toml", 0x19EF, &code.assembled);
    let image = rtvc_core::asm::parse_rtvc_asm_v1(&toml).unwrap();
    let owned: Vec<(u16, Vec<u8>)> = image
        .segments
        .iter()
        .map(|s| (s.addr, s.bytes.clone()))
        .collect();
    let refs: Vec<(u16, &[u8])> = owned.iter().map(|(a, b)| (*a, b.as_slice())).collect();
    let linear = flatten_cas_image(&refs).unwrap();
    let cas = encode_tvc_cas(&linear.bytes, TVC_CAS_TYPE_BASIC, 0xFF, TVC_CAS_LOAD_ADDR);
    (result, cas)
}

/// A cold BASIC 1.2 boot with the cassette injected, ready for `RUN`.
pub fn boot_with_cas(cas: &[u8]) -> rtvc_core::tvc::Tvc {
    let mut tvc = boot_tvc12();
    wait_until(&mut tvc, 400, |tvc| peek16(tvc, 0x1722) == 0x19EF);
    for _ in 0..120 {
        tvc.run_for_a_frame();
    }
    assert!(tvc.load_cas(cas));
    tvc
}

/// Types `text` without the final Enter.
pub fn type_chars(tvc: &mut rtvc_core::tvc::Tvc, text: &str) {
    for ch in text.chars() {
        let code = ch as u32;
        tvc.key_down(code);
        tvc.key_press(ch);
        for _ in 0..8 {
            tvc.run_for_a_frame();
        }
        tvc.key_up(code);
        for _ in 0..3 {
            tvc.run_for_a_frame();
        }
    }
}

pub fn examples_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../info/c80")
}
