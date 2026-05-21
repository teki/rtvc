# AGENTS.md — rtvc

## Project

Videoton TV Computer emulator in Rust, ported from `../jstvc`.

- Single Rust binary crate with additional test binary.
- `src/z80.rs` — complete Z80 CPU emulator (all documented + many undocumented opcodes).
- `src/mmu.rs` — TVC memory management unit with bank switching.
- `src/fuse_test.rs` — FUSE test harness ported from `jstvc/tests/test.js`.

## Toolchain

- `Cargo.toml` uses `edition = "2024"`, which requires Rust ≥ 1.85.

## Commands

```bash
# Build main binary
cargo build

# Run main binary
cargo run

# Build test binary
cargo build --bin fuse_test

# Run FUSE tests (1334 tests, should all pass)
cargo run --bin fuse_test
```

## Architecture

- Z80 emulator closely follows the JavaScript implementation in `../jstvc/src/z80.js`.
- `FakeMMU` in `mmu.rs` provides flat 64KB memory for CPU tests.
- Full `MMU` in `mmu.rs` implements TVC bank switching (not yet wired to main binary).
- Tests are in `tests/` (copied from `../jstvc/tests/`):
  - `tests.in` / `tests.expected` — FUSE test vectors
  - `zexdoc.com` / `zexall.com` — ZEXDOC/ZEXALL test programs

## Testing

FUSE tests are the primary validation. The test harness:
1. Parses each test case from `tests/tests.in`
2. Loads register state and memory into `FakeMMU` + `Z80`
3. Executes `z80.step(mmu, runtime)` for the specified t-states
4. Compares resulting registers/memory against `tests/tests.expected`
5. Prints `descr ......... OK` for each passed test

All 1334 FUSE tests should pass. If any fail, the harness prints a diff and stops.
