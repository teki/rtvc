# High Boot Floppy (HBF) Card & FD1793 Controller Documentation

This document provides a language-independent architectural guide for building and understanding the Floppy Disk Controller (FDC) expansion card for the Videoton TV Computer (TVC) emulator. It covers the layout of the **HBF Expansion Card** and the emulation of the **Western Digital FD1793 (WD1793) FDC chip**.

This documentation is based on the implementations in [src/hbf.js](file:///Users/teki/dev/jstvc/src/hbf.js) and [src/fd1793.js](file:///Users/teki/dev/jstvc/src/fd1793.js).

## Table of Contents

- [Overview](#overview)
- [HBF Card Memory Mapping](#hbf-card-memory-mapping)
- [HBF Card I/O Registers](#hbf-card-io-registers)
- [FD1793 Floppy Disk Controller Emulation](#fd1793-floppy-disk-controller-emulation)
- [Disk Image Structure (FDisk)](#disk-image-structure-fdisk)

---

## Overview

The High Boot Floppy (HBF) card is a hardware expansion cartridge (usually mounted in Card Slot 0) that provides floppy disk boot support for the TVC. The card contains:
1. **A 16 KB Boot ROM** containing TVC-DOS.
2. **4 KB of private RAM** used by TVC-DOS for buffer workspace.
3. **A Western Digital FD1793 Floppy Disk Controller** chip to interface with up to four 5.25" or 3.5" disk drives.

The machine orchestrator mounts the card into the memory and I/O bus, passing read/write requests to the HBF cartridge when the expansion segment is active.

---

## HBF Card Memory Mapping

When the main CPU memory mapper maps the **EXT (Expansion)** bank to Page 3 (`0xC000 - 0xFFFF`), the HBF card responds to memory accesses within the lower 8 KB of the page range (`0xC000 - 0xDFFF` / relative address `0x0000 - 0x1FFF`). 

The HBF maps its ROM and RAM within this 8 KB space as follows:

```text
EXT Space (0xC000 - 0xDFFF):
+-----------------------------------+-----------------------------------+
|   4 KB ROM Page (0xC000-0xCFFF)    |   4 KB Private RAM (0xD000-0xDFFF)|
|     (Selected from _rom0.._rom3)   |                                   |
+-----------------------------------+-----------------------------------+
 0x0000                              0x1000                              0x1FFF (Relative)
```

### 1. Active ROM Page Selection (0x0000–0x0FFF)
The 16 KB floppy ROM is divided into four **4 KB pages** (`ROM0`, `ROM1`, `ROM2`, `ROM3`). Accessing memory below `0x1000` reads the currently active page. Writes to this range are ignored.

The active page is selected by writing to the **HBF ROM Page Register** (Port 8).

### 2. Private RAM (0x1000–0x1FFF)
The HBF card's local 4 KB RAM buffer is mapped directly to the upper half of the cartridge range. Reads and writes are both fully enabled in this zone.

---

## HBF Card I/O Registers

When the CPU accesses expansion card ports mapped to Slot 0 (`0x10 - 0x1F`) or Slot 1 (`0x20 - 0x2F`), the machine orchestrator forwards the accesses to [HBF](file:///Users/teki/dev/jstvc/src/hbf.js#L11). The port number is masked as `port_number & 0x0F` (offsets 0–15) and handled as follows:

| Port Offset | Read / Write | Target | Description / Behavior |
|:---:|:---:|:---:|---|
| **0** | Read | FDC | **FDC Status Register** (Clears INTRQ) |
| **0** | Write | FDC | **FDC Command Register** (Starts floppy execution) |
| **1** | R/W | FDC | **FDC Track Register** (Current physical track) |
| **2** | R/W | FDC | **FDC Sector Register** (Target sector to read/write) |
| **3** | R/W | FDC | **FDC Data Register** (Transfer buffer; reading clears DRQ) |
| **4** | Read | FDC | **Fast Status Register**: Returns `INTRQ` in bit 0 and `DRQ` in bit 7. |
| **4** | Write | FDC | **FDC Parameter Register**: Selects active drive and side (see below). |
| **8** | Write | HBF Card | **ROM Page Selector**: Bits 4-5 select active 4 KB ROM bank:<br>`0x00` $\rightarrow$ ROM0, `0x10` $\rightarrow$ ROM1, `0x20` $\rightarrow$ ROM2, `0x30` $\rightarrow$ ROM3. |

### FDC Parameter Register Bit Layout (Port 4 Write)

```text
Bit: [  7   ]  [  6   ]  [  5   ]  [  4   ]  [  3   ]  [  2   ]  [  1   ]  [  0   ]
     [ Side ]  [ MON  ]  [ DDEN ]  [ HLD  ]  [ DS3  ]  [ DS2  ]  [ DS1  ]  [ DS0  ]
```
- **Side (bit 7)**: Selects side/head. `0` = Side 0 (bottom), `1` = Side 1 (top).
- **MON (bit 6)**: Motor On control.
- **DDEN (bit 5)**: Double Density mode enable.
- **HLD (bit 4)**: Head Load delay state.
- **DS0–DS3 (bits 0–3)**: Active Drive Select. Sets which virtual floppy drive is active (0 to 3). If no drive bit is set, it defaults to Drive 0.

---

## FD1793 Floppy Disk Controller Emulation

The [FD1793](file:///Users/teki/dev/jstvc/src/fd1793.js#L240) class emulates the Western Digital FD1793 FDC. It maintains internal register states (`_status`, `_track`, `_sector`, `_data`, `_intrq`) and drives the active disk's state machine.

### FDC Registers and States

- **Status Register (`_status`)**: Returns FDC execution state flags:
  - `0x80` (`ST_NOTREADY`): Active drive is empty or motor is spun down.
  - `0x40` (`ST_READONLY` / `ST_WRFAULT`): Write protection or write fault.
  - `0x20` (`ST_HEADLOADED` / `ST_RECTYPE`): Head loaded / Record Type.
  - `0x10` (`ST_SEEKERR` / `ST_RECNF`): Seek error / Record Not Found.
  - `0x08` (`ST_CRCERR`): Data CRC mismatch.
  - `0x04` (`ST_TRACK0` / `ST_LOSTDATA`): Head is at track 0 / Data overrun.
  - `0x02` (`ST_INDEX` / `ST_DRQ`): Index pulse detected / Data Request active.
  - `0x01` (`ST_BUSY`): Controller is actively executing a command.
- **Fast status read (Port 4)**: Emulates a quick polling path for Z80 assembly to verify data ready (`DRQ` in bit 7, `INTRQ` in bit 0) without triggering the standard status read register clear.

### Supported Commands (`command(val)`)

- **Restore (`0x00`)**: Moves the drive head to Track 0. Resets track register to 0. Sets `INTRQ` flag.
- **Seek (`0x01`)**: Seeks the track specified in the data register (`_data`). Updates the track register and sets `INTRQ`.
- **Read Sector (`0x08 / 0x09`)**: Reads a sector into the controller. Sets `ST_BUSY` and fetches the first byte into `_data`, raising the `DRQ` flag.
- **Read Address (`0x0C`)**: Reads the next sector's physical ID address field (6 bytes: Track, Side, Sector, Size code, CRC1, CRC2) into the data register.

---

## Disk Image Structure (FDisk)

The [FDisk](file:///Users/teki/dev/jstvc/src/fd1793.js#L70) class simulates the physical disk medium. It loads raw MS-DOS compatible `.dsk` sector dumps.

### BIOS Parameter Block (BPB) Parsing
When a disk image is inserted via `loadDsk`, the emulator parses the FAT12 boot sector to determine the physical geometry of the medium:
- **Sector Size**: Read from offset 11–12 (typically 512 bytes).
- **Sectors per Track**: Read from offset 24–25 (typically 9).
- **Heads**: Read from offset 26–27 (typically 1 or 2).
- **Total Sectors**: Read from offset 19–20.

### Address Translation (Seek Seek Seek)
To read a sector, the head's byte offset inside the flat `.dsk` buffer is calculated as:

$$\text{Byte Offset} = \left(\text{Track} \times (\text{Sectors/Track} \times \text{Heads}) + (\text{Sectors/Track} \times \text{Side}) + (\text{Sector} - 1)\right) \times \text{Sector Size}$$
