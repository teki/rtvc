---
name: development
description: How to build, run, test, and benchmark the TVC emulator in Rust
---

# TVC Emulator Development & Testing

This skill provides step-by-step instructions and references for compiling, executing, and validating the `rtvc` emulator.

## Commands

### Compilation and Execution

- **Build the main emulator binary:**
  ```bash
  cargo build
  ```
- **Run the main emulator binary:**
  ```bash
  cargo run
  ```

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
