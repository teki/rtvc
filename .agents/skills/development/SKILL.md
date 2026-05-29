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
- **Run the main emulator binary (opens egui window):**
  ```bash
  cargo run --bin rtvc
  ```
  - Place ROM files (`TVC12_D3.64K`, `TVC12_D4.64K`, `TVC12_D7.64K`) in `roms/` for the TVC to boot.
  - Use the "Log" button to toggle the IO port log panel.
  - Use the "Reset" button to reset the emulator.
  - Use the "Save Snapshot" and "Load Snapshot" buttons to write/read `.rtvcsnap.zip` or raw `.rtvcsnap` files.
  - Use the "Save Screenshot" button to write the current framebuffer as a 4:3 PNG (`768x576`).
  - PAL 4:3 display aspect ratio is applied to the framebuffer.

- **Profile the native emulator with samply:**
  ```bash
  samply record cargo run --bin rtvc
  ```
  - Uses sampling instead of compile-time instrumentation.
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
  - See [docs/snapshot.md](../../../docs/snapshot.md) for snapshot format and bundle details.

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

### ASM/DASM Round-Trip Test

Test the assembler and disassembler with a comprehensive set of Z80 instructions:

```bash
cargo run --bin asm_test
```

This prints the encoded bytes for each instruction, then disassembles them back to verify round-trip correctness.

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
