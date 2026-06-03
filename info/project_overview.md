# Project Overview & Architecture — rtvc

## Project Scope

`rtvc` is a Videoton TV Computer (TVC) emulator written in Rust, derived from the earlier [teki/jstvc](https://github.com/teki/jstvc) emulator.

The project is structured as a Rust library crate with a native desktop binary plus multiple test and utility binaries defined in [Cargo.toml](../Cargo.toml). The emulator core is shared by native and WebAssembly frontends.

Future build-target and UI direction is tracked in [info/future_plan.md](future_plan.md). Check it before changing Cargo features, video model selection, web UI, native UI, or storage behavior.

### Crate Files and Directory Structure

- [Cargo.toml](../Cargo.toml) — Package configuration specifying package edition, MIT license, features, library crate types, and binaries.
- [src/lib.rs](../src/lib.rs) — Shared library entry point exporting the emulator core modules, native UI modules, and optional WASM bindings.
- [src/main.rs](../src/main.rs) — Entry point for the native TVC emulator binary (eframe/egui application).
- [src/z80.rs](../src/z80.rs) — Complete Z80 CPU emulator (supporting all documented and many undocumented opcodes).
- [src/disasm.rs](../src/disasm.rs) — Compact Z80 disassembler built from opcode bit fields plus explicit prefix/special-case decoding, returning decoded bytes, mnemonic text, flags, short behavior notes, and T-state timing metadata.
- [info/z80opcodes.md](z80opcodes.md) — Merged Z80 opcode, flag, effect, and T-state reference used to keep disassembler metadata understandable.
- [src/bus.rs](../src/bus.rs) — Z80-facing `CpuBus` trait and flat `FakeBus` test implementation.
- [src/mmu.rs](../src/mmu.rs) — TVC memory management unit implementing bank switching.
- [src/vid.rs](../src/vid.rs) — TVC video controller (MC6845 CRTC emulation).
- [info/sound.md](sound.md) — TVC programmable sound generator, timer interrupt, and PCM sample output model.
- [src/key.rs](../src/key.rs) — TVC keyboard matrix with dynamic auto-mapping.
- [src/tvc.rs](../src/tvc.rs) — System bus and machine orchestrator (TvcBus + Tvc).
- [src/tape.rs](../src/tape.rs) — Cassette tape playback/input state.
- [src/sound.rs](../src/sound.rs) — Programmable sound generator, 4-bit DAC/amplitude register, PCM sample renderer, and shared interrupt source.
- [src/audio.rs](../src/audio.rs) — Native-only `cpal` output sink for draining generated PCM samples to the default host audio device.
- [src/expansion.rs](../src/expansion.rs) — Expansion slot/card routing for memory and I/O windows.
- [src/emu.rs](../src/emu.rs) — Native-only high-level emulator wrapper (Emu struct with run state, filesystem ROM loading, and zipped program loading).
- [src/ui.rs](../src/ui.rs) — Native-only egui/eframe GUI application (EmuApp) with screen display and IO log panel.
- [src/wasm.rs](../src/wasm.rs) — Lightweight WASM bindings exposing `Tvc` control, ROM/disk loading, keyboard input, and framebuffer access for a browser canvas UI.
- [src/snapshot.rs](../src/snapshot.rs) — Chunked snapshot format helpers shared by native and WASM snapshot save/load APIs.
- [src/tvc_snapshot.rs](../src/tvc_snapshot.rs) — TVC-specific snapshot chunk save/load glue used by `Tvc`.
- [src/log.rs](../src/log.rs) — Logger trait and ring-buffer log implementation.
- [src/cas2wav.rs](../src/cas2wav.rs) — CAS-to-WAV utility binary using [src/cas.rs](../src/cas.rs) tape intervals and emitting legacy-compatible 44.1 kHz unsigned 8-bit PCM.
- [src/fuse_test.rs](../src/fuse_test.rs) — FUSE test harness executable.
- [src/zex_test.rs](../src/zex_test.rs) — Z80 Instruction exercise test runner (zexall/zexdoc).
- [src/perf_test.rs](../src/perf_test.rs) — Performance benchmark suite running `zexdoc`.
- [tests/](../tests/) — Contains CPU test fixtures:
  - `tests.in` / `tests.expected` — FUSE test vectors.
  - `zexdoc.com` / `zexall.com` — ZEXDOC/ZEXALL binary test programs.

## Architecture

- **Z80 CPU**: The CPU emulator is implemented in [src/z80.rs](../src/z80.rs) and supports all documented and many undocumented Z80 opcodes.
- **MMU**:
  - `FakeBus` in `bus.rs` provides a flat, 64 KB CPU bus specifically for running CPU tests (like FUSE and ZEX).
  - `TvcMmu` in `mmu.rs` implements TVC bank switching (mapping external/internal memory banks into four 16 KB pages), wired to the main binary via `TvcBus`.
- **CPU Bus Trait**: `CpuBus` in `bus.rs` is the Z80-facing memory and I/O interface used by the CPU core. Test memory and the full TVC bus both implement it. The compact disassembler in [src/disasm.rs](../src/disasm.rs) also reads through this trait, so it can decode instructions from either `FakeBus` or the full TVC bus. Its `DisassembledInstruction` metadata uses the `SZHPNC` flag order from [info/z80href.txt](z80href.txt) and T-state notation from [info/z80inst.txt](z80inst.txt), including conditional forms such as `12/7`; [info/z80opcodes.md](z80opcodes.md) merges the opcode, flag, effect, and timing references in one maintained document.
- **Video**: `Vid` struct emulates the MC6845 CRTC and supports two runtime video schedules: `VidModel::FastFrame` (`draw_frame()` after one screen-time CPU budget) and `VidModel::Interleaved` (`stream_some()`/`render_stream()` after each CPU instruction).
- **Sound**: `SoundTimer` models the TVC's 12-bit programmable divider, fixed 4-bit sound stage, amplitude register, DAC mode, and shared sound interrupt. It renders mono 44.1 kHz `f32` PCM samples that frontends can drain through `Tvc::take_audio_samples()`.
- **Keyboard**: `Key` struct implements a row/column matrix with dynamic auto-mapping from host keyboard codes to TVC layout.
- **System Bus**: `TvcBus` wraps the MMU, Video, Keyboard, tape interface, sound timer, logger, and expansion slots. It implements `CpuBus`, dispatching CPU memory and I/O accesses to the relevant device. Expansion memory and I/O routing lives here, while `TvcMmu` remains the internal memory mapper.
- **Machine Orchestrator**: `Tvc` owns the bus, Z80 CPU, framebuffer, and runtime `VidModel` setting, providing `run_for_a_frame()` over 62500 CPU cycles. Interleaved mode is bounded by this host screen-time budget and draws a black background with moving white stripes after several consecutive host ticks without a synchronized CRTC frame.
- **Library Boundary**: [src/lib.rs](../src/lib.rs) exposes the emulator core as an `rlib` for native tooling and as a `cdylib` for WASM packaging.
- **Native Emulator Wrapper**: `Emu` wraps `Tvc` with run state, ROM loading from `roms/`, and zipped program discovery from `progs/`. Native lookup checks the current working directory first, then directories beside the executable so release archives can include ready-to-run `roms/` and `progs/` folders. It is compiled only with the `native` feature.
- **Native GUI**: `EmuApp` (eframe/egui) displays the TVC screen at PAL 4:3 aspect ratio, routes keyboard input to the TVC, exposes the video model as a runtime setting, drains generated audio samples into the native `cpal` sink, and shows an optional IO log panel. While running, it requests continuous repaints and generates TVC frames from a 50 Hz real-time gate so display refreshes reuse the latest texture instead of running the emulator once per host repaint. It is compiled only with the `native` feature.
- **WASM Facade**: `WasmTvc` in [src/wasm.rs](../src/wasm.rs) exposes a small `wasm-bindgen` API around `Tvc`, including `runFrame()`, `setVidModel()`, audio sample draining, key events, ROM/disk loading, and direct framebuffer pointer/length access for JavaScript canvas rendering. The generated lightweight web bundle feeds drained audio samples to a browser `AudioWorklet`. The WASM build does not include cpal, egui, eframe, or zip.
- **Snapshots**: [info/snapshot.md](snapshot.md) defines the custom `RTVCSNAP` chunked state format, while `tvc_snapshot.rs` maps `Tvc` state to those chunks. User-facing snapshot and web bundle commands are in [README.md](../README.md).
- **Cassette WAV Utility**: `cargo run --bin cas2wav -- input.cas output.wav [tape-name]` converts CAS images into the same 44.1 kHz unsigned 8-bit PCM waveform as the legacy [tools/cas2wav](../tools/cas2wav) converter.
- **Profiling**: Use a sampling profiler such as `samply` against the native binary when profiling CPU performance.

## Toolchain

- Rust Edition: `2024` (requires Rust ≥ 1.85).
- Default feature: `native`, which enables `cpal` 0.17, `egui` 0.31, `eframe` 0.31, and `zip` 2 for the desktop application.
- WASM feature: `wasm`, which enables only `wasm-bindgen` for the browser-facing API. Build it with `--no-default-features --features wasm`.
- Native `Tvc::new()` defaults to `VidModel::Interleaved`. WASM constructors default to `VidModel::FastFrame`; browser callers can still switch modes through the WASM string API, which accepts `fast-frame` and `interleaved` plus the legacy aliases `simple` and `realistic`.
- Package dependencies and metadata are managed in [Cargo.toml](../Cargo.toml).
- License: MIT for emulator code. ROMs, cassette/disk images, snapshots, screenshots, manuals, and other historical or third-party machine materials may be present for preservation, compatibility testing, or convenience, but are outside the project license unless explicitly stated.
