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

[src/emulator/asm.rs](../src/emulator/asm.rs) exposes two entry points:

- `assemble_line(source, pc)` assembles one instruction or `DB`/`DEFB` line at
  the given program counter.
- `assemble_program(source, origin)` runs the two-pass helper assembler and
  returns emitted segments, symbols, emitted source-line metadata, flattened
  bytes, and `next_addr`.

The TCP debugger uses `assemble_program` for its `assemble` command, so a single
debugger request may contain either one instruction or a small source block.

### Command Line

Assemble a helper source to JSON:

```bash
cargo run --bin rtvc-asm -- --origin 8000H helper.asm -o helper.json
```

Use `-` as the input path to read source from stdin:

```bash
printf 'ORG 8000H\nSTART: NOP\n' | cargo run --bin rtvc-asm -- --origin 7000H -
```

Options:

| Option | Meaning |
| --- | --- |
| `--origin <addr>` | Initial assembly address before source-level `ORG`; defaults to `0`. |
| `-o <path>`, `--output <path>` | Write JSON to a file; omitted means stdout. |
| `-`, as input path | Read source from stdin. |

`<addr>` accepts decimal, `0x` hexadecimal, `$` hexadecimal, and `H`-suffixed
hexadecimal forms.

### Disassembler Command Line

Disassemble a binary blob into `rtvc-asm` source:

```bash
cargo run --bin rtvc-disasm -- --origin C000H roms/TVC12_D4.64K -o data/TVC12_D4.64K.asm
```

ROM symbol metadata and explicit data ranges can be supplied to keep known
tables as `DB` statements while emitting instructions for code:

```bash
cargo run --bin rtvc-disasm -- \
  --origin C000H \
  --symbols data/rom_symbols_1_2.json --bank sys --bank-offset 0000H \
  --data-range C003H-C228H \
  roms/TVC12_D4.64K -o data/TVC12_D4.64K.asm
```

Options:

| Option | Meaning |
| --- | --- |
| `--origin <addr>` | CPU address for the first input byte; defaults to `0`. |
| `-o <path>`, `--output <path>` | Write assembly source to a file; omitted means stdout. |
| `--title <text>` | Add a listing title comment. |
| `--symbols <path>` | Load ROM labels and comments from a ROM symbol JSON document. |
| `--bank <name>` | Select a symbol bank such as `sys` or `exth`; required with `--symbols`. |
| `--bank-offset <addr>` | Physical bank offset corresponding to the first input byte. |
| `--data-range <start-end>` | Emit an inclusive CPU-address range as `DB`; may be repeated. |
| `-`, as input path | Read binary bytes from stdin. |

`rtvc-disasm` uses the emulator's own Z80 disassembler and checks each emitted
instruction against `assemble_line()`. Unsupported or boundary-crossing forms
fall back to `DB`, so generated files remain byte-exact assembler input.

### Debugger Client

Inside [scripts/rtvc_debug.py](../scripts/rtvc_debug.py):

```text
asm [addr]
asmfile helper.asm [origin]
loadasm helper.json
```

- `asm` keeps the old one-instruction interactive patch workflow.
- `asmfile` sends a source file to the debugger assembler and writes the
  returned segments to mapped memory.
- `loadasm` loads `rtvc-asm-v1` JSON from disk and writes every segment to
  mapped memory.

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

Supported directives:

| Directive | Forms | Notes |
| --- | --- | --- |
| `ORG` | `ORG expr` | Sets the current assembly address. Multiple `ORG` directives create multiple output segments. |
| `EQU` | `LABEL EQU expr` or `LABEL: EQU expr` | Defines a constant symbol. |
| `DB`, `DEFB` | `DB expr[, expr...]` | Emits bytes. String literals are also accepted. |
| `DW`, `DEFW` | `DW expr[, expr...]` | Emits little-endian 16-bit words. |
| `DS`, `DEFS` | `DS count[, fill]` | Emits `count` bytes, filled with zero or `fill`. |

String literals in `DB`/`DEFB` must be ASCII. Supported escapes are `\0`, `\n`,
`\r`, `\t`, `\\`, `\"`, and `\'`.

## Expressions

Expressions are intentionally limited:

- decimal numbers;
- `0x1234`, `$1234`, and `1234H` hexadecimal;
- `0b1010` and `1010B` binary;
- labels and `EQU` symbols;
- `$` as the current address;
- `+` and `-` operators.

There is no operator precedence beyond left-to-right `+`/`-`, and there are no
parenthesized arithmetic expressions. Parentheses are still used for Z80 memory
operands such as `(4000H)` and `(IX+2)`.

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
  `INI`, `INIR`, `IND`, `INDR`, `OUTI`, `OTIR`, `OUTD`, and `OTDR`.

Unsupported mnemonics or unsupported operand forms are errors. The assembler
does not implement macros, includes, conditional assembly, local-scope rules,
relocation records, listing files, or third-party assembler compatibility
syntax.

## JSON Output

`rtvc-asm` emits versioned JSON only. The format is intended to be consumed by
future linker/injection tools without losing address or symbol information:

```json
{
  "format": "rtvc-asm-v1",
  "source": "helper.asm",
  "requested_origin": 28672,
  "origin": 32768,
  "next_addr": 32777,
  "segments": [
    {
      "addr": 32768,
      "len": 9,
      "bytes": [33, 6, 128, 195, 0, 128, 79, 75, 0]
    }
  ],
  "symbols": {
    "MSG": 32774,
    "START": 32768
  },
  "lines": [
    {
      "line": 2,
      "addr": 32768,
      "len": 3,
      "source": "START: LD HL,MSG"
    }
  ]
}
```

Fields:

| Field | Meaning |
| --- | --- |
| `format` | Always `rtvc-asm-v1` for this version. |
| `source` | Source path, or `-` for stdin. |
| `requested_origin` | CLI/API origin before any source-level `ORG`. |
| `origin` | Address of the first emitted segment, or the requested origin if nothing is emitted. |
| `next_addr` | Current assembly address after the final emitted statement or `ORG`. |
| `segments` | Addressed byte ranges. Multiple ranges appear when source uses non-contiguous `ORG` values. |
| `symbols` | Uppercase symbol names mapped to 16-bit values. |
| `lines` | Emitted source lines with source line number, output address, and byte length. |

`segments[].bytes` is the canonical data for loaders. `segments[].len` must
match the length of `segments[].bytes`.

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
   cargo run --bin rtvc-asm -- --origin 7000H helper.asm -o helper.json
   ```

3. Load it into the active machine through the debugger client:

   ```text
   rtvc> loadasm helper.json
   ```

4. Use debugger commands such as `disasm`, `read`, breakpoints, and CPU stepping
   to inspect or execute the loaded helper.

## Error Behavior

Assembler errors include source line numbers for `assemble_program`:

```text
line 2: unknown symbol 'MISSING'
line 4: relative target 9000H is out of range from 8000H
```

`loadasm` validates that JSON has `format: "rtvc-asm-v1"`, at least one segment,
16-bit segment addresses, byte arrays containing integers in `0..255`, and
matching `len` values when present.
