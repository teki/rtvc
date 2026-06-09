---
name: development
description: How to build, run, test, and benchmark the TVC emulator in Rust
---

# TVC Emulator Development & Testing

This skill provides step-by-step instructions and references for compiling, executing, and validating the `rtvc` emulator.

## Commands

### Compilation and Execution

- **Build the native emulator binary:**
  ```bash
  cargo build
  ```
  - The Windows MSVC `rtvc` executable is linked with an 8 MiB stack by [build.rs](../../../build.rs); keep this setting when changing startup allocation patterns because the emulator state can overflow the platform's default debug-build stack.
  - On Linux, the native audio backend uses `cpal` and may require ALSA development files such as `libasound2-dev` on Debian/Ubuntu or `alsa-lib-devel` on Fedora.
- **Run the main emulator binary (opens egui window):**
  ```bash
  cargo run --bin rtvc
  ```
  To start directly from a snapshot:
  ```bash
  cargo run --bin rtvc -- snapshots/load_tape.rtvcsnap.zip
  ```
  To mount a disk, mount a tape, or inject a tape directly on startup:
  ```bash
  cargo run --bin rtvc -- -d path/to/disk.dsk
  cargo run --bin rtvc -- -t path/to/tape.cas
  cargo run --bin rtvc -- -i path/to/tape.cas
  ```
  - Place ROM files (`TVC12_D3.64K`, `TVC12_D4.64K`, `TVC12_D7.64K`) in `roms/` for the TVC to boot.
  - Use the Machine menu to run/pause, reset, select machine type, and select video model.
  - Use the View > IO Log menu item to toggle the IO port log panel.
  - Use the File menu to write/read `.rtvcsnap.zip` snapshots, load tape/disk files via open file dialog, and save the current framebuffer as a 4:3 PNG (`768x576`).
  - Use the Tape and Disk menus to load cassette and floppy media (either from local list or by browsing for any file). Selecting an entry immediately loads it.
  - The bottom status bar shows tape activity, loaded tape/disk media, run state, FPS, ROM state, audio status, and recent file status, plus a Reset button in the bottom right corner.
  - Native app preferences are stored in `rtvc.toml`, checked in the current working directory first and then beside the executable.
  - PAL 4:3 display aspect ratio is applied to the framebuffer.

- **Profile the native emulator with samply:**
  ```bash
  cargo build --profile profiling --bin rtvc
  samply record ./target/profiling/rtvc
  ```
  - Uses sampling instead of compile-time instrumentation.
  - The `profiling` profile inherits release optimizations, keeps debug info, and disables release symbol stripping so samply can resolve Rust symbols.
  - Keep ROM files in `roms/` as for normal native runs.

- **Check the lightweight WASM library build:**
  ```bash
  rustup target add wasm32-unknown-unknown
  cargo check --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown
  ```
  - The `wasm` feature exposes [src/wasm.rs](../../../src/wasm.rs) through `wasm-bindgen`.
  - `web-vid-simple` selects `VidModel::FastFrame` as the WASM constructor default.
  - This build intentionally excludes the native `egui`/`eframe` UI stack and the zipped-disk filesystem helper.
  - JavaScript should render the returned framebuffer bytes to a browser canvas.

- **Check the interleaved WASM video build:**
  ```bash
  cargo check --lib --no-default-features --features wasm,web-vid-realistic --target wasm32-unknown-unknown
  ```
  - `web-vid-realistic` selects `VidModel::Interleaved` as the WASM constructor default.

- **Bundle a lightweight web snapshot upload:**
  ```bash
  cargo install wasm-bindgen-cli --version 0.2.122
  cargo bundle-web path/to/game.rtvcsnap
  # or:
  cargo xtask bundle-web path/to/game.rtvcsnap
  ```
  - Builds the small `wasm,web-vid-simple` target.
  - Emits a static bundle under `dist/<snapshot-name>-web/`.
  - See [info/snapshot.md](../../../info/snapshot.md) for snapshot format and bundle details.

- **Bundle a lightweight web skeleton without an embedded snapshot:**
  ```bash
  cargo xtask bundle-web-skeleton
  # or choose an output directory:
  cargo xtask bundle-web-skeleton dist/rtvc-snapshot-player
  ```
  - Runs the bundler and builds the small `wasm,web-vid-simple` target with the optimized Cargo release profile.
  - Emits a static snapshot player under `dist/rtvc-web-skeleton/` by default.
  - Users can copy `snapshot.rtvcsnap.zip` or `snapshot.rtvcsnap` beside `index.html` and serve the directory with any static web server.

- **Build and bundle the full egui web application:**
  ```bash
  cargo check --lib --no-default-features --features wasm-full --target wasm32-unknown-unknown
  cargo xtask bundle-web-full
  # or choose an output directory:
  cargo xtask bundle-web-full package/web-full
  ```
  - Runs the bundler and builds the `wasm-full` feature with the complete egui/eframe emulator UI using the optimized Cargo release profile.
  - Emits a static application under `dist/rtvc-web-full/` by default.
  - Browser audio uses an `AudioWorklet`; the audio context resumes after user interaction.
  - Recent tape and disk bytes are stored in IndexedDB. Small UI preferences remain in `localStorage`.
  - Browser keyboard input uses DOM `KeyboardEvent.code` for key identity and `KeyboardEvent.key` for layout-aware character mapping, including AltGr.
  - Serve the output directory over HTTP; opening `index.html` directly with `file://` is not supported.

- **Serve the web emulator / docs website locally:**
  On macOS / Linux:
  ```bash
  python scripts/serve_docs.py
  ```
  On Windows:
  ```cmd
  scripts\serve_docs.bat
  ```
  - Serves the `docs/` directory on an available port (defaulting to 8000).
  - Automatically opens the web emulator in the default web browser.
  - Automatically handles MIME mapping (crucial for WebAssembly on Windows) and disables browser caching.

- **Build release packages on GitHub Actions:**
  ```bash
  git tag v0.1.0
  git push origin v0.1.0
  ```
  - The release workflow builds `rtvc.exe` on `windows-latest`, a macOS x64 binary on `macos-15-intel`, and a macOS Apple Silicon binary on `macos-15`.
  - Release builds use LTO, one codegen unit, stripped symbols, and `panic = "abort"` to keep binaries smaller.
  - It uploads `rtvc-windows-x64.zip`, `rtvc-macos-x64.zip`, and `rtvc-macos-arm64.zip`.
  - The GitHub release body is extracted from the matching `## v<version>` section in `CHANGES.md`.
  - The Windows archive contains the native binary, `README.md`, `LICENSE`, `roms/`, `progs/`, and `web/`.
  - The macOS archives contain an ad hoc signed `RTVC.app` bundle, `README.md`, and `LICENSE`. The release workflow does not use paid Developer ID signing or notarization, so users may need to remove the browser quarantine flag with `xattr -dr com.apple.quarantine RTVC.app` before first launch.
  - The app bundle includes `roms/`, `progs/`, and `web/` beside `Contents/MacOS/rtvc` so Finder launches can find runtime assets.
  - The native app searches `roms/` and `progs/` in the current working directory first, then beside the executable for extracted release archives and app bundles.
  - The bundled `web/` directory is the full browser emulator UI. Serve it over HTTP to open local CAS, DSK, ZIP, and snapshot files in a browser.

- **Convert a TVC CAS cassette image to WAV:**
  ```bash
  cargo run --bin cas2wav -- progs/TVBALL.CAS /tmp/TVBALL.WAV
  ```
  - Writes unsigned 8-bit mono PCM at 44.1 kHz.
  - The optional third argument overrides the filename stored in the generated tape header.
  - The generated bytes are intended to match the legacy [tools/cas2wav](../../../tools/cas2wav) converter for the same tape filename.

### Testing

- **Run FUSE tests (1334 tests):**
  These tests are adapted from the FUSE ZX Spectrum emulator test vectors. They are **fast to run** and are the primary validation suite used to verify correctness during active development.
  - Build the test binary:
    ```bash
    cargo build --bin fuse_test
    ```
  - Run the tests:
    ```bash
    cargo run --bin fuse_test
    ```

- **Run ZEX (Z80 Instruction Exercise) tests:**
  These tests (`zexdoc` and `zexall`) are **even stricter** Z80 instruction validators. However, they are **very time-consuming** to execute and are therefore **rarely run** (e.g., during final validation or major CPU core refactors).
  - Run both `zexdoc` and `zexall`:
    ```bash
    cargo run --bin zex_test
    ```
  - Run `zexdoc` only:
    ```bash
    cargo run --bin zex_test zexdoc
    ```
  - Run `zexall` only:
    ```bash
    cargo run --bin zex_test zexall
    ```

### Performance Benchmarking

- **Run performance benchmark:**
  ```bash
  cargo run --bin perf_test
  ```

## FUSE Testing Details

The primary validation suite consists of 1334 FUSE tests (adapted from the FUSE ZX Spectrum emulator test vectors).

### Test Harness Execution Steps

1. **Parse Input**: Parses each test case definition from [tests/tests.in](../../../tests/tests.in).
2. **Setup State**: Initializes `FakeMmu` memory and `Z80` CPU registers matching the starting conditions of the test.
3. **Execute**: Runs `z80.step(mmu, runtime)` for the specified number of T-states.
4. **Compare**: Verifies resulting CPU registers and modified memory locations against the expected outcomes in [tests/tests.expected](../../../tests/tests.expected).
5. **Output**: Prints `<test_description> ......... OK` for each passing test case.

If any test case fails, the harness outputs a detailed diff of the expected vs actual state and immediately aborts execution. All 1334 FUSE tests must pass successfully.
