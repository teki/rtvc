# C80 Compiler Progress

Last updated: 2026-09-07

## Current increment

- **ID:** E04
- **State:** passed
- **Authorization:** user assigned the execution plan and asked to continue

## Per-increment state

| ID | State | Last successful gate | Notes |
| --- | --- | --- | --- |
| E00 | passed | T00 | Baselines, feature proposal, contract table |
| E01 | passed | T01 | Source model, diagnostics, lexer/parser |
| E02 | passed | T02 | Scalar typing, definite assignment, typed IR |
| E03 | passed | T03 | Register-only executable functions; harness |
| E04 | passed | T04 | Branches, globals, register loops |
| E05–E12 | not started | — | Next: E05 calls, spills, @stackcall |

## T03 validation

```text
cargo test --lib --no-default-features --features cli-tools compiler::
  32 passed (before E04 tests)
```

Tiny leaves remain `RET` and `ADD HL,DE; RET`.

## T04 validation

```text
cargo test --lib --no-default-features --features cli-tools compiler::
  37 passed; 0 failed; 119 filtered out

cargo check --lib --no-default-features --features cli-tools
  ok

git diff --check
  clean
```

T04 fixtures: signed overflow compares, short-circuit suppressed stores, zero/one/many loop iterations (count T-states 66/127/371), nested if, continue, JP-only long branches, repeated global writes, bus-scripted rereads. Local counter has no PUSH/IX.

## Implementation notes

- Multi-block functions keep locals in stable registers (C/B/E/…, DE/BC); A/HL stay scratch.
- ABI-to-stable prologue moves are ordered so destinations are not pending sources (F-007).
- Globals emit `DB`/`DW` in source order with functions. Calls are still skipped.
- Docs: [`info/c80.md`](../../info/c80.md)

## Next action

E05: calls, live-value preservation, spills/frames, explicit `@stackcall`, stack bounds.
