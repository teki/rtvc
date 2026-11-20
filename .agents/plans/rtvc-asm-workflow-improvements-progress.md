# rtvc-asm Workflow Improvements: Progress

Plan: [rtvc-asm-workflow-improvements.md](rtvc-asm-workflow-improvements.md)

## Status

Items 1 (full expressions), 2 (file-aware errors) and 3 (`PHASE`/`DEPHASE`)
are done. Items 4–10 are not started. Next: item 4, `INCLUDE`, `INCBIN` and
`IMPORT`.

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
- Item 2: `AsmError` stores `message` and `line: Option<usize>`.
  `line_error` sets the line only when it is missing, so the innermost line
  wins and the old double `line N: line N:` prefix cannot happen.
  `display_in(file)` renders `file:N: message`. `rtvc-asm` prints source
  errors (assembler and `%NAME%`) bare as `file:line: message`; other
  failures keep `rtvc-asm:`. `Display` is unchanged, so the debugger,
  `rtvc-tocas`, and C80 messages read as before.
- Item 3: `PHASE address` / `DEPHASE`. Both passes track `pc` (run
  address, seen by labels and `$`) and `load` (where bytes go); they differ
  only inside a block. Segments are emitted at `load`. `AssembledLine` gained
  `run_addr`; the TOML writes `run_addr` only when it differs from `addr`,
  and the debugger `assemble` JSON always includes it.

## Decisions

- Item 2: file names and columns are not stored in `AsmError` yet; that
  waits for `INCLUDE` (item 4), when a line can belong to another file.
- Item 3: a label on the `PHASE` line is the load address, one on
  `DEPHASE` the run address past the block (labels are bound before the
  directive acts). No implicit symbols such as a block length; write
  `END-RUN` with labels.
- Item 3: `PHASE` cannot nest; `ORG` and `BASIC_START` inside a block are
  errors, and an unclosed block is reported at its `PHASE` line.
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

## Validation (item 3)

- `cargo test --lib asm`: 36 passed (3 new `PHASE` tests).
  `cargo test --lib`: 277 passed. `cargo test -p rtvc-c80` and
  `cargo test --bin rtvc-asm`: all passed. ROM listings ok.
- The previous and new `rtvc-asm` give identical output for all 43 sources
  in `coding/`, `info/c80/out/`, `roms/`, and `tvc-ports`.

## Validation (item 2)

- `cargo test --lib asm`: 33 passed. `cargo test --bin rtvc-asm`: 8 passed.
- Manual: file, stdin (`-:1:`), placeholder, and option errors print as
  documented. No script in `scripts/`, `xtask/`, or `tvc-ports` parsed the
  old `rtvc-asm: line N:` text.

## Validation (item 1)

- `cargo test --lib asm`: 32 passed (4 new expression tests).
- `cargo test --lib`: 273 passed. `cargo test -p rtvc-c80`: all passed.
- `python3 scripts/check_rom_listings.py`: all TVC12 and VT-DOS listings ok.
- Old and new `rtvc-asm` give identical output for every source in
  `coding/`, `info/c80/out/`, and `tvc-ports` (ports and knowledge).
  `lsqintro/intro.asm` fails an `ASSERT` ("TUNE must fill its page") and
  `knowledge/48.rom.asm` has a `%` placeholder error with both versions;
  those are pre-existing.
