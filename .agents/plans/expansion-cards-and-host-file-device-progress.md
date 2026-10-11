# Expansion Cards and Host File Device — Progress

Plan: [expansion-cards-and-host-file-device.md](expansion-cards-and-host-file-device.md).

## Status

Implemented. All plan steps are done; reference documentation is
authoritative for the finished behavior:
[info/rtvc.md — Expansion cards / Host file card](../../info/rtvc.md#expansion-cards)
and [info/tvc.md — Operating system card discovery](../../info/tvc.md#operating-system-card-discovery).

## Decisions made during implementation

- `SlotDevice` has no `advance(cycles)`: no card is time-based yet, and calling
  it per instruction would cost time for nothing. Add it with the first card
  that needs it.
- `Tvc::add_rom` lost its `*DOS*` → HBF slot 0 fallback. Only the lightweight
  WASM `addRom` keeps it, for JavaScript compatibility.
- Disk units in `Emu` are numbered two per HBF card in slot order, so the
  existing drive-indexed APIs keep addressing the first card.
- Host file card ports: offset 3 is a file-flags register (input exhausted,
  input open, output open) so the ROM can maintain the OS end-of-file flag at
  `0x0B6E`. Separate input/output close, set-position, and seek commands
  replace the plan's single close and get-size/get-position commands; seek to
  end with offset 0 returns the size, seek current with offset 0 the position.
- The ROM derives the port base from `IX` on every call instead of caching it
  in a free card-table byte, and checks status after each byte rather than
  after a whole `INIR`/`OTIR` block, so it can stop exactly at end of file.
- Seek functions 6-8 with `BC:DE` offsets follow ep128emu's interface facts so
  software written for its `FILE` device works.
- Creating an output while one is open commits the first file instead of
  failing.
- The ROM sets `0x0B6B` (buffered flag) to `type - 0x11` on open input and
  derives the created file type from it.

## Verified

- VT-DOS 1.2 with the HBF card in slot 1 or 2 drives the FDC through that
  slot's ports and lists a directory.
- BASIC 1.2: `FILE` appears in the card table at `0x0070` with the card in
  slot 1, with and without HBF in slot 0; `SAVE`, `LOAD`, and loading an
  existing `TVBALL.CAS` work through the card; the saved file also loads
  through the tape inject path.
- `cargo test`, FUSE tests, `cargo check --bins`, both lightweight WASM
  variants, `wasm-full`, xtask, and the lightweight dependency-tree check.

## Not done / possible follow-ups

- BASIC 2.2 card table location was not checked for the host file card.
- Manual GUI check of the Machine > Expansion host file options and the web upload/download
  flow has not been done.
