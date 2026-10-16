# TVC BASIC 1.2 ROM listings

These are standalone, assembler-readable listings for the three 8 KiB ROM
images used by the TVC 64K BASIC 1.2 profile. The ROM image is the byte-level
source of truth. The two reference books supply the routine names, data-table
boundaries, explanations, and comments, which are rewritten and embedded in
the generated ASM rather than left as external reading requirements.

## Listings

| Chip image | Physical bank | Physical offset | Canonical CPU origin | Other CPU-visible address |
| --- | --- | ---: | ---: | ---: |
| [TVC12_D4.64K.asm](TVC12_D4.64K.asm) | SYS | `0000H` | `C000H` | `0000H` during page-0 SYS mapping |
| [TVC12_D3.64K.asm](TVC12_D3.64K.asm) | SYS | `2000H` | `E000H` | `2000H` during page-0 SYS mapping |
| [TVC12_D7.64K.asm](TVC12_D7.64K.asm) | EXTH | `0000H` | `E000H` | — |

`D3` and `D7` deliberately remain separate even though both can be visible at
`E000H-FFFFH`: they are different physical banks selected by different TVC
memory-map states. The named mapping on the `ORG` line at the top of each
listing records the alternate CPU address without emitting extra bytes.

## How to read a listing

- `ORG` is the CPU-visible address used for labels, absolute operands, and
  relative branches.
- `ORG address, map-name, mapped-address` records a named transformation from
  the canonical CPU address to an alternate CPU-visible address. It is
  assembler metadata and emits no bytes.
- Instruction boundaries come from tracing control flow from the reset entry,
  the device and BASIC jump tables, the BASIC function records, and the
  cross-ROM `FFF0H` gateway calls. Untraced bytes keep their linear decode
  unless a data label heads them.
- `DB`/`DW`/`DS` lines are data: tables, strings, copied RAM initialization
  bytes, character matrices, and the inline operands that follow some calls.
  Inline operands carry a decoding comment:
  - `RST 08H` + error code (`00H` means no error, and execution continues at
    that byte);
  - `RST 18H` + BASIC-stack bytecode, ending with the first byte that has bit
    7 set (operation 5, `CONST`, takes one extra byte);
  - `RST 30H` + OS function byte (class in bits 6-4, function in bits 3-0,
    with the target routine named);
  - `CALL PRINT_INLINE_TEXT` (`FE79H`) + a counted string.
- A `DB` of one or two bytes commented `= <instruction> when run from here`
  is the first byte of an instruction that deliberately hides the entry point
  below it (the `LD A,n` / `LD BC,nn` skip idiom).
- The `EQU` block before `ORG` names the U0 RAM cells the listing uses and,
  in the two SYS listings, the routines it calls in the other SYS half.
  Addresses whose meaning is not documented stay numeric.
- `LD HL,nnnn` that feeds the `FFF0H` page-and-jump gateway targets the other
  ROM bank (SYS to EXT or EXT to SYS); its comment names the routine there.
- `Lxxxx` labels are mechanical branch/call targets. Named labels are curated
  routine or table landmarks.
- Rich routine blocks include purpose, algorithm, inputs, outputs, side
  effects, destroyed state, and important implementation notes where known.
- These parts of `TVC12_D4.64K.asm` are commented instruction by instruction
  and are the reference style for further inline annotation: the RST 30H
  dispatcher (`RST30_DISPATCH` through `DEVICE_EXIT`, with the U0 bridge code
  in its header), the interrupt path (`IRQ_HANDLER` through
  `INTERNAL_IRQ_EXIT`), the kernel functions (`KERNEL_JUMP_TABLE` through
  `SLOT_NUM`), the keyboard driver (`KEYBOARD_JUMP_TABLE` through the four
  code tables), the block I/O helpers (`OUT_CHARS_SAFE`, `IN_CHARS_SAFE`),
  and the printer, sound, and cassette-forwarding drivers. In
  `TVC12_D7.64K.asm`, the cassette motor, text, physical block read/write,
  timing, and CRC routines (`CAS_MOTOR_CONTROL` through `CAS_CRC_UPDATE`)
  are annotated the same way, with a summary of the recording format.
- The counted device jump tables, `DEVICE_JUMP_TABLE_POINTERS`, the BASIC
  statement table (with each entry's keyword token), the RST 18H operation
  table, and the editor key table are `DW` entries naming their targets.

## Verifying

The listings are maintained directly as standalone source files. Assemble
them with `rtvc-asm` and compare the binary output with the corresponding ROM
image:

```text
rtvc-asm --format bin --origin C000H roms/TVC12_D4.64K.asm -o /tmp/TVC12_D4.64K.bin
rtvc-asm --format bin --origin E000H roms/TVC12_D3.64K.asm -o /tmp/TVC12_D3.64K.bin
rtvc-asm --format bin --origin E000H roms/TVC12_D7.64K.asm -o /tmp/TVC12_D7.64K.bin
```

Mapping entries are preserved in TOML output as `[[mappings]]` metadata but do
not change binary output. Landmark notes live in the matching
`TVC12_*.64K.comments.json` files. The matching `TVC12_*.64K.symbols.json`
files are the emulator debugger's BASIC 1.2 symbol database; they are
generated from these listings by `cargo xtask rom-symbols` (curated prose is
merged by bank and offset), not hand-maintained.

When a book diagram and a ROM byte disagree, retain the ROM byte and record the
uncertainty in an ASM comment. Do not repair executable bytes by copying OCR
text directly.

## VT-DOS 1.2 HBF disk ROM

[VT-DOS12-DISK.ROM](VT-DOS12-DISK.ROM) is the 16 KiB HBF card ROM. The card
maps one of four 4 KiB pages at CPU `C000H-CFFFH` (`D000H-DFFFH` is the card's
private RAM). Each listing is one page, assembled at `C000H`, because that is
the only CPU window where the bytes are visible.

| Listing | HBF page | File offset |
| --- | ---: | ---: |
| [VT-DOS12-DISK.page0.asm](VT-DOS12-DISK.page0.asm) | 0 | `0000H` |
| [VT-DOS12-DISK.page1.asm](VT-DOS12-DISK.page1.asm) | 1 | `1000H` |
| [VT-DOS12-DISK.page2.asm](VT-DOS12-DISK.page2.asm) | 2 | `2000H` |
| [VT-DOS12-DISK.page3.asm](VT-DOS12-DISK.page3.asm) | 3 | `3000H` |

Page 0 is the MOPS entry page. `C00BH` is the initializer pointer (`C21CH`),
and `C00DH` is the counted function table (functions 0 through 0BH). Function
0 is also the IRQ vector; its target `DBC1H` is in HBF RAM. Pages 1-3 repeat
the `MOPS`/`DISK` signature and are reached after the card writes its page
register. Page 1 holds the CLI message table and page 3 holds the command
table (`CD`, `DIR`, `COPY`, `FORMAT`, and the rest).

Landmark notes live in the matching `VT-DOS12-DISK.pageN.comments.json` files.
`DB` ranges cover the MOPS header, those tables, the filename and banner
strings, and unused fill at the end of pages 0 and 3. Other bytes are a linear
disassembly, so a data table that was not marked can still appear as
instructions.

Cartridge images use the `.crt` extension and fill the 16 KiB CART bank.
[VT-DOS12-CART.crt](VT-DOS12-CART.crt) is the VT-DOS cartridge.
[pack_man.crt](pack_man.crt) is a MOPS game cartridge used to test autostart.
Neither file is the HBF card ROM above.

Assemble a page and compare it with the 4 KiB slice at that file offset:

```text
rtvc-asm --format bin --origin C000H roms/VT-DOS12-DISK.page0.asm -o /tmp/VT-DOS12-DISK.page0.bin
```
