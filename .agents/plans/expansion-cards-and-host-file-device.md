# Expansion Card Abstraction and Host File Device

Backlog entry: [TODO.md — Storage And Expansion](../../TODO.md#storage-and-expansion).

## Goal

1. Replace the HBF-only expansion handling with a card abstraction modeled on
   the TVC expansion connector, so any slot can hold any card and several cards
   of the same kind can coexist (for example two HBF floppy cards).
2. Add a virtual **host file card** that gives TVC software a `FILE` device
   backed by a host directory (native) or a browser file store (web). It talks
   to its own ROM through ordinary card I/O ports. The Z80 core is not touched.

The idea comes from ep128emu's TVC FileIO ROM (`tvcfileio.s`): an extension ROM
with a `MOPS` device header that the TVC OS registers as a device. Only the idea
is reused. ep128emu traps an unused `ED FE` opcode and keeps CAS handling in
Z80 code; rtvc uses card I/O ports, keeps CAS handling in Rust, and ships a
newly written ROM. ep128emu's ROM is GPL and rtvc is MIT, so no ep128emu code
or ROM bytes may be copied.

## Background

### What the hardware gives a card

From the TVC hardware book (`tvchardver`, sections on the memory pager, I/O
decoding, and table 5 "Bővítő egységek") and the existing
[TVC reference](../../info/tvc.md#expansion-system):

| Connector signal | Meaning for a card |
| --- | --- |
| `NSLOTn` | I/O select for the slot's 16 ports (`0x10`/`0x20`/`0x30`/`0x40` + offset). The card decodes the low address bits itself; all address lines plus `NBRD`/`NBWR` are on the connector. |
| `NEXPn` | Memory select for the 8 KiB IOMEM window `0xC000-0xDFFF`, active only when page 3 maps EXT and port `0x03` bits 6-7 select this slot. |
| ID lines | Two bits per slot, read together through `0x5A`/`0x5E`. `00` serial (RS232), `01` video game module (the boot ROM calls it `VGB`), `10` floppy, `11` empty or "identify another way". |
| Interrupt line | Active-low request visible in `0x59`/`0x5D` bits 0-3. Writes to `0x58`/`0x59`/`0x5A`/`0x5B` bit 7 clear/disable (`0`) or enable (`1`) slot 0/1/2/3 requests. |
| `NRESET`, clock | Card reset and timing. |
| `NNMI` | Cards may request NMI. |
| `NSLOWEXP` | Wait-state request (32k/64k boards only). |
| `NPHANTOM`, `DMA`/`NBUSAK`, `NP2`/`NP3`, `MUX`, light pen, sound input | Rare or board-specific; out of scope for this plan. |

**Port assignment answer:** the TVC does not assign ports dynamically. The slot
position decides the port range, so the same card answers on `0x11` in slot 0
and `0x21` in slot 1. The emulator must therefore route per slot to separate
card instances, and card firmware must discover which slot it lives in.

### How the TVC OS discovers cards

From the commented system ROM listing (`KL-tvcrom`, `F050`-`F13A`, plus
`EXP_DIRECT` at `C4D0` and `SLOT_NUM` at `C50E`):

1. The boot code reads `0x5A`, then for each slot pages the slot into IOMEM
   (`OUT 03`) and fills a 0x30-byte entry of the card table at
   `0x0040 + slot * 0x30` (`0x0040`, `0x0070`, `0x00A0`, `0x00D0`).
2. ID `00`/`01`/`10` produce the built-in names `RS232`/`VGB`/`DISK` without
   reading card memory. ID `11` makes the OS look for `MOPS` at `0xC000`. If
   present, the length-prefixed name that follows is copied into the entry;
   otherwise the entry is marked `0xFF` (no card).
3. Entries with identical names get increasing unit numbers at entry `+7`, so
   two `DISK` cards are distinguishable to software.
4. For every identified non-serial card the OS pages the card in, sets
   `IX = entry address`, and calls the address stored at `0xC00B`. The init
   vector is read from the fixed address `0xC00B`, so a `MOPS` header must be
   `"MOPS"`, length byte `4`, a 4-character name, the IRQ vector at `0xC009`,
   and the init vector at `0xC00B`. The device entry vector table follows.

Consequence: a `MOPS` card must report ID `11`, and its ROM can compute its own
slot from `IX` during init: `slot = (IX - 0x40) / 0x30`, so
`port_base = 0x10 + slot * 0x10`.

### Current rtvc state

- [expansion.rs](../../src/emulator/expansion.rs): `ExpansionSlots` stores
  `[Option<HBF>; 4]`, routes `0x10-0x4F` by slot, builds the `0x5A` ID byte,
  and maps the EXT window to the slot selected by port `0x03`.
- [tvc.rs](../../src/emulator/tvc.rs): `add_rom` attaches any ROM whose name
  contains `DOS` as HBF in slot 0; disk APIs use `slot0()`/`slot0_mut()`;
  writes to `0x58-0x5B` are ignored; `0x59` reports `pend_it` without slot
  interrupt bits; FDC logging is hard-wired to ports `0x10-0x14`.
- [tvc_snapshot.rs](../../src/emulator/tvc_snapshot.rs): `META` stores a
  `has_hbf` flag and an optional `HBF ` chunk for slot 0.
- [cas.rs](../../src/emulator/cas.rs): CAS constants, `encode_tvc_cas`, and the
  144-byte header model (128-byte container block + 16-byte program header).

## Design Decisions

1. **Card interface mirrors the connector.** One `ExpansionCard` trait with:
   - `id_bits() -> u8` (two bits, `NIDENT`);
   - `mem_read(offset) / mem_write(offset, val)` for the 8 KiB window, called
     only while selected; `mem_peek(offset)` for side-effect-free debugger reads;
   - `io_read(port: u16) / io_write(port: u16, val)` with the full 16-bit
     address (cards normally decode only the low nibble);
   - `irq_active() -> bool`, plus slot-level enable/clear handled by
     `ExpansionSlots` from `0x58-0x5B` writes;
   - `reset()`, and `advance(cycles)` for cards with time-based behavior;
   - snapshot write/read with a stable per-card kind tag.

   NMI, wait states, audio input and the other rare signals stay out of the
   trait until a card needs them.
2. **Static dispatch with an enum.** `enum Card { Hbf(Hbf), HostFile(HostFileCard) }`
   implements the trait by matching. This keeps EXT-window accesses cheap,
   avoids downcasting for card-specific media APIs, and keeps the lightweight
   WASM build small.
3. **Per-slot instances everywhere.** Disk media, dirty flags, snapshots, UI and
   logging address a card by slot. No `slot0()` shortcuts remain outside a thin
   compatibility layer during migration.
4. **Host file card reports ID `11`** and is discovered through its `MOPS`
   header like any third-party card.
5. **CAS handling lives in Rust.** The card presents the TVC OS with the stream
   the cassette routines expect, which starts at the 16-byte program header
   (file offset `0x80`). Rust strips the 128-byte container block on read and
   rebuilds it on close after writing, sharing helpers with
   [cas.rs](../../src/emulator/cas.rs).
6. **Whole-file buffering.** Opening reads the whole file into the card;
   closing an output file commits the buffer to the store. TVC files are small,
   this makes the card snapshot-friendly, and the web store can stay
   synchronous from the emulator's point of view.
7. **ROM is rtvc-original**, assembled with rtvc's own assembler and checked
   into the repository together with its source.

## Host File Card

### Port protocol (card-local offsets, initial proposal)

Finalize while writing the ROM, then document the final version in
[info/rtvc.md](../../info/rtvc.md).

| Offset | Read | Write |
| ---: | --- | --- |
| `0` | status of last command: `0x00` OK, otherwise a TVC error code | command |
| `1` | next data byte of the open input file | append data byte to the open output file |
| `2` | — | append one file name byte (cleared by the "name clear" command) |
| `4-7` | 32-bit parameter / result, little-endian | 32-bit parameter |
| `8` | card flags (bit 0: register as default device) | — |
| `F` | signature byte for ROM self-check | — |

Commands: reset/close all, name clear, open input, open output (create or
truncate), close, get size, get position, set position. Block reads and writes
use `INIR`/`OTIR` on offset `1`. Reading past the end sets the end-of-file
status, and the ROM checks status after each block.

### ROM responsibilities

- `MOPS` header with the fixed layout above and device name `FILE`.
- Init: derive slot and port base from `IX`, store the base in a free byte of
  the card table entry, close any open file, and optionally make `FILE` the
  default input/output device. The TVC OS has no device prefixes in file names
  (`FILE:name` is Enterprise EXOS syntax; VT-DOS's `B:` is a drive letter parsed
  by the VT-DOS ROM). Channel 5 is routed by the bytes at `0x0B05` (input) and
  `0x0B0D` (output): `5` selects cassette, `0x80 | slot` selects a card. This is
  documented in [info/vt-dos.md](../../info/vt-dos.md#cassette-and-floppy-simultaneous-use).
  The OS calls card inits in slot order, so with HBF and the file card both
  claiming the default, the higher slot wins; the card flag lets the user
  choose.
- Device entries (character I/O, block I/O, open, close, verify, seek): load
  the port base, issue port commands, and translate status into the OS's
  return conventions.
- No CAS logic and no file name rewriting in the ROM.

### Rust side

- `HostFileCard`: name buffer, command decoder, open-file state (name, mode,
  buffer, position, dirty flag), status register, snapshot support.
- File name mapping: TVC names arrive as typed (no device prefix). Map to
  lowercase host names, replace unsafe characters, resolve case-insensitively,
  try `<name>.cas` before an exact match, and never escape the configured root.
- `HostFileStore` trait with a native directory backend and a web backend.
  `wasm-full` mirrors files in IndexedDB with upload/download in the UI.
  Lightweight WASM exposes put/get/list functions to JavaScript, or leaves the
  card unavailable if that is simpler.
- Error codes: ep128emu returns `0xA1` for file not found and the TVC 2.2 OS
  reports it as "System error 161". Derive the rest (file not open, already
  open, end of file, write error) from the system ROM's error table and record
  the source.

## Implementation Order

1. **Verify OS details** in the ROM listing and by debugger tracing:
   register contents on device calls (`IX` value, function selector), free
   bytes in a card table entry, and the OS error codes.
   Also check whether VT-DOS in a non-zero slot finds its FDC (it may hard-code
   `0x10`). Record findings in [info/tvc.md](../../info/tvc.md) (OS card
   discovery belongs next to the expansion section) or
   [info/vt-dos.md](../../info/vt-dos.md).
2. **Card abstraction refactor, no behavior change.** Introduce the trait and
   `Card` enum, move HBF behind it, make routing and the ID byte fully per
   slot, and replace `slot0()` callers with slot-aware APIs. Existing tests and
   VT-DOS boot must pass unchanged.
3. **Slot interrupts.** Implement `0x58-0x5B` bit-7 enable/clear writes and
   merge card `irq_active()` lines into `0x59`/`0x5D` bits 0-3. HBF keeps its
   current behavior unless the FDC interrupt is already wired.
4. **Multiple cards.** Configurable slot layout (for example
   `slots = ["hbf", "hostfile", "", ""]`), with existing `*-vtdos` machine
   types mapping to HBF in slot 0. Per-slot disk menus when more than one HBF
   is present, and FDC logging driven by the slot's port range.
5. **Snapshots.** Per-slot chunks (for example `SLT0`-`SLT3`, each with a card
   kind tag and card state). Keep loading old `HBF ` chunks into slot 0 and
   keep the `META` `has_hbf` compatibility check for old snapshots.
6. **CAS container helpers** in [cas.rs](../../src/emulator/cas.rs): split the
   128-byte container block encode/decode out of `encode_tvc_cas` so both the
   tape path and the host file card use one implementation.
7. **Host file card in Rust** with the native store, port protocol and unit
   tests.
8. **ROM** source and binary, plus a test that reassembles the source and
   compares it with the checked-in binary.
9. **Integration:** desktop configuration for the root directory, `wasm-full`
   file store and UI, optional lightweight WASM API.
10. **Documentation:** see below.

## Validation

- Unit tests for slot routing: two HBF instances in slots 0 and 1 respond only
  on their own ranges, keep independent disk state, and produce ID byte `0xFA`.
- Unit tests for interrupt enable/clear and status bits per slot.
- Snapshot round trips for each card kind, plus loading an existing
  HBF-in-slot-0 snapshot.
- CAS helper tests: decode/encode of the 128-byte block is byte-identical to
  the current `encode_tvc_cas` output and round-trips existing `progs/*.CAS`.
- Host file card tests: name mapping, `.cas` fallback, not-found status `0xA1`,
  read to end of file, seek, write and close producing a valid CAS file,
  rejection of paths outside the root.
- Boot integration test with the card in slot 1: the card table entry at
  `0x0070` contains `FILE` after boot, with and without HBF in slot 0.
- End-to-end: `LOAD` an existing CAS through the card and run it, `SAVE` a
  BASIC program, then reload it in the tape path and through the card.
- Build checks for native, lightweight WASM variants, and `wasm-full`, as
  listed in [info/rtvc.md](../../info/rtvc.md).

## Documentation Updates

- [info/tvc.md](../../info/tvc.md): connector signals, ID code meanings, slot
  interrupt enable/clear writes, OS card discovery (card table, `MOPS` header
  layout, unit numbers, init call convention). Hardware facts only.
- [info/rtvc.md](../../info/rtvc.md): card architecture, slot configuration,
  snapshot chunks, host file card protocol, file name mapping, storage
  backends.
- [README.md](../../README.md) and [README.hu.md](../../README.hu.md): how to
  enable the card and choose the directory.
- Hungarian references under `info.hu/` following `L10N.md`.
- [TODO.md](../../TODO.md): tick the entry when done.

## Out of Scope

- NMI, wait states, `NPHANTOM`, DMA/bus request, sound input from cards, and
  the light pen line.
- Emulating real third-party cards other than HBF.
- Running ep128emu's ROM or supporting its `ED FE` trap.
