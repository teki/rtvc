# rtvc-asm Repetition and Cycle Timing

## Motivation

Raster-timed TVC code, such as the Laser Squad intro in `tvc-ports`
(`ports/laser-squad/lsqintro/intro.asm`), is written as straight-line blocks
whose length must be an exact number of T-states. With the current assembler
this produced a 3,856-line source in which 3,354 lines are `NOP`: one
3,245-`NOP` start-up delay and many 20–25-`NOP` runs that balance branch paths.

Two separate problems showed up:

1. **Volume.** Padding cannot be expressed compactly. `DS n` happens to emit
   `NOP`s (fill 0), but nothing says so and other fillers are impossible.
2. **Correctness.** Nothing checks the timing. Every `*_MUSIC2` line in the
   intro was 13 T-states short (187 instead of 200) and nobody noticed until
   an emulator measurement showed the frame drifting by 1,542 T-states. A
   tool that counts T-states would have reported it at assembly time.

The second problem matters more. A repetition directive alone makes padding
shorter to write, but it does not make it right.

## How other Z80 assemblers handle this

| Assembler | Repetition | Macros | Conditionals | Cycle counting |
| --- | --- | --- | --- | --- |
| sjasmplus | `DUP n[,index]` / `EDUP`, alias `REPT`/`ENDR`; zero count skips | yes, per-instance `.local` labels, named arguments | `IF`/`IFDEF`/…, Lua scripting | not documented |
| RASM | `REPEAT n[,var[,start[,step]]]` / `REND`; `REPEAT`…`UNTIL`; `WHILE`/`WEND` | `MACRO name,p1,…` with `{p1}` substitution, defaults, variadic | `IF`/`ELSEIF`/`IFDEF`/`IFUSED`, `SWITCH`/`CASE`, `ASSERT` | `TICKER START,var` … `TICKER STOP,var` measures the enclosed instructions (in CPC NOP units, or T-states in Spectrum mode); `getnop(…)` gives an instruction sequence's cost; used for `DEFS 64-…` padding |
| zmac | `rept n` / `endm`; `irp`, `irpc` | yes, `?s` and `local` labels | `if`/`ifdef`/`ifeq`… | `t(addr)` T-state count up to an address, `tilo`/`tihi` for branch variants, `ocf()` opcode fetches, `sett` to reset; cycle column in the listing; e.g. `time equ t($)-t(code)` |
| Pasmo | `REPT n` with optional loop variable, `IRP` | yes, `LOCAL` | `IF`/`IFDEF` | not documented |
| z88dk z80asm | `REPT`/`REPTC`/`REPTI` … `ENDR` (constant count) | yes | yes | not documented |

Observations:

- Every mainstream Z80 assembler has a counted block repeat. The spelling
  varies (`REPT`/`ENDR`, `REPT`/`ENDM`, `DUP`/`EDUP`, `REPEAT`/`REND`).
- Only the demo-oriented assemblers (RASM for CPC, zmac's cycle-counting
  operators) treat timing as a first-class concern. Those are exactly the
  features raster code needs.
- Full macro systems are the largest piece of work and the hardest to keep
  small; the assembler reference says rtvc-asm is "intentionally small".

## Options

### A. Block repetition: `REPT n` / `ENDR`

```asm
        REPT 20
        NOP
        ENDR
```

- Expand textually before pass 1, nested, with a constant count resolved from
  already-defined `EQU`s. A zero count skips the block.
- Optional counter, as in sjasmplus and Pasmo: `REPT 4, I` makes `I` an
  ordinary symbol for each copy. Labels inside a repeated block are an error
  unless the counter is used to make them unique; this avoids adding a local
  label scope.
- Accept `DUP`/`EDUP` as aliases only if a real source needs them.
- Cost: small. It is a preprocessing pass over lines, plus line-number
  mapping so errors and `[[lines]]` TOML metadata still point at the source.

### B. Macros

Named, parameterised blocks with per-expansion local labels. This solves
repetition and much more, but it also needs argument parsing, local label
scope, recursion limits and error mapping. Recommendation: **defer**. The
cases seen so far are padding and repeated straight-line blocks, which A, C
and D cover. Revisit if ports start copying the same parameterised routine.

### C. Cycle counting and assertions (zmac/RASM style)

Give the assembler the T-state cost of each instruction and let the source
state its timing intent:

```asm
LINE_A:  OUT (06H),A
         ...
LINE_B:
         ASSERT CYCLES(LINE_A, LINE_B) = 200
```

- `CYCLES(from, to)` sums the nominal T-states of the straight-line
  instructions emitted between two labels in source order.
- Conditional branches need a rule. Proposal: inside a measured range,
  `JR cc`/`JP cc`/`DJNZ`/`RET cc`/`CALL cc` count as **not taken** by
  default, and an explicit taken path is measured as its own range. This is
  simpler than zmac's `tilo`/`tihi` and matches how balanced raster paths are
  written (each branch arm is a separate range that must add up to the same
  total).
- `ASSERT expr [, message]` fails assembly when the expression is zero. It is
  useful beyond timing (alignment, overlap, size checks that port build
  scripts currently do in Python).
- Add a T-state column to an optional listing output (`--format lst`, or
  `--listing <path>` alongside TOML), as zmac does, and record per-line
  T-states in the `[[lines]]` TOML so tools can reuse them.
- The timing source must be the same one the emulator uses. The Z80 core and
  the FUSE-derived [z80_bus_cycles_table.rs](../../src/emulator/z80_bus_cycles_table.rs)
  already know each instruction's T-states and memory-cycle positions. Expose
  a small lookup from that data rather than writing a second timing table.
- **TVC limitation:** video RAM accesses add 0.5–2 T-states each, depending
  on the beam phase (see `info/tvc.md`, "Video RAM wait states"). Static
  counts are nominal. In the intro, the copy lines are 200 T nominal but
  measured 202–203 T. The assembler can still help: it knows which
  instructions access memory, so the listing can mark memory accesses, and a
  `CYCLES` result can be reported as `nominal` plus "N memory accesses that
  may wait". Exact totals still need the emulator.
- Cost: medium. Instruction cost lookup, a two-label sum, `ASSERT`, an
  expression evaluator that can compare (`=`, `<`, `>`), and listing output.
  The expression evaluator today only has left-to-right `+`/`-`, so this
  likely also brings `*`, `/`, parentheses and comparisons.

### D. Delay generation: `DELAY n [, clobber]`

Emit the shortest instruction sequence that takes exactly `n` T-states:

```asm
        DELAY 80            ; no registers or flags changed
        DELAY 27, F         ; may change flags, e.g. AND 0FFH + NOPs
        DELAY 12980, A, F   ; may use a counted loop in A
```

- With no clobbers it may only use state-preserving forms: `NOP` (4),
  `JR $+2` (12), `INC SP`/`DEC SP` (12), `PUSH rr`/`POP rr` (21),
  `EX (SP),HL` twice (38). Some small values (for example 5, 6, 7) are
  impossible without clobbering and must be an error that says why.
- With a clobbered register it may generate a `DEC r`/`JR NZ` loop for long
  delays, which is what replaces the intro's 3,245-`NOP` start-up delay.
- Generated code must touch no memory, so video RAM waits never apply and
  the delay is exact on the TVC.
- Depends on C's instruction costs. Cost: small once C exists.

### E. Emulator-side measurement (related, not assembler work)

Checking the intro took about 10 minutes of single-stepping over the
debugger socket for two frames. A debugger feature that records the cycle
counter at chosen addresses without stopping (for example "cycle probes" that
log `pc, cycles` into a ring buffer, readable like the instruction trace)
would make this a sub-second check and would cover what static counting
cannot: video RAM waits and real branch paths. Track it as a separate rtvc
debugger plan if wanted.

## Recommendation

1. **A** (`REPT`/`ENDR` with optional counter): quick win, removes the source
   bloat, no design risk.
2. **C** (instruction costs, `CYCLES`, `ASSERT`, listing T-states): the
   feature that would have caught the intro's 13-T-state bug.
3. **D** (`DELAY`): built on C; makes padding exact and compact.
4. **E** as a separate debugger plan for the waits static counting cannot see.
5. **B** (macros): not now.

## Integration Points

- [src/emulator/asm.rs](../../src/emulator/asm.rs): repetition pre-pass,
  directive parsing, expression evaluator, `ASSERT`, `CYCLES`, `DELAY`.
- [src/bin/rtvc_asm.rs](../../src/bin/rtvc_asm.rs): listing output option.
- [src/emulator/z80_bus_cycles.rs](../../src/emulator/z80_bus_cycles.rs) or
  the Z80 core: a public per-instruction T-state lookup for the assembler.
- `rtvc-asm-v1` TOML: optional per-line `tstates` in `[[lines]]`. Adding a
  field is backward compatible for `tomllib` readers in `tvc-ports`.
- [info/assembler.md](../../info/assembler.md): document every new directive,
  including the branch-counting rule and the video RAM limitation.

## Validation

- Unit tests: nested and zero-count `REPT`, counter symbols, line mapping in
  errors; `CYCLES` over known sequences, including prefixed and conditional
  instructions; `ASSERT` failure messages; `DELAY` for every `n` from 4 to
  300 with each clobber set, checked by summing the emitted instructions.
- Cross-check: for every opcode, the assembler's T-state value equals what the
  Z80 core charges when executing it from non-video RAM.
- Real source: convert `lsqintro/intro.asm` to `REPT`/`DELAY`/`ASSERT`; the
  emitted bytes must be identical where the timing is unchanged, and the
  assertions must flag the original 187-T-state `MUSIC2` lines.
