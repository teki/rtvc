# rtvc Assembler Reference

This document describes the helper Z80 assembler implemented in
[src/emulator/asm.rs](../src/emulator/asm.rs), the `rtvc-asm` command-line tool in
[src/bin/rtvc_asm.rs](../src/bin/rtvc_asm.rs), the ROM-oriented `rtvc-disasm`
tool in [src/bin/rtvc_disasm.rs](../src/bin/rtvc_disasm.rs), and the debugger
loading workflow in [scripts/rtvc_debug.py](../scripts/rtvc_debug.py).

The assembler is intentionally small. It is meant for debugger patches, TVC
helper routines, and Spectrum-porting shims, not as a full replacement for a
macro assembler.

## Entry Points

### Rust API

[src/emulator/asm.rs](../src/emulator/asm.rs) exposes these entry points:

- `assemble_line(source, pc)` assembles one instruction or `DB`/`DEFB` line at
  the given program counter.
- `assemble_program(source, origin)` runs the two-pass helper assembler and
  returns emitted segments, symbols, emitted source-line metadata, flattened
  bytes, and `next_addr`. It has no file access.
- `assemble_program_with(source, origin, name, files)` does the same for the
  source called `name`, reading `INCLUDE`, `INCBIN`, and `IMPORT` files
  through an `AsmFiles` implementation. The assembler itself never touches
  the file system, so it also runs in the debugger and in WebAssembly builds.

The TCP debugger uses `assemble_program` for its `assemble` command, so a single
debugger request may contain either one instruction or a small source block.

### Command Line

Assemble a helper source to TOML:

```bash
cargo run --bin rtvc-asm -- --origin 8000H helper.asm -o helper.toml
```

Use `-` as the input path to read source from stdin:

```bash
printf 'ORG 8000H\nSTART: NOP\n' | cargo run --bin rtvc-asm -- --origin 7000H -
```

Options:

| Option | Meaning |
| --- | --- |
| `--origin <addr>` | Initial assembly address before source-level `ORG`; defaults to `0`. |
| `--format <toml\|cas\|bin>` | Output format; defaults to `toml`. `cas` expects one contiguous segment: a `BASIC_START` program, or any `ORG` from `1A3EH` to `BFFFH` (see [Runnable TVC BASIC Programs](#runnable-tvc-basic-programs)). `bin` expects one contiguous segment. |
| `-d <NAME=VALUE>`, `--define <NAME=VALUE>` | Replace `%NAME%` placeholders in assembly code before assembly. Repeat for multiple values; a missing placeholder value is an error. |
| `--no-breakpoints` | Leave [`BREAK`](#breakpoints) marks out of the output. |
| `--entry <symbol>` | With `--format cas`, the label the copy stub jumps to; defaults to `START`, or the first byte. |
| `-I <dir>`, `--include-dir <dir>` | Also look for `INCLUDE`, `INCBIN`, and `IMPORT` files in `dir`, after the directory of the file that names them. Repeat for several directories. |
| `-o <path>`, `--output <path>` | Write output to a file; omitted means stdout. |
| `-`, as input path | Read source from stdin. |

`<addr>` accepts decimal, `0x` hexadecimal, `$` hexadecimal, and `H`-suffixed
hexadecimal forms.

Build-generated values can be kept out of assembly sources with `%NAME%`
placeholders:

```asm
        LD BC,%BLOCK_SIZE%
```

```bash
rtvc-asm -d BLOCK_SIZE=0748H helper.asm -o helper.toml
```

Definitions are case-insensitive by name and substituted as text. Omitting
`-d BLOCK_SIZE=...` in this example fails with `missing definition for
%BLOCK_SIZE%`. Substitution applies only to assembly code: placeholders in
semicolon comments and quoted string literals are left unchanged. A semicolon
inside a quoted string is likewise not treated as the start of a comment.

### Disassembler Command Line

Disassemble a binary blob into `rtvc-asm` source:

```bash
cargo run --bin rtvc-disasm -- --origin C000H roms/TVC12_D4.64K -o roms/TVC12_D4.64K.asm
```

ROM symbol metadata and explicit data ranges can be supplied to keep known
tables as `DB` statements while emitting instructions for code:

```bash
cargo run --bin rtvc-disasm -- \
  --origin C000H \
  --symbols roms/TVC12_D4.64K.symbols.json \
  --comments roms/TVC12_D4.64K.comments.json \
  --bank sys --bank-offset 0000H \
  --data-range C003H-C228H \
  roms/TVC12_D4.64K -o roms/TVC12_D4.64K.asm
```

Options:

| Option | Meaning |
| --- | --- |
| `--origin <addr>` | CPU address for the first input byte; defaults to `0`. |
| `-o <path>`, `--output <path>` | Write assembly source to a file; omitted means stdout. |
| `--title <text>` | Add a listing title comment. |
| `--symbols <path>` | Load ROM labels and comments from a ROM symbol JSON document. |
| `--comments <path>` | Load address-keyed comments from a JSON document; may be repeated. |
| `--bank <name>` | Select a symbol bank such as `sys` or `exth`; required with `--symbols`. |
| `--bank-offset <addr>` | Physical bank offset corresponding to the first input byte. |
| `--data-range <start-end>` | Emit an inclusive CPU-address range as `DB`; may be repeated. |
| `-`, as input path | Read binary bytes from stdin. |

`rtvc-disasm` uses the emulator's own Z80 disassembler and checks each emitted
instruction against `assemble_line()`. Unsupported or boundary-crossing forms
fall back to `DB`, so generated files remain byte-exact assembler input. Branch
and call targets inside the input byte range are emitted as simple `Lxxxx`
labels, including targets that intentionally enter the middle of a linear
instruction.
Address comments are emitted as normal semicolon comments and do not affect
round-tripping.

### Debugger Client

Inside [scripts/rtvc_debug.py](../scripts/rtvc_debug.py):

```text
asm [addr]
asmfile helper.asm [origin]
loadasm helper.toml
```

- `asm` keeps the old one-instruction interactive patch workflow.
- `asmfile` sends a source file to the debugger assembler and writes the
  returned segments to mapped memory.
- `loadasm` loads `rtvc-asm-v1` TOML from disk and writes every segment to
  mapped memory.
- Both add the breakpoints that [`BREAK`](#breakpoints) lines recorded.

## Source Format

Source is line-oriented. Semicolon comments are ignored except inside quoted
strings:

```asm
; comment
START:  LD HL,MSG   ; inline comment
MSG:    DB "OK",0
```

Labels use `name:` syntax and are case-insensitive. Stored symbol names are
uppercase. Valid label characters are ASCII letters, digits, `_`, and `.`; a
label must start with an ASCII letter, `_`, or `.`.

A label that starts with `.` is local to the last global label above it,
the last `name:` label whose name does not start with `.`. It is stored as
`GLOBAL.LOCAL`, and `.local` in an expression means the same name, so each
routine can have its own `.loop`. From elsewhere, use the full name:

```asm
CLEAR:  LD B,0
.loop:  LD (HL),A           ; CLEAR.LOOP
        INC HL
        DJNZ .loop
        RET
FILL:   LD B,C
.loop:  LD (HL),A           ; FILL.LOOP, a different label
        ...
        JP CLEAR.loop
```

- An `EQU` constant does not start a new scope; `.name EQU value` defines a
  local constant.
- `.name` is not replaced inside strings, character literals, or comments.
  Listings keep the `.name` spelling; errors and symbols use the full name.
- A local label before the first global label keeps its `.name`.

Supported directives:

| Directive | Forms | Notes |
| --- | --- | --- |
| `ORG` | `ORG expr[, map-name, mapped-address ...]` | Sets the current CPU assembly address. Each optional name/address pair declares a persistent address mapping from this origin. Multiple `ORG` directives create multiple output segments. |
| `BASIC_START` | `BASIC_START` | Emits a tokenized TVC BASIC autorun line at `19EFH`, pads to `1A30H`, and defines `BASIC_START` as the first machine-code instruction address called by `USR(6704)`. |
| `EQU` | `LABEL EQU expr` or `LABEL: EQU expr` | Defines a constant symbol. |
| `DB`, `DEFB` | `DB expr[, expr...]` | Emits bytes. String literals are also accepted. |
| `DW`, `DEFW` | `DW expr[, expr...]` | Emits little-endian 16-bit words. |
| `DS`, `DEFS` | `DS count[, fill]` | Emits `count` bytes, filled with zero or `fill`. |
| `REPT`, `ENDR` | `REPT count[, counter]` … `ENDR` | Assembles the enclosed lines `count` times. See [Repetition](#repetition). |
| `ASSERT` | `ASSERT condition[, "message"]` | Fails assembly when the condition is false. See [Assertions](#assertions). |
| `PHASE`, `DEPHASE` | `PHASE address` … `DEPHASE` | Assembles the enclosed code for a run address while its bytes stay at the load address. See [Assembling for a Run Address](#assembling-for-a-run-address). |
| `INCLUDE` | `INCLUDE "path"` | Assembles another source file in place. See [Files](#files-include-incbin-and-import). |
| `INCBIN` | `INCBIN "path"[, offset[, length]]` | Emits the bytes of a file. |
| `IMPORT` | `IMPORT "module.toml"[, PREFIX]` | Defines the symbols of another module's TOML output. |
| `BREAK` | `BREAK` | Marks the next instruction as a debugger breakpoint. See [Breakpoints](#breakpoints). |
| `STRUCT`, `ENDS` | `STRUCT name[, base]` … `ENDS` | Defines a size and field offsets; emits no bytes. See [Structures](#structures). |
| `MACRO`, `ENDM` | `MACRO name [param[=default], ...]` … `ENDM` | Defines a macro, called by name. See [Macros](#macros). |

String literals in `DB`/`DEFB` must be ASCII. Supported escapes are `\0`, `\n`,
`\r`, `\t`, `\\`, `\"`, and `\'`.

## Repetition

`REPT count` … `ENDR` assembles the enclosed lines `count` times:

```asm
        REPT 20
        NOP
        ENDR
```

- The count is an expression. It is evaluated when assembly reaches the
  `REPT` line, so it may use `$`, `EQU` symbols, and labels defined above it,
  including a label on the `REPT` line itself. Symbols defined later are an
  error.
- A count of zero skips the block. Blocks may be nested.
- An optional counter name, `REPT count, NAME`, is replaced in each copy by
  the copy number, `0` to `count - 1`, in decimal. The replacement matches
  whole words, ignores case, and skips double-quoted strings. The counter is
  not a symbol and does not appear in the output.
- A label on the `REPT` line marks the first copy; a label on the `ENDR` line
  marks the address after the last copy. A global label inside the block is
  defined once per copy, so it is a duplicate-label error when the count is
  more than one.
- A [local label](#source-format) defined inside the block belongs to its
  copy, so loops can be repeated. Within a copy, `.name` means that copy's
  label; a `.name` the block does not define means the enclosing routine's
  label. The copies are stored as `GLOBAL._Rn_c.NAME`, where `n` numbers the
  `REPT` blocks that have local labels and `c` is the copy:

  ```asm
  WAIT3:  REPT 3
          LD B,40
  .delay: DJNZ .delay         ; WAIT3._R1_0.DELAY, WAIT3._R1_1.DELAY, ...
          ENDR
          RET
  ```
- Each copy keeps the source line number of the original line, both in error
  messages and in the TOML `[[lines]]` table.
- Expansion is limited to 1,048,576 lines in total.

## Macros

`MACRO name params` … `ENDM` defines a block that is assembled wherever
`name` appears in the mnemonic position:

```asm
        MACRO SETPAL index, colour=0    ; TVC palette ports 60H-63H
        LD A,colour
        OUT (60H+index),A
        ENDM

        SETPAL 1, 0F0H
        SETPAL 2            ; colour defaults to 0
```

- A macro must be defined before it is called. Names are case-insensitive.
- Each parameter is replaced in the body by the argument text, as whole
  words, ignoring case, and not inside double-quoted strings: the same rule
  as a [`REPT`](#repetition) counter. A parameter with `=default` may be
  left out or passed empty (`SETPAL 3,`).
- Arguments are split at commas outside parentheses and quotes. Wrap an
  argument in `<…>` to pass commas: `TWICE <LD A,(HL)>`.
- Argument counts are checked: too many arguments, or a missing one
  without a default, is an error naming the macro and the parameter.
- A [local label](#source-format) defined in the body belongs to that one
  expansion, so a macro may contain loops. A `.name` the body does not
  define means the caller's label. Expansions are stored as
  `GLOBAL._Mn.NAME`, numbered by call.
- A label on the call line marks the first expanded instruction.
- Macros may call other macros and may contain `REPT` or `INCLUDE`. A macro
  that calls itself, directly or through others, is an error: without
  conditional assembly it could never stop. Expansion is limited to
  1,048,576 lines in total.
- A macro may not be named like a Z80 mnemonic or an rtvc-asm directive,
  and a parameter may not be named like a register or condition (`A`,
  `HL`, `IX`, `NZ`, `C`, …), so neither needs escaping. Macros cannot be
  defined inside macros.
- Errors inside an expansion name the call and the body line:
  `game.asm:40: in macro SETPAL (macros.inc:3): unknown symbol 'X'`.
- Expanded lines belong to the call's source line in the TOML `[[lines]]`
  table and in the debugger, with the expanded text as their `source`.

### The z80macros.inc library

[coding/lib/z80macros.inc](../coding/lib/z80macros.inc) holds the 16-bit
moves and arithmetic the Z80 lacks, plus loop and unrolling helpers:
`LD_HL_DE` and the other pair moves, `SUB16_HL`, `CP16_HL`, `NEG_HL`,
`ADD_A_TO_HL`/`DE`/`BC`, `DEC_BC_JR_NZ`, `DELAY_B`, `OUTI_N`, and `LDI_N`.
Their names say they are macros, and each one's comment gives its T-states,
bytes, and every register it changes, so nothing hides a cost. A test runs
every macro on the Z80 core and checks those comments.

```bash
cargo run --bin rtvc-asm -- -I coding/lib game.asm
```

```asm
        INCLUDE "z80macros.inc"
        LD HL,(SCORE)
        LD DE,(BEST)
        CP16_HL DE          ; 30 T, 4 bytes; C when SCORE < BEST
```

## Structures

`STRUCT name` … `ENDS` describes a record layout. It emits no bytes; it
defines `name` as the size and `name.FIELD` as each member's offset, which
suits `(IX+d)` and `(IY+d)` addressing:

```asm
        STRUCT POS
X       BYTE
Y       BYTE
        ENDS

        STRUCT SPRITE
AT      POS             ; nested: SPRITE.AT, SPRITE.AT.X, SPRITE.AT.Y
FRAME   WORD
        BLOCK 2         ; unnamed padding
FLAGS   BYTE
        ENDS            ; SPRITE = 7

        LD A,(IX+SPRITE.FLAGS)  ; LD A,(IX+6)
        LD DE,SPRITE            ; the size, 7
SPRITES: DS SPRITE*8            ; eight sprites
```

- A member line is `FIELD KIND` or `FIELD: KIND`; without a field name it
  only reserves space. `KIND` is `BYTE` (1), `WORD` (2), `BLOCK size`, or
  the name of a struct defined earlier, whose fields are also defined under
  the member (`SPRITE.AT.Y`).
- `BLOCK size` is an expression evaluated where the struct is defined, so
  it may use symbols defined above it, including other structs' sizes.
- `STRUCT name, base` adds `base` to every field, for fixed layouts such as
  system variables or I/O registers: `STRUCT VARS, 0B000H` makes the first
  field `0B000H`. The size is unchanged.
- Names are ordinary global symbols containing a dot, so they never clash
  with [local labels](#source-format), which start with a dot. A struct
  name, field, or size that is already defined is a duplicate-label error.
- Only member lines, blank lines, and comments may appear inside the block.
- `LD A,(SPRITE.X)` is a memory operand at address `SPRITE.X`, not a field
  of some instance; see [Parentheses and memory
  operands](#parentheses-and-memory-operands). Write `(IX+SPRITE.X)`, or
  `(SPRITES+SPRITE.X)` for the first instance.

## Assertions

`ASSERT condition[, "message"]` stops assembly with an error when the
condition is false:

```asm
LINE_START:
        ...
LINE_END:
        ASSERT LINE_END-LINE_START = 12, "copy loop changed size"
```

- The condition is either one comparison between two expressions or a single
  expression that must be nonzero. Comparisons are `=` or `==`, `!=` or `<>`,
  `<`, `<=`, `>`, and `>=`.
- Assertions are checked in the second pass, after every symbol is defined, so
  they may refer to labels later in the source. `$` is the address of the
  `ASSERT` line.
- The error names the line, the message (or the condition when there is no
  message), and the values compared, for example
  `line 9: assertion failed: copy loop changed size (left is 13, right is 12)`.

## Breakpoints

`BREAK` marks the next instruction as a debugger breakpoint. It emits no
bytes:

```asm
START:  LD HL,TABLE
        BREAK               ; stop here when loaded through the debugger
        LD A,(HL)
```

- The breakpoint address is the run address at the `BREAK` line, so a
  `BREAK` inside `PHASE` stops where the copied code runs.
- Each `BREAK` is listed in the TOML `[[breakpoints]]` table with `addr`,
  `line`, and `file` for an included file, and in the debugger `assemble`
  reply.
- `asmfile` and `loadasm` in the debugger client add these breakpoints
  after writing the code; `bp list` shows them and `bp rm` removes them.
- `rtvc-asm --no-breakpoints` leaves them out, for release builds.

## Files: INCLUDE, INCBIN, and IMPORT

```asm
        INCLUDE "tvc-ports.inc"          ; assemble another source here
        IMPORT "boot.toml", BOOT_        ; another module's symbols
        ORG 8000H
START:  CALL BOOT_KEY_SCAN
FONT:   INCBIN "font.bin"                ; the whole file
GLYPH:  INCBIN "font.bin", 64*3, 64      ; offset 192, 64 bytes
```

- `INCLUDE "path"` assembles the lines of another source file in place of
  the `INCLUDE` line. A label on the `INCLUDE` line marks the address of
  the included code. Included files may include others, up to 32 deep; a
  file that includes itself, directly or not, is an error. `%NAME%`
  placeholders apply to included files as to the main source.
- `INCBIN "path"[, offset[, length]]` emits the bytes of a file. Offset and
  length are expressions that may use symbols defined above the line, and
  a `REPT` counter: `REPT 4, I` … `INCBIN "font.bin", I*64, 8` … `ENDR`.
  Without a length the rest of the file is emitted. A range past the end of
  the file is an error.
- `IMPORT "module.toml"[, PREFIX]` defines every symbol of an `rtvc-asm-v1`
  TOML file, as written by `rtvc-asm`, as a constant. With a prefix, `KEY`
  becomes `PREFIXKEY`, so `BOOT_` gives `BOOT_KEY`. The symbols can be
  used below the `IMPORT` line in the first pass and everywhere in the
  second. A name that is already defined is an error. Imported symbols are
  not written to this module's `[symbols]`.
- A relative path is looked up next to the file that names it, then in
  each `-I` directory in order. An absolute path is used as it is.
- Lines from included files are listed in the TOML `files` array and
  marked with `file` in `[[lines]]`; errors in them name that file:
  `tvc-ports.inc:12: unknown symbol 'PORT'`.
- The debugger `assemble` command has no file access, so these
  directives are errors there.

## Assembling for a Run Address

Code that is copied before it runs can be assembled where it runs while its
bytes stay where they are loaded. `PHASE address` starts such a block and
`DEPHASE` ends it:

```asm
        ORG 8000H
COPY:   LD HL,SUPPORT_LOAD
        LD DE,SUPPORT_RUN
        LD BC,SUPPORT_LEN
        LDIR
        JP SUPPORT_RUN

SUPPORT_LOAD:
        PHASE 21BCH
SUPPORT_RUN:
        LD A,(FLAGS)        ; FLAGS is a 21BCH-based address
        ...
FLAGS:  DB 0
SUPPORT_END:
        DEPHASE
SUPPORT_LEN EQU SUPPORT_END-SUPPORT_RUN
```

- Inside the block, `$` and every label are run addresses. The bytes go on
  at the load address, in the same output segment as the code around them.
- After `DEPHASE`, assembly continues at the load address just past the
  block: in the example, the byte after `FLAGS`.
- A label on the `PHASE` line is the load address of the block; a label on
  the `DEPHASE` line is the run address just past it.
- The run address may use symbols defined above the `PHASE` line only.
- `PHASE` cannot nest. `ORG` and `BASIC_START` inside a block, `DEPHASE`
  without `PHASE`, and a `PHASE` with no `DEPHASE` are errors.
- `label@map` applies to the run address, like any other label value.
- In the TOML `[[lines]]` table, lines inside the block have
  `run_addr` next to `addr`, which stays the load address.

## Expressions

Values are 32-bit signed integers while an expression is evaluated, and are
range-checked where they are used (a byte operand must be `0..255`, a word
`0..65535`).

Primary values:

- decimal numbers;
- `0x1234`, `$1234`, and `1234H` hexadecimal;
- `0b1010` and `1010B` binary;
- labels and `EQU` symbols;
- `$` as the current address;
- `label@map` address transformations;
- parenthesised subexpressions;
- function calls (see [Functions](#functions)).

A word that reads as a number is a number, as before: `C000H` is
`0C000H`, not a symbol. Function names are only special directly before `(`.

### Functions

| Function | Value |
| --- | --- |
| `LOW(x)` | `x & 0FFH` |
| `HIGH(x)` | `(x >> 8) & 0FFH` |
| `ABS(x)` | absolute value |
| `MIN(a, b, …)`, `MAX(a, b, …)` | smallest or largest of two or more values |
| `SIN(angle, period, amplitude)` | `amplitude × sin(2π × angle / period)`, rounded |
| `COS(angle, period, amplitude)` | the same with `cos` |

`SIN` and `COS` measure the angle in steps of a full turn of `period` steps,
which must be positive; angles outside one turn wrap around. Results are
rounded half away from zero, so `SIN(1,12,1)` (sin 30° = 0.5) is `1`. They
are computed the same way on every build, so a table assembles to the same
bytes in the debugger and on the command line. With [`REPT`](#repetition)
they build lookup tables:

```asm
SINE:   REPT 256, I
        DB 128+SIN(I, 256, 127)     ; 128 ± 127 sine wave
        ENDR
```

Operators, from highest to lowest precedence, with C meanings:

| Precedence | Operators | Notes |
| --- | --- | --- |
| 1 | unary `-`, `+`, `~`, `!` | `~` is bitwise not; `!x` is 1 when `x` is 0, otherwise 0 |
| 2 | `*`, `/`, `%` | `/` and `%` truncate towards zero; dividing by zero is an error |
| 3 | `+`, `-` | |
| 4 | `<<`, `>>` | shift count must be `0..31`; `>>` keeps the sign |
| 5 | `&` | |
| 6 | `^` | |
| 7 | `\|` | |

Binary operators of equal precedence group left to right, so `10-4-3` is
`3`. Comparisons are not general operators; they belong to
[`ASSERT`](#assertions).

```asm
GLYPHS      EQU 89
GLYPHS_SIZE EQU GLYPHS*64          ; 1640H
TABLE       EQU 0B000H
TABLE_PAGE  EQU HIGH(TABLE)        ; 0B0H
            LD A,(TABLE+2*2)       ; memory operand
            OUT (60H+INDEX),A
```

### Parentheses and memory operands

An operand whose whole text is one parenthesised group is a Z80 memory
operand: `LD A,(TABLE+4)` loads from `TABLE+4`, and `OUT (60H+1),A` writes
port `61H`. An operand that only starts and ends with parentheses, such as
`(2+1)*(1+1)`, is an expression. To use a parenthesised constant as an
immediate value, start it with `+` or `0+`:

```asm
        LD A,(X+1)      ; load from address X+1
        LD A,+(X+1)     ; load the value X+1
```

`DB`, `DW`, `EQU`, and other directives have no memory operands, so
`DB (X+1)*2` is always an expression.

A [struct](#structures) field is a number, so the same rule applies:
`LD A,(SPRITE.X)` loads from the address equal to the field offset. Add a
base: `LD A,(IX+SPRITE.X)` or `LD A,(PLAYER+SPRITE.X)`.

In the first pass an expression that uses a symbol defined later is
assumed to be a placeholder value, and arithmetic errors it may cause are
ignored; the second pass evaluates it with every symbol known.

## Runnable TVC BASIC Programs

Use `BASIC_START` instead of `ORG` for small experiments that should load as an
autorun TVC BASIC program. The directive emits the same one-line bootstrap
shape used by `TVBALL.CAS`:

```asm
        BASIC_START

        LD A,02H
FLASH:  OUT (00H),A
        XOR 0AH
        JP FLASH
```

Assemble directly to CAS:

```bash
cargo run --bin rtvc-asm -- --format cas experiment.asm -o experiment.cas
```

A program assembled at any other `ORG` also becomes a runnable CAS. rtvc-asm
puts the same BASIC line in front of it, followed at `1A30H` by a 14-byte stub
that copies the program to its `ORG` and jumps to it:

```asm
        ORG 8000H
START:  LD A,02H
FLASH:  OUT (00H),A
        XOR 0AH
        JP FLASH
```

- The program must be one contiguous segment between `1A3EH` (just after the
  stub) and `BFFFH`. The BASIC stack and system variables are below `19EFH`,
  so this range is free while BASIC is running.
- The CAS payload, BASIC line and stub included, can be at most 42,256 bytes.
- The stub jumps to `START` when it is defined, otherwise to the first byte.
  `--entry NAME` picks another label inside the program.
- A `RET` returns to BASIC, which then stores the `USR` result just after the
  loaded payload. Use `RET` only when the program leaves that area alone.

Use `--format toml` when the emitted TOML should be loaded through the debugger
with `loadasm`, and `--format bin` when only the contiguous machine-code bytes
are needed.

## Instruction Coverage

The instruction encoder supports the Z80 forms currently implemented by
`assemble_line`, including:

- `LD`, `INC`, `DEC`;
- `ADD`, `ADC`, `SBC`, `SUB`, `AND`, `XOR`, `OR`, `CP`;
- `JP`, `JR`, `DJNZ`, `CALL`, `RET`, `RST`;
- `PUSH`, `POP`, `EX`, `EXX`;
- `IN`, `OUT`, `IM`;
- `BIT`, `RES`, `SET`, `RLC`, `RRC`, `RL`, `RR`, `SLA`, `SRA`, `SLL`, `SRL`;
- fixed forms such as `NOP`, `HALT`, `DI`, `EI`, `NEG`, `RETN`, `RETI`,
  `RRD`, `RLD`, `LDI`, `LDIR`, `LDD`, `LDDR`, `CPI`, `CPIR`, `CPD`, `CPDR`,
  `INI`, `INIR`, `IND`, `INDR`, `OUTI`, `OTIR`, `OUTD`, and `OTDR`;
- the undocumented forms the disassembler prints: `SLL`, `IN F,(C)`,
  `OUT (C),0`, the IX/IY halves `IXH`, `IXL`, `IYH`, `IYL` in `LD`, `INC`,
  `DEC`, and 8-bit ALU instructions (for example `LD IXH,12H`,
  `ADD A,IYL`), and indexed rotate/shift, `RES`, and `SET` forms that also
  copy the result to a register (for example `RLC (IX+5),B`,
  `SET 3,(IY-1),A`).

Every instruction the disassembler decodes assembles back to the same
instruction. An index half cannot be combined with `H`, `L`, `(HL)`, or a
half of the other index register, because the prefix would change their
meaning.

Unsupported mnemonics or unsupported operand forms are errors. The assembler
does not implement macros, conditional assembly, local-scope rules,
relocation records, listing files, or third-party assembler compatibility
syntax.

## TOML Output

`rtvc-asm` emits versioned TOML. The format is intended to be consumed by
future linker/injection tools without losing address or symbol information,
while remaining pleasant to inspect and annotate:

```toml
format = "rtvc-asm-v1"
source = "helper.asm"
requested_origin = 0x7000
origin = 0x8000
next_addr = 0x8009

[symbols]
MSG = 0x8006
START = 0x8000

[[segments]]
addr = 0x8000
len = 0x09
bytes = [
  0x21, 0x06, 0x80, 0xC3, 0x00, 0x80, 0x4F, 0x4B, 0x00,                       # |!.....OK.|
]
```

Fields:

| Field | Meaning |
| --- | --- |
| `format` | Always `rtvc-asm-v1` for this version. |
| `source` | Source path, or `-` for stdin. |
| `requested_origin` | CLI/API origin before any source-level `ORG`. |
| `origin` | Address of the first emitted segment, or the requested origin if nothing is emitted. |
| `next_addr` | Current assembly address after the final emitted statement or `ORG`. |
| `files` | Present when files were included: the main source, then each included file. `[[lines]]` entries from an included file have `file`, an index into this array. |
| `segments` | Addressed byte ranges. Multiple ranges appear when source uses non-contiguous `ORG` values. |
| `symbols` | Uppercase symbol names mapped to 16-bit values. |
| `mappings` | Named `ORG` transformations with `name`, `source_base`, and `mapped_base`. |
| `breakpoints` | `BREAK` marks with `addr`, `line`, and `file` when the line is in an included file. Omitted with `--no-breakpoints`. |
| `lines` | Emitted source lines with source line number, output address (`addr`), byte length, and `run_addr` when the line is inside a `PHASE` block. |

`segments[].bytes` is the canonical data for loaders. `segments[].len` must
match the length of `segments[].bytes`.

### Named address mappings

An `ORG` can declare one or more named transformations from its address space
to another address space:

```asm
ORG C000H, SYS0, 0000H, SYS1, 4000H

START:  JP MAIN@SYS0
MAIN:   NOP

ORG C100H
TABLE:  DW TABLE@SYS0, TABLE@SYS1, MAIN

ORG D000H, DATA0, 8000H
OTHER:  JP TABLE@SYS0
```

This captures `SYS0` as `C000H -> 0000H` and `SYS1` as `C000H -> 4000H`.
The mapping declaration emits no bytes. `START` is assembled at `C000H`,
`MAIN` at `C003H`, and `TABLE` at `C100H`. A mapped reference applies:

```text
label@map = label - source_base + mapped_base
```

Therefore the `DW` values are `0100H`, `4100H`, and `C003H`, producing the
little-endian bytes `00 01 00 41 03 C0`. The final jump uses `TABLE@SYS0`,
which is `0100H`, so it emits `C3 00 01`. `DATA0` captures `D000H -> 8000H`;
the later `ORG` does not alter any earlier mapping. Plain `TABLE` continues
to mean `C100H`.

Mappings are unique within one assembled module, and both an unknown mapping
reference and a duplicate mapping declaration are errors.

Mapping references can be used in any address expression accepted by the
assembler, including instruction operands, `EQU`, `DB`, and `DW` values.

For the TVC BASIC 1.2 ROM set, the common mappings are:

| Chip | Bank offset | Canonical CPU origin | Other CPU alias |
| --- | ---: | ---: | ---: |
| `TVC12_D4.64K` | `0000H` | `C000H` | `0000H` |
| `TVC12_D3.64K` | `2000H` | `E000H` | `2000H` |
| `TVC12_D7.64K` | `0000H` | `E000H` | — |

## Typical Workflow

1. Write a small helper source:

   ```asm
   ORG 8000H
   START:  LD HL,MSG
           JP START
   MSG:    DB "OK",0
   ```

2. Assemble it:

   ```bash
   cargo run --bin rtvc-asm -- --origin 7000H helper.asm -o helper.toml
   ```

3. Load it into the active machine through the debugger client:

   ```text
   rtvc> loadasm helper.toml
   ```

4. Use debugger commands such as `disasm`, `read`, breakpoints, and CPU stepping
   to inspect or execute the loaded helper.

## Error Behavior

`rtvc-asm` reports source errors as `file:line: message` on standard error,
with no program-name prefix, so editors and build tools can jump to the line.
Standard input is named `-`. `%NAME%` placeholder errors use the same form:

```text
helper.asm:2: unknown symbol 'MISSING'
helper.asm:4: relative target 9000H is out of range from 8000H
helper.asm:7: missing definition for %BLOCK_SIZE%
-:1: unknown symbol 'Q'
```

Other failures, such as bad options or an output format the program does not
fit, keep the `rtvc-asm:` prefix.

The debugger `assemble` command has no file name and reports
`line 2: unknown symbol 'MISSING'`.

In the Rust API, `AsmError` keeps the parts apart: `message()` is the text,
`line()` is the 1-based source line when there is one, `Display` renders
`line N: message`, and `display_in(file)` renders `file:N: message`. A line
inside a `REPT` copy is the line of the original source line.

`loadasm` validates that TOML has `format = "rtvc-asm-v1"`, at least one segment,
16-bit segment addresses, byte arrays containing integers in `0..255`, and
matching `len` values when present.
