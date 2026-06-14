# ROM Symbol Database

The portable BASIC 1.2 ROM symbol database is stored in
[data/rom_symbols_1_2.json](../data/rom_symbols_1_2.json). It is intended to
support debugger annotations, AI-assisted execution tracing, developer lookup,
and generated HTML or Rust tables.

## Address Identity

A TVC CPU address is not a unique ROM location because the MMU can map different
physical banks into the same CPU page. Consumers must resolve both:

- physical bank (`sys` or `exth`);
- offset within that bank.

The `address` field is the usual CPU-visible address. The `offset` field is the
stable physical-bank identity. In particular, both SYS and EXTH contain symbols
with `0xFxxx` CPU addresses.

SYS normally occupies `0xC000-0xFFFF`, but it also appears at `0x0000-0x3FFF`
during reset and some paging transitions. EXTH occupies `0xE000-0xFFFF`; the
BASIC 1.2 extension code starts at EXTH offset `0x1000`, visible at CPU address
`0xF000`.

## Scope and Confidence

The initial database is curated rather than a raw OCR import. It concentrates
on execution landmarks, useful callable routines, and major ROM tables.
Descriptions were normalized into short English summaries from the two
Hungarian ROM books. The checked-in ROM binaries and their SHA-256 hashes
identify the byte images to which the table applies.

`usage` values have these meanings:

- `trace`: useful as an execution or event landmark;
- `call`: potentially useful to a machine-code developer;
- `data`: table, constant, text, or other non-code structure.

An entry marked `call` is not an ABI guarantee. Callers must still respect ROM
paging, alternate registers, BASIC stack state, work variables, and any
subsystem-specific setup named in the entry.

Blank input or output fields mean the calling convention has not yet been
curated. They must not be interpreted as "no inputs" or "no outputs."

## Sources

- Kaszanyiczki Laszlo, *A Videoton TV-Computer ROM listaja*: primary routine
  and address index.
- Ludanyi Laszlo, *A TV-Computer ROM programja*: explanations and calling
  details.
- `TVC12_D3.64K`, `TVC12_D4.64K`, and `TVC12_D7.64K`: authoritative BASIC 1.2
  ROM bytes and physical-bank placement.

Future BASIC 2.2 work should use the same schema in a separate JSON file.
Addresses should be transferred by matching code and data against the 2.2 ROM,
not by assuming a constant offset from BASIC 1.2.

The integrated debugger loads this JSON at runtime from an embedded string. It
uses the MMU's currently mapped physical ROM bank when annotating PC and
disassembly addresses. The Events pane can install only `trace` entries as
bank-aware execution tracepoints; no tracepoint lookup occurs while that option
is disabled.
