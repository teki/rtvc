# rtvc-asm Workflow Improvements: Progress

Plan: [rtvc-asm-workflow-improvements.md](rtvc-asm-workflow-improvements.md)

## Status

Items 1–10 are done: full expressions, file-aware errors, `PHASE`/`DEPHASE`,
`INCLUDE`/`INCBIN`/`IMPORT`, `BREAK`, runnable CAS output for any `ORG`,
table functions, local labels, `STRUCT`, and macros with `z80macros.inc`.
Open from the plan: migrating `tvc-ports/ports/laser-squad/build.py` to
`PHASE`/`IMPORT`/`INCBIN` (in that repo), the known gaps below, and the
later operand-shape macro overloads. `CYCLES`/`DELAY` stay in the timing
plan.

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
- Item 4: `SourceLoader` reads files while the source is parsed, before
  pass 1: `INCLUDE` is replaced by the included lines (its own line stays,
  with its labels), `INCBIN` and `IMPORT` keep the file contents in the
  statement. File access goes through the `AsmFiles` trait and
  `assemble_program_with`; `assemble_program` has none, so the debugger
  rejects the directives. `rtvc-asm` implements the trait as `CliFiles`
  with `-I`, and applies `%NAME%` placeholders to included sources.
  `ParsedLine` carries its file index and name, and every pass error goes
  through `ParsedLine::error`, so errors in included files name the file.
  `AsmError` gained `file()`; `AssembledProgram` gained `files`,
  `AssembledLine` gained `file`; the TOML writes `files` and `file` only
  when something was included.
- Item 5: `BREAK` records the run address (`pc`) in pass 2 as an
  `AsmBreakpoint { addr, file, line }` in `AssembledProgram::breakpoints`.
  The TOML writes `[[breakpoints]]` (`file` only when nonzero),
  `parse_rtvc_asm_v1` reads them back, and the debugger `assemble` reply
  has `breakpoints`. `rtvc-asm --no-breakpoints` clears them. In
  [rtvc_debug.py](../../scripts/rtvc_debug.py), `asmfile` and `loadasm`
  call `breakpoint_add` for each after writing the segments.
- Item 6: `tvc_cas_payload` in [src/emulator/asm.rs](../../src/emulator/asm.rs)
  builds the bytes BASIC loads at `19EFH`. A `BASIC_START` segment is used
  as it is. Any other single segment gets the `BASIC_START` line plus a
  14-byte stub at `1A30H` (`LD HL / LD DE / LD BC / LDDR / JP entry`) and
  the program at `1A3EH`. `rtvc-asm --format cas` gained `--entry NAME`;
  `rtvc-tocas` uses the same function for `.asm` inputs.

- Item 7: `ExprParser::parse_function` takes comma-separated arguments
  (`split_operands` already keeps commas inside parentheses) and checks
  each function's argument count. `SIN`/`COS` go through `wave`.
- Item 8: pass 1 tracks `scope`, the last colon label not starting with
  `.`. `scope_locals` rewrites `.name` words in a line's source
  (`rewrite_locals` skips strings, character literals, and the comment) to
  `SCOPE.NAME`, re-parses it, and restores the original `source` for
  listings. `REPT` expansion first renames the locals its body defines
  (`rept_locals`, outside nested blocks) to `._Rn_c.NAME` per copy; the
  scope rewrite then prefixes the global label.
- Item 9: `Statement::Struct { name, base }` and `Statement::Ends`. Pass 1
  calls `define_struct` when it reaches `STRUCT`: `struct_member` reads each
  member from the line's source text, and the layout (size plus fields,
  nested ones flattened as `AT.Y`) is kept in a `StructLayout` map for later
  nesting. The block's lines then become `Empty` with no labels, so pass 2
  and listings emit nothing for them.
- Item 10: `SourceLoader` expands macros while it reads the source, so
  every later stage sees ordinary lines. `define_macro` keeps the body as
  text; `process` handles each line (files, `ENDM` and `MACRO` errors,
  macro calls); `expand` checks arguments (`split_macro_args`, with
  `<...>`), substitutes parameters with `substitute_words` (the `REPT`
  counter rule, now shared), renames body locals to `._Mn.NAME`, and feeds
  each line back through `process` with the call chain for recursion
  checks. `ParsedLine::context` (`in macro NAME (file:line)`, nested
  outermost first) prefixes errors that do not yet have a line, and REPT
  copies and local-label rewrites keep it.
- Item 10: [coding/lib/z80macros.inc](../../coding/lib/z80macros.inc) with
  pair moves (through a `LD_PAIR` helper), `SUB16_HL`, `CP16_HL`, `NEG_HL`,
  `ADD_A_TO_*`, `DEC_BC_JR_NZ`, `DELAY_B`, `OUTI_N`, `LDI_N`.
  `z80macros_match_their_documented_costs` runs each on the Z80 core with
  `FakeBus`, checks results, and compares measured T-states, bytes, and
  changed registers with each `N T, M bytes. Changes: ...` comment.

## Decisions

- Item 10: macros expand at load time, not in pass 1: definitions are
  textual and must come before use, and the debugger `assemble` command
  (no file access) gets macros too. An expansion's lines take the call's
  line number and file, so `[[lines]]` and breakpoints point at the call.
- Item 10: the generic 16-bit move is not one `LD16 dst,src` macro: without
  conditionals a macro cannot split `HL` into `H` and `L`, and `PUSH`/`POP`
  would cost 21 T. The library uses `LD_PAIR hi,lo,hi,lo` plus named
  `LD_HL_DE`-style wrappers, which also keeps each cost exact.
- Item 10: an empty argument takes the default; a macro defined after its
  call is not found (the call reports the encoder's error for an unknown
  word). Parameter substitution, like the `REPT` counter, still applies
  inside `'x'` character literals and comments; only double-quoted strings
  are skipped.

- Item 9: `BLOCK n` and the base are evaluated strictly in pass 1, like
  `EQU`, so they may only use symbols defined above. A member line's first
  word is the field unless it is `BYTE`, `WORD` or `BLOCK` (unnamed
  padding). A label on the `STRUCT` line is an error (the name is an
  operand); `ORG`, `PHASE` and `REPT` inside a struct are errors. The base
  shifts fields only, never the size.

- Item 8: `EQU` does not set the scope, so constants between a routine's
  lines keep its locals reachable. A `'` after a word character is `AF'`,
  not a character literal. Locals before any global label keep `.NAME`.
  `REPT` copy names are stored in the symbol table, so the debugger sees
  them.
- Item 8: renamed `REPT` copies keep the renamed text in `source`, like
  counter copies keep the substituted text, because a nested `REPT` is
  re-parsed from it.

- Item 7: `wave` reduces the angle with `rem_euclid` before using `f64`,
  then snaps the product to 1e-9 before rounding half away from zero.
  Without the snap `SIN(1,12,1)` gave 0 (sin 30° is 0.49999999999999994),
  and an ulp of difference between native and WASM `sin` could change a
  byte. A non-positive period is an error, suppressed in pass 1 when the
  arguments use undefined symbols.
- Item 7: `MIN`/`MAX` take two or more arguments; `ABS` wraps like the
  other operators.

- Item 6: the destination must lie within `1A3EH`–`BFFFH`. It is never below
  the source, so `LDDR` is always a safe copy, and the stub is never
  overwritten. The ROM's stack top is `16ACH`; SP was `1698H` inside `USR` in
  the boot test, so the range cannot touch the stack. No `DI` and no new SP:
  interrupts keep running, and a `RET` goes back to BASIC.
- Item 6: the entry must be inside the copied program; it defaults to
  `START`, else the first byte. Only one contiguous segment is supported, and
  the whole payload must fit in 42,256 bytes. `--entry` with `BASIC_START` is
  an error.
- Item 6: TOML inputs to `rtvc-tocas` still have no relocating loader.

- Item 2: file names and columns are not stored in `AsmError` yet; that
  waits for `INCLUDE` (item 4), when a line can belong to another file.
- Item 3: a label on the `PHASE` line is the load address, one on
  `DEPHASE` the run address past the block (labels are bound before the
  directive acts). No implicit symbols such as a block length; write
  `END-RUN` with labels.
- Item 3: `PHASE` cannot nest; `ORG` and `BASIC_START` inside a block are
  errors, and an unclosed block is reported at its `PHASE` line.
- Item 4: imported symbols are not written to the importing module's
  `[symbols]`, so modules do not re-export each other's names. `IMPORT`
  files do not appear in `files`, which lists sources only.
- Item 4: a `REPT` copy with a counter is parsed again from its text;
  `keep_loaded_files` gives it the file contents already read, so
  `INCBIN "f", I*8, 8` works inside `REPT`.
- Item 4: `IMPORT` uses `parse_rtvc_asm_v1`, so it needs the `asm-toml`
  feature (on in native and CLI builds); without it the directive is an
  error.
- Item 5: the debugger `assemble` command stays pure (it neither writes
  memory nor sets breakpoints); the client does both, so a future editor
  pane can decide for itself.
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

## Known gaps

- `rtvc-tocas` assembles `.asm` with `assemble_program`, which has no file
  access, so `INCLUDE`, `INCBIN`, and `IMPORT` fail there. Use
  `rtvc-asm --format cas` for such sources, or pass a `CliFiles`-style
  reader to `rtvc-tocas` if it is needed.
- The debugger `asmfile` command sends source text only; sending its path
  so the debugger can resolve includes is still open.

## Validation (item 10)

- `cargo test --lib asm`: 52 passed (4 new: arguments and defaults,
  `<...>`, nested macros and `REPT` with per-expansion locals, errors with
  call and definition lines through `INCLUDE`, and the library run on the
  Z80 core). Changing one documented clobber in the library made the test
  fail, as intended. CLI: `-I coding/lib` with `INCLUDE "z80macros.inc"`
  assembles, and errors print as `file:6: in macro M (file:3): ...`.
- `cargo test --lib`: 293 passed; CLI, C80, and ROM listings ok; no new
  clippy warnings. Old and new `rtvc-asm` give identical TOML and CAS
  output for all 55 sources.

## Validation (item 9)

- `cargo test --lib asm`: 48 passed (1 new: sizes, offsets, nesting, base,
  padding, `(IX+d)`, `DS NAME*n`, and eight error cases). The reference
  example assembles to the documented symbols and bytes.
- `cargo test --lib`: 289 passed; CLI, C80, and ROM listings ok. Old and
  new `rtvc-asm` give identical TOML and CAS output for all 55 sources.

## Validation (item 8)

- `cargo test --lib asm`: 47 passed (2 new: scopes, case, `EQU` locals,
  strings, `AF'`, errors with full names, `REPT` copies with nesting and a
  counter). No source in `coding/`, `info/c80/out/`, `roms/`, or `tvc-ports`
  starts a label or operand word with `.` (re-checked 2026-11-21).
- `cargo test --lib`: 288 passed; CLI, C80, and ROM listings ok. Old and
  new `rtvc-asm` give identical TOML and CAS output for all 55 sources.

## Validation (item 7)

- `cargo test --lib asm`: 45 passed (1 new, covering values, wrap, `.5`
  rounding, a `REPT` sine table, a forward reference, and argument
  errors). `cargo test --lib`: 286 passed; CLI, C80, and ROM listings ok.
- Old and new `rtvc-asm` give identical TOML and CAS output for all 55
  sources.

## Validation (item 6)

- `cargo test --bin rtvc-asm`: 14 passed (4 new); `rtvc-tocas`: 8 passed.
  `cargo test --lib`: 285 passed, including
  `relocating_asm_cas_runs_at_its_org`. That test loads a CAS at `A000H`
  through the host-file card; it autostarts, the code is copied and runs,
  and SP is below `19EFH`. C80 and ROM listings ok.
- Old and new `rtvc-asm`: TOML identical for all 55 sources. CAS differs only
  in error wording, and `lasersqd_ls_7530.asm` (`ORG 7530H`) now builds.

## Validation (item 5)

- `cargo test --lib asm`: 43 passed (2 new); `cargo test --bin rtvc-asm`:
  11 passed (1 new). `cargo test --lib`: 284 passed. C80, `rtvc-tocas`,
  and ROM listings ok.
- Live: a headless `rtvc --port` session took `asmfile` (breakpoint at the
  instruction after `BREAK`) and `loadasm` of a TOML with `[[breakpoints]]`;
  `bp list` showed both.
- The previous and new `rtvc-asm` give identical output for all 43 sources.

## Validation (item 4)

- `cargo test --lib asm`: 41 passed (5 new). `cargo test --bin rtvc-asm`:
  10 passed (2 new, including `CliFiles` lookup on a temporary directory).
  `cargo test --lib`: 282 passed. C80, `rtvc-tocas`, and ROM listings ok;
  `--no-default-features` builds without new warnings.
- Manual CLI run: `INCLUDE` via `-I`, `IMPORT` of a TOML written by
  `rtvc-asm`, `INCBIN` with an offset, and an error inside an included
  file reported as `lib/bad.inc:2: …`.
- The previous and new `rtvc-asm` give identical output for all 43 sources.

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
