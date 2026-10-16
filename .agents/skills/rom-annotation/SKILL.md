---
name: rom-annotation
description: "Use when annotating or correcting the hand-maintained ROM listings in roms/ (TVC12_D3/D4/D7 BASIC 1.2, VT-DOS12-DISK pages): inline comments, labels, EQUs, decoding data vs code, and checking claims in the emulator. Covers the comment style, the byte-identical check, symbol regeneration, the emulator probe client, known pitfalls, and progress."
---

# ROM Listing Annotation

The listings in [roms/](../../../roms/README.md) are hand-maintained source
files that must assemble byte-identical to the ROM images. The goal is a
listing that reads like `roms/48.rom.asm`: the explanation is next to the
instructions, not only in a header.

## Rules

- **Bytes are the authority.** Every edit must keep the listing
  byte-identical. Run the check after every edit, not at the end.
- **Do not trust old prose.** Header text that came from OCR'd books or the
  first automated pass was wrong in several places (examples below). Before
  keeping or rewriting a claim, read the code; if it is still unclear, check
  it in the emulator. Say in the listing when a claim was checked in the
  emulator.
- **Do not regenerate listings** with `rtvc-disasm` over `roms/*.asm`: it
  discards all annotation.
- Commit only when the user asks (AGENTS.md policy).

## Style

Follow the annotated parts of `roms/TVC12_D4.64K.asm` (`IRQ_HANDLER`,
`RST30_DISPATCH`, the keyboard and video drivers) and the D3 runtime core.

- A short header per routine: the dashed title box, a few sentences on what
  it does and why, `Entry:` / `Exit:` lines. Leave out what the inline
  comments already say.
- A comment on almost every instruction, starting at column 41 (instruction
  `ljust(36)` + ` ; `). Say what the instruction achieves, not what it is
  ("A = 80H if enabled", not "rotate right").
- Section summary blocks (`; ====`) for shared data formats: the stack number
  format, video RAM layout, the cassette recording format.
- Name labels by meaning (`KB_INT_DECODE`, `LINE_LEFT`); replace `Lxxxx` when
  you annotate a routine. Keep an old wrong-but-published name as a second
  label on the same line if other tools may use it
  (`CHANNEL_CHAR_IO:` + `PRINT_NUMBER_OR_SPECIAL:`).
- Data is `DB` / `DW` / `DS`: jump tables as `DW` with the target name and a
  comment per entry; strings as `DB 08H, "VIDEOTON"`; long fills as `DS`.
- Inline operands are `DB` with a decoding comment: `RST 08H` + error code,
  `RST 18H` + bytecode, `RST 30H` + function byte, inline counted text.
- A byte that hides an entry point (the `LD A,n` / `LD BC,nn` skip idiom)
  stays `DB xxH ; = <instruction> when run from here, hiding the entry below`.
- RAM cells get `EQU` names in the block before `ORG`; keep the block sorted
  by address. Calls into the other SYS half use names imported as `EQU`.
- `LD HL,nnnn` feeding the `FFF0H` page-and-jump gateway targets the other
  ROM bank: keep it hex and name the target in a comment.

## Workflow per chunk

1. Read the chunk without its old comments:
   `sed -n 'A,Bp' roms/X.asm | grep -v '^;' | grep -v '^$'`.
2. Find callers of each internal label (`grep -n '\bLxxxx\b' roms/*.asm`) to
   learn the calling convention before naming it.
3. Check unclear behaviour in the emulator (below).
4. Write the new section to a scratch file and splice it in by line range
   (replace from a unique start line up to a unique stop line). Keep the
   section's instruction sequence identical; only labels and comments
   change.
5. Rename references to the replaced `Lxxxx` labels **in that file only**
   (see pitfalls), and update gateway comments in the other listings.
6. Check, regenerate symbols, test:

```bash
python3 scripts/check_rom_listings.py
```

```bash
cargo xtask rom-symbols
```

```bash
cargo xtask rom-symbols --check
```

```bash
cargo test --release --lib
```

7. Update [roms/README.md](../../../roms/README.md) (the list of annotated
   ranges) and the `CHANGES.md` Unreleased entry.

`cargo xtask rom-symbols` only covers the three TVC12 listings. New named
labels become symbol stubs; moving a label to another address drops its
curated prose (acceptable, but say so). EQU names are skipped.

## Checking in the emulator

Build and start a headless emulator, then drive it with
[scripts/rtvc_probe.py](../../../scripts/rtvc_probe.py):

```bash
cargo build --release --bin rtvc --bin rtvc-asm
```

```bash
./target/release/rtvc --headless --port 8765
```

```python
import sys; sys.path.insert(0, "scripts")
from rtvc_probe import Probe
p = Probe(8765); p.wait_boot()
p.run_asm("""
        LD IY,7F00H
        LD HL,1234
        CALL 0FA2BH             ; INTEGER_TO_STACK
""")
print(p.mem(0x7F00 - 9, 9))
p.breakpoint(0xC412); print(p.continue_to_break())   # registers at a hit
p.type_text("PRINT 6*7\n"); p.screenshot("/tmp/shot.png")
```

Techniques that worked:

- **Call a ROM routine directly** from a test program with registers set up,
  then read memory (number format, `CH_POS`, `CH_DEF`, the block I/O bug).
- **Breakpoints plus `status`** to see register values at a ROM address over
  several hits (interrupt path, `PAINT` flags). Log values in a loop.
- **Type BASIC and take a screenshot** to see user-visible behaviour, then
  read the image. Read stored program bytes from `TEXT_START` (1722H) to see
  tokenization.
- **Key presses** with `key_tap(code, mods)` (JS codes: 16 SHIFT, 17 CTRL,
  18 ALT, 20 LOCK) and RAM reads to watch keyboard state.

Pitfalls of the probe:

- Each test program must set SP (`run_asm` does); otherwise it inherits
  whatever SP the CPU was paused with and can end up halted.
- `instruction_trace_list` hangs on large traces: use breakpoints.
- Typing too fast drops keys while BASIC is busy (keep the 0.15 s delay).
- Kill the emulator when done; each run needs its own port if an old one is
  still shutting down.

## Pitfalls found so far

- **Renames across files.** D3, D4 and D7 each have their own `Lxxxx`
  labels; the same address name in D7 is a different routine. Rename only in
  the file that owns the label, then fix gateway comments by hand.
- **Mechanical symbolization mistakes.** The first pass named `LD BC/DE`
  immediates that matched RAM addresses; some are counts or screen
  coordinates (`LD BC,USRTAB+7` was `LD BC,0028H`). Check `LD rr,NAME` lines
  that feed `LDIR`, `ADD`, `RST 30H` or `CAS_WRITE_PERIODS`.
- **The other ROM bank.** In SYS code, `LD HL,Fxxx` into the gateway and
  `JP F000H` after `OUT (02H),C0H` reach the EXT ROM.
- **Wrong old prose found and fixed:** the RST 30H direction bit (bit 7 set
  = input), `V_FLAG` bits, `CH_DEF` range (80H-DFH), `CH_POS`, keyboard lock
  modes, `CAS_BORDER_SAVE`, misplaced labels (`CASSETTE_STATUS_STRINGS`,
  `BASIC_ERROR_MESSAGES`), misnamed routines (`PRINT_NUMBER_OR_SPECIAL`,
  `EXPANSION_INPUT/OUTPUT_DISPATCH`).
- **ROM bugs worth recording:** `OUT_CHARS_SAFE` / `IN_CHARS_SAFE` return
  into BC instead of reporting FAH when a block is above HI_MEM.

## Sources

- The ROM images and the code: authoritative.
- OCR'd books in `/Users/teki/dev/tvcdocs`: `KL-tvcrom/KL-tvcrom.md`
  (Kaszanyiczki: per-line Hungarian comments, good for routine purposes) and
  `TVC_ROM/TVC_ROM.md` (Lüdányi: explanations; leaves some routines, such as
  `PAINT`, unexplained). OCR quality is poor; use them as hints.
- [info/tvc.md](../../../info/tvc.md) for hardware ports and formats.

## Tracing code boundaries

[tools/rom_trace.py](tools/rom_trace.py) follows control flow from entry
points and reports where a listing hides code in `DB`, decodes data as code,
or splits instructions wrongly. It knows the TVC inline-operand conventions
(`RST 08H`/`18H`/`30H`); add handlers for other conventions (for example
`CALL PRINT_INLINE_TEXT` + counted string) through `Image.inline_calls`.
Computed jumps are invisible to it: add their targets with `--entry`,
`--counted-table` or `--word-table`.

```bash
python3 .agents/skills/rom-annotation/tools/rom_trace.py roms/VT-DOS12-DISK.ROM --origin C000H --length 1000H --entry C21CH --counted-table C00DH --listing roms/VT-DOS12-DISK.page0.asm
```

The TVC12 mechanical pass used this tracer plus a one-off emitter that
rebuilt each listing from the trace (keeping comments and named labels by
address, re-decoding code with `rtvc-disasm`, symbolizing operands) and was
checked byte-identical. For VT-DOS, either fix the reported places by hand
or write a similar emitter for the page listings.

## VT-DOS

The same workflow applies to the four `VT-DOS12-DISK.pageN.asm` listings, with
these differences:

- They have not had the tracing pass yet; `rom_trace.py` on page 0 already
  shows code inside `DB` lines. Do that first, page by page. Each page is a
  4 KiB window at C000H; calls to other pages go through the card's page
  register and look like ordinary calls into C000H-CFFFH, so trace each page
  with its own entry points.
- `cargo xtask rom-symbols` does not cover them; they only have
  `comments.json` files. There is no symbol regeneration step.
- D000H-DFFFH is the card's RAM (`DBC1H` is the IRQ vector target), not ROM.
- In the emulator, VT-DOS needs the HBF disk card in a slot and a disk image.

## Progress

Annotated inline (see roms/README.md for the exact ranges):

- D4: interrupt path, RST 30H dispatcher, kernel, keyboard, printer, sound,
  cassette forwarders, block I/O helpers, video and graphics (including
  PAINT), jump tables.
- D7: cassette motor, text, physical block read/write, timing, CRC.
- D3: runtime core (FB5BH-FFFFH), number conversion and BASIC-stack helpers
  (F80EH-FB5AH).

Planned order for the rest:

1. D3 floating-point arithmetic (`FP_ADD` .. `FP_GET_BCD_NIBBLE`).
2. D3 expression evaluator and symbol table (`EVAL_NUMERIC_EXPRESSION` ..
   `STACK_ELEMENT_SCAN`).
3. D3 functions (`EVAL_NUMERIC_ARGUMENT` .. `BASIC_VERNUM`).
4. D3 statements (E000H .. EA67H).
5. D4 editor (`EDITOR_JUMP_TABLE` .. `EDITOR_SHIFT_RIGHT`), then the D4
   startup, command loop, tokenizer and program-line handling.
6. D7 card handling, serial driver, cassette open/close/character I/O.
7. VT-DOS pages.

Update this list when a chunk is committed.
