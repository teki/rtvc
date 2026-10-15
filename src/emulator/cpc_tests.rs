use super::*;

fn rom_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("roms")
        .join(name)
}

fn booted_cpc() -> Option<Cpc> {
    let os = std::fs::read(rom_path("CPC_OS_464.ROM")).ok()?;
    let basic = std::fs::read(rom_path("CPC_BASIC_1.0.ROM")).ok()?;
    let mut cpc = Cpc::new();
    cpc.load_roms(&os, &basic).unwrap();
    Some(cpc)
}

/// Programs the firmware's default 50 Hz CRTC setup.
fn program_default_crtc(bus: &mut CpcBus) {
    let regs = [63, 40, 46, 0x8E, 38, 0, 25, 30, 0, 7, 0, 0, 0x30, 0x00];
    for (index, value) in regs.into_iter().enumerate() {
        bus.out8(0x00, index as u8, 0xBC);
        bus.out8(0x00, value, 0xBD);
    }
}

#[test]
fn default_crtc_frame_is_312_lines_with_six_interrupts() {
    let mut bus = CpcBus::new();
    program_default_crtc(&mut bus);
    // Settle into the first VSYNC, then count one whole frame.
    let mut chars = 0;
    while !bus.vid.take_frame_ready() {
        bus.run_chars(1);
        chars += 1;
        assert!(chars < 40_000, "no VSYNC");
    }

    let mut interrupts = 0;
    let mut frame_chars = 0;
    while !bus.vid.take_frame_ready() {
        bus.run_chars(1);
        frame_chars += 1;
        if bus.irq_pending() {
            interrupts += 1;
            bus.acknowledge_interrupt();
        }
    }
    assert_eq!(frame_chars, 312 * 64);
    assert_eq!(interrupts, 6);
}

#[test]
fn ppi_vsync_bit_follows_crtc() {
    let mut bus = CpcBus::new();
    program_default_crtc(&mut bus);
    let mut seen_high = false;
    let mut seen_low = false;
    for _ in 0..312 * 64 * 2 {
        bus.run_chars(1);
        if bus.in8(0, 0xF5) & 1 != 0 {
            seen_high = true;
        } else {
            seen_low = true;
        }
    }
    assert!(seen_high && seen_low);
}

#[test]
fn keyboard_row_is_read_through_psg_register_14() {
    let mut bus = CpcBus::new();
    bus.set_key(8, 5, true); // A

    // Select AY register 14.
    bus.out8(0, 0x82, 0xF7);
    bus.out8(0, 14, 0xF4);
    bus.out8(0, 0xC0, 0xF6);
    bus.out8(0, 0x00, 0xF6);
    // Port A as input, read row 8.
    bus.out8(0, 0x92, 0xF7);
    bus.out8(0, 0x48, 0xF6);
    assert_eq!(bus.in8(0, 0xF4), 0xDF);
    bus.out8(0, 0x47, 0xF6);
    assert_eq!(bus.in8(0, 0xF4), 0xFF);
}

#[test]
fn gate_array_controls_rom_mapping() {
    let mut bus = CpcBus::new();
    bus.load_roms(&[0x11; ROM_SIZE], &[0x22; ROM_SIZE]).unwrap();
    bus.w8(0x0000, 0xAA);
    bus.w8(0xC000, 0xBB);
    assert_eq!(bus.r8(0x0000), 0x11);
    assert_eq!(bus.r8(0xC000), 0x22);

    bus.out8(0, 0x8C | 0x01, 0x7F);
    assert_eq!(bus.r8(0x0000), 0xAA);
    assert_eq!(bus.r8(0xC000), 0xBB);
}

#[test]
fn mode_change_applies_at_next_hsync() {
    let mut bus = CpcBus::new();
    program_default_crtc(&mut bus);
    bus.out8(0, 0x82, 0x7F);
    assert_eq!(bus.vid.mode(), 1);
    bus.run_chars(64);
    assert_eq!(bus.vid.mode(), 2);
}

#[test]
fn firmware_boots_to_basic_and_runs_typed_command() {
    let Some(mut cpc) = booted_cpc() else {
        eprintln!("skipping: CPC ROMs not present");
        return;
    };
    for _ in 0..150 {
        cpc.run_for_a_frame();
    }
    // The firmware programs mode 1 with the 0xC000 screen.
    assert_eq!(cpc.bus.vid.crtc().reg(12), 0x30);
    assert_eq!(cpc.bus.vid.mode(), 1);
    assert!(cpc.bus.ram()[0xC000..].iter().any(|&byte| byte != 0));

    // POKE a marker through BASIC to prove the keyboard path works.
    for ch in "poke &4000,42\r".chars() {
        cpc.key_press(ch);
        for _ in 0..3 {
            cpc.run_for_a_frame();
        }
    }
    for _ in 0..25 {
        cpc.run_for_a_frame();
    }
    assert_eq!(cpc.bus.ram()[0x4000], 42);
}

/// Loads `CPC_TEST_TAPE` with `RUN"` at full speed and writes PPM screenshots
/// to `CPC_TEST_OUT` every 500 frames, for checking real tapes:
/// `CPC_TEST_TAPE=game.cdt CPC_TEST_OUT=/tmp/cpc cargo test --release -- --ignored loads_tape`
#[test]
#[ignore]
fn loads_tape_from_env() {
    let (Ok(tape_path), Ok(out_dir)) = (
        std::env::var("CPC_TEST_TAPE"),
        std::env::var("CPC_TEST_OUT"),
    ) else {
        return;
    };
    let mut cpc = booted_cpc().expect("CPC ROMs");
    let tape = crate::tzx::Tape::parse(&std::fs::read(&tape_path).unwrap()).unwrap();
    cpc.bus.insert_tape(tape);
    for _ in 0..150 {
        cpc.run_for_a_frame();
    }
    for ch in "run\"\r".chars() {
        cpc.key_press(ch);
        for _ in 0..3 {
            cpc.run_for_a_frame();
        }
    }
    for _ in 0..25 {
        cpc.run_for_a_frame();
    }
    cpc.key_press(' ');
    let max_frames: usize = std::env::var("CPC_TEST_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(30_000);
    for frame in 0..max_frames {
        cpc.run_for_a_frame();
        if frame % 500 == 0 || frame + 1 == max_frames {
            let tape = cpc.bus.tape().unwrap();
            eprintln!(
                "frame {frame}: tape {}% motor {} loading {} pc {:04X}",
                tape.progress_percent(),
                cpc.bus.motor_on(),
                cpc.tape_loading(),
                cpc.z80.state.pc
            );
            write_ppm(&cpc, &format!("{out_dir}/frame{frame:05}.ppm"));
        }
    }
}

fn write_ppm(cpc: &Cpc, path: &str) {
    use crate::cpc_vid::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
    let mut data = format!("P6 {FRAMEBUFFER_WIDTH} {FRAMEBUFFER_HEIGHT} 255\n").into_bytes();
    for pixel in cpc.framebuffer() {
        data.extend_from_slice(&pixel.to_le_bytes()[..3]);
    }
    std::fs::write(path, data).unwrap();
}

#[test]
fn typed_symbol_replaces_the_host_keys_that_produced_it() {
    let mut cpc = Cpc::new();
    // Host Shift+; producing ':' must not leave the CPC Shift down (':' is
    // unshifted on the CPC; Shift+: is '*').
    cpc.key_down(16);
    cpc.key_press(':');
    let mut expected = [0xFF; KEYBOARD_ROWS];
    expected[3] &= !(1 << 5);
    assert_eq!(cpc.bus.keyboard, expected);
    cpc.key_up(16);
    cpc.run_for_a_frame();
    assert_eq!(cpc.bus.keyboard, [0xFF; KEYBOARD_ROWS]);
}
