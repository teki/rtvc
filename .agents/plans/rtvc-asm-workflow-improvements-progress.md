# rtvc-asm Workflow Improvements: Progress

Plan: [rtvc-asm-workflow-improvements.md](rtvc-asm-workflow-improvements.md)

## Status

Item 1 (full expressions) is done. Items 2–10 are not started. Next: item 2,
file-aware errors.

## Completed

- Item 1: a precedence-climbing evaluator (`ExprParser` in
  [src/emulator/asm.rs](../../src/emulator/asm.rs)) replaces the
  left-to-right `+`/`-` loop. Unary `- + ~ !`, `* / %`, `+ -`, `<< >>`, `&`,
  `^`, `|`, parentheses, `LOW()`, `HIGH()`, and `label@map` as a primary.
- `is_wrapped` decides memory operands: only an operand that is one whole
  parenthesised group is memory; `(2+1)*(1+1)` is an expression.
- `ASSERT`'s comparison splitter skips `<<` and `>>`.
- Docs: [info/assembler.md](../../info/assembler.md),
  [info.hu/assembler.md](../../info.hu/assembler.md) (Expressions, plus the
  missing `REPT` and `ASSERT` sections), [CHANGES.md](../../CHANGES.md).

## Decisions

- Values are `i32` with wrapping arithmetic; range checks stay at the point
  of use (`eval_byte`, `eval_word`).
- A word that parses as a number is a number before it is a symbol
  (`C000H`), matching the old evaluator.
- In pass 1, any undefined symbol or mapping makes the whole expression
  `unknown_value`, and division-by-zero or shift errors are suppressed once
  an undefined value has been seen. Before, the placeholder was added into
  the sum.
- Function names are recognised only directly before `(`; an unknown name
  before `(` is an error.
- `assemble_line` (the single-instruction debugger path) still accepts
  numbers only; expressions need `assemble_program`, as before.

## Validation

- `cargo test --lib asm`: 32 passed (4 new expression tests).
- `cargo test --lib`: 273 passed. `cargo test -p rtvc-c80`: all passed.
- `python3 scripts/check_rom_listings.py`: all TVC12 and VT-DOS listings ok.
- Old and new `rtvc-asm` give identical output for every source in
  `coding/`, `info/c80/out/`, and `tvc-ports` (ports and knowledge).
  `lsqintro/intro.asm` fails an `ASSERT` ("TUNE must fill its page") and
  `knowledge/48.rom.asm` has a `%` placeholder error with both versions;
  those are pre-existing.
