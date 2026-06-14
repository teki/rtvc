# Project Overview & Architecture — rtvc

## Project Scope

`rtvc` is a Videoton TV Computer (TVC) emulator written in Rust, derived from the earlier [teki/jstvc](https://github.com/teki/jstvc) emulator.

The project is structured as a Rust library crate with a native desktop binary plus multiple test and utility binaries defined in [Cargo.toml](../Cargo.toml). The emulator core is shared by native and WebAssembly frontends.

### Crate Files and Directory Structure

- [Cargo.toml](../Cargo.toml) — Package configuration specifying package edition, MIT license, features, library crate types, and binaries.
- [src/lib.rs](../src/lib.rs) — Shared library entry point exporting the emulator core modules, native UI modules, and optional WASM bindings.
- [src/main.rs](../src/main.rs) — Entry point for the native TVC emulator binary (eframe/egui application).
- [src/z80.rs](../src/z80.rs) — Complete Z80 CPU emulator (supporting all documented and many undocumented opcodes).
- [src/disasm.rs](../src/disasm.rs) — Compact Z80 disassembler built from opcode bit fields plus explicit prefix/special-case decoding, returning decoded bytes, mnemonic text, flags, short behavior notes, and T-state timing metadata.
- [src/asm.rs](../src/asm.rs) — Small dependency-free, single-line Z80 assembler used by the debugger, including relative branches, data bytes, and documented base/prefixed forms.
- [info/z80opcodes.md](z80opcodes.md) — Merged Z80 opcode, flag, effect, and T-state reference used to keep disassembler metadata understandable.
- [src/bus.rs](../src/bus.rs) — Z80-facing `CpuBus` trait and flat `FakeBus` test implementation.
- [src/mmu.rs](../src/mmu.rs) — TVC memory management unit implementing bank switching.
- [src/vid.rs](../src/vid.rs) — TVC video controller (MC6845 CRTC emulation).
- [info/sound.md](sound.md) — TVC programmable sound generator, timer interrupt, and PCM sample output model.
- [src/key.rs](../src/key.rs) — TVC keyboard matrix with dynamic auto-mapping.
- [src/tvc.rs](../src/tvc.rs) — System bus and machine orchestrator (TvcBus + Tvc).
- [src/tape.rs](../src/tape.rs) — Cassette tape playback/input state.
- [src/sound.rs](../src/sound.rs) — Programmable sound generator, 4-bit DAC/amplitude register, PCM sample renderer, and shared interrupt source.
- [src/audio.rs](../src/audio.rs) — Platform-specific PCM sink: native `cpal` output and a full-web bridge to the bundled browser `AudioWorklet`.
- [src/app_state.rs](../src/app_state.rs) — Hand-written preference/state loader and saver using native `rtvc.toml` or browser `localStorage`.
- [src/expansion.rs](../src/expansion.rs) — Expansion slot/card routing for memory and I/O windows.
- [src/emu.rs](../src/emu.rs) — High-level emulator wrapper with run state, native filesystem media, embedded full-web ROMs, and byte-backed browser media.
- [src/ui.rs](../src/ui.rs) — Shared native/full-web egui application with platform-specific file, storage, audio, and keyboard integration.
- [src/workspace.rs](../src/workspace.rs) — Native/full-web developer workspace modes, dock pane rendering, layout persistence, and TVC keyboard capture policy.
- [src/debugger.rs](../src/debugger.rs) — Native-only TCP socket debugger command handler, supporting both native GUI and headless execution modes.
- [data/rom_symbols_1_2.json](../data/rom_symbols_1_2.json) — Portable, bank-aware BASIC 1.2 ROM symbol database for debugger annotations, AI traces, developer lookup, and generated help.
- [data/snapshots/](../data/snapshots/) — Checked-in emulator state fixtures, including the clean BASIC 1.2 VT-DOS boot snapshot embedded for Gamebase launches.
- [info/rom_symbols.md](rom_symbols.md) — ROM symbol schema, addressing rules, source provenance, and maintenance guidance.
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

## Build Targets and Frontends

The project supports three frontend targets over the same emulator core:

| Target | Cargo features | Frontend and platform integration |
| --- | --- | --- |
| Native desktop | Default `native` feature | Shared egui/eframe UI with optional `egui_dock` developer workspace, native `cpal` audio, native file dialogs, filesystem media, and zipped program loading. Video model selection is a runtime UI setting. |
| Lightweight web | `--no-default-features --features wasm,web-vid-simple` | Small `wasm-bindgen` API for a JavaScript-owned canvas UI, Web Audio, keyboard events, and file selection. It excludes cpal, egui, eframe, and zip. |
| Full web | `--no-default-features --features wasm-full` | Shared egui/eframe application with optional `egui_dock` developer workspace, browser file upload/download, an `AudioWorklet` initialized from a user gesture, `localStorage` preferences, and up to five recent tape and disk entries per media kind in IndexedDB. |

The lightweight `web-vid-simple` and `web-vid-realistic` features are mutually exclusive compatibility selectors retained for existing build commands. Lightweight WASM constructors default to `VidModel::FastFrame` in either build; callers can select `VidModel::Interleaved` through the runtime API.

Browser-only dependencies must remain behind explicit web features. In particular, changes to the full web application must not pull egui/eframe, zip, IndexedDB helpers, or native filesystem assumptions into the lightweight WASM target. Current build and dependency-tree validation commands are maintained in the [development and testing skill](../.agents/skills/development/SKILL.md).

## Architecture

- **Z80 CPU**: The CPU emulator is implemented in [src/z80.rs](../src/z80.rs) and supports all documented and many undocumented Z80 opcodes.
- **MMU**:
  - `FakeBus` in `bus.rs` provides a flat, 64 KB CPU bus specifically for running CPU tests (like FUSE and ZEX).
  - `TvcMmu` in `mmu.rs` implements TVC bank switching (mapping external/internal memory banks into four 16 KB pages), wired to the main binary via `TvcBus`.
- **CPU Bus Trait**: `CpuBus` in `bus.rs` is the Z80-facing memory and I/O interface used by the CPU core. Test memory and the full TVC bus both implement it. The compact disassembler in [src/disasm.rs](../src/disasm.rs) also reads through this trait, so it can decode instructions from either `FakeBus` or the full TVC bus. Its `DisassembledInstruction` metadata uses `SZHPNC` flag order and conditional T-state notation such as `12/7`; the single-line assembler in [src/asm.rs](../src/asm.rs) provides the reverse developer workflow without adding a parser dependency. [info/z80opcodes.md](z80opcodes.md) is the maintained opcode, flag, effect, and timing reference.
- **Video**: `Vid` struct emulates the MC6845 CRTC and supports two runtime video schedules: `VidModel::FastFrame` (`draw_frame()` after one screen-time CPU budget) and `VidModel::Interleaved` (`stream_some()`/`render_stream()` after each CPU instruction).
- **Sound**: `SoundTimer` models the TVC's 12-bit programmable divider, fixed 4-bit sound stage, amplitude register, DAC mode, and shared sound interrupt. It renders mono 44.1 kHz `f32` PCM samples that frontends can drain through `Tvc::take_audio_samples()`.
- **Keyboard**: `Key` struct implements a row/column matrix with dynamic auto-mapping from host keyboard codes to TVC layout.
- **System Bus**: `TvcBus` wraps the MMU, Video, Keyboard, tape interface, sound timer, logger, and expansion slots. It implements `CpuBus`, dispatching CPU memory and I/O accesses to the relevant device. Expansion memory and I/O routing lives here, while `TvcMmu` remains the internal memory mapper.
- **Machine Orchestrator**: `Tvc` owns the bus, Z80 CPU, framebuffer, and runtime `VidModel` setting, providing `run_for_a_frame()` over 62500 CPU cycles. Interleaved mode is bounded by this host screen-time budget and draws a black background with moving white stripes after several consecutive host ticks without a synchronized CRTC frame.
- **Library Boundary**: The `rtvc_core` target in [src/lib.rs](../src/lib.rs) exposes the emulator core as an `rlib` for native tooling and as a `cdylib` for WASM packaging. Its distinct target name avoids Windows debug-symbol collisions with the `rtvc` executable.
- **Emulator Wrapper**: `Emu` wraps `Tvc` with run state and media lifecycle. Native builds load ROMs from `roms/` and discover zipped programs in `progs/`; full-web builds embed the required ROMs and retain mounted media bytes so machine reloads can restore browser-selected tape and disk content.
- **Native App State**: `rtvc.toml` stores native emulator preferences and restorable media state, including machine type, video model, the fast-boot ROM patch toggle, and loaded tape/disk filenames. It is loaded from the current working directory first, then beside the executable; saving uses the loaded path and can fall back to the executable directory. The separate `rtvc-workspace.json` beside the active config stores the versioned developer mode and dock layout. Native Gamebase CAS/DSK members are cached under `rtvc-media/` beside that active config so downloaded media can participate in recents and restart restoration.
- **egui GUI**: `EmuApp` owns scheduling, audio draining, menus, status, and Gamebase. [src/workspace.rs](../src/workspace.rs) renders either the default simple screen or a dockable developer workspace containing Screen and IO Log panes. Developer-mode TVC keyboard input requires an explicit Screen click and Escape releases capture. The shared File menu includes a lazily loaded TVC Gamebase browser; native fetches run on worker threads through `ureq`, while full-web fetches use the browser Fetch API. Gamebase launch restores the clean TVC 1.2 VT-DOS snapshot embedded in the app, attaches or injects the selected media, starts emulation, and types `RUN` for CAS or `LOAD "*"` for DSK using frame-paced virtual keystrokes.
- **WASM Facade**: `WasmTvc` in [src/wasm.rs](../src/wasm.rs) exposes a small `wasm-bindgen` API around `Tvc`, including `runFrame()`, `setVidModel()`, audio sample draining, key events, ROM/disk loading, and direct framebuffer pointer/length access for JavaScript canvas rendering. The generated lightweight web bundle feeds drained audio samples to a browser `AudioWorklet`. The WASM build does not include cpal, egui, eframe, or zip.
- **Full Web Application**: `WebHandle` in [src/wasm.rs](../src/wasm.rs) starts the complete egui application. Its static bundle is generated by `cargo xtask bundle-web-full`, uses an `AudioWorklet`, stores recent media bytes in IndexedDB, stores small preferences in `localStorage`, and stores the workspace document under `rtvc_workspace_v1`. Browser storage failures are reported through the UI.
- **Native Application Icons**: [assets/rtvc-app-icon.svg](../assets/rtvc-app-icon.svg) is the source artwork. Native windows embed the PNG derivative, Windows executables embed the ICO resource through [build.rs](../build.rs), and macOS bundles copy the ICNS resource through [scripts/package-macos-app.sh](../scripts/package-macos-app.sh).
- **Socket Debugger**: The TCP socket server in [src/debugger.rs](../src/debugger.rs) runs a non-blocking TCP interface, accepting debugger client connections in both headless and native GUI modes. It accepts JSON commands to inspect state, single-step execution, continue/pause execution, assemble or disassemble instructions, save screenshots/snapshots, read raw memory banks, and inject inputs. A python client REPL script is provided in [scripts/rtvc_debug.py](../scripts/rtvc_debug.py) for interactive command-line debugging, including an assembler sub-prompt that reports encoded bytes without modifying machine memory.
- **Snapshots**: [info/snapshot.md](snapshot.md) defines the custom `RTVCSNAP` chunked state format, while `tvc_snapshot.rs` maps `Tvc` state to those chunks. User-facing snapshot and web bundle commands are in [README.md](../README.md).
- **Cassette WAV Utility**: `cargo run --bin cas2wav -- input.cas output.wav [tape-name]` converts CAS images into the same 44.1 kHz unsigned 8-bit PCM waveform as the legacy converter.
- **Profiling**: Use a sampling profiler such as `samply` against the native binary when profiling CPU performance.

## Toolchain

- Rust Edition: `2024` (requires Rust ≥ 1.85).
- Default feature: `native`, which enables `cpal` 0.17, `egui` 0.31, `eframe` 0.31, `egui_dock` 0.16, `zip` 2, `png` 0.17, GIF decoding through `image`, HTTPS through `ureq`, and native debugger/catalog/workspace JSON support through `serde`/`serde_json` for the desktop application.
- WASM feature: `wasm`, which enables only `wasm-bindgen` for the browser-facing API. Build it with `--no-default-features --features wasm`.
- Full-web feature: `wasm-full`, which enables egui/eframe, `egui_dock`, PNG and GIF decoding, file dialogs, JSON catalog/workspace parsing, zip media, and browser integration without enabling native `cpal`.
- Native `Tvc::new()` defaults to `VidModel::Interleaved`. WASM constructors default to `VidModel::FastFrame`; browser callers can still switch modes through the WASM string API, which accepts `fast-frame` and `interleaved` plus the legacy aliases `simple` and `realistic`.
- Package dependencies, feature definitions, binary targets, and metadata are managed in [Cargo.toml](../Cargo.toml). `serde` and `serde_json` are enabled by native builds for debugger JSON-RPC and by the full-web build for Gamebase catalog parsing; the lightweight web tier excludes them.
- License: MIT for emulator code. ROMs, cassette/disk images, snapshots, screenshots, manuals, and other historical or third-party machine materials may be present for preservation, compatibility testing, or convenience, but are outside the project license unless explicitly stated.
