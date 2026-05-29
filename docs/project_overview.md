# Project Overview & Architecture — rtvc

## Project Scope

`rtvc` is a Videoton TV Computer (TVC) emulator written in Rust, ported from the JavaScript implementation [../jstvc](../../jstvc).

The project is structured as a Rust library crate with a native desktop binary plus multiple test and utility binaries defined in [Cargo.toml](../Cargo.toml). The emulator core is shared by native and WebAssembly frontends.

Future build-target and UI direction is tracked in [docs/future_plan.md](future_plan.md). Check it before changing Cargo features, video model selection, web UI, native UI, or storage behavior.

### Crate Files and Directory Structure

- [Cargo.toml](../Cargo.toml) — Package configuration specifying package edition, features, library crate types, and binaries.
- [src/lib.rs](../src/lib.rs) — Shared library entry point exporting the emulator core modules, native UI modules, and optional WASM bindings.
- [src/main.rs](../src/main.rs) — Entry point for the native TVC emulator binary (eframe/egui application).
- [src/z80.rs](../src/z80.rs) — Complete Z80 CPU emulator (supporting all documented and many undocumented opcodes).
- [src/mmu.rs](../src/mmu.rs) — TVC memory management unit implementing bank switching and flat memory helper.
- [src/vid.rs](../src/vid.rs) — TVC video controller (MC6845 CRTC emulation).
- [src/key.rs](../src/key.rs) — TVC keyboard matrix with dynamic auto-mapping.
- [src/tvc.rs](../src/tvc.rs) — System bus and machine orchestrator (TvcBus + Tvc).
- [src/emu.rs](../src/emu.rs) — Native-only high-level emulator wrapper (Emu struct with run state, filesystem ROM loading, and zipped disk loading).
- [src/ui.rs](../src/ui.rs) — Native-only egui/eframe GUI application (EmuApp) with screen display and IO log panel.
- [src/wasm.rs](../src/wasm.rs) — Lightweight WASM bindings exposing `Tvc` control, ROM/disk loading, keyboard input, and framebuffer access for a browser canvas UI.
- [src/snapshot.rs](../src/snapshot.rs) — Chunked snapshot format helpers shared by native and WASM snapshot save/load APIs.
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
- **Video**: `Vid` struct emulates the MC6845 CRTC and supports both `VidModel::Simple` (`draw_frame()` renders directly from VRAM once per frame) and `VidModel::Realistic` (`stream_some()`/`render_stream()` incrementally follow CRTC timing).
- **Keyboard**: `Key` struct implements a row/column matrix with dynamic auto-mapping from host keyboard codes to TVC layout.
- **System Bus**: `TvcBus` wraps MMU, Video, Keyboard, and Logger, implementing the `Mmu` trait for Z80 memory and I/O access with port dispatch.
- **Machine Orchestrator**: `Tvc` owns the bus, Z80 CPU, framebuffer, and runtime `VidModel` setting, providing `run_for_a_frame()` over 62500 CPU cycles.
- **Library Boundary**: [src/lib.rs](../src/lib.rs) exposes the emulator core as an `rlib` for native tooling and as a `cdylib` for WASM packaging.
- **Native Emulator Wrapper**: `Emu` wraps `Tvc` with run state, ROM loading from `roms/`, and zipped disk discovery from `disks/`. It is compiled only with the `native` feature.
- **Native GUI**: `EmuApp` (eframe/egui) displays the TVC screen at PAL 4:3 aspect ratio, routes keyboard input to the TVC, exposes the video model as a runtime setting, and shows an optional IO log panel. It is compiled only with the `native` feature.
- **WASM Facade**: `WasmTvc` in [src/wasm.rs](../src/wasm.rs) exposes a small `wasm-bindgen` API around `Tvc`, including `runFrame()`, `setVidModel()`, key events, ROM/disk loading, and direct framebuffer pointer/length access for JavaScript canvas rendering. The WASM build does not include egui, eframe, or zip.
- **Snapshots**: [docs/snapshot.md](snapshot.md) defines the custom `RTVCSNAP` chunked state format. User-facing snapshot and web bundle commands are in [README.md](../README.md).
- **Profiling**: Use a sampling profiler such as `samply` against the native binary when profiling CPU performance.

## Toolchain

- Rust Edition: `2024` (requires Rust ≥ 1.85).
- Default feature: `native`, which enables `egui` 0.31, `eframe` 0.31, and `zip` 2 for the desktop application.
- WASM feature: `wasm`, which enables only `wasm-bindgen` for the browser-facing API. Build it with `--no-default-features --features wasm`.
- Web video defaults: `web-vid-simple` and `web-vid-realistic` select the default `VidModel` for WASM constructors. They are mutually exclusive; omitting both defaults to simple for lightweight web builds.
- Package dependencies and metadata are managed in [Cargo.toml](../Cargo.toml).
