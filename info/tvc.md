# TVC Machine Emulator Core Documentation

This document provides a language-independent architectural guide for building and understanding the main machine orchestrator (`Tvc`) of the Videoton TV Computer emulator. It is based on the implementation in [src/tvc.rs](../src/tvc.rs).

## Table of Contents

- [Overview](#overview)
- [System Clock and Timings](#system-clock-and-timings)
- [Execution Loop and Core Step](#execution-loop-and-core-step)
- [I/O Port Mapping](#io-port-mapping)
- [Interrupt Controller and Priorities](#interrupt-controller-and-priorities)
- [Media Loader (.CAS and .DSK)](#media-loader-cas-and-dsk)
- [Debugger Console Commands](#debugger-console-commands)

---

## Overview

The `Tvc` type in [src/tvc.rs](../src/tvc.rs) acts as the system bus and hardware orchestrator. It instantiates the other primary modules—the CPU, MMU, Video (CRTC), Keyboard, and Audio subsystems—and handles communication between them:

- **CPU**: Z80 core ([z80.md](z80.md))
- **MMU**: Memory Management Unit ([mmu.md](mmu.md))
- **Video**: Motorola 6845 CRTC ([vid.md](vid.md))
- **Sound**: Sound generator / timer ([sound.md](sound.md))
- **Keyboard**: Row/Column scanner matrix
- **Extensions**: Expansion cards (such as the Floppy Controller)

All hardware I/O writes (`OUT`) and reads (`IN`) issued by the CPU are intercepted by the machine orchestrator and routed to the correct virtual ports.

---

## System Clock and Timings

A TVC emulator must maintain timing synchronization between components. The Z80 CPU clock frequency serves as the master system clock reference:

- **Master CPU Frequency**: 3,125,000 Hz (3.125 MHz).
- **Z80 Clock Cycle (T-state)**: Exactly **320 ns**.
- **Horizontal Sync Period (Scanline)**: 64 µs.
  - Calculated as: $3,125,000 \times 0.000064 = 200$ CPU clock cycles per scanline.
- **Vertical Sync Period (Frame)**: 20 ms (50 Hz).
  - Calculated as: $3,125,000 / 50 = 62,500$ CPU clock cycles per frame.

All components (Video, Sound, Interrupts) are updated in terms of CPU clock cycles elapsed since their last update.

---

## Execution Loop and Core Step

The emulation advances frame-by-frame using `run_for_a_frame()` (equivalent to the JS `runForAFrame`):

1. **CPU Run**: Executes CPU instructions via `step(0)` until `FRAME_CLOCKS` (62500) cycles have been consumed, checking breakpoints each step.
2. **Fast Frame Video**: When `VidModel::FastFrame` is active, runs the CPU for one screen-time budget and then calls `vid.draw_frame(vidmem, framebuffer)` to render a complete 608×288 frame from the current video state.
3. **Interleaved Video**: When `VidModel::Interleaved` is active, advances `vid.stream_some()` after each CPU instruction and consumes completed scan data through `vid.render_stream()`.
4. **Sound and Tape**: Advances cassette playback and the sound generator by the elapsed instruction cycles. Sound samples are generated as mono 44.1 kHz `f32` PCM and can be drained through `Tvc::take_audio_samples()`.
5. **Interrupt Handling**: In interleaved mode, a CRTC cursor match immediately latches the active-low cursor interrupt, calls `z80.irq()` if interrupts are enabled, and advances the CRTC by the IRQ service duration. This keeps the CPU and CRTC aligned for software that times drawing from the last-pixel screen interrupt.
6. **Presentation**: Sets `frame_complete = true` whenever a presentable framebuffer is ready for the UI. Interleaved mode does not wait indefinitely for CRTC sync; it keeps presenting the monitor surface while trying to relock, and only replaces it with a black lost-sync background with moving white stripes after several consecutive host ticks without a synchronized frame.

The native egui UI does not run one TVC frame for every host repaint. While the emulator is running, the UI requests continuous repaints and gates TVC frame generation from real time at 50 Hz. On displays refreshing faster than 50 Hz, host repaints reuse the latest texture until the next TVC frame is due. If generating a TVC frame takes too long, the UI drops the backlog and generates at most one new TVC frame per repaint callback. The FPS readout reports generated TVC frames only.

Native builds default to `VidModel::Interleaved` and expose the video model as a runtime setting. WASM builds default to `VidModel::FastFrame`. JavaScript can call `setVidModel()` with `fast-frame` or `interleaved`; legacy `simple` and `realistic` names are still accepted.

---

## I/O Port Mapping

The orchestrator maps the Z80 CPU I/O space. When the CPU executes an `IN` or `OUT` instruction, the orchestrator intercepts the port address (8-bit) and maps it as follows:

### Port Writes (`writePort` at Port `addr`)

| Port (Hex) | Module | Description |
|:---:|:---:|---|
| `0x00` | Video | Border color register (IGRB format) |
| `0x02` | MMU | Memory mapping register (maps RAM/ROM banks to pages) |
| `0x03` | Keyboard / Expansion | Bits 0-3: Selects the active keyboard scan row.<br>Bits 6-7: Cartridge expansion mapping (`_extCartMapping`). |
| `0x04` | Audio | Sound frequency generator low byte. |
| `0x05` | Audio / Tape | Bits 0-3: sound frequency high nibble.<br>Bit 4: routes the oscillator through the amplitude control.<br>Bit 5: sound interrupt enable flag.<br>Bits 6-7: Tape motor control outputs (`0` off, `1` on). |
| `0x06` | Multi-Port | Bits 0-1: Video display mode (2-color, 4-color, 16-color).<br>Bits 2-5: Sound amplitude / 4-bit DAC level.<br>Bit 7: Printer acknowledgment trigger. |
| `0x07` | Interrupt Controller | Acknowledges and clears the shared Cursor / Audio Interrupt. |
| `0x0C - 0x0F` | MMU | Video page mapping bank selector (for TVC 64K+ expandability). |
| `0x58` | Expansion card 0 | Write-enable / interrupt-enable configuration for Card Slot 0. |
| `0x59` | Expansion card 1 | Write-enable / interrupt-enable configuration for Card Slot 1. |
| `0x5A` | Expansion card 2 | Write-enable / interrupt-enable configuration for Card Slot 2. |
| `0x5B` | Expansion card 3 | Write-enable / interrupt-enable configuration for Card Slot 3. |
| `0x60 - 0x63` | Video | Sets Palette registers 0 to 3. |
| `0x70 - 0x7F` | Video (CRTC) | Mirrored MC6845 CRTC ports. Even addresses select the CRTC address register; odd addresses write the selected CRTC data register. |
| `0x10 - 0x1F` | Slot 0 Card | Direct pass-through of writes (Port offset `addr & 0x0F`) to Card 0 module. |
| `0x20 - 0x2F` | Slot 1 Card | Direct pass-through of writes (Port offset `addr & 0x0F`) to Card 1 module. |

### Port Reads (`readPort` at Port `addr`)

| Port (Hex) | Module | Description |
|:---:|:---:|---|
| `0x58` | Keyboard | Reads the column state of the currently selected keyboard scan row. |
| `0x59` / `0x5D` | Interrupt / System | Reads pending interrupts (bits 0-4) and system flags:<br>Bit 7: Printer ACK status.<br>Bit 6: Color / BW monitor selection flag.<br>Bit 5: Tape (cassette) input stream bit. |
| `0x5A` | Expansion Slots | Reads slot occupancy / card identifier codes. |
| `0x5B` / `0x5F` | Audio Timer | Resets/restarts the sound oscillator counter from the programmed divisor. |
| `0x70 - 0x7F` | Video (CRTC) | Mirrored MC6845 CRTC ports. Even address-register reads return `0xFF`; odd data-register reads follow CRTC register access permissions. |
| `0x10 - 0x1F` | Slot 0 Card | Direct pass-through read (offset `addr & 0x0F`) from Card 0 module. |
| `0x20 - 0x2F` | Slot 1 Card | Direct pass-through read (offset `addr & 0x0F`) from Card 1 module. |

---

## Interrupt Controller and Priorities

The TVC handles peripheral interrupts through a custom latch state stored in `_pendIt`. This status byte maps the interrupt request lines:

- **Bit 4**: Shared Cursor/Audio interrupt.
- **Bits 0-3**: Extension Slots 0 to 3 card interrupts.

### Interrupt Generation & Lifecycle
1. When the CRTC beam matches the cursor position, it triggers a Cursor Interrupt (setting bit 4 in `_pendIt` to `0` since it is active-low).
2. The sound oscillator can also trigger the same bit when port `0x05` bit 5 enables sound IT. The 12-bit divisor is written through ports `0x04` and `0x05`; reading `0x5B` or `0x5F` restarts the counter. The audible oscillator path uses `195312.5 / (4096 - n)` Hz for divisor `n`, with `0xFFF` stopping the oscillator.
3. The orchestrator checks if the interrupt is enabled. If Z80 interrupts are enabled (`irqEnabled()`), it halts execution and fires a Z80 interrupt service routine via `_z80.irq()`.
4. The CPU services the interrupt, using software state to distinguish cursor and sound timer requests because they share the same status bit.
5. The Z80 services write to Port `0x07`, which clears the shared interrupt flag, restoring bit 4 of `_pendIt` to `1` (idle).

---

## ROM Loading

The system ROM files are loaded at startup from the `roms/` directory:

| File | Target Bank | Description |
|:---|:---|---|
| `TVC12_D3.64K` | SYS (upper 8KB, offset 0x2000) | System ROM upper half |
| `TVC12_D4.64K` | SYS (lower 16KB) | System ROM lower half |
| `TVC12_D7.64K` | EXTH (8KB) | Extension ROM |

The `TvcMmu::add_rom()` method dispatches by filename matching the JS reference behavior.
Cartridge ROMs use `load_cart_rom()` which maps into the CART bank.

## IO Logging

`TvcBus` contains a `log: Log` field (ring buffer, 200 entries). Every port write (`OUT`) and port read (`IN`) is logged with the format:
- `OUT 0xXX <- 0xYY`
- `IN  0xXX -> 0xYY`

The UI exposes the log via a toggleable bottom panel with a "Clear" button.

---

## Media Loader (.CAS and .DSK)

The orchestrator supports loading cassette tape and floppy disk formats at runtime through the media-loading helpers in [src/tvc.rs](../src/tvc.rs), [src/emu.rs](../src/emu.rs), and [src/wasm.rs](../src/wasm.rs):

### 1. Cassette Tape (`.cas`)
- The TVC cassette image contains raw BASIC/binary data.
- **Loading mechanism**:
  - The MMU map is temporarily changed to `0xB0` (bringing RAM mapping into memory view).
  - The emulator skips the first 144 bytes of the `.cas` header.
  - The remaining bytes are written directly into RAM starting at target memory address `6639` (`0x19EF`).
  - The MMU map is restored to its original state.

### 2. Floppy Disk (`.dsk`)
- Floppy disk files contain physical sectors for TVC-DOS.
- **Loading mechanism**:
  - Checks if a floppy controller expansion card is attached to Card Slot 0 (`_ext0`).
  - Passes the file payload to `_ext0.loadDisk(name, data)` to simulate insertion of the floppy disk into the virtual drive.

### 3. Archive (`.zip`)
- Automatically decompresses `.zip` files containing `.cas` or `.dsk` images and routes the extraction payload recursively.

---

## Debugger Console Commands

The orchestrator exposes a powerful suite of console commands for debugging from the developer console:

- **Breakpoints**:
  - `db(addr)` / `dd(addr)`: Add or delete an execution breakpoint on the PC.
  - `dbm(addr)` / `ddm(addr)`: Add or delete a memory read/write breakpoint.
- **Dumping State**:
  - `dreg()`: Dumps CPU registers, disassembly at PC, stack context, and MMU status.
  - `dmem(addr, lines, bytesPerLine)`: Hex and ASCII dumps of memory at target address.
- **Execution**:
  - `dstep(breakOnNext)`: Single-steps the CPU instruction and prints the disassembled output, executing until the target instruction completes.
  - `dasm(addr, length)`: Disassembles a block of memory starting at `addr`.
