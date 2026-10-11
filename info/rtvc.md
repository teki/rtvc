# rtvc Implementation and Usage Reference

This document describes the Rust emulator in this repository. For the
implementation-neutral machine specification, see
[TVC Technical Reference](tvc.md).

## Contents

- [Project architecture](#project-architecture)
- [Build targets](#build-targets)
- [Machine execution](#machine-execution)
- [Video emulation](#video-emulation)
- [Sound emulation](#sound-emulation)
- [Keyboard input](#keyboard-input)
- [Media handling](#media-handling)
- [Command-line helper assembler](#command-line-helper-assembler)
- [Command-line BASIC compiler](#command-line-basic-compiler)
- [Command-line CAS converter](#command-line-cas-converter)
- [ZX Spectrum TAP conversion](#zx-spectrum-tap-conversion)
- [ROM loading and fast boot](#rom-loading-and-fast-boot)
- [Snapshot format](#snapshot-format)
- [Native and web UI](#native-and-web-ui)
- [Debugger](#debugger)
- [ROM symbol database](#rom-symbol-database)
- [Configuration and persistence](#configuration-and-persistence)
- [C80 compiler](#c80-compiler)
- [Testing and validation](#testing-and-validation)

## Project Architecture

`rtvc` is a Rust library crate with native, headless, lightweight WebAssembly,
and full-web frontends.

| File | Responsibility |
| --- | --- |
| [src/z80.rs](../src/z80.rs) | Z80 execution core |
| [src/bus.rs](../src/bus.rs) | CPU bus trait and flat test bus |
| [src/mmu.rs](../src/mmu.rs) | TVC bank switching and ROM placement |
| [src/vid.rs](../src/vid.rs) | CRTC state, TVC pixel decoding, renderers |
| [src/tvc.rs](../src/tvc.rs) | TVC machine bus, timing, interrupts, devices |
| [src/zx82.rs](../src/zx82.rs) | initial Spectrum 48K memory, ULA, frame timing, and full-frame renderer |
| [src/key.rs](../src/key.rs) | keyboard matrix and host-key adaptation |
| [src/sound.rs](../src/sound.rs) | sound divider, timer, DAC, PCM generation |
| [src/cas.rs](../src/cas.rs) | CAS container encoding and CAS-to-pulse-interval conversion |
| [src/tape.rs](../src/tape.rs) | cassette transport and signal sampling |
| [src/emulator/expansion.rs](../src/emulator/expansion.rs) | `SlotDevice` card interface, `Card` enum, four-slot routing and slot interrupts |
| [src/emulator/hbf.rs](../src/emulator/hbf.rs) | HBF card memory and registers |
| [src/emulator/host_file.rs](../src/emulator/host_file.rs) | host file card, its port protocol, name mapping, and file stores |
| [roms/RTVC-HOSTFILE.asm](../roms/RTVC-HOSTFILE.asm) | rtvc-original `FILE` device ROM for the host file card |
| [src/emulator/asm.rs](../src/emulator/asm.rs) | Z80 single-line and two-pass helper assembler |
| [src/emulator/basic.rs](../src/emulator/basic.rs) | TVC BASIC tokenizer and detokenizer |
| [src/emulator/disasm.rs](../src/emulator/disasm.rs) | Z80 disassembler and debugger instruction metadata |
| [c80/](../c80/) | C80 compiler crate (`rtvc-c80`; not part of `cargo build`) |
| [src/bin/rtvc_asm.rs](../src/bin/rtvc_asm.rs) | command-line assembler that emits `rtvc-asm-v1` TOML |
| [src/bin/rtvc_basic.rs](../src/bin/rtvc_basic.rs) | command-line BASIC compiler that emits CAS or raw program bytes |
| [src/bin/rtvc_tocas.rs](../src/bin/rtvc_tocas.rs) | command-line converter that writes sibling `.cas` files from `.bas`, `.asm`, and `rtvc-asm-v1` TOML |
| [src/bin/rtvc_tap2toml.rs](../src/bin/rtvc_tap2toml.rs) | ZX Spectrum TAP parser that emits structured `rtvc-zx-tap-v1` TOML |
| [src/fd1793.rs](../src/fd1793.rs) | FD1793 floppy controller with two-drive read/write support |
| [src/emu.rs](../src/emu.rs) | machine selection, media, run state |
| [src/machine.rs](../src/machine.rs) | explicit TVC/Zx82 machine boundary and shared debugger operations |
| [src/ui.rs](../src/ui.rs) | shared native/full-web egui application |
| [src/workspace.rs](../src/workspace.rs) | simple/developer layouts |
| [src/debug_ui.rs](../src/debug_ui.rs) | integrated debugger panes |
| [src/debugger.rs](../src/debugger.rs) | native TCP debugger |
| [src/snapshot.rs](../src/snapshot.rs) | generic chunk reader/writer |
| [src/tvc_snapshot.rs](../src/tvc_snapshot.rs) | TVC snapshot serialization |
| [src/wasm.rs](../src/wasm.rs) | lightweight and full-web bindings |
| [src/zx82_main.rs](../src/zx82_main.rs) | experimental native/headless Zx82 runner |

The CPU sees only the `CpuBus` interface. `FakeBus` supplies flat memory for CPU
tests; `TvcBus` supplies the real TVC memory and I/O behavior. This keeps Z80
validation independent from machine emulation.

## Build Targets

| Target | Features | Notes |
| --- | --- | --- |
| Native desktop | default `native` | egui/eframe, cpal audio, filesystem media, zip support, TCP debugger |
| Native CLI tools | `cli-tools` without default features | disk, assembler, BASIC compiler, CAS converter, disassembler, CAS-to-WAV, and TAP conversion utilities without desktop UI/audio dependencies |
| C80 compiler | workspace crate `rtvc-c80` | `cargo run -p rtvc-c80`; uses `rtvc_core` without UI features |
| Native headless | default `native`, `--headless` CLI | machine loop and TCP debugger without GUI |
| Integrated Zx82 | default `native` and `wasm-full` | Spectrum 48K state loading through the shared application and debugger |
| Standalone Zx82 | default `native`, `cargo run --bin zx82` | focused Spectrum core runner |
| Lightweight web | `wasm,web-vid-simple` | small wasm-bindgen API, JavaScript-owned canvas and audio |
| Compatibility lightweight web | `wasm,web-vid-realistic` | same API; runtime video selection remains available |
| Full web | `wasm-full` | complete egui UI, browser files, IndexedDB, AudioWorklet |

The lightweight WASM target intentionally excludes egui, eframe, cpal, zip,
native filesystem code, and the C80 compiler. Native desktop and `wasm-full`
also omit C80. Browser-only dependencies must remain behind web features. C80
is a separate workspace crate (`c80/`); it is not a Cargo feature of `rtvc`.

Rust edition 2024 is used, requiring Rust 1.85 or newer.

## Machine Execution

`Emu` owns an explicit `Machine` enum. The implemented variants are `Tvc` and
`Zx82`; common scheduling, framebuffer, input, breakpoint, mapped-memory,
disassembly, and stepping operations dispatch through this boundary.

The normal scheduler uses a 62,500-cycle host frame budget:

1. execute one Z80 instruction;
2. add its T-states to the machine clock;
3. advance tape transport and sound by those T-states;
4. advance interleaved video when selected;
5. latch device interrupts and invoke the Z80 interrupt path when accepted;
6. check execution breakpoints and optional ROM tracepoints;
7. stop at the budget or a debugger condition.

`debug_step_instruction()` uses the same device advancement path. Debugger
stepping therefore changes video, tape, sound, interrupts, and machine time,
not just CPU registers.

The native UI requests repaints continuously while running but generates TVC
frames on a 50 Hz real-time gate. Faster host refreshes reuse the current
texture. When emulation falls behind, the UI drops backlog rather than running
multiple catch-up frames in one repaint.

`Zx82` executes the Spectrum ROM against a fixed 16 KiB ROM and 48 KiB RAM
map, offers one interrupt every 69,888 T-states, and draws a 352 x 296
framebuffer from bitmap and attribute memory. The shared application maps host
keys onto the eight-by-five Spectrum matrix and exposes Zx82 through the dock
and TCP debuggers. Both `VidModel` values are retained, but Zx82 currently
draws a completed frame for either selection. Plain 48K `.z80` versions 1, 2,
and 3 are supported; expanded-machine and peripheral-dependent states are
rejected.

## Video Emulation

`VidModel` has two runtime modes.

### Interleaved

After each CPU instruction, `Vid::stream_some()` advances the CRTC at the
two-T-state character-clock ratio. The implementation has explicit CRTC, TVC
video-generator, bounded signal-ring, and television-receiver stages. The ring
carries eight final packed IGRB pixels per character clock plus shaped sync and
blanking state; it never carries a VRAM byte that can be recolored later.

This mode preserves mid-frame palette, border, mode, start-address, and CRTC
changes. A cursor match latches the shared interrupt at the corresponding beam
position; IRQ service time is also applied to video advancement.

The television stage can see only final IGRB video, blanking, HSYNC, and VSYNC.
It measures repeated sync edges and accepts PAL-like horizontal periods of
90-110 character clocks and vertical periods of 310-340 lines. The first
observed VSYNC is only a measurement origin; vertical lock requires a later
period inside that window. Capture starts 22 lines after VSYNC and fills 288
lines, so periods shorter than 310 cannot complete the public surface. A
discontinuity, missing sync, or dropped ring samples invalidates lock and
discards the partial raster. It cannot complete a frame without both
horizontal and vertical lock.

The public surface remains 608 x 288. Its 76-character-clock horizontal aperture
starts 19 character clocks after observed HSYNC; its vertical aperture starts
22 observed lines after VSYNC. These are connected-TV sampling policy, not CRTC
line or frame limits. Stable reprogrammed CRTC timing within the receiver's PAL
tolerances can therefore display without matching the firmware's normal
register values.

The external SN74LS123 sync widths are currently isolated approximations (eight
character clocks for horizontal sync and four lines for vertical sync) because
the available textual hardware description establishes the shaping circuit but
does not provide verified board-variant RC durations. Edge timing, receiver
lock, NVRCL vertical blanking set by VS, and MA9 video re-enable are modeled;
the pulse-width constants should be replaced after schematic-component or
oscilloscope verification. If valid sync is absent for several host ticks,
rtvc displays its lost-sync surface while continuing emulation.

Native `Tvc::new()` defaults to Interleaved.

### Fast frame

The CPU runs for the host frame budget, then `Vid::draw_frame()` renders the
whole 608 x 288 framebuffer from the current VRAM, palette, and CRTC state.

This is faster and simpler but cannot reproduce raster changes made during the
frame. Lightweight WASM constructors default to Fast frame. JavaScript can call
`setVidModel("interleaved")`; `simple` and `realistic` remain accepted aliases.

### Current CRTC policy

rtvc implements the TVC port mirrors and TVC-compatible CPU register access.
It treats R12-R13 as readable/writable, R14-R15 as readable/writable, R16-R17
as read-only, and write-only reads as `0xFF`.

The visible MC6845 cursor shape/blink is not drawn because TVC software normally
uses the cursor output for timing and draws the visible cursor in bitmap memory.
Interlace, display-enable skew, cursor skew, and light-pen strobing remain
limited or deferred.

## Sound Emulation

`SoundTimer` advances from CPU cycles and models:

- the 12-bit programmable period;
- the four-bit following counter;
- counter bit 3 as oscillator output;
- the amplitude register and direct DAC mode;
- the shared sound interrupt;
- phase restart on reads from `0x5B`/`0x5F`.

The core generates mono 44.1 kHz `f32` PCM and applies a small DC-blocking
high-pass filter to approximate the AC-coupled output path. Pending PCM is
bounded to one second.

Native output uses `cpal`, duplicates mono to all host channels, converts to the
selected sample format, and performs lightweight resampling when 44.1 kHz is
unavailable. Web output uses an `AudioWorklet`; audio starts after a browser
user gesture.

`Tvc::sound_sample_rate()` reports the rate and `Tvc::take_audio_samples()`
drains generated samples. The lightweight WASM API exposes equivalent methods.

## Keyboard Input

The core stores the active-low 11 x 8 TVC matrix. Host adaptation is separate:

- native input prefers egui's physical key identity and uses text events for
  layout-aware character mapping;
- full web uses `KeyboardEvent.code` for physical identity and
  `KeyboardEvent.key` for generated characters;
- AltGr is tracked separately from ordinary Alt;
- synthesized TVC Shift compensates when host and TVC layouts require
  different modifier states;
- key release clears all modifier-map candidates to prevent stuck keys;
- focus loss, canvas blur, and visibility loss release held keys.

In Developer mode, the user must click the Screen pane to capture TVC input.
Escape, focus loss, hiding the pane, or clicking another pane releases capture.
Simple mode routes keyboard input directly.

## Media Handling

### Cassette playback

Mounted CAS files are converted to pulse intervals in CPU cycles. Tape position
advances only while playback is active and a motor bit is set. Port `0x59`
samples the current interval level.

`cargo run --bin rtvc-cas2wav -- input.cas output.wav [tape-name]` writes compatible
unsigned 8-bit mono 44.1 kHz WAV output.

### Direct cassette injection

The optional fast injection path is an emulator convenience, not TVC hardware:

1. save the current MMU map;
2. set map `0xB0` to expose RAM through all CPU windows;
3. skip the 144-byte CAS header;
4. copy payload to BASIC program address `0x19EF`;
5. if the payload starts with a tokenized BASIC program, set `TEXT`/`CHAIN`/`TOP`
   (`1722H`/`1724H`/`1726H`) so `LIST` and `RUN` see it;
6. restore the previous map.

A non-zero CAS autostart byte (`-i` / `--inject`) queues `RUN` after injection,
matching Gamebase. The Tape menu Inject action still leaves you at the prompt.
Many machine-code programs include a small BASIC loader that calls code near
`0x1B00`.

### Floppy and archives

DSK bytes can be attached to drives A: (0) and B: (1) of an HBF card. With
several HBF cards, `Emu` numbers disk units two per card in slot order: units
0 and 1 are the first card's A: and B:, units 2 and 3 the second card's, and so
on. Single-card behavior and the drive-based APIs are unchanged. The Disk menu
and status bar label the units `ex<slot> A:` when more than one card is
present, and `EMUI` stores one media name per unit. The disk
model parses FAT12 boot-sector geometry and supports the controller paths
needed by the included software, including restore, seek, step in, step out,
read sector, write sector, read address, and force interrupt. FD1793 behavior
is not yet a complete cycle-accurate implementation.

The `rtvc-dsk` utility can inspect legacy TVC/MSX-DOS style FAT12 images whose
boot sectors omit the PC `55 AA` signature and reuse later BPB bytes for boot
code. It can also create formatted images and copy host files into them:

```bash
cargo run --bin rtvc-dsk -- new720 game.dsk
cargo run --bin rtvc-dsk -- put game.dsk:HELLO.TXT local.txt
cargo run --bin rtvc-dsk -- dir game.dsk
cargo run --bin rtvc-dsk -- cat game.dsk:HELLO.TXT
cargo run --bin rtvc-dsk -- get game.dsk:HELLO.TXT local-copy.txt
```

The CLI accepts up to two `-d` arguments: the first mounts on drive A:, the
second on drive B:. The Disk menu provides Drive A: and Drive B: sub-menus
with Open, New 360K Disk, New 720K Disk, Save, and Eject actions per drive.
New disks are formatted as FAT12 images with TVC-compatible boot-sector bytes.
Native `.dsk` files loaded from an existing host path are written back
automatically after emulated sector writes. Browser-loaded disks, ZIP members,
and unsaved empty disks remain in memory until the user chooses Save Disk.

Native builds can open ZIP archives and recursively select CAS or DSK members.
The lightweight WASM core excludes zip support.

### Expansion cards

[expansion.rs](../src/emulator/expansion.rs) models the connector. Each
slot holds an optional `Card` (`Hbf` or `HostFile`), and `Card` implements the
`SlotDevice` trait by matching on the variant: two-bit `id_bits`, EXT-window
`mem_read`/`mem_peek`/`mem_write`, `io_read`/`io_write` with the full 16-bit
port address, `irq_active`, `reset`, and snapshot state. Any card kind may sit
in any slot, and several cards of one kind may coexist. NMI, wait states and
time-based `advance` stay out of the trait until a card needs them.

`ExpansionSlots` routes `0x10-0x4F` and the EXT window per slot, builds the
`0x5A` identifier byte, and keeps the per-slot interrupt enables written
through bit 7 of `0x58-0x5B` (disabled at reset). Port `0x59`/`0x5D` bits 0-3
report enabled card requests, and the CPU interrupt line is the OR of the
shared cursor/sound request and those slot requests. Neither current card
raises an interrupt. FDC write logging covers the ports of every slot holding
an HBF card and names the slot for non-zero slots.

`Tvc::attach_card(slot, ExpansionCard, rom)` plugs a card in, and
`load_disk`, `disk_dirty` and `clear_disk_dirty` take a slot and a drive.
`Tvc::add_rom` no longer turns a `*DOS*` ROM into an HBF card; machine
setup places cards through `MachineType::rom_uses` (HBF ROM files) and
`attach_builtin_cards` (cards with an embedded ROM). The lightweight WASM
`addRom` keeps the old `*DOS*` meaning for JavaScript callers.

### Host file card

The host file card (`hostfile`, ID bits `11`) gives TVC software a `FILE`
device backed by a host directory (native) or browser storage (web). Its
ROM, [RTVC-HOSTFILE.asm](../roms/RTVC-HOSTFILE.asm), is original rtvc code
assembled with `rtvc-asm` and embedded with `include_bytes!`; a unit test
reassembles the source and compares it with the checked-in
`RTVC-HOSTFILE.ROM`. The idea comes from ep128emu's `tvcfileio.s`, but no code
is shared: rtvc uses card I/O ports instead of an `ED FE` trap and keeps CAS
handling in Rust.

The ROM's `MOPS` header registers the name `FILE`. Its init routine derives
the slot from `IX`, resets the card, and, when the card's default-device flag
is set, writes `0x80 | slot` to `0x0B05`/`0x0B0D` so BASIC `LOAD`, `SAVE`,
`OPEN` and friends use the card instead of the cassette. Cards are initialized
in slot order, so with HBF and the host file card both claiming channel 5 the
higher slot wins. Without the flag, `POKE 2821,128+slot` and
`POKE 2829,128+slot` select the card manually. Implemented device functions:
character I/O, block I/O, open/create, close, verify, and seek from start,
current position or end (`BC:DE` offset, functions 6-8, as in ep128emu's ROM).

Card-local ports:

| Offset | Read | Write |
| ---: | --- | --- |
| `0` | status of the last command or data access | command |
| `1` | next input byte | append output byte |
| `2` | — | append one file name byte |
| `3` | bit 0 input exhausted, bit 1 input open, bit 2 output open | — |
| `4-7` | 32-bit parameter/result, little-endian | 32-bit parameter |
| `8` | bit 0 register as default device | — |
| `F` | signature `'H'` | — |

Commands: `00` no-op (status `0`), `01` close all without committing, `02`
clear name, `03` open input (parameter byte 0 returns the CAS file type), `04`
create output (parameter byte 0 selects `0x11` unbuffered or `0x01`
buffered), `05` close input, `06` close output and commit, `07-09` input seek
set/current/end, `0A-0C` output seek set/current/end (signed offset for
current/end; the new position is returned in the parameter). Opening a new
input replaces the old one; creating an output while one is open commits the
first. Closing a file that is not open succeeds.

Status codes: `0xA1` file not found, `0xAD` host store failure, `0xE7` invalid
CAS container, `0xE9` file not open, `0xEC` end of file or seek beyond the
end, `0xFF` unknown command; the ROM adds `0xE8` for verify mismatches.

Files are buffered whole: opening reads the file, closing an output writes it.
`.cas` files are presented to the OS from offset `0x80`, the 16-byte program
header, which is the stream the cassette routines expect. The 128-byte
container block is stripped on read and rebuilt with
`cas::encode_cas_container` on close; files with other extensions are raw
bytes. A program saved through the card therefore loads through the tape path
too.

Name mapping: the TVC name is trimmed, lowercased, and every character outside
`a-z 0-9 - _ .` becomes `_` (a leading `.` too), so the result is always one
safe path component; an empty name becomes `start`. Opening for input tries
`<name>.cas`, then `<name>`, matched case-insensitively against the store's
file list. Creating a file without an extension writes `<name>.cas`.

Stores implement `HostFileStore` (`list`, `read`, `write`). `DirStore` serves
one directory, created on first write, and rejects any name that is not a
single normal path component. `MemoryStore` backs the browser builds and
queues card writes. Open files, the name buffer and the parameter register are
saved in the card's snapshot chunk; the store contents are not.

When a host file card is fitted, Machine > Expansion shows a Host files section
under the slot choices, with the default-device checkbox (applied at the next
reset) and:

- Native: the folder and a Choose folder button. The default is an existing
  `files/` in the working directory, then one beside the executable, otherwise
  a new `files/` beside the `roms/` directory in use, created on the first
  save.
- Full web: Upload file and Download. Files live in the `recent-media`
  IndexedDB store with kind `hostfile` and no recent-list limit; card writes
  are mirrored to IndexedDB each frame.
- Lightweight WASM: `attachCard(slot, "hostfile" | "vtdos12", rom)`,
  `putHostFile(name, bytes)`, `getHostFile(name)`, `listHostFiles()` and
  `loadSlotDisk(slot, drive, name, bytes)`; files are kept in memory.

### Gamebase

Gamebase launches load the embedded clean VT-DOS boot snapshot, choose the
matching TVC 1.2 VT-DOS machine, attach or inject media, start emulation, and
type `RUN` for CAS or `LOAD "*"` for DSK.

## Command-Line Helper Assembler

`cargo run --bin rtvc-asm -- [--origin <addr>] [-o output.toml] input.asm`
assembles small Z80 helper sources through the same two-pass assembler used by
the debugger. It currently emits versioned TOML for later linker and
injection tooling. The TCP debugger client can load this output with
`loadasm <path.toml>`. See [assembler.md](assembler.md) for source syntax, TOML
schema, command-line options, and debugger loading details.

## Command-Line BASIC Compiler

`cargo run --bin rtvc-basic -- input.bas -o output.cas` tokenizes numbered TVC
BASIC source into a CAS image using the BASIC 1.2 keyword table. Default CAS
headers match a BASIC `SAVE`. Pass `--format bin` for the raw payload, `--auto`
to set the autostart byte, and `-` to read source from stdin. The compiler
rejects payloads above the standard 64K TVC BASIC memory ceiling of 42,256
bytes, including the program terminator. Runtime allocations and machine
memory reservations can require a smaller program.

See [basic.md](basic.md#tokenized-program-format) for the in-memory line format
and tokenization rules.

## Command-Line CAS Converter

`cargo run --bin rtvc-tocas -- input.bas helper.asm out.toml` converts one or more
`.bas`, `.asm`, and `rtvc-asm-v1` TOML sources to sibling `.cas` files. `.bas`
inputs use the same path as `rtvc-basic`; `.asm` inputs use the same path as
`rtvc-asm --format cas`. TOML inputs are sorted by address and flattened from
`19EFH` through the highest exclusive end, with gaps zero-filled. The first
segment must start at `19EFH` and contain a tokenized BASIC program or launch
stub (trailing machine code after the `00H` terminator is allowed). Relocated
BASIC-only images at `4000H` are rejected; exclusive end must be at most
`C000H`. The CAS header uses the ASM autostart profile (`FFH`, load `19EFH`).
The command prints payload versus padding sizes. The linear payload owns the
whole span: zeroes in gaps are part of the cassette image. There is no
relocating loader. The output path is the source path with the extension
replaced by `.cas`. Files with any other extension are printed and skipped.

## ZX Spectrum TAP Conversion

`cargo run --bin rtvc-tap2toml -- input.tap -o output.toml` converts a standard
ZX Spectrum 48K TAP image to versioned `rtvc-zx-tap-v1` TOML. CODE blocks
become byte-array segments, PROGRAM blocks include decoded BASIC lines, and
non-standard data flags are preserved as raw blocks. The output also records
the original tape order, SHA-256 provenance, and TVC bridge mapping hints.

Use `-` as the input path to read from standard input. If `-o` is omitted, the
TOML is written to standard output.

## ROM Loading and Fast Boot

`TvcMmu::add_rom()` maps known ROM filenames into SYS and EXTH; unknown ROM
bytes are treated as a cartridge image.

The optional Fast boot setting applies guarded, reversible patches to known
BASIC 1.2 and 2.2 ROM byte sequences. It replaces the two-pattern RAM test with
a zero-fill and skips the firmware boot screen while preserving the calling
contract expected by BASIC. Patches are applied only when both filename and
original bytes match, and disabling the option restores the original bytes.

This feature is deliberately documented here rather than in the hardware
reference because it modifies firmware behavior.

## Snapshot Format

Snapshots begin with:

```text
RTVCSNAP
u16 version
```

The remainder is a little-endian chunk stream:

```text
u8[4] chunk_id
u32   payload_length
u8[]  payload
```

Unknown chunks are ignored. Version 2 uses:

| Chunk | Contents |
| --- | --- |
| `META` | plus model, video model, machine clock, frame state |
| `CPUZ` | Z80 registers, interrupt and HALT state |
| `MMU ` | RAM, model-appropriate VRAM, paging state |
| `VID ` | TVC mode, CRTC state, palette, border |
| `SLT0`-`SLT3` | one per occupied slot: card kind tag (`1` HBF, `2` host file) and card state |
| `HBF ` | legacy: first HBF card's RAM and controller state; still loaded |
| `BUS ` | interrupt latch, expansion selection, tape and sound state, slot interrupt enables |
| `EMUT` | optional UI machine selection and ROM revision |
| `EMUI` | optional selected media references |

`EMUT` stores is-plus, ROM revision, a legacy slot-0 HBF flag, the cartridge,
and one card byte per slot (`0` empty, `1` VT-DOS 1.2 HBF, `2` host file
card). The three-byte pre-cartridge form still loads. The `META` slot-0 HBF
flag is still checked, and a legacy `HBF ` chunk loads into the first HBF
slot of the restored layout. A slot chunk whose kind tag does not match the
card in that slot rejects the snapshot.

Keyboard state, logs, pending frontend PCM, ROM bytes, and disk bytes are not
serialized. The wrapper reconstructs the selected machine from normal ROM
resources and reattaches accessible disk media by filename.

Version 2 intentionally rejects version 1 snapshots.

Native save/load accepts raw `.rtvcsnap` and ZIP-wrapped
`.rtvcsnap.zip`. The checked-in
[boot12dos.rtvcsnap.zip](../snapshots/boot12dos.rtvcsnap.zip) is a stable
post-boot fixture and the Gamebase launch base.

`cargo bundle-web <snapshot>` creates a lightweight static snapshot player.
`cargo xtask bundle-web-skeleton` builds the player without an embedded
snapshot. `cargo xtask bundle-web-full` builds the complete web UI.

## Native and Web UI

### Modes and panes

Simple mode shows the 4:3 TVC screen. Developer mode uses `egui_dock`; its
default layout places Screen above IO Log.

Debugger Layout opens CPU, Disassembly, Memory, Breakpoints, ROM Symbols,
Events, Frame History, Instruction Trace, Screen, and IO Log panes. Pane
rendering does not advance emulation. Memory/disassembly ranges and event
histories are bounded.

### Persistence

Native preferences are stored in `rtvc.toml`, searched in the working directory
and then beside the executable. The versioned dock layout is stored separately
as `rtvc-workspace.json`.

Full web stores small preferences in `localStorage`, recent media bytes and
host files in IndexedDB, and the workspace under `rtvc_workspace_v1`. Lightweight WASM has no
egui workspace dependency.

## Debugger

### Integrated debugger

The dock debugger acts on the active `Machine` through `Emu` and is available
in native and full web. Both TVC and Zx82 provide run/pause/reset, instruction
stepping, bounded run-to-IRQ, mapped memory, disassembly, and breakpoints. Raw
banks, ROM symbols, trace landmarks, and IO logs remain TVC-specific.
The TVC CPU pane shows the configured CRTC display start address, video-interrupt
cursor address, and its zero-based active-screen raster line together as
`VID START AAAA  IRQ AAAA/R`.

The TVC-only Frame History pane offers two in-memory recording modes:
- **Per frame** keeps an adjustable 1–30 seconds at one snapshot per completed frame.
- **Long term** keeps every frame for the latest second, one snapshot per second
  from 1–10 seconds ago, and one per ten seconds from 10–30 seconds ago. Older
  snapshots expire automatically, bounding history to at most 61 snapshots.
  Ages use emulated time at 50 frames per second, so pausing does not age history.

The newest thumbnail is at the top-left. Per-frame mode lays out one second
(50 frames) per row. Long-term mode uses separate newest-first rows for frames,
seconds, and tens-of-seconds snapshots.

Switching modes clears the timeline while preserving the recording/stopped state.
Record starts a new timeline; Stop retains it; Clear empties it while preserving
the current recording state. Back Snapshot, Forward Snapshot, Return to Live,
and clickable thumbnails restore a selected snapshot and pause execution. The
pane reports actual frame/time offsets from the newest snapshot and current
memory use. Resuming or instruction-stepping from an older snapshot discards
its newer branch and continues the timeline from that emulated time.

History restore shares the normal TVC snapshot codec but loads into the current
machine so attached media remain in place. Keyboard state is released and
queued text is cleared after restore. Disk image bytes are not snapshotted or
rolled back. Save Selected Snapshot uses the normal snapshot-file writer, so
the resulting file can be loaded through the regular UI, command line, or TCP
debugger.

The Instruction Trace pane is available for both TVC and Zx82. Record clears
the previous trace and starts a configurable 1,000–1,000,000 instruction ring
buffer; Stop retains the captured entries and Clear discards them. Entries are
shown newest first and contain the pre-instruction clock, opcode bytes, main
and alternate Z80 registers, interrupt state, elapsed cycles, memory writes,
port writes, and whether an interrupt was accepted immediately afterward. TVC
entries additionally contain the main and video paging register values.

Tracing is an optional diagnostic path. The buses collect write effects only
while an instruction is actively being traced, and the normal execution path
does not allocate trace entries while recording is disabled. Reset and state
load clear existing entries so a trace cannot silently span unrelated machine
states. Instruction traces are not stored in snapshot files.

### TCP debugger

Native GUI and headless modes expose newline-delimited JSON on localhost:

```bash
cargo run --bin rtvc -- --port 8089
cargo run --bin rtvc -- --headless --port 8080
```

The debugger listener is bound before the GUI or headless emulation loop
starts. If the requested port is already in use, `rtvc` prints the bind error
and exits with a nonzero status instead of leaving an emulator instance running
without its debugger socket.

| Command | Purpose |
| --- | --- |
| `status` | CPU registers, clock, run/HALT state |
| `stats` | rolling host-time FPS |
| `step` | execute one or more complete machine instructions |
| `continue`, `pause`, `reset` | execution control |
| `breakpoint_add`, `breakpoint_remove`, `breakpoint_list` | breakpoints |
| `read_memory` | mapped memory or raw `u0`-`u3`, `vid0`-`vid3`, `sys`, `cart`, `exth` |
| `write_memory` | write bytes to the active machine's mapped CPU address space |
| `disassemble`, `assemble` | Z80 developer tools; `assemble` accepts one instruction or a small source block |
| `save_snapshot`, `load_snapshot` | snapshot files |
| `save_screenshot` | 4:3 PNG |
| `key` | key down/up or frame-paced typed text |
| `key_press` | hold a host key code for a number of 50 Hz frames, then release it |
| `instruction_trace_start`, `instruction_trace_stop` | start a new bounded trace or stop recording |
| `instruction_trace_clear`, `instruction_trace_status` | discard entries or report trace state |
| `instruction_trace_list` | return the newest captured instructions, including registers and writes |
| `close_app` | normal application shutdown |

Requests and responses are one JSON object per line. A running emulator emits
`{"event":"breakpoint","pc":...}` asynchronously when a breakpoint is hit.

Typed text is queued and each character is pressed and released over completed
emulator frames. For example, the following enters a complete BASIC command
without leaving a matrix key held:

```json
{"cmd":"key","action":"press","char":"load \"*\"\r"}
```

Frame-timed input uses, for example,
`{"cmd":"key_press","key":49,"duration":3}` to hold key code 49 (`1`) for
three completed emulator frames. The countdown advances only while the machine
is running. A repeated request for the same key replaces its remaining
duration. Reset, snapshot/state load, focus loss, or an explicit key-up releases
and cancels pending timed keys. The interactive debugger client exposes the same
operation as `key_press 49 3`, with `kp` as a short alias.

Instruction tracing can be driven with JSON such as
`{"cmd":"instruction_trace_start","capacity":100000}` followed by
`{"cmd":"instruction_trace_list","limit":100}`. List responses are capped
at 10,000 entries per request. The interactive client provides the equivalent
`trace start [capacity]`, `trace stop`, `trace clear`, `trace status`, and
`trace list [count]` commands; `itrace` is an alias.

The `assemble` command uses rtvc's built-in two-pass helper assembler. It
supports labels, `ORG` with persistent named address mappings, `EQU`,
`DB`/`DEFB`, `DW`/`DEFW`, `DS`/`DEFS`, simple `+`/`-` expressions, and `$` as
the current address. Responses keep the single-line compatibility fields
(`addr`, `len`, `bytes`, `next_addr`) and also include `segments`, `symbols`,
and emitted line-address metadata for multi-line source; mapping declarations
are returned in `mappings`. See
[assembler.md](assembler.md) for the detailed assembler reference.

The interactive client is [scripts/rtvc_debug.py](../scripts/rtvc_debug.py).
Its `asm` command keeps the one-instruction interactive patch workflow,
`asmfile <path> [origin]` assembles a helper source file and writes all returned
segments to mapped memory, and `loadasm <path.toml>` writes segments from
`rtvc-asm-v1` TOML.

## ROM Symbol Database

BASIC 1.2 execution landmarks, callable routines, and data live in one symbol
file per ROM image:
[TVC12_D4.64K.symbols.json](../roms/TVC12_D4.64K.symbols.json),
[TVC12_D3.64K.symbols.json](../roms/TVC12_D3.64K.symbols.json), and
[TVC12_D7.64K.symbols.json](../roms/TVC12_D7.64K.symbols.json).
`cargo xtask rom-symbols` generates them from the standalone listings in
[roms/](../roms/); do not hand-edit addresses. Label addresses come from the
real assembler, and curated prose is merged by `(bank, offset)`.

A CPU address alone is not a stable ROM identity because SYS and EXTH can
occupy overlapping CPU ranges. Entries are keyed by physical bank and image
offset, and the debugger resolves the live CPU address to that key through the
MMU, so every landmark fires in whatever paging view the CPU reaches it
through. The debugger annotates only when the relevant bank is currently mapped.
Disasm and breakpoints in the symbol list open the CPU address where that byte
is visible under the current map, and stay disabled when it is not. Per-image
comments use the same bank and offset, so the `E000H-FFFFH` window shows the
SYS or EXTH note that is actually mapped.

`usage` values are `trace`, `call`, and `data`. A `call` label is not an ABI
guarantee; software must still satisfy paging, register, BASIC state, and work
variable requirements. BASIC 2.2 needs a separately matched database rather
than a constant address shift.

## Configuration and Persistence

The Machine menu's Preset submenu keeps five TVC presets: 64K+ BASIC 1.2 or
2.2 with the HBF card in expansion slot 0, and the three machines without a
card (64K 1.2, 64K+ 1.2, 64K+ 2.2). The rows below it override memory (64K or
64K+) and the system ROM pair (BASIC 1.2 or 2.2, SYS and EXTH together); the
Cartridge submenu picks a `roms/*.crt` image or none; and the Expansion
submenu sets, independently for each of the four connectors, the card it
holds: none, the VT-DOS 1.2 HBF card, or the host file card. Several HBF cards
may be fitted. Slot `ex0` is I/O `0x10-0x1F`, connector CH1. System and Video
are single rows of choices, so the top level stays short.

A preset stays highlighted while the machine matches it. Any other mix is
shown by its id, for example `64ktvc12+ex1:vtdos12+cart:pack_man`. Each card
adds a `+ex<slot>:<card>` part, with card names `vtdos12` and `hostfile`;
naming a slot twice or an unknown card makes the id invalid.

`rtvc.toml` also stores the host file card settings beside the machine id:
`host_file_dir` (native folder) and `host_file_default` (claim channel 5 at
boot; default `true`, applied after reset). A
cartridge image is a 16 KiB `.crt` file in `roms/`. BASIC autostarts an image
whose first four bytes are `MOPS`, which covers `pack_man.crt` and
`VT-DOS12-CART.crt`. The HBF card and the VT-DOS cartridge are separate: the
card serves BASIC disk commands, and the cartridge is the CART bank used by
the `DOS` command. Disk images follow the connector that holds the card.

Native and full-web applications store that id in `rtvc.toml` (or browser
local storage), together with the video model, fast-boot setting, and
restorable media references. Older preset ids such as `64k-plus-1.2-vtdos`
still load. Snapshots record the same machine. A snapshot written before
cartridge and slot selection loads as the matching preset with the HBF card
in slot 0 and an empty cartridge socket.

The native application searches runtime ROM and program assets in the current
working directory first and beside the executable second. Packaged macOS apps
and extracted release archives therefore work without depending on the launch
directory.

## C80 Compiler

The C80 compiler is the [`rtvc-c80`](../c80/) crate. Implemented language, leaf codegen, branches
(shortened to `JR` when the displacement fits), scalar globals, calls
(register and `@stackcall`), arrays, pointers, prefixed strings, packed
structs, `for`/`do-while`, compound assignment, increment/decrement,
multi-unit projects, mixed BASIC/C80 linking, manual assembly entry, explicit
inline `asm` operands, listing maps, instruction timing, stack provenance, and
provenance-preserving peephole/branch-shortening are described
in [C80 Language Reference](c80.md) and the
[C80 Tutorial](c80/tutorial.md). Generated
functions are assembled with the existing helper assembler. The host CLI is
[`rtvc-c80`](../c80/src/bin/rtvc_c80.rs): `rtvc-c80 build INPUT` with
`--emit-asm`, `--emit-segments` (`rtvc-asm-v1`), and/or `--emit-bin`.
`--no-optimize` keeps baseline `JP` lowering. A TVC
manifest may name one `[basic]` file. `--emit-segments` writes one
`rtvc-asm-v1` TOML with C80/ASM segments and the tokenized BASIC payload at
`19EFH`. `rtvc-tocas` flattens that TOML into one CAS. The tutorial mixed
examples use `19EFH`/`3000H` and `compile.sh` writes `.cas` files. Callers
may also pass owned in-memory snapshots into `rtvc_c80::compile` or
`rtvc_c80::project::compile_project`. `CompilationResult::map()` is the
in-process listing; `--emit-map` is not a CLI flag. The compiler is the
`rtvc-c80` workspace crate (`cargo run -p rtvc-c80`) and is absent from native
desktop, `cli-tools`, `wasm-full`, and lightweight `wasm` builds of `rtvc`.

## Testing and Validation

The maintained commands and platform checklist live in
[the development skill](../.agents/skills/development/SKILL.md).

The fast CPU validation path is the 1,334-case FUSE suite:

```bash
cargo run --bin fuse_test
```

ZEXDOC/ZEXALL are stricter and slower:

```bash
cargo run --bin zex_test
```

Cross-target changes should at least validate native, lightweight WASM,
alternate lightweight WASM, full-web WASM, xtask, and the lightweight
dependency tree. Hardware behavior changes should update
[TVC Technical Reference](tvc.md); repository architecture, formats, or UI
changes should update this document.
