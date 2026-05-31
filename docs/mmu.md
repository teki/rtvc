# Memory Management Unit (MMU) Documentation

This document provides a language-independent architectural guide for building the Memory Management Unit (MMU) for a Videoton TV Computer (TVC) emulator. It is based on the logic in `jstvc` but abstracts away the JavaScript-specific details to serve as a reference for any implementation.

## Table of Contents

- [Overview](#overview)
- [Address Space and Paging](#address-space-and-paging)
- [Memory Banks](#memory-banks)
- [Main Paging Register (Memory Map)](#main-paging-register-memory-map)
- [Video Paging Register (TVC 64K+)](#video-paging-register-tvc-64k)
- [Read and Write Semantics](#read-and-write-semantics)
- [Implementation Strategy](#implementation-strategy)

---

## Overview

The TVC uses a Z80 CPU with a 64 KB (16-bit) address space. Because the system can be equipped with much more physical memory (RAM, Video RAM, System ROMs, and Cartridges), the memory is bank-switched. 

The 64 KB address space is divided into **four 16 KB pages**. Software maps different physical memory "banks" into these four pages by writing to specific hardware I/O ports.

---

## Address Space and Paging

The CPU sees memory from `0x0000` to `0xFFFF`, split into four fixed 16 KB windows:

| Page | Address Range     | Size  | Selection Controlled By |
|------|-------------------|-------|-------------------------|
| 0    | `0x0000 - 0x3FFF` | 16 KB | Main map port bits 3, 4 |
| 1    | `0x4000 - 0x7FFF` | 16 KB | Main map bit 2, Video map bits 0, 1 |
| 2    | `0x8000 - 0xBFFF` | 16 KB | Main map bit 5, Video map bits 2, 3 |
| 3    | `0xC000 - 0xFFFF` | 16 KB | Main map bits 6, 7      |

---

## Memory Banks

An emulator should allocate internal arrays or buffers for the following distinct memory blocks. Each block (except `EXTH`) is 16 KB (`0x4000` bytes).

### RAM Banks (Read/Write)
- **U0**: Base system RAM 0
- **U1**: Base system RAM 1
- **U2**: Base system RAM 2
- **U3**: Base system RAM 3 / Expansion RAM

### Video RAM Banks (Read/Write)
- **VID0**: Base Video RAM (present on all models)
- **VID1, VID2, VID3**: Expanded Video RAM (present only on TVC 64K+ / TVC 64K Plus models).

### ROM Banks (Read-Only)
- **SYS**: System ROM containing the OS and BASIC.
- **CART**: Cartridge ROM slot.

### External / Expansion (Special)
The TVC has an expansion system mapped to Page 3. It is split into two halves:
- **EXT (0xC000 - 0xDFFF)**: 8 KB. Usually an external expansion device (RAM or memory-mapped I/O) handled via external module callbacks.
- **EXTH (0xE000 - 0xFFFF)**: 8 KB. High extension ROM (e.g., DOS ROM).

---

## Main Paging Register (Memory Map)

The primary memory configuration is updated via an 8-bit port write (the mapping port). This determines which bank is visible in each of the four pages.

Let `M` be the 8-bit value written to the mapping port.

### Page 0 (`0x0000 - 0x3FFF`)
Controlled by bits 3 and 4 (`M & 0x18`):
- `0x00`: **SYS** (System ROM)
- `0x08`: **CART** (Cartridge ROM)
- `0x10`: **U0** (RAM)
- `0x18`: **U3** (RAM) on TVC 64K+, otherwise **U0**

### Page 1 (`0x4000 - 0x7FFF`)
Controlled by bit 2 (`M & 0x04`):
- `0x04` (TVC 64K+ only): Maps a **Video RAM** bank (see [Video Paging](#video-paging-register-tvc-64k)).
- `0x00` (or standard TVC): **U1** (RAM)

### Page 2 (`0x8000 - 0xBFFF`)
Controlled by bit 5 (`M & 0x20`):
- `0x20`: **U2** (RAM)
- `0x00`: Maps a **Video RAM** bank. On standard TVC, this is always **VID0**. On TVC 64K+, it is selected by the Video Paging Register.

### Page 3 (`0xC000 - 0xFFFF`)
Controlled by bits 6 and 7 (`M & 0xC0`):
- `0x00`: **CART** (Cartridge ROM)
- `0x40`: **SYS** (System ROM)
- `0x80`: **U3** (RAM)
- `0xC0`: **EXT** (Expansion space) — *Special handling applies here; see Read and Write Semantics.*

### Complete Map Summary

| Logical page | CPU range | Select bits | Value | 64K+ mapping | 32K/64K mapping |
|---|---:|---:|---:|---|---|
| Page 0 | `0x0000-0x3FFF` | `M & 0x18` | `0x00` | SYS ROM | SYS ROM |
| Page 0 | `0x0000-0x3FFF` | `M & 0x18` | `0x08` | CART ROM | CART ROM |
| Page 0 | `0x0000-0x3FFF` | `M & 0x18` | `0x10` | U0 RAM | U0 RAM |
| Page 0 | `0x0000-0x3FFF` | `M & 0x18` | `0x18` | U3 RAM | U0 RAM |
| Page 1 | `0x4000-0x7FFF` | `M & 0x04` | `0x00` | U1 RAM | U1 RAM |
| Page 1 | `0x4000-0x7FFF` | `M & 0x04` | `0x04` | Video RAM selected by `V & 0x03` | U1 RAM |
| Page 2 | `0x8000-0xBFFF` | `M & 0x20` | `0x00` | Video RAM selected by `V & 0x0C` | VID0 RAM |
| Page 2 | `0x8000-0xBFFF` | `M & 0x20` | `0x20` | U2 RAM | U2 RAM |
| Page 3 | `0xC000-0xFFFF` | `M & 0xC0` | `0x00` | CART ROM | CART ROM |
| Page 3 | `0xC000-0xFFFF` | `M & 0xC0` | `0x40` | SYS ROM | SYS ROM |
| Page 3 | `0xC000-0xFFFF` | `M & 0xC0` | `0x80` | U3 RAM | U3 RAM |
| Page 3 low half | `0xC000-0xDFFF` | `M & 0xC0` | `0xC0` | EXT card window | EXT card window |
| Page 3 high half | `0xE000-0xFFFF` | `M & 0xC0` | `0xC0` | EXTH ROM | EXTH ROM |

---

## Video Paging Register (TVC 64K+)

The TVC 64K+ features additional Video RAM banks. A secondary 8-bit port controls Video RAM mapping and which bank the CRTC (video chip) displays.

Let `V` be the 8-bit value written to the video mapping port. This only has an effect on TVC 64K+ models.

### Page 1 Video Bank
If Page 1 is configured to show Video RAM (`M & 0x04` is true), bits 0 and 1 of `V` (`V & 0x03`) select the bank:
- `0x00`: **VID0**
- `0x01`: **VID1**
- `0x02`: **VID2**
- `0x03`: **VID3**

### Page 2 Video Bank
If Page 2 is configured to show Video RAM (`M & 0x20` is false), bits 2 and 3 of `V` (`V & 0x0C`) select the bank:
- `0x00`: **VID0**
- `0x04`: **VID1**
- `0x08`: **VID2**
- `0x0C`: **VID3**

### Active Display Bank
Bits 4 and 5 of `V` (`V & 0x30`) dictate which Video RAM bank the CRTC reads to generate the screen image:
- `0x00`: **VID0**
- `0x10`: **VID1**
- `0x20`: **VID2**
- `0x30`: **VID3**

---

## Read and Write Semantics

The MMU must expose at least 8-bit read/write (`r8`, `w8`) and 16-bit read/write (`r16`, `w16`) functions to the CPU.

When the CPU accesses an `address` (0x0000 - 0xFFFF):
1. **Determine the Page:** `page_index = address >> 14` (results in 0, 1, 2, or 3).
2. **Determine the Local Offset:** `offset = address & 0x3FFF` (0x0000 to 0x3FFF).
3. **Lookup the Bank:** Use the current memory map configuration to find the active bank for `page_index`.

### Writes (`w8`)
- If the target bank is **RAM** (U0-U3 or VID0-VID3), write the value to `bank[offset]`.
- If the target bank is **ROM** (SYS or CART), ignore the write (ROM is read-only).
- **Special EXT handling:** If `page_index == 3` and the mapped bank is `EXT` (`M & 0xC0 == 0xC0`):
  - If `offset < 0x2000` (address `0xC000 - 0xDFFF`): Write to the external expansion module (if one is attached).
  - If `offset >= 0x2000` (address `0xE000 - 0xFFFF`): Ignore the write (this is the EXTH ROM).

### Reads (`r8`)
- If the target bank is a standard RAM or ROM block, return `bank[offset]`.
- **Special EXT handling:** If `page_index == 3` and the mapped bank is `EXT`:
  - If `offset < 0x2000` (address `0xC000 - 0xDFFF`): Return the value from the external expansion module (or `0xFF` if none attached).
  - If `offset >= 0x2000` (address `0xE000 - 0xFFFF`): Return `EXTH_ROM[offset - 0x2000]`.

### 16-bit Access (`r16`, `w16`)
The Z80 is little-endian.
- `r16(address)`: Return `r8(address) | (r8(address + 1) << 8)`.
- `w16(address, value)`: Call `w8(address, value & 0xFF)` then `w8(address + 1, value >> 8)`.
- `w16reverse(address, value)`: Some specific CPU instructions (like `EX (SP),HL`) require writing the high byte first, then the low byte. Call `w8(address + 1, value >> 8)` then `w8(address, value & 0xFF)`.

---

## Implementation Strategy

1. **Bank Pointers:** Instead of copying memory around, keep an array of 4 pointers/references representing the currently visible block for Pages 0, 1, 2, and 3.
2. **Update Map on Write:** When the CPU writes to the memory map or video map I/O ports, update these 4 pointers based on the logic described above.
3. **Hot Path:** `r8` and `w8` are called millions of times per second. They should be highly optimized: resolve `page_index = addr >> 14`, look up the pointer array, and access the underlying buffer.
4. **Initialization:** Initialize `_mapVal` / `_mapValVid` to a sentinel value (e.g., `-1` in JS, `0xFF` in Rust `u8`) that differs from the first real `setMap`/`setVidMap` call to avoid early-return guards blocking initial page configuration.
4. **EXT Exception:** The only branch in the hot path should be checking if the access is in Page 3 and if that page is mapped to EXT, to route calls to the expansion module or EXTH ROM.
