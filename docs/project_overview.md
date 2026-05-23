# Project Overview & Architecture — rtvc

## Project Scope

`rtvc` is a Videoton TV Computer (TVC) emulator written in Rust, ported from the JavaScript implementation `../jstvc`.

The project is structured as a single Rust binary crate with multiple test and utility binaries defined in `Cargo.toml`.

### Crate Files and Directory Structure

- [Cargo.toml](../Cargo.toml) — Package configuration specifying package edition and binaries.
- [src/main.rs](../src/main.rs) — Entry point for the main TVC emulator binary (eframe/egui application).
- [src/z80.rs](../src/z80.rs) — Complete Z80 CPU emulator (supporting all documented and many undocumented opcodes).
- [src/mmu.rs](../src/mmu.rs) — TVC memory management unit implementing bank switching and flat memory helper.
- [src/vid.rs](../src/vid.rs) — TVC video controller (MC6845 CRTC emulation).
- [src/key.rs](../src/key.rs) — TVC keyboard matrix with dynamic auto-mapping.
- [src/tvc.rs](../src/tvc.rs) — System bus and machine orchestrator (TvcBus + Tvc).
- [src/emu.rs](../src/emu.rs) — High-level emulator wrapper (Emu struct with run state and ROM loading).
- [src/ui.rs](../src/ui.rs) — egui/eframe GUI application (EmuApp) with screen display and IO log panel.
- [src/log.rs](../src/log.rs) — Logger trait and ring-buffer log implementation.
- [src/fuse_test.rs](../src/fuse_test.rs) — FUSE test harness executable.
- [src/zex_test.rs](../src/zex_test.rs) — Z80 Instruction exercise test runner (zexall/zexdoc).
- [src/perf_test.rs](../src/perf_test.rs) — Performance benchmark suite running `zexdoc`.
- [tests/](../tests/) — Contains test files copied from the JS implementation:
  - `tests.in` / `tests.expected` — FUSE test vectors.
  - `zexdoc.com` / `zexall.com` — ZEXDOC/ZEXALL binary test programs.
  - `test.js` — Original JS test runner for comparison.

## Architecture

- **Z80 CPU**: The CPU emulator closely follows the design and behavior of the JavaScript implementation in `../jstvc/src/z80.js`.
- **MMU**:
  - `FakeMmu` in `mmu.rs` provides a flat, 64 KB memory space specifically for running CPU tests (like FUSE and ZEX).
  - `TvcMmu` in `mmu.rs` implements TVC bank switching (mapping external/internal memory banks into four 16 KB pages), wired to the main binary via `TvcBus`.
- **Video**: `Vid` struct emulates the MC6845 CRTC with `draw_frame()` rendering directly from VRAM to a 608×288 framebuffer.
- **Keyboard**: `Key` struct implements a row/column matrix with dynamic auto-mapping from host keyboard codes to TVC layout.
- **System Bus**: `TvcBus` wraps MMU, Video, Keyboard, and Logger, implementing the `Mmu` trait for Z80 memory and I/O access with port dispatch.
- **Machine Orchestrator**: `Tvc` owns the bus, Z80 CPU, and framebuffer, providing `run_for_a_frame()` (62500 CPU cycles + `draw_frame()`).
- **Emulator Wrapper**: `Emu` wraps `Tvc` with run state and ROM loading from `roms/` directory.
- **GUI**: `EmuApp` (eframe/egui) displays the TVC screen at PAL 4:3 aspect ratio, routes keyboard input to the TVC, and shows an optional IO log panel.

## Toolchain

- Rust Edition: `2024` (requires Rust ≥ 1.85).
- GUI: `egui` 0.31 and `eframe` 0.31.
- Package dependencies and metadata are managed in [Cargo.toml](file:///Users/teki/dev/rtvc/Cargo.toml).
