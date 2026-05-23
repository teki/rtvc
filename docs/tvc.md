# TVC Machine Emulator Core Documentation

This document provides a language-independent architectural guide for building and understanding the main machine orchestrator class (`TVC`) of the Videoton TV Computer emulator. It is based on the implementation in [src/tvc.js](file:///Users/teki/dev/jstvc/src/tvc.js).

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

The [TVC](file:///Users/teki/dev/jstvc/src/tvc.js#L13) class acts as the system bus and hardware orchestrator. It instantiates the other primary modules—the CPU, MMU, Video (CRTC), Keyboard, and Audio subsystems—and handles communication between them:

- **CPU**: Z80 core ([z80.md](file:///Users/teki/dev/jstvc/docs/z80.md))
- **MMU**: Memory Management Unit ([mmu.md](file:///Users/teki/dev/jstvc/docs/mmu.md))
- **Video**: Motorola 6845 CRTC ([vid.md](file:///Users/teki/dev/jstvc/docs/vid.md))
- **Sound**: Sound generator / timer
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

The emulation advances frame-by-frame using the [runForAFrame](file:///Users/teki/dev/jstvc/src/tvc.js#L111) loop:

1. **CPU Step**: Executes one CPU instruction via `step(0)` and returns the cycles consumed (`cpuTime`).
2. **Breakpoint Check**: Checks if the current Program Counter (`PC`) matches a debug breakpoint.
3. **Video Advancement**: Call [streamSome(cpuTime)](file:///Users/teki/dev/jstvc/src/vid.js#L207) to advance the CRTC beam counters by the same cycle duration.
4. **Interrupt Handling**:
   - If the Video Controller signals a cursor interrupt trigger and Z80 interrupts are enabled (`irqEnabled()`), it asserts an IRQ (`_z80.irq()`).
   - The pending interrupt flag (`_pendIt` bit 4) is cleared.
   - The video stream is advanced by the interrupt duration.
5. **Frame Termination**: Check `_vid.renderStream()`. If a full screen frame has successfully been rendered, the framebuffer is refreshed via `_fb.refresh()` and the loop exits.
6. **Emulation Speed Limit**: The loop limits execution time so it does not exceed 2x the standard frame clocks in a single run (to prevent emulation runaway).

---

## I/O Port Mapping

The orchestrator maps the Z80 CPU I/O space. When the CPU executes an `IN` or `OUT` instruction, the orchestrator intercepts the port address (8-bit) and maps it as follows:

### Port Writes (`writePort` at Port `addr`)

| Port (Hex) | Module | Description |
|:---:|:---:|---|
| `0x00` | Video | Border color register (IGRB format) |
| `0x02` | MMU | Memory mapping register (maps RAM/ROM banks to pages) |
| `0x03` | Keyboard / Expansion | Bits 0-3: Selects the active keyboard scan row.<br>Bits 6-7: Cartridge expansion mapping (`_extCartMapping`). |
| `0x04` | Audio | Sound frequency generator (Low byte). |
| `0x05` | Audio | Bits 0-3: Sound frequency (High byte).<br>Bit 4: Sound Output enable switch.<br>Bit 5: Sound interrupt enable/disable flag. |
| `0x06` | Multi-Port | Bits 0-1: Video display mode (2-color, 4-color, 16-color).<br>Bits 2-5: Sound amplitude level.<br>Bit 7: Printer acknowledgment trigger. |
| `0x07` | Interrupt Controller | Acknowledges and clears the Cursor / Audio Interrupt. |
| `0x0C - 0x0F` | MMU | Video page mapping bank selector (for TVC 64K+ expandability). |
| `0x58` | Expansion card 0 | Write-enable / interrupt-enable configuration for Card Slot 0. |
| `0x59` | Expansion card 1 | Write-enable / interrupt-enable configuration for Card Slot 1. |
| `0x5A` | Expansion card 2 | Write-enable / interrupt-enable configuration for Card Slot 2. |
| `0x5B` | Expansion card 3 | Write-enable / interrupt-enable configuration for Card Slot 3. |
| `0x60 - 0x63` | Video | Sets Palette registers 0 to 3. |
| `0x70` | Video (CRTC) | Programs the MC6845 CRTC Address Index pointer. |
| `0x71` | Video (CRTC) | Programs the selected MC6845 CRTC Register Data. |
| `0x10 - 0x1F` | Slot 0 Card | Direct pass-through of writes (Port offset `addr & 0x0F`) to Card 0 module. |
| `0x20 - 0x2F` | Slot 1 Card | Direct pass-through of writes (Port offset `addr & 0x0F`) to Card 1 module. |

### Port Reads (`readPort` at Port `addr`)

| Port (Hex) | Module | Description |
|:---:|:---:|---|
| `0x58` | Keyboard | Reads the column state of the currently selected keyboard scan row. |
| `0x59` | Interrupt / System | Reads pending interrupts (bits 0-4) and system flags:<br>Bit 7: Printer ACK status.<br>Bit 6: Color / BW monitor selection flag.<br>Bit 5: Tape (cassette) input stream bit. |
| `0x5A` | Expansion Slots | Reads slot occupancy / card identifier codes. |
| `0x10 - 0x1F` | Slot 0 Card | Direct pass-through read (offset `addr & 0x0F`) from Card 0 module. |
| `0x20 - 0x2F` | Slot 1 Card | Direct pass-through read (offset `addr & 0x0F`) from Card 1 module. |

---

## Interrupt Controller and Priorities

The TVC handles peripheral interrupts through a custom latch state stored in `_pendIt`. This status byte maps the interrupt request lines:

- **Bit 4**: Cursor/Audio interrupt.
- **Bits 0-3**: Extension Slots 0 to 3 card interrupts.

### Interrupt Generation & Lifecycle
1. When the CRTC beam matches the cursor position, it triggers a Cursor Interrupt (setting bit 4 in `_pendIt` to `0` since it is active-low).
2. The orchestrator checks if the interrupt is enabled. If Z80 interrupts are enabled (`irqEnabled()`), it halts execution and fires a Z80 interrupt service routine via `_z80.irq()`.
3. The CPU services the interrupt, performing its routine (keyboard scanning, system timers, cassette sound).
4. The Z80 services write to Port `0x07`, which clears the interrupt flag, restoring bit 4 of `_pendIt` to `1` (idle).

---

## Media Loader (.CAS and .DSK)

The orchestrator supports loading cassette tape and floppy disk formats at runtime via the [loadImg](file:///Users/teki/dev/jstvc/src/tvc.js#L86) function:

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
