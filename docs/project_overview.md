# Project Overview & Architecture — rtvc

## Project Scope

`rtvc` is a Videoton TV Computer (TVC) emulator written in Rust, ported from the JavaScript implementation `../jstvc`.

The project is structured as a single Rust binary crate with multiple test and utility binaries defined in `Cargo.toml`.

### Crate Files and Directory Structure

- [Cargo.toml](../Cargo.toml) — Package configuration specifying package edition and binaries.
- [src/main.rs](../src/main.rs) — Entry point for the main TVC emulator binary.
- [src/z80.rs](../src/z80.rs) — Complete Z80 CPU emulator (supporting all documented and many undocumented opcodes).
- [src/mmu.rs](../src/mmu.rs) — TVC memory management unit implementing bank switching and flat memory helper.
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
  - The full `Mmu` in `mmu.rs` implements TVC bank switching (mapping external/internal memory banks into four 16 KB pages) but is not yet wired to the main binary.

## Toolchain

- Rust Edition: `2024` (requires Rust ≥ 1.85).
- Package dependencies and metadata are managed in [Cargo.toml](file:///Users/teki/dev/rtvc/Cargo.toml).
