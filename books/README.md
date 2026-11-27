# Reference books

Two Hungarian TVC books, converted from scans to Markdown. Every page was proofread against the
scans, so the text is a reliable transcription. It is still a secondary source. The ROM images,
the code and [info/tvc.md](../info/tvc.md) remain the authority for the emulator. Promote a fact into
`info/tvc.md` only after checking it against the hardware or the ROM.

| Book | File | Content |
| --- | --- | --- |
| Kaszanyiczki László: *A Videoton TV-Computer ROM listája* | [tvc_rom-kaszanyiczki.md](tvc_rom-kaszanyiczki.md) | Full commented disassembly of the system ROM (BASIC 1.2), with a Hungarian comment on almost every line. The closing chapters cover error handling, interrupt handling and calling BASIC routines directly. |
| Benedek–Monok: *TV-Computer hardver* (1988) | [tvchardver/tvchardver.md](tvchardver/tvchardver.md) | Hardware description: functional units, memory paging, I/O decoding, timing (including CLOCK STRETCH), the display unit, PAL and UHF, the Z80A, the 6845 CRTC and the power supply. It also has the I/O and register tables, 67 figures (schematics and timing diagrams, as images) and the Z80 instruction tables. |

## Finding things

- Both files mark each page with an HTML comment that gives the 1-based PDF page: `<!-- page P (pdf N) -->` in the ROM book, `<!-- pdf N -->` in the hardware book. Cite by PDF page.
- **ROM book:**
  - The code is in ```` ```asm ```` blocks, one line per instruction: `ADDR  MNEMONIC  OPERANDS  ; comment`.
  - Search for an address with `grep -n '^C4D0 ' books/tvc_rom-kaszanyiczki.md`, and for a routine by its Hungarian comment text.
  - Hex values are written without the `H` suffix in the listing and with it in the prose (`C4D0H`).
- **Hardware book:**
  - Chapters are `#` headings and sections `##` / `###`, following the book's numbering (e.g. `## 2.2 …`, `### 2.2.4 …`).
  - Figures are images in `tvchardver/`, each followed by its caption (`16. ábra A video időzítő jelek`), so search for `"N. ábra"`.
  - Register and bit-field tables (memory paging, I/O ports, colours, CRTC R0–R17) are HTML tables in chapter 4.3 "Táblázatok".
- Signal names follow the book: a leading `N` means active low (`NMREQ`, `NIORQ`, `NM1`, `NWR6`).

## Corrections to the print

- **ROM book:**
  - The listing was checked for address sequence and `DB` byte counts.
  - Typos in the printed book are kept as printed, except for a few address typos that were corrected by arithmetic.
  - The ő/ű accents follow modern spelling.
- **Hardware book:**
  - Obvious errors in the printed book are corrected.
  - Each correction is listed in [tvchardver/ERRATA.md](tvchardver/ERRATA.md), along with doubtful cases that were left as printed.
  - Before relying on a surprising detail, check it in the errata. Examples are the Z80 table entries, the palette/CRTC port addresses and the `NSLOT` numbering.
