# C80 Compiler Progress

Last updated: 2026-09-08

## Current increment

- **ID:** E13
- **State:** passed
- **Authorization:** user “continue” after E12 reaffirm

## Per-increment state

| ID | State | Last successful gate | Notes |
| --- | --- | --- | --- |
| E00–E04 | passed | T04 | Committed as 3c9e25e |
| E05 | passed | T05 | Committed as 4f78d0e |
| E06 | passed | T06 | Committed as 80e51a9 |
| E07 | passed | T07 | Committed as 627a9a2 |
| E08 | passed | T08 | Committed as 39b9ee8 |
| E09 | passed | T09 | Committed as 8924b9b |
| E10 | passed | T10 | Committed as 093125d |
| E11 | passed | T11 artifact | Mixed TOML includes BASIC; USR loads CLI file |
| E12 | passed | T12 | Reaffirmed after T11 artifact gate |
| E13 | passed | T13 | `rtvc-tocas` flattens TOML from 19EFH |

## T13 evidence (2026-09-08)

- `cargo test --lib --no-default-features --features cli-tools cas::` — 5 passed
  (out-of-order gaps, adjacent, BASIC-only, overlap/`4000H`/overflow/empty BASIC)
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools` — 9 passed
  (`.bas`/`.asm` regressions, TOML gap + ASM autostart, cassette `load_cas` USR)

`flatten_cas_image` sorts segments, zero-fills `[19EFH, highest end)`, requires
a tokenized BASIC stub at `19EFH`, and rejects overlap, empty BASIC, and
exclusive end past `C000H`. `rtvc-tocas` wraps that linear image with
`encode_tvc_cas` (type BASIC, autostart `FFH`, load `19EFH`) and reports
payload vs padding. Cassette T13 uses `Tvc::load_cas`, not `loadasm`.

A `[basic]` region is a path only; BASIC starts at `19EFH` and the linker
rejects overlap with C80/ASM. The tutorial mixed/`tvc-usr`/pong examples
place C80 at `3000H` and `compile.sh` emits `.cas` through `rtvc-tocas`.

C80 `cpu::in`/`cpu::out` are typed port intrinsics (TVC immediate ports,
`BC`-addressed dynamic I/O on other targets). `cpu::di`/`cpu::ei` emit
`DI`/`EI`. `cpu::ldir(hl, de, bc)` is inline Z80 `LDIR` (HL source, DE dest;
BC=0 copies 65536 bytes). `cpu` is a reserved builtin namespace like `project`.

## Next action

E14 (optional compression) remains unassigned. Stop unless the user assigns
it or Phase 2.
