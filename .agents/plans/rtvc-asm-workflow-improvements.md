# rtvc-asm Workflow Improvements

## Motivation

[rtvc-asm](../../info/assembler.md) began as a debugger patch assembler. It
now builds whole port modules in `tvc-ports`, raster demos in
[coding/](../../coding/), and is the target language of
[C80](../../info/c80.md). Real sources show where it falls short:

- The Laser Squad build script (`tvc-ports/ports/laser-squad/build.py`)
  reads one module's TOML `[symbols]`, moves 20+ addresses from where the
  code was assembled to where it runs (`TVC_SUPPORT_ADDR + sym -
  source_base`), and passes them back to another module with `-d`. It also
  checks overlaps and region sizes in Python and splices a converted font
  into the CAS payload.
- Code that is copied before it runs gets its run addresses by hand:
  `TVC_SUPPORT_RUN EQU 21BCH`, `TVC_LARGE_NATIVE_LOW_RUN EQU 2880H`.
- Arithmetic lives in comments: `TVC_LARGE_NATIVE_LOW_SIZE EQU 1640H ; 89
  glyphs * 64 bytes`. High bytes are repeated by hand:
  `TVC_CONVERT_TABLE_PAGE EQU 0B0H` next to `TVC_CONVERT_TABLE_RUN EQU
  0B000H`.
- Errors print `line N:` without a file name, so editors cannot jump to them.

This plan follows on from
[rtvc-asm Repetition and Cycle Timing](rtvc-asm-repetition-and-timing.md),
which delivered `REPT`/`ENDR` and `ASSERT` and still owns `CYCLES`,
T-state listings and `DELAY`. That plan's evaluator note (`*`, `/` and
parentheses are needed) is taken over by item 1 here.

## What makes 8-bit assemblers popular

Research on 2026-11-20 across Z80, 6502 and 6809 tools (sources at the end).
The reasons users give are mostly about workflow, not language features:

| Assembler | Scene | Reasons given |
| --- | --- | --- |
| sjasmplus | Z80, Spectrum/Next | `DEVICE` simulates paged memory; `SAVESNA`/`SAVENEX` write runnable images; source-level debug files read by DeZog; Lua; fast; responsive maintenance |
| RASM | Z80, CPC | very fast on huge projects; writes snapshots, cartridges and AMSDOS files; bank tags (`{PAGESET}`); exports symbols and breakpoints into snapshots |
| Pasmo | Z80 | "simple to use, and it just works" |
| ca65/ld65 | 6502, NES and others | segments and a linker config describing the memory map; `.proc` scopes; multi-file projects |
| 64tass, ACME | 6502, C64 | native Turbo Assembler syntax heritage, local labels, macros, long-standing standards |
| KickAssembler | 6502, C64 demos | in-source scripting that generates sine tables, vectors and graphics data |
| DASM | 6502, Atari 2600 | habit, tutorials and shared include files; editor-parsable errors |
| BeebAsm | 6502, BBC Micro | BBC BASIC assembler syntax; writes files straight into disc images |
| LWASM | 6809/6309, CoCo/Dragon | complete toolchain: listings, symbol tables, OS-9 objects |
| AS (Alfred Arnold) | many CPUs incl. 6800 family | one tool for many CPUs; cross-reference listing; macro-expanded source dump |

Recurring themes:

1. **It produces the thing you run**: snapshot, disc, cartridge or tape
   image, not a raw binary to wrap later.
2. **It works with the emulator and editor**: symbols, source lines and
   breakpoints in the debugger; errors an editor can jump to.
3. **It understands the machine's memory**: paging, banks, memory maps.
4. **It generates data in the source** instead of an external script.
5. **It is simple and dependable.**
6. **It scales**: speed, scopes, includes, modules.

Large macro systems are rarely the reason people give.

rtvc-asm's distinctive advantage is that it lives inside the emulator and
debugger. Themes 1–3 are where it can be better than general tools, so they
rank higher here than macros.

## Scope

In scope, in implementation order:

1. Full expressions.
2. File-aware errors.
3. `PHASE`/`DEPHASE`.
4. `INCLUDE`, `INCBIN`, and symbol import.
5. Debugger integration: `BREAK` and richer `[[lines]]`.
6. TVC output beyond `BASIC_START`.
7. Data-generation functions.
8. Local labels.
9. `STRUCT` offset tables.
10. Macros, plus a documented Z80 macro library.

`CYCLES`/`DELAY`/listing stay in the existing timing plan and can be
interleaved with this one.

Out of scope: conditional assembly (`IF`/`IFDEF`; not needed at the
moment, revisit if one source must build several variants), embedded
scripting (Lua, JavaScript), built-in "fake" instructions, relocatable
object files and a separate linker, compatibility syntax for other
assemblers, and multi-CPU support. The `Zx82` and `Cpc` machines use the same Z80
encoder, so nothing here is TVC-only except item 6.

## Design

### 1. Full expressions

Replace the left-to-right `+`/`-` evaluator with a precedence-climbing
parser over 16-bit-wrapping integers (evaluate in `i32`, range-check at the
point of use as today).

- Operators, high to low: unary `-` `~` `!`; `*` `/` `%`; `+` `-`;
  `<<` `>>`; `&`; `^`; `|`. Comparisons stay in `ASSERT` only, so
  that a bare `<` or `>` never changes the meaning of an existing operand.
- Parentheses. Ambiguity with Z80 memory operands: an operand whose whole
  text is `( … )` remains a memory operand, as now (`LD A,(LABEL+1)`).
  To load a parenthesised constant, write `LD A,0+(X*2)` or `LD A,+(X*2)`.
  Document this rule; it matches sjasmplus and Pasmo behaviour.
- Functions: `LOW(x)`, `HIGH(x)`. Function names are reserved only when
  directly followed by `(`.
- `/` and `%` by zero is an error with the line.
- Keep `label@map` as a primary expression.

Integration: the expression evaluator in
[asm.rs](../../src/emulator/asm.rs); `ASSERT` reuses it for both sides.

### 2. File-aware errors

Errors become `file:line: message` (stdin is `-`). Line metadata gains a
file index once `INCLUDE` exists; `[[lines]]` gains an optional `file`
field (index into a new top-level `files` array). The debugger `assemble`
command keeps `line N:` because it has no file. This is the DASM lesson:
editors parse this format for free.

### 3. `PHASE addr` / `DEPHASE`

Between `PHASE` and `DEPHASE`, bytes continue at the current load address,
but `$` and labels take values from the phase address. This is sjasmplus
`DISP`/`ENT` and RASM `RORG`.

- `[[lines]]` records both load address and run address.
- Labels defined inside a phase block are run addresses. To get a load
  address, place a label just before `PHASE`.
- `PHASE` cannot nest; `ORG` inside a phase block is an error.
- Relationship to `label@map`: mappings transform a finished address into
  another view; `PHASE` sets what the address is. They compose.

This replaces the hand-written `*_RUN` EQUs and the address arithmetic in
`build.py`.

### 4. `INCLUDE`, `INCBIN`, and symbol import

- `INCLUDE "path"`: textual inclusion, path relative to the including
  file, include depth limit (for example 32), cycle detection.
- `INCBIN "path"[, offset[, length]]`: emits file bytes. Offset and length
  are expressions.
- `IMPORT "module.toml"[, PREFIX]`: reads an `rtvc-asm-v1` TOML and defines
  its symbols as constants, optionally with a name prefix, so one module
  can call another without `-d`. Duplicate names are an error.
- CLI: `-I <dir>` adds search directories.
- The debugger's `assemble` command has no file system context: it rejects
  all three with a clear error. `asmfile` in
  [rtvc_debug.py](../../scripts/rtvc_debug.py) should send the path so a
  later step can allow them there.

### 5. Debugger integration

rtvc will gain an assembly editor pane (see the Developer Workspace
section of [TODO.md](../../TODO.md)), and the debugger will be wired into
it. Everything here is the data that editor needs, so design it for both
consumers: the CLI/TOML path used today and an in-process editor that
calls `assemble_program` directly. The `AssembledProgram` value, not the
TOML text, is the source of truth; TOML is one rendering of it.

- `BREAK` emits no bytes and records the current address in a new TOML
  `breakpoints` array. `loadasm` sets them through the debugger
  connection, like RASM's breakpoint export. An optional
  `--no-breakpoints` CLI flag drops them.
- `[[lines]]` already maps addresses to source lines. With item 2 it
  becomes enough for a source view: the editor highlights the line for the
  current PC and shows breakpoints in the gutter. Record that in
  [c80-compiler.md](c80-compiler.md) Phase 2 so C80's integrated view uses
  the same data rather than a new format.
- Macro and `REPT` expansions (item 10) record both the call-site line and
  the definition line, so the editor can step through a macro body or
  treat the call as one line.
- Errors are structured (file, line, column when known, message) in the
  Rust API; the `file:line:` string is only the CLI rendering. The editor
  can then underline errors without parsing text.
- Keep the TOML backward compatible: new fields only, `format` stays
  `rtvc-asm-v1` unless a field changes meaning.

### 6. TVC output beyond `BASIC_START`

Today `--format cas` requires one `BASIC_START` segment. Extend it so any
single contiguous program can become a runnable CAS:

- Generate the BASIC bootstrap and an `LDIR` stub that copies the payload
  from the BASIC area to its `ORG` address and jumps to an entry label
  (`--entry NAME`, default `START`).
- Before implementing, confirm in [info/tvc.md](../../info/tvc.md) which
  memory is safe to overwrite during the copy for each target range
  (system variables, stack, video RAM paging). Add what is learned there.
- Later candidate: write files into a VT-DOS disk image, the BeebAsm model.
  Depends on the disk image support in
  [expansion-cards-and-host-file-device.md](expansion-cards-and-host-file-device.md).

### 7. Data-generation functions

Demos use KickAssembler scripts mostly to compute tables. Most of that is
covered by `REPT` with a counter plus item 1. Add a small set of pure
functions:

- `SIN(angle, period, amplitude)` and `COS(…)`: integer results, rounded
  half away from zero, computed with `f64` so results are identical on
  native and WASM.
- `MIN`, `MAX`, `ABS`.

Example:

```asm
SINE:   REPT 256, I
        DB 128+SIN(I, 256, 127)
        ENDR
```

No user-defined functions and no loops beyond `REPT`.

### 8. Local labels

A label starting with `.` is local to the nearest preceding global label
and is stored as `GLOBAL.LOCAL` (the form already allowed in symbol
names). References to `.name` resolve within the current global label.
Inside a `REPT` block, local labels belong to the copy, which removes the
current duplicate-label error for loops inside repeated blocks.

Compatibility check: no current source in `coding/`, `roms/` or
`tvc-ports` starts a label with `.` (checked 2026-11-20). The
disassembler does not emit such labels. Re-check before implementing.

### 9. `STRUCT` offset tables

Structures are among the most-used features of sjasmplus and ca65, and on
the Z80 they pair with `(IX+d)`/`(IY+d)` addressing. Keep them as offset
tables only; no instances or initialisers.

```asm
        STRUCT SPRITE
X       BYTE
Y       BYTE
FRAME   WORD
FLAGS   BYTE
        ENDS

        LD A,(IX+SPRITE.Y)
        LD DE,SPRITE        ; size, 5
        DS SPRITE*8         ; eight sprites
```

- Member kinds: `BYTE`, `WORD`, `BLOCK n`, and a nested struct name.
- Defines `NAME` as the size and `NAME.FIELD` as offsets. These are global
  names containing a dot, so they never clash with item 8's local labels,
  which start with a dot.
- `STRUCT NAME, base` starts offsets at `base`, which also describes I/O
  or system-variable layouts (ca65 uses `.org` in a struct for this).
- Emits no bytes. Instances are `DS NAME` with a label.
- Pitfall from sjasmplus: `LD A,(SPRITE.X)` is a memory operand at address
  `SPRITE.X`, not a field access. Document it next to the parenthesis rule
  from item 1.

### 10. Macros

#### What people value in macros

From the sjasmplus, ca65 and KickAssembler manuals and the NESdev
"synthetic instructions" page (2026-11-20):

| Feature | Where | Why it is liked |
| --- | --- | --- |
| Named, parameterised blocks | all | repeated sequences such as port writes and 16-bit moves |
| Labels local to each expansion | sjasmplus (`.label`), ca65 (`.local`) | loops inside macros without duplicate-label errors; ca65 makes you declare each one, which users find error-prone |
| Synthetic instructions | NESdev, ca65 `macpack generic`, sjasmplus fakes | operations the CPU lacks (`NEG` on 6502, 16-bit `LD` on Z80), used "almost as if they were natively supported" |
| Named arguments and defaults | sjasmplus named args, ca65 empty args with `.ifblank` | readable calls with many parameters |
| Operand-aware commands | KickAssembler pseudocommands | `mov16 #10 : $1000` inspects each argument's addressing mode and emits matching code |
| Argument count | ca65 `.paramcount` | optional arguments |

Pitfalls the manuals themselves list:

- Macro bodies are re-parsed text, so edge cases parse differently
  (sjasmplus).
- Fake or synthetic instructions hide costs: extra T-states and clobbered
  flags or registers (sjasmplus says the expansion "might not be the
  fastest way"; NESdev lists clobbers per sequence).
- Macros named like mnemonics need escape syntax (`@djnz` in sjasmplus,
  `:adc` in KickAssembler).
- Parameter names that match registers clash (`a`, `x`, `y` in ca65; on the
  Z80 that would be `A`, `HL`, `C`).
- Wrong argument counts pass silently in sjasmplus and ca65 classic macros.

#### Design

```asm
        MACRO SETPAL index, colour=0    ; TVC palette ports 60H-63H
        LD A,colour
        OUT (60H+index),A
        ENDM

        SETPAL 1, 0F0H
        SETPAL 2            ; colour defaults to 0
```

- `MACRO name [param[=default], ...]` … `ENDM`. Invoked by name in the
  mnemonic position. Must be defined before use.
- Substitution is whole-word, case-insensitive, outside strings: the same
  code path as the `REPT` counter, so there is one substitution rule.
- Argument checking is strict, which is better than sjasmplus and ca65:
  too many arguments, or a missing argument without a default, is an
  error that names the macro and parameter.
- An argument containing commas is wrapped in `<…>`, as in sjasmplus. This
  is rarely needed on the Z80.
- Local labels: a `.label` inside a macro body is local to that expansion
  (item 8's scope rule, with each expansion as its own scope). No `.local`
  declarations.
- Forbidden names: a macro may not be named like a Z80 mnemonic or
  directive, and a parameter may not be named like a register or
  condition (`A`…`L`, `HL`, `IX`, `NZ`, …). Both are errors at definition
  time, so no escape syntax is needed.
- Macros may call other macros. Recursion is an error, since without
  conditional assembly it can never stop. Expansions count towards the
  existing 1,048,576-line limit.
- Errors inside an expansion name both places: `game.asm:40: in macro
  SETPAL (macros.inc:3): …`.
- Not now: `EXITM`, `IF` inside macros, variadic arguments, and
  `.paramcount`-style tests, since all of them need conditional assembly.

#### Synthetic instructions as a library, not built in

sjasmplus's fake instructions (`LD HL,DE` → `LD H,D : LD L,E`) are popular,
but they hide cost and flag effects, which conflicts with the C80 rule of
judging output against hand-written Z80. Instead, ship an includable
`z80macros.inc` with explicit names and a comment on each macro giving
its T-states, bytes and clobbers. Examples: `LD16 HL,DE`, `NEG16 HL`,
`SUB16 HL,DE` (`OR A : SBC HL,DE`), `ADD_A_TO HL`. These read as macros at
the call site, so nobody mistakes them for single instructions. Once
`CYCLES` exists, each one gets an `ASSERT` on its cost.

#### Later: operand-shape overloads

KickAssembler's pseudocommands are the most loved macro feature found,
but they rely on scripting. A Z80 equivalent without conditionals is
overloading by operand shape:

```asm
        MACRO LD16 dst:reg16, src:reg16     ; LD16 HL,DE
        ...
        MACRO LD16 dst:reg16, src:mem       ; LD16 BC,(IX+4)
```

Each parameter can declare a shape (`reg8`, `reg16`, `imm`, `mem`,
`index`), and the first definition whose shapes match is used. This is
worth designing only after basic macros have been used in real ports.

## Integration Points

- [src/emulator/asm.rs](../../src/emulator/asm.rs): evaluator, directives,
  include stack, local label scope, phase state, line metadata.
- [src/bin/rtvc_asm.rs](../../src/bin/rtvc_asm.rs): `-I`, `--entry`,
  file-aware errors, CAS bootstrap, `breakpoints` output.
- [src/emulator/cas.rs](../../src/emulator/cas.rs): CAS encoding (reuse).
- [src/debugger/debugger.rs](../../src/debugger/debugger.rs): reject file
  directives in `assemble`.
- [scripts/rtvc_debug.py](../../scripts/rtvc_debug.py): `loadasm` sets
  breakpoints; `asmfile` passes a path.
- [c80/](../../c80/) C80 emitter: may use local
  labels and `PHASE` once available; not required.
- `tvc-ports/ports/laser-squad/build.py`: first real user of `PHASE`,
  `IMPORT` and `INCBIN`; migrate it as the acceptance test, in that repo.
- Docs: [info/assembler.md](../../info/assembler.md) and
  [info.hu/assembler.md](../../info.hu/assembler.md) for every feature;
  [info/rtvc.md](../../info/rtvc.md) if the TOML format or debugger
  behaviour changes; [CHANGES.md](../../CHANGES.md).

## Validation

- Unit tests in [asm_tests.rs](../../src/emulator/asm_tests.rs) for each
  item: precedence table, memory operand vs parenthesised constant,
  division by zero; `PHASE` labels and `[[lines]]` addresses; include
  depth and cycle errors; `INCBIN` ranges; `IMPORT` duplicates; local
  labels across globals and inside `REPT`; `SIN` values against a fixed
  table; `STRUCT` offsets, sizes and nesting; macro defaults, strict
  argument counts, forbidden names, per-expansion locals, recursion
  rejected, and error messages naming both call and definition lines.
- `z80macros.inc`: each macro assembled and run in a headless emulator
  test that checks its result and documented clobbers.
- Round trip: `rtvc-disasm` output for the ROM listings in `roms/` still
  reassembles byte-identically.
- C80: the C80 conformance tests still pass, since C80 output goes through
  this assembler.
- Integration: a CAS built with item 6 boots in the emulator (headless
  test, like the existing `BASIC_START` tests in
  [tvc_tests.rs](../../src/emulator/tvc_tests.rs)).
- Acceptance: Laser Squad's `build.py` drops its address arithmetic and
  `-d` round trip and produces an identical CAS.

## Sources

- sjasmplus: <https://z00m128.github.io/sjasmplus/documentation.html>,
  <https://github.com/z00m128/sjasmplus/>; DeZog:
  <https://www.github.com/maziac/DeZog/blob/main/design/AddingNewAssemblers.md>
- Pasmo/SkoolKit: <https://skoolkit.ca/posts/2010/10/adventures-with-assemblers>;
  RC2014 list: <https://groups.google.com/g/rc2014-z80/c/HMOw0fMZ-YI>
- RASM: <https://cpcwiki.eu/index.php/RASM>,
  <https://github.com/EdouardBERGE/rasm>
- ca65/ld65: <https://cc65.github.io/doc/ld65.html>,
  <https://prog.world/building-nes-games-in-6502-assembly-refactoring/?amp>
- 64tass: <https://sourceforge.net/projects/tass64/>; KickAssembler:
  <https://www.c64-wiki.com/wiki/KickAssembler>, pseudocommands:
  <http://theweb.dk/KickAssembler/webhelp/content/ch07s03.html>
- ca65 macros and structs: <https://cc65.github.io/doc/ca65.html>
- NESdev synthetic instructions:
  <https://www.nesdev.org/wiki/Synthetic_Instructions>
- DASM: <https://atariage.com/forums/topic/297291-what-assemblers-are-being-used-these-days-for-2600-dev/>
- BeebAsm: <https://github.com/tom-seddon/beebasm>
- LWTOOLS: <https://www.lwtools.ca/>; AS: <https://manpages.opensuse.org/Tumbleweed/asl/asl.1.en.html>
