# rtvc-asm Repetition and Cycle Timing: Progress

Plan: [rtvc-asm-repetition-and-timing.md](rtvc-asm-repetition-and-timing.md)

## Status

Option A (`REPT`/`ENDR`) is done. `ASSERT` from option C is done; the rest
of option C (`CYCLES`, instruction T-states, listing column) and option D
(`DELAY`) are not started.

## Completed

- `REPT count[, counter]` / `ENDR` in
  [src/emulator/asm.rs](../../src/emulator/asm.rs). Blocks are expanded when
  pass 1 reaches the `REPT` line, so counts may use `$`, `EQU` symbols and
  labels defined above, including a label on the `REPT` line itself. Nesting,
  zero counts and a 1,048,576-line expansion limit are supported.
- `ASSERT condition[, "message"]`, checked in pass 2 with all symbols known.
  One comparison (`=`/`==`, `!=`/`<>`, `<`, `<=`, `>`, `>=`) or a nonzero
  expression.
- Tests in [src/emulator/asm_tests.rs](../../src/emulator/asm_tests.rs);
  reference in [info/assembler.md](../../info/assembler.md).

## Decisions

- The counter is textual: each copy's source line has the counter name
  replaced by the copy number (`0` to `count - 1`) and is parsed again. This
  avoids per-copy symbol scopes. The counter is not a symbol.
- Labels inside a repeated block are not made unique. They are a
  duplicate-label error when the count is more than one; there is no local
  label scope.
- Copies keep the original source line number in errors and in the TOML
  `[[lines]]` table, which therefore has one entry per copy.
- Comparisons are parsed only by `ASSERT`. Full expressions (precedence,
  `*`, `/`, parentheses) arrived with item 1 of the
  [workflow improvements plan](rtvc-asm-workflow-improvements-progress.md).

## Validation

- `cargo test --lib asm`: 28 passed.
- `cargo test`: all library and integration tests passed.
- `cargo clippy --all-targets`: no new warnings (85 before and after).
- `cargo fmt --check`: clean.

## Next Steps

1. Option C: a public per-instruction T-state lookup shared with the Z80
   core, `CYCLES(from, to)`, and a T-state column in `[[lines]]` and an
   optional listing.
2. Option D: `DELAY n[, clobbers]`.
