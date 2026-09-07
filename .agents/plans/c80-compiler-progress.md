# C80 Compiler Progress

Last updated: 2026-09-07

## Current increment

- **ID:** E12
- **State:** passed
- **Authorization:** user asked to commit after each phase, then continue

## Per-increment state

| ID | State | Last successful gate | Notes |
| --- | --- | --- | --- |
| E00–E04 | passed | T04 | Committed as 3c9e25e |
| E05 | passed | T05 | Committed as 4f78d0e |
| E06 | passed | T06 | Committed as 80e51a9 |
| E07 | passed | T07 | Committed as 627a9a2 |
| E08 | passed | T08 | Committed as 39b9ee8 |
| E09 | passed | T09 | Committed as 8924b9b |
| E10 | passed | T10 | Committed as 093125d |
| E11 | passed | T11 | Mixed BASIC/C80 linking and USR fixture |
| E12 | passed | T12 | Provenance-preserving opts and final compiler gate |

## E12 gate

Focused:

- `cargo test --lib --no-default-features --features cli-tools compiler::` — 110 passed
- `cargo test --lib --no-default-features --features cli-tools asm::tests:: -- --skip disasm` — 15 passed
- `cargo test --lib --no-default-features --features cli-tools disasm::tests::` — 7 passed
- `cargo test --lib --no-default-features --features cli-tools basic::` — 15 passed
- `cargo test --test rtvc_c80 --no-default-features --features cli-tools` — 5 passed
- `cargo test --bin rtvc-c80 --no-default-features --features cli-tools` — 1 passed
- `cargo check --lib --no-default-features --features cli-tools` — ok
- `git diff --check` — clean

Matrix:

- `cargo test --no-default-features --features cli-tools` — 229 lib + CLI/integration tests passed
- `cargo run --no-default-features --bin fuse_test` — 1333 passed, 1 failed (`76` HALT fetch count; pre-existing CPU baseline, not a compiler change)
- `cargo check` — ok
- `cargo check --bins` — ok
- `cargo check --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown` — ok
- `cargo check --lib --no-default-features --features wasm,web-vid-realistic --target wasm32-unknown-unknown` — ok
- `cargo check --lib --no-default-features --features wasm-full --target wasm32-unknown-unknown` — ok
- `cargo check --manifest-path xtask/Cargo.toml` — ok
- `cargo tree --no-default-features --features wasm,web-vid-simple -e normal --target wasm32-unknown-unknown` — `wasm-bindgen` present; no cpal/egui/eframe/zip

T12 fixtures: `short_branches_become_jr_long_stay_jp`, `baseline_and_optimized_match_effects_and_improve_cost`, `compile_latency_fixture_completes`, plus updated `zero_one_many_iterations_and_nested_branches` and `long_branches_use_jp_not_jr`.

Measured quality (`count` loop, origin `0x8000`):

- bytes 28 → 21
- 5-iter T-states 371 → 338
- 0/1/5-iter T-states 66/127/371 → 48/106/338

Compile latency: 20 compiles of the packed-struct/pointer-walk/`for`/`if` fixture in 291 ms (debug `cli-tools` libtest, macOS 15.6 darwin 25.6.0, arm64, rustc 1.98.0).

## Limitations

- Optimizer is on by default (`CompileInput.optimize`, CLI `--no-optimize` to disable).
- Forward conditional `JP` may become `JR`; backward and unconditional jumps stay `JP`.
- `DJNZ` is not emitted (F-017).
- Mutable memory is not CSE'd or store-forwarded.
- No editor, no `--emit-map`, no release artifacts.
- Compiler stays behind `compiler` (`native`, `cli-tools`, `wasm-full`; not lightweight `wasm`).

## Next action

Phase 1 compiler increments E00–E12 are complete in this tree. Stop unless the user assigns Phase 2 (editor/load) or other work.
