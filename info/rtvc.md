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
- [ROM loading and fast boot](#rom-loading-and-fast-boot)
- [Snapshot format](#snapshot-format)
- [Native and web UI](#native-and-web-ui)
- [Debugger](#debugger)
- [ROM symbol database](#rom-symbol-database)
- [Configuration and persistence](#configuration-and-persistence)
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
| [src/cas.rs](../src/cas.rs) | CAS-to-pulse-interval conversion |
| [src/tape.rs](../src/tape.rs) | cassette transport and signal sampling |
| [src/expansion.rs](../src/expansion.rs) | four-slot expansion routing |
| [src/hbf.rs](../src/hbf.rs) | HBF card memory and registers |
| [src/fd1793.rs](../src/fd1793.rs) | current FD1793 and flat-disk model |
| [src/emu.rs](../src/emu.rs) | machine selection, media, run state |
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
| Native headless | default `native`, `--headless` CLI | machine loop and TCP debugger without GUI |
| Experimental Zx82 | default `native`, `cargo run --bin zx82` | standalone Spectrum 48K boot runner; not yet part of the shared application |
| Lightweight web | `wasm,web-vid-simple` | small wasm-bindgen API, JavaScript-owned canvas and audio |
| Compatibility lightweight web | `wasm,web-vid-realistic` | same API; runtime video selection remains available |
| Full web | `wasm-full` | complete egui UI, browser files, IndexedDB, AudioWorklet |

The lightweight WASM target intentionally excludes egui, eframe, cpal, zip,
and native filesystem code. Browser-only dependencies must remain behind web
features.

Rust edition 2024 is used, requiring Rust 1.85 or newer.

## Machine Execution

`Tvc` owns the Z80, `TvcBus`, framebuffer, clock, breakpoints, and selected
`VidModel`.

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

The initial `Zx82` core is deliberately separate from `Emu` while the shared
machine boundary is extracted. It executes the Spectrum ROM against a fixed
16 KiB ROM and 48 KiB RAM map, offers one interrupt every 69,888 T-states, and
draws a 352 x 296 framebuffer from bitmap and attribute memory. The standalone
runner maps host keys onto the eight-by-five Spectrum matrix, including
Spectrum shift chords for editing, arrows, and common punctuation. Both
`VidModel` values are retained, but Zx82 currently draws a completed frame for
either selection.

## Video Emulation

`VidModel` has two runtime modes.

### Interleaved

After each CPU instruction, `Vid::stream_some()` advances the CRTC at the
two-T-state character-clock ratio. It writes character state into a circular
stream and `render_stream()` behaves like a monitor responding to HSYNC and
VSYNC.

This mode preserves mid-frame palette, border, mode, start-address, and CRTC
changes. A cursor match latches the shared interrupt at the corresponding beam
position; IRQ service time is also applied to video advancement.

The monitor renderer produces a 608 x 288 surface. It waits for sync, applies
the expected TVC porch positioning, and draws 76 output character clocks per
line. If valid sync is absent for several host ticks, rtvc displays a black
lost-sync surface with moving white stripes while continuing emulation.

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

`cargo run --bin cas2wav -- input.cas output.wav [tape-name]` writes compatible
unsigned 8-bit mono 44.1 kHz WAV output.

### Direct cassette injection

The optional fast injection path is an emulator convenience, not TVC hardware:

1. save the current MMU map;
2. set map `0xB0` to expose RAM through all CPU windows;
3. skip the 144-byte CAS header;
4. copy payload to BASIC program address `0x19EF`;
5. restore the previous map.

The UI suggests `RUN` after injection. Many machine-code programs include a
small BASIC loader that calls code near `0x1B00`.

### Floppy and archives

DSK bytes are attached to drive 0 of an HBF card. The current disk model parses
FAT12 boot-sector geometry and supports the controller paths needed by the
included software, including restore, seek, read sector, and read address.
FD1793 behavior is not yet a complete cycle-accurate implementation.

Native builds can open ZIP archives and recursively select CAS or DSK members.
The lightweight WASM core excludes zip support.

### Gamebase

Gamebase launches load the embedded clean VT-DOS boot snapshot, choose the
matching TVC 1.2 VT-DOS machine, attach or inject media, start emulation, and
type `RUN` for CAS or `LOAD "*"` for DSK.

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
| `HBF ` | optional HBF RAM and controller state |
| `BUS ` | interrupt latch, expansion selection, tape and sound state |
| `EMUT` | optional UI machine selection and ROM revision |
| `EMUI` | optional selected media references |

Keyboard state, logs, pending frontend PCM, ROM bytes, and disk bytes are not
serialized. The wrapper reconstructs the selected machine from normal ROM
resources and reattaches accessible disk media by filename.

Version 2 intentionally rejects version 1 snapshots.

Native save/load accepts raw `.rtvcsnap` and ZIP-wrapped
`.rtvcsnap.zip`. The checked-in
[boot12dos.rtvcsnap.zip](../data/snapshots/boot12dos.rtvcsnap.zip) is a stable
post-boot fixture and the Gamebase launch base.

`cargo bundle-web <snapshot>` creates a lightweight static snapshot player.
`cargo xtask bundle-web-skeleton` builds the player without an embedded
snapshot. `cargo xtask bundle-web-full` builds the complete web UI.

## Native and Web UI

### Modes and panes

Simple mode shows the 4:3 TVC screen. Developer mode uses `egui_dock`; its
default layout places Screen above IO Log.

Debugger Layout opens CPU, Disassembly, Memory, Breakpoints, ROM Symbols,
Events, Screen, and IO Log panes. Pane rendering does not advance emulation.
Memory/disassembly ranges and event histories are bounded.

### Persistence

Native preferences are stored in `rtvc.toml`, searched in the working directory
and then beside the executable. The versioned dock layout is stored separately
as `rtvc-workspace.json`.

Full web stores small preferences in `localStorage`, recent media bytes in
IndexedDB, and the workspace under `rtvc_workspace_v1`. Lightweight WASM has no
egui workspace dependency.

## Debugger

### Integrated debugger

The dock debugger acts directly on `Emu` and is available in native and full
web. It provides run/pause/reset, instruction stepping, bounded run-to-IRQ,
mapped and raw-bank memory views, disassembly, breakpoints, ROM symbols, and
structured events.

### TCP debugger

Native GUI and headless modes expose newline-delimited JSON on localhost:

```bash
cargo run --bin rtvc -- --port 8089
cargo run --bin rtvc -- --headless --port 8080
```

| Command | Purpose |
| --- | --- |
| `status` | CPU registers, clock, run/HALT state |
| `stats` | rolling host-time FPS |
| `step` | execute one or more complete machine instructions |
| `continue`, `pause`, `reset` | execution control |
| `breakpoint_add`, `breakpoint_remove`, `breakpoint_list` | breakpoints |
| `read_memory` | mapped memory or raw `u0`-`u3`, `vid0`-`vid3`, `sys`, `cart`, `exth` |
| `disassemble`, `assemble` | Z80 developer tools |
| `save_snapshot`, `load_snapshot` | snapshot files |
| `save_screenshot` | 4:3 PNG |
| `key` | key down/up or typed character |
| `close_app` | normal application shutdown |

Requests and responses are one JSON object per line. A running emulator emits
`{"event":"breakpoint","pc":...}` asynchronously when a breakpoint is hit.

The interactive client is [scripts/rtvc_debug.py](../scripts/rtvc_debug.py).

## ROM Symbol Database

[data/rom_symbols_1_2.json](../data/rom_symbols_1_2.json) contains curated
BASIC 1.2 execution landmarks, callable routines, and data.

A CPU address alone is not a stable ROM identity because SYS and EXTH can
occupy overlapping CPU ranges. Consumers resolve both physical bank and offset.
The debugger annotates only when the relevant bank is currently mapped.

`usage` values are `trace`, `call`, and `data`. A `call` label is not an ABI
guarantee; software must still satisfy paging, register, BASIC state, and work
variable requirements. BASIC 2.2 needs a separately matched database rather
than a constant address shift.

## Configuration and Persistence

Machine choices combine standard/Plus memory, BASIC 1.2/2.2 ROMs, and optional
VT-DOS. Native and full-web applications retain machine type, video model,
fast-boot setting, and restorable media references.

The native application searches runtime ROM and program assets in the current
working directory first and beside the executable second. Packaged macOS apps
and extracted release archives therefore work without depending on the launch
directory.

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
