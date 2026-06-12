# Snapshot Format

`rtvc` uses a custom, versioned snapshot format for emulator save/load state. The format is machine-specific rather than trying to reuse Spectrum `.sna`, Spectrum `.z80`, CPC `.sna`, or RetroArch save states.

## Why a Custom Format

Common snapshot formats are tied to the machine that defined them:

- ZX Spectrum `.sna` and `.z80` encode Spectrum-specific memory layout and hardware state.
- CPC `.sna` is designed for Amstrad CPC hardware and CRTC state.
- RetroArch save states are libretro-core serialization blobs, not a portable cross-emulator format.

The TVC needs its own state model: Z80 registers, TVC MMU banks, video RAM, CRTC/video state, extension hardware, and future device state.

## File Structure

Snapshot files begin with:

```text
RTVCSNAP
u16 version
```

The rest of the file is a sequence of chunks:

```text
u8[4] chunk_id
u32   chunk_length
u8[]  chunk_payload
```

Chunk payloads are little-endian. Unknown chunks are ignored, so future versions can add optional state.

## Version 2 Chunks

- `META` — machine type, video model, emulator clock, frame-complete flag.
- `CPUZ` — Z80 register arrays and interrupt/halt state.
- `MMU ` — 64 KiB TVC RAM, model-appropriate video RAM, paging registers, and plus-model state.
- `VID ` — selected video mode, CRTC registers, palette, and border color.
- `HBF ` — optional VT-DOS/HBF mutable state, including its 4 KiB RAM and FDC registers.
- `BUS ` — pending interrupt, extension mapping, tape transport, and sound generator/timer state.
- `EMUT` — optional emulator machine selection (`64K`/`64K+`, ROM version, VT-DOS presence). The native and lightweight WASM wrappers use it to load the required ROM resources.
- `EMUI` — optional UI media references: the selected `progs/` filename and mounted disk filename. The core loader ignores it.

Keyboard and log state are intentionally reset when loading a snapshot.

ROM and disk image bytes are not stored. The loader reconstructs the selected
machine from the normal ROM resources and reattaches accessible disk media by
filename, including a matching entry from the recent-disk list when available.
A 64K 1.2 machine stores one 16 KiB video bank; Plus machines store all four
video banks.

Version 2 intentionally does not load version 1 snapshots.

The sound portion of `BUS ` stores the frequency/control registers, timer counter, running flag, amplitude register, oscillator phase, and fractional PCM sample scheduler state. Pending frontend audio samples are intentionally not serialized.

## Runtime APIs

- [Tvc::save_snapshot](../src/tvc.rs) returns snapshot bytes.
- [Tvc::load_snapshot](../src/tvc.rs) restores snapshot bytes.
- [Emu::save_snapshot](../src/emu.rs) and [Emu::load_snapshot](../src/emu.rs) wrap the core API for native code.
- [WasmTvc::saveSnapshot](../src/wasm.rs) and [WasmTvc::loadSnapshot](../src/wasm.rs) expose the API to JavaScript.

Native snapshots include `EMUT` so loading restores the exact native machine selection among the five UI machine types. Core-only version 2 snapshots without `EMUT` require the caller to prepare the matching machine and ROM resources before loading.

Native snapshots also include `EMUI` so the program dropdown returns to the selected cassette or disk archive after loading. If the selected disk/archive is still accessible in `progs/`, native loading reattaches it; cassette selections are restored so pressing Play can recreate the tape generator from the original file.

## Compression

Native save/load supports raw `.rtvcsnap` files and `.rtvcsnap.zip` files. Compressed snapshots are zip archives containing a `.rtvcsnap` entry.

The native app can also start directly from a snapshot path:

```bash
cargo run --bin rtvc -- snapshots/boot12dos.rtvcsnap.zip
```

The checked-in [boot12dos.rtvcsnap.zip](../snapshots/boot12dos.rtvcsnap.zip)
fixture is a clean, fully booted TVC 1.2 VT-DOS machine. Tests that do not need
to exercise the boot process can load it to begin immediately from a stable
post-boot state.

Zip compression is intentionally kept out of the lightweight WASM build. Web bundles may include a zipped snapshot, but browser JavaScript decompresses it before calling `WasmTvc::loadSnapshot`.

User-facing snapshot and web bundle commands are documented in [../README.md](../README.md).

## Lightweight Web Bundles

`cargo bundle-web path/to/game.rtvcsnap` builds the lightweight WASM target and emits a self-contained static player under `dist/<snapshot-name>-web/` with the supplied snapshot copied in as `snapshot.rtvcsnap` or `snapshot.rtvcsnap.zip`.

`cargo xtask bundle-web-skeleton` builds the same player without embedding a snapshot and writes it to `dist/rtvc-web-skeleton/` by default. An explicit output directory may be supplied, for example:

```bash
cargo xtask bundle-web-skeleton dist/rtvc-snapshot-player
```

Release archives include the full web emulator as `web/`. Use `cargo xtask bundle-web-full [out-dir]` when you want the normal browser emulator UI; keep `bundle-web` and `bundle-web-skeleton` for lightweight snapshot-specific pages.
