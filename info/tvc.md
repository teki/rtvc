# TVC Technical Reference

Videoton TV Computer hardware information for emulator authors and low-level
software developers.

This is an implementation-neutral description of the TVC machine. It documents
the memory and I/O maps, timing relationships, interrupt wiring, video data
path, keyboard matrix, sound/tape hardware, and expansion devices. It does not
repeat the generic Z80, MC6845, or FD1793 specifications; consult the original
component documentation for behavior not changed by the TVC wiring.

The companion [rtvc Implementation Reference](rtvc.md) describes how this
repository models the machine, including renderer choices, snapshots, debugger
interfaces, UI behavior, and build targets.

## Contents

**Overview**

- [Technical data](#technical-data)
- [Machine variants](#machine-variants)
- [Clock and timing](#clock-and-timing)
- [Reset state](#reset-state)

**Hardware programming**

- [CPU and bus conventions](#cpu-and-bus-conventions)
- [Memory map](#memory-map)
- [I/O map](#io-map)
- [Interrupt system](#interrupt-system)
- [Video system](#video-system)
- [Keyboard](#keyboard)
- [Sound and timer](#sound-and-timer)
- [Cassette interface](#cassette-interface)
- [Expansion system](#expansion-system)
- [HBF floppy interface](#hbf-floppy-interface)

**Media and firmware**

- [CAS cassette representation](#cas-cassette-representation)
- [Raw DSK images](#raw-dsk-images)
- [System ROM placement](#system-rom-placement)

**Implementation notes**

- [Minimum emulator model](#minimum-emulator-model)
- [Accuracy-sensitive behavior](#accuracy-sensitive-behavior)
- [External component references](#external-component-references)

---

## Technical Data

| Item | TVC value |
| --- | --- |
| CPU | Zilog Z80-compatible CPU |
| CPU clock | 3,125,000 Hz |
| CPU T-state | 320 ns |
| CPU address space | 64 KiB, divided into four 16 KiB paging windows |
| CPU I/O space | TVC devices use the low 8-bit port address |
| Base RAM | Four logical 16 KiB RAM banks, U0-U3 |
| Video RAM | One 16 KiB bank on standard machines; four banks on TVC 64K+ |
| System ROM window | 16 KiB SYS bank |
| Cartridge ROM window | 16 KiB CART bank |
| Extension ROM | 8 KiB EXTH bank at the high half of the EXT mapping |
| Video controller | MC6845-compatible CRTC used as a timing/address generator |
| CRTC character clock | 1,562,500 Hz, exactly one clock per two CPU T-states |
| Normal line period | 64 us, 200 CPU T-states |
| Normal frame rate | 50 Hz |
| Nominal frame budget | 62,500 CPU T-states |
| Normal active picture | 512 x 240 source pixels |
| TVC color space | 16 colors encoded as intensity, green, red, blue |
| Sound | One programmable divider/counter channel plus 4-bit amplitude/DAC |
| Keyboard | 10 rows x 8 columns, active-low (rows 8-9 carry joysticks) |
| Cassette | Software-decoded input and software-toggled output |
| Expansion | Four I/O slots and one selected 8 KiB memory window |

The Z80 and MC6845 are ordinary components from the CPU's point of view. The
TVC-specific behavior lies in the memory mapper, port decode, CRTC-to-VRAM
address wiring, pixel serializer, palette, and interrupt connections.

## Machine Variants

The memory mapper supports two relevant hardware classes:

| Variant | Main RAM behavior | Video RAM behavior |
| --- | --- | --- |
| TVC 64K | U0-U3 are available; page 0 selection `11b` aliases U0 | One 16 KiB video bank, VID0 |
| TVC 64K+ | Page 0 selection `11b` maps U3 | Four independently selectable 16 KiB video banks, VID0-VID3 |

ROM revisions such as BASIC 1.2 and BASIC 2.2 change firmware, not the core bus
architecture described here. VT-DOS systems add an HBF expansion card and DOS
ROM rather than changing the base machine.

## Clock and Timing

The 3.125 MHz CPU clock is the useful master time base for emulation:

```text
CPU T-state          1 / 3,125,000 s = 320 ns
CRTC character clock 1,562,500 Hz    = 2 CPU T-states
normal scanline      64 us            = 200 CPU T-states
normal 50 Hz frame   20 ms            = 62,500 CPU T-states
```

With the normal firmware CRTC programming, a line contains 100 character
clocks. A character clock fetches one byte and serializes it into eight output
pixel periods. The standard active area is 64 characters by 60 character rows,
with four raster lines per row: 512 by 240 source pixels.

Software can reprogram the CRTC. An accurate emulator must therefore derive
display timing from CRTC state rather than assuming that every program keeps
the firmware defaults. The 62,500-cycle value is still a convenient host
scheduling quantum, not a substitute for advancing the CRTC.

## Reset State

At hardware reset:

- the Z80 begins from its normal reset state;
- the main paging register is effectively `0x00`;
- the video paging register is effectively `0x00`;
- page 0 contains SYS ROM;
- page 1 contains U1 RAM;
- page 2 contains VID0;
- page 3 contains CART ROM;
- the selected expansion memory slot is slot 0;
- pending interrupt inputs are inactive;
- sound, tape motor, keyboard state, and CRTC state return to their reset
  conditions.

Firmware programs the CRTC, palette, paging, and interrupt timing during boot.
Power-on RAM contents are not specified here and should not be relied upon by
software.

## CPU and Bus Conventions

### Z80 use

The TVC does not modify the Z80 instruction set. CPU emulators should implement
normal Z80 interrupt, HALT, refresh, and I/O semantics. This reference only
describes the devices visible through memory and I/O.

### I/O cycle timing

Devices see a port access during the I/O machine cycle, not at the end of the
instruction. The Z80 I/O cycle starts after the instruction's opcode and
operand cycles (T7 for `OUT (n),A` and `IN A,(n)`, T8 for `IN r,(C)` and
`OUT (C),r`, T9 for `INI`-type and T12 for `OUTI`-type instructions; add four
T-states per index prefix), and the device latches or drives the data bus
about three T-states into that cycle: the I/O cycle is T1, T2, an automatic
wait state TW, and T3. The Zilog manual's cycle list for `INI`/`INIR`
(4, 5, 3, 4) is inconsistent with its own description; the port read comes
first, as (4, 5, 4, 3), which the FUSE vectors confirm. A border, palette, or sound-timer access
therefore takes effect 10-15 T-states into an 11-16 T-state instruction, which
in raster-timed effects is a shift of five to seven character positions.

### Video RAM wait states

Video RAM is time-multiplexed between the CPU and the CRTC. The DEB signal
(CCLK, the character clock) gives the CPU the first 320 ns of every 640 ns
character clock and the CRTC the second; each access cycle latches the
addressed byte into a shared output register at the end of its half. A CPU
cycle that addresses a video bank therefore has to read the register after the
end of a CPU half that it was able to address, and before the following CRTC
half overwrites it.

The TVC does not use the Z80 `/WAIT` input for this: `/WAIT` can only insert
whole T-states. Instead a clock-stretch circuit stops the CPU clock for one to
four 160 ns half T-states:

- in a memory read or write, the high phase of T3 is held until the falling
  edge of T3, where the Z80 samples read data, follows a DEB 1-to-0 edge;
- in an opcode fetch (M1), the low phase of T2 is held until the rising edge
  of T3, where the Z80 samples the opcode, follows a DEB 1-to-0 edge. The M1
  path needs half a clock less setup.

The stretch depends only on where T1 of the cycle falls relative to DEB. With
T1 starting 0, 1, 2, or 3 half T-states after DEB rises:

| T1 offset from DEB rising | 0 | 1 | 2 | 3 |
| --- | --- | --- | --- | --- |
| memory read or write stretch | 2 | 1 | 4 | 3 |
| opcode fetch stretch | 3 | 2 | 1 | 4 |

Counting the CPU clock in half T-states (`h`, taken at T1 of the memory cycle,
with `h mod 4 == 0` where DEB falls, at the character clock boundary), this is:

```text
memory read or write   4 - (h & 3)        half T-states
opcode fetch (M1)      4 - ((h + 3) & 3)  half T-states
```

Every video RAM access costs between half a T-state and two T-states, and code
running from video RAM settles at six T-states per `NOP`. Other memory and I/O
cycles are not stretched. Source: Benedek–Monok, *TV-Computer hardver* (1988),
section 2.2.4 and figures 18–22 (clock-stretch circuit and its timing
diagrams); ep128emu implements the same rule.

### Opcode fetch wait state on 32K and 64K boards

The HBA and HBA-1 main boards (32K and 64K machines) also have a real `/WAIT`
circuit. A flip-flop arms for one CPU clock on every M1, and `/WAIT` is
asserted (one whole wait state) only if a slow source requests it at the same
time: the cartridge (`FAST`), an expansion card (`NSLOWEXP`), or the system ROM
when board jumper TB1 (16 KiB part) or TB9 (8 KiB extension) is fitted. The
HBA-2 (64K+) board has no such circuit. The jumper fitting of shipped machines
is not documented here, so the effect on normal ROM execution is unknown.

### Port decode

TVC peripheral descriptions use an 8-bit port number. For a Z80 implementation
that exposes the full 16-bit I/O address, route devices from the low byte unless
a particular expansion card documents additional decoding.

### Byte order

The Z80 is little-endian. A 16-bit word at address `A` stores the low byte at
`A` and the high byte at `A+1`.

## Memory Map

### CPU paging windows

The 64 KiB CPU address space is divided into four fixed 16 KiB windows:

| Page | CPU range | Main paging controls |
| --- | --- | --- |
| 0 | `0x0000-0x3FFF` | bits 3-4 |
| 1 | `0x4000-0x7FFF` | bit 2, plus video-map bits 0-1 |
| 2 | `0x8000-0xBFFF` | bit 5, plus video-map bits 2-3 |
| 3 | `0xC000-0xFFFF` | bits 6-7 |

Port `0x02` is the main paging register. Let its value be `M`.

### Main paging register, port `0x02`

| CPU window | Selection | Mapping |
| --- | --- | --- |
| Page 0 | `M & 0x18 == 0x00` | SYS ROM |
| Page 0 | `M & 0x18 == 0x08` | CART ROM |
| Page 0 | `M & 0x18 == 0x10` | U0 RAM |
| Page 0 | `M & 0x18 == 0x18` | U3 on 64K+, U0 on standard TVC |
| Page 1 | `M & 0x04 == 0x00` | U1 RAM |
| Page 1 | `M & 0x04 == 0x04` | selected video RAM on 64K+; U1 otherwise |
| Page 2 | `M & 0x20 == 0x00` | selected video RAM; VID0 on standard TVC |
| Page 2 | `M & 0x20 == 0x20` | U2 RAM |
| Page 3 | `M & 0xC0 == 0x00` | CART ROM |
| Page 3 | `M & 0xC0 == 0x40` | SYS ROM |
| Page 3 | `M & 0xC0 == 0x80` | U3 RAM |
| Page 3 | `M & 0xC0 == 0xC0` | EXT/EXTH split mapping |

ROM writes have no effect. RAM and video RAM are read/write.

### EXT/EXTH split mapping

When page 3 selects EXT:

```text
0xC000-0xDFFF  selected expansion card memory window, 8 KiB
0xE000-0xFFFF  EXTH ROM, 8 KiB
```

The active card for the low 8 KiB window is selected by port `0x03` bits 6-7.
An absent card normally reads as `0xFF`; writes are ignored.

### Video paging register, ports `0x0C-0x0F`

The four ports are mirrors. Let the written value be `V`. The register affects
TVC 64K+ machines only.

| Use | Bits | Selection |
| --- | --- | --- |
| Video RAM mapped into page 1 | `V & 0x03` | `0`: VID0, `1`: VID1, `2`: VID2, `3`: VID3 |
| Video RAM mapped into page 2 | `(V >> 2) & 0x03` | `0`: VID0, `1`: VID1, `2`: VID2, `3`: VID3 |
| Video RAM displayed by the video circuit | `(V >> 4) & 0x03` | `0`: VID0, `1`: VID1, `2`: VID2, `3`: VID3 |

CPU-visible video memory and displayed video memory are selected independently.
This allows drawing into one bank while another bank is being scanned out.

## I/O Map

### Summary

| Port | Read | Write |
| --- | --- | --- |
| `0x00` | unspecified | border color |
| `0x02` | unspecified | main memory paging |
| `0x03` | unspecified | keyboard row and expansion memory slot |
| `0x04` | unspecified | sound divisor bits 0-7 |
| `0x05` | unspecified | sound divisor bits 8-11, sound controls, cassette motors |
| `0x06` | unspecified | video mode, sound amplitude, printer strobe/ack control |
| `0x07` | unspecified | clear shared cursor/sound interrupt |
| `0x0C-0x0F` | unspecified | 64K+ video paging |
| `0x10-0x1F` | slot 0 registers | slot 0 registers |
| `0x20-0x2F` | slot 1 registers | slot 1 registers |
| `0x30-0x3F` | slot 2 registers | slot 2 registers |
| `0x40-0x4F` | slot 3 registers | slot 3 registers |
| `0x50-0x57` | toggle cassette output, return bus value | toggle cassette output |
| `0x58` | selected keyboard row | slot 0 interrupt enable/clear (bit 7) |
| `0x59`, `0x5D` | interrupt and system status | slot 1 interrupt enable/clear (bit 7) |
| `0x5A`, `0x5E` | expansion type identifiers | slot 2 interrupt enable/clear (bit 7) |
| `0x5B`, `0x5F` | restart sound divider/counter | slot 3 interrupt enable/clear (bit 7) |
| `0x60-0x63` | unspecified | palette registers 0-3 |
| `0x70-0x7F` | mirrored CRTC access | mirrored CRTC access |

The slot interrupt controls are described under
[Interrupts and identification](#interrupts-and-identification). Other bits of
those writes are not characterized here; preserve them as unknown.

### Port `0x03`

```text
bits 0-3  keyboard matrix row, 0-9
bits 4-5  not described here
bits 6-7  expansion card selected for the EXT memory window
```

The expansion I/O ranges are fixed per slot and are not affected by bits 6-7.

### Status ports `0x59` and `0x5D`

```text
bit 7  printer ACK/status
bit 6  color/monochrome monitor status
bit 5  cassette input level (ECIN)
bit 4  shared cursor/sound interrupt, active low
bits 0-3 expansion slot interrupt inputs, active low
```

The two addresses are mirrors.

### Expansion identification, `0x5A` and `0x5E`

The status byte contains a two-bit card type for each slot:

```text
bits 1-0  slot 0
bits 3-2  slot 1
bits 5-4  slot 2
bits 7-6  slot 3
```

The identifier lines are wired on each card:

| Bits | Card class | OS name |
| --- | --- | --- |
| `00` | serial (RS232) | `RS232` |
| `01` | video game module | `VGB` |
| `10` | floppy interface | `DISK` |
| `11` | empty slot, or a card that identifies itself through a `MOPS` ROM header | from the header |

An unoccupied slot reports type `3` (`11b`). The HBF card reports type `2`
(`10b`). See [Operating system card discovery](#operating-system-card-discovery).

## Interrupt System

The status bits in port `0x59`/`0x5D` are active low:

| Bit | Source |
| --- | --- |
| 0 | expansion slot 0 |
| 1 | expansion slot 1 |
| 2 | expansion slot 2 |
| 3 | expansion slot 3 |
| 4 | shared CRTC cursor or sound timer |

The CRTC cursor output and sound timer share bit 4 and the CPU interrupt input.
Software must use its own device state and timing expectations to determine the
source. The request is latched on the falling edge of the cursor pulse or of
the enabled sound counter's bit 3: a one-character cursor raises it one
character clock after the cursor character, not when the cursor starts.

The usual frame interrupt is not generated by the CRTC VSYNC output. Firmware
programs the CRTC cursor address and cursor raster line so that CURSOR becomes
active at the final byte of the visible picture. That pulse requests the shared
interrupt at approximately 50 Hz.

Writing any value to port `0x07` acknowledges and clears the shared bit-4
request. Expansion cards have card-specific acknowledgement rules.

An emulator must keep the request latched until acknowledged. It should present
the request to the Z80 according to normal Z80 interrupt-enable and interrupt
mode behavior; a request is not lost merely because interrupts are temporarily
disabled.

## Video System

### TVC use of the MC6845

The MC6845 supplies character/raster counters, memory addresses, display enable,
sync, and cursor timing. It does not define TVC pixel formats or colors. TVC
logic:

1. transforms the CRTC memory and raster address into a 14-bit VRAM address;
2. reads one byte from the selected display VRAM bank;
3. serializes that byte according to the TVC video mode;
4. maps pixel values through four palette registers or direct 16-color bits;
5. uses the CRTC CURSOR output as an interrupt source.

Generic MC6845 counter and register behavior should come from a component
reference. The details below are the TVC-specific connection.

### CRTC ports

Only address line A0 reaches the CRTC, so the pair is mirrored:

```text
0x70, 0x72, ... 0x7E  CRTC address register
0x71, 0x73, ... 0x7F  selected CRTC data register
```

The TVC hardware documentation permits CPU reads of the start-address pair
R12-R13. R14-R15 are readable/writable cursor address registers; R16-R17 are
read-only light-pen latches. Reads of write-only registers should return an
open-bus value such as `0xFF`. High address-register values are not valid CRTC
register selections.

### Normal firmware CRTC programming

The operating system initializes the MC6845 through ports `0x70` and `0x71`.
The following values explain the normal TVC display geometry and cursor
interrupt setup. They are useful as a reference trace, not immutable hardware
constants.

| Register | TVC value | MC6845 meaning | TVC use/comment |
| --- | ---: | --- | --- |
| R0 | 99 | Horizontal total, in character clocks, minus one | 100 character clocks per line |
| R1 | 64 | Horizontal displayed character count | 64 displayed bytes, 512 source pixels |
| R2 | 75 | Horizontal sync position | Places TVC horizontal timing relative to the displayed area |
| R3 | `0x32` | Sync width register | TVC uses the horizontal field for blanking/sync timing; vertical sync width is not taken directly from the CRTC |
| R4 | 77 | Vertical total, in character rows, minus one | 78 character rows per generated frame before vertical adjust |
| R5 | 2 | Vertical total adjust, in scanlines | Adds 2 scanlines to the frame |
| R6 | 60 | Vertical displayed character-row count | 60 displayed character rows, 240 active scanlines |
| R7 | 66 | Vertical sync position | Places vertical sync after the active display |
| R8 | 0 | Interlace mode and display/cursor skew | Non-interlaced operation; skew features are not used |
| R9 | 3 | Maximum raster address within a character row | Four scanlines per displayed character row |
| R10 | 3 | Cursor start raster address plus cursor mode bits | Cursor starts on raster line 3; cursor output is used as a timing interrupt source |
| R11 | 3 | Cursor end raster address | Same as R10 so the cursor pulse is one scanline high |
| R12-R13 | 0 | Start address: first CRTC refresh-memory address after vertical blanking | Display begins at TVC video memory address 0 |
| R14-R15 | `0x0EFF` | Cursor address: CRTC refresh-memory address compared against MA | Startup firmware uses this address with R10/R11 to request the normal frame interrupt |
| R16-R17 | read-only | Light pen address latch | Routed to the expansion connector; not used by normal TVC firmware |

These values yield `(77 + 1) * (3 + 1) + 2 = 314` generated scanlines.

With the normal 64-byte line width, the final displayed character address is:

```text
(60 displayed character rows * 64 characters per row) - 1 = 3839 = 0x0EFF
```

Splitting this CRTC memory address into cursor registers gives `R14=0x0E` and
`R15=0xFF`. Since `R9=3` and `R10=3`, this cursor position requests the normal
shared interrupt on raster line 3 of the last displayed character row: the last
active scanline.

Do not assume all running software keeps this cursor address. For example, the
checked-in clean VT-DOS boot snapshot (`snapshots/boot12dos.rtvcsnap.zip`)
currently contains `R14-R15 = 0x0AFF` with the same `R10=3`, which places the
cursor interrupt at character row 43, column 63, raster line 3. This value is
programmed by the VT-DOS disk ROM while it is temporarily mapped at `0xC000`;
the routine writes CRTC registers from a DOS table rather than leaving the
startup firmware's `0x0EFF` cursor address intact.

The difference is visible to raster-timed code. A program that changes the
border or palette from the cursor interrupt and expects the interrupt on the
last active paper line behaves correctly after the plain firmware setup, but
will trigger much earlier after VT-DOS has changed R14-R15 unless the program
reprograms the CRTC itself. Games and ports that inherit the current CRTC state
must either tolerate the live cursor position or explicitly restore the
expected CRTC cursor registers.

### VRAM address wiring

Let:

- `MA` be the low 12 bits of the CRTC memory address;
- `RA` be the raster address within the character row.

The physical offset in a 16 KiB video bank is:

```text
vram = ((RA & 0x03) << 6)
     |  (MA & 0x003F)
     | ((MA & 0x0FC0) << 2)
```

Equivalently, VRAM bits 0-5 receive MA0-MA5, bits 6-7 receive RA0-RA1,
and bits 8-13 receive MA6-MA11.

With the normal 64-byte line width, each group of four adjacent raster lines is
stored in one contiguous 256-byte block:

```text
row 0 raster 0: offsets 0x0000-0x003F
row 0 raster 1: offsets 0x0040-0x007F
row 0 raster 2: offsets 0x0080-0x00BF
row 0 raster 3: offsets 0x00C0-0x00FF
row 1 raster 0: offsets 0x0100-0x013F
...
```

At the published normal cursor position, `MA=0x0EFF` and `RA=3` produce VRAM
offset `0x3BFF`, the final byte of the 512 x 240 picture.

### Video mode, port `0x06` bits 0-1

| Bits | Mode | Pixels per byte | Color source |
| --- | --- | ---: | --- |
| `00` | 2-color | 8 | palette entries 0-1 |
| `01` | 4-color | 4 | palette entries 0-3 |
| `10` | 16-color | 2 | pixel contains direct IGRB color |
| `11` | 16-color | 2 | same serializer class as mode 2 |

#### Two-color byte

Bits are emitted from bit 7 to bit 0. A zero selects palette 0 and a one
selects palette 1.

#### Four-color byte

The two bits of each pixel are split between nibbles:

```text
pixel 0 = (bit3 << 1) | bit7
pixel 1 = (bit2 << 1) | bit6
pixel 2 = (bit1 << 1) | bit5
pixel 3 = (bit0 << 1) | bit4
```

#### Sixteen-color byte

The left pixel uses odd-numbered bits and the right pixel uses even-numbered
bits:

```text
left  = [bit7 intensity, bit5 green, bit3 red, bit1 blue]
right = [bit6 intensity, bit4 green, bit2 red, bit0 blue]
```

Each source pixel is normally repeated horizontally so every fetched byte
occupies eight output pixel periods in all three modes.

### Palette and color encoding

Ports `0x60-0x63` hold palette entries 0-3. TVC color values use:

```text
bit 6  intensity
bit 4  green
bit 2  red
bit 0  blue
```

The unused alternating bits are ignored for palette colors. Intensity clear
means the dim level; intensity set means the bright level. A digital renderer
may use `0x7F` and `0xFF` for the two nonzero channel levels.

Port `0x00` sets the border color. Bits 7, 5, 3, and 1 of the written value are
the significant intensity, green, red, and blue bits; each is duplicated into
the adjacent low bit in the internal border register:

```text
value written         = IxGxRxB
border register value = IIGGRRBB
```

### Output pipeline, blanking, and sync

The CRTC does not drive a television directly. Its MA/RA, display-enable,
horizontal-sync, and vertical-sync outputs feed the TVC display circuitry:

1. MA/RA select a video-memory byte and the color shifter serializes it;
2. the palette or direct-color path resolves that byte to final IGRB levels;
3. display enable controls the WSE border flip-flop, selecting paper while DE
   is active and the border latch otherwise;
4. NVRCL clears the final video register during horizontal or vertical
   retrace;
5. the raw CRTC HS and VS outputs trigger separate SN74LS123 monostables, whose
   outputs are the externally shaped horizontal and vertical sync signals;
6. the shaped sync signals are combined by a NOR gate to form active-low
   composite sync (NCSYNC) for the PAL/RGB output circuitry.

Vertical blanking is latched when VS starts. CRTC address output MA9 resets the
blanking latch and permits video again. This makes blanking a property of the
TVC circuitry driven by CRTC signals, rather than a palette color or an
implicit range of normal firmware register values.

Software may reprogram the CRTC. The counters and address generator continue
to run even when a programmed sync-position comparison is unreachable; that
output simply produces no pulse. A television-facing emulator must therefore
derive picture lock from the shaped output sync signals, not from R0/R2 or
R4/R7 values and not from the normal 100-by-314 firmware geometry.

### Cursor interrupt

The hardware cursor is primarily used as a timing signal. Normal software
draws its text and visible cursor into bitmap memory itself. R14-R15 and the
cursor raster setting define where CURSOR requests the shared interrupt.
Startup firmware documents the cursor at the final byte of the active display,
but software may move it; an emulator must derive the interrupt position from
the live CRTC registers rather than from a fixed frame boundary.

VT-DOS is a normal example of such software-visible state: it may leave
R14-R15 at `0x0AFF`, moving the shared cursor/sound interrupt from the last
active paper line to row 43. Programs that do not reprogram the CRTC inherit
that timing.

An emulator that only raises an interrupt at a fixed host-frame boundary will
miss programs that change CRTC geometry or time raster work from the cursor
pulse.

### Interlace, skew, and light pen

Normal TVC output is non-interlaced. CRTC interlace and skew behavior is
component-variant-sensitive and uncommon in TVC software. The light-pen strobe
is routed to the expansion connector and should latch the current refresh
address into R16-R17 when implemented.

## Keyboard

The keyboard is a 10 x 8 active-low matrix (hardware book §3.3.2, Figure 10):

- write the row number to port `0x03` bits 0-3;
- read the eight column bits from port `0x58`;
- a zero bit means pressed;
- a one bit means released;
- an unconnected row (10-15) reads `0xFF`.

### Matrix

| Row | C0 | C1 | C2 | C3 | C4 | C5 | C6 | C7 |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | `5` | `3` | `2` | `0` | `6` | `í` | `1` | `4` |
| 1 | `^` | `8` | `9` | `ü` | `*` | `ó` | `ö` | `7` |
| 2 | `t` | `e` | `w` | `;` | `z` | `@` | `q` | `r` |
| 3 | `]` | `i` | `o` | `ő` | `[` | `ú` | `p` | `u` |
| 4 | `g` | `d` | `s` | `\` | `h` | `<` | `a` | `f` |
| 5 | Backspace/Delete | `k` | `l` | `á` | Return | `ű` | `é` | `j` |
| 6 | `b` | `c` | `x` | Shift | `n` | Lock | `y` | `v` |
| 7 | Alt | `,` | `.` | Esc | Ctrl | Space | `-` | `m` |
| 8 | Ins | Up | Down | Fire | Accel | Right | Left | - |
| 9 | - | Up | Down | Fire | Accel | Right | Left | - |

Rows 8 and 9 are the joystick rows; Ins is the only keyboard key on them. Row 8
is shared by the keyboard's built-in joystick (up/down/left/right only, no fire
or accelerator) and one external joystick port. Row 9 is the other external
port. The hardware book is inconsistent about which port (CH6 left, CH7 right)
is on which row: the text and Figure 10 put the right joystick on row 8, while
the connector table and block diagram put CH6 on row 8 and CH7 on row 9.
Rows 10-15 select nothing and read `0xFF`.

Host keyboard auto-mapping, Unicode input, AltGr handling, and stuck-key
prevention are emulator UI concerns and are documented in
[rtvc.md](rtvc.md#keyboard-input).

## Sound and Timer

### Registers

The sound path uses three writes:

```text
port 0x04 bits 0-7   frequency divisor low byte
port 0x05 bits 0-3   frequency divisor high nibble
port 0x05 bit 4      route timer oscillator to amplitude control
port 0x05 bit 5      enable timer interrupt
port 0x06 bits 2-5   four-bit amplitude/DAC value
```

Let `n` be the 12-bit divisor:

```text
n = ((port05 & 0x0F) << 8) | port04
```

The audible oscillator frequency is:

```text
f = 3,125,000 / 16 / (4096 - n)
  = 195,312.5 / (4096 - n) Hz
```

`n = 0xFFF` stops the oscillator.

### Divider and output

The programmable stage counts `4096-n` CPU clocks. Its carry advances a
four-bit counter. Bit 3 of that counter is the square-wave output, producing
eight low carries followed by eight high carries.

When port `0x05` bit 4 is set, that square wave gates the four-bit amplitude.
When bit 4 is clear, the amplitude value is presented directly as a DAC level,
allowing software-generated stepped waveforms.

The physical audio path is unipolar before AC coupling. A faithful audio output
model should remove DC rather than treating a static DAC level as permanent
speaker displacement.

### Timer restart and interrupt

Reading port `0x5B` or `0x5F`:

- reloads the programmable divider from the current divisor;
- clears/restarts the four-bit counter phase;
- returns an unused/open-bus value.

The restart happens during the `IN` instruction's I/O cycle (see
[I/O cycle timing](#io-cycle-timing)), so the clocks of that instruction
before the port access still count toward the old period.

If port `0x05` bit 5 is enabled, a full wrap of the four-bit counter requests
the shared cursor/sound interrupt. The cassette firmware uses this as a roughly
20 ms timing source with an appropriate divisor.

Raster-timed programs can use the timer as a line interrupt: divisor `0xFE7`
gives a 25-clock period and an interrupt every 400 T-states, two normal
scanlines. Such an effect lands at the hardware raster position only if the
port accesses, the falling-edge cursor interrupt that starts the timer, and
interrupt acknowledge are all timed within the instruction.

## Cassette Interface

The cassette hardware is intentionally simple; ROM software performs the
modulation and decoding.

### Input

The amplified, squared cassette input appears as ECIN at status-port bit 5:

```text
IN 0x59 or 0x5D
bit 5 = current cassette input level
```

Loading software measures transition intervals to distinguish the FSK tones.
For cycle-accurate emulation, change bit 5 at the recorded pulse boundaries,
not at audio-buffer boundaries.

### Output

Any I/O access, read or write, to `0x50-0x57` toggles the cassette output
flip-flop. Software controls the interval between toggles to generate the
recording waveform.

### Motor

Port `0x05` bits 6 and 7 drive the cassette motor-control outputs. The tape
transport advances while either output is active:

```text
motor_on = (port05 & 0xC0) != 0
```

An emulator should freeze tape position while both bits are clear.

## Expansion System

The TVC has four logical expansion slots.

### I/O windows

| Slot | I/O range | Card-local register |
| ---: | --- | --- |
| 0 | `0x10-0x1F` | low nibble |
| 1 | `0x20-0x2F` | low nibble |
| 2 | `0x30-0x3F` | low nibble |
| 3 | `0x40-0x4F` | low nibble |

Each slot receives 16 card-local port offsets. Missing devices normally read
`0xFF` and ignore writes.

### Memory window

Only one card at a time appears in the `0xC000-0xDFFF` EXT window. Port `0x03`
bits 6-7 select slots 0-3. The `0xE000-0xFFFF` half remains EXTH ROM.

### Connector signals

Each connector carries the full address and data bus, read/write strobes,
reset and clock, plus these per-slot signals:

| Signal | Meaning for a card |
| --- | --- |
| `NSLOTn` | I/O select for the slot's 16 ports; the card decodes the low address bits. |
| `NEXPn` | memory select for `0xC000-0xDFFF` while page 3 maps EXT and port `0x03` bits 6-7 select the slot |
| ID lines | the two identifier bits read through `0x5A`/`0x5E` |
| interrupt | active-low request, visible in `0x59`/`0x5D` bits 0-3 |
| `NNMI` | non-maskable interrupt request |
| `NSLOWEXP` | opcode-fetch wait-state request (32k/64k boards; see [Opcode fetch wait state](#opcode-fetch-wait-state-on-32k-and-64k-boards)) |

Port assignment is positional: the same card answers on `0x11` in slot 0 and on
`0x21` in slot 1. Card firmware must therefore discover its own slot.

### Interrupts and identification

Each slot has one active-low status/interrupt bit in port `0x59`/`0x5D`.
Writing to `0x58`/`0x59`/`0x5A`/`0x5B` controls slot 0/1/2/3: bit 7 set
enables that slot's request, bit 7 clear disables and clears it. The
BASIC 1.2 system ROM's interrupt handler writes only to the ports of slots
that are requesting service: `0x80` when its IRQ-STAT mask at `0x0B1F`
enables that slot, `0x00` when it does not. It then calls the interrupt
vector of every requesting card, enabled or not.
Port `0x5A`/`0x5E` reports the two-bit card type identifiers described in the
I/O map.

### Operating system card discovery

The BASIC 1.2 system ROM (EXT bank, `F050`-`F13A` in the commented listing)
discovers cards at boot:

1. It reads `0x5A` and, for each slot, selects the slot's IOMEM with port
   `0x03` and fills a 0x30-byte card table entry at `0x0040 + slot * 0x30`
   (`0x0040`, `0x0070`, `0x00A0`, `0x00D0`, in U0).
2. Identifiers `00`/`01`/`10` produce the built-in names `RS232`/`VGB`/`DISK`
   without reading card memory. Identifier `11` makes the ROM look for `MOPS`
   at `0xC000`; if found, the length-prefixed name that follows is copied into
   the entry, otherwise the entry starts with `0xFF` (no card).
3. Entries with identical names get increasing unit numbers at entry `+7`.
4. For each identified non-serial card, the ROM pages the card in, sets
   `IX` to the entry address, and calls the address stored at `0xC00B`
   through the `0xFFF9` gateway with page map `U0 U1 U2 EXT`.

A `MOPS` ROM header therefore has a fixed layout:

| Address | Contents |
| --- | --- |
| `0xC000` | `"MOPS"` |
| `0xC004` | name length, `4` |
| `0xC005` | 4-character device name |
| `0xC009` | word, interrupt routine |
| `0xC00B` | word, init routine |
| `0xC00D` | function count; must exceed the highest function number used |
| `0xC00E` | words, function 0, 1, ... routine addresses |

Function 0 is also the interrupt entry: the expansion IRQ dispatcher reads the
vector at `0xC00E` for each requesting slot. Init code can compute its slot
from `IX`: `slot = ((IX & 0xF0) - 0x10) >> 6`, giving port base
`0x10 + slot * 0x10`. Bytes from entry `+0x0F` onward are not used by the OS.

Device I/O reaches a card through `RST 30H`. The class selector bytes at
`0x0B00-0x0B07` (input) and `0x0B08-0x0B0F` (output) name the device for each
class; `0x80 | slot` selects a card, so cassette class 5 is redirected by
`0x0B05`/`0x0B0D`. The ROM validates the slot, pages the card in, looks up the
requested routine in the counted table, and calls it with:

- `A` = the function byte (bit 7 set for input), flags from `OR A`;
- `BC`, `DE` = the caller's values, returned to the caller;
- `IX` = card table entry `+8` for the slot (`0x0048 + slot * 0x30`);
- page map `U0 U1 U2 EXT`, so U0 system variables are addressable.

The routine returns a status in `A` (`0` for success). An out-of-range
function number returns `0xFF`, and a missing card resets the selector to its
default. Cassette-class routines are: 1 character I/O (`C` = byte), 2 block
I/O (`DE` address, `BC` length; returns the next address and remaining count),
3 open input / create output (`DE` = length-prefixed name), 4 close,
5 verify. The cassette routines keep the buffered-file flag at `0x0B6B` and
the end-of-file flag at `0x0B6E`.

Status codes used by the cassette routines include `0xE7` (read failure or
invalid header), `0xE8` (verify mismatch), `0xE9` (file not open), `0xEB`
(already open) and `0xEC` (end of file). VT-DOS uses `0xA1` for "File not
found" and `0xAD` for "Disk operation aborted". BASIC 1.2 prints unknown OS
codes as `System error NNN`, for example `System error 161` for `0xA1`.

VT-DOS 1.2 works from a non-zero slot: with the HBF card in slot 1 or 2 it
drives the FDC through that slot's ports and lists a disk directory.

## HBF Floppy Interface

The HBF card normally occupies slot 0 and provides VT-DOS boot support. It
contains:

- a 16 KiB ROM divided into four 4 KiB pages;
- 4 KiB private RAM;
- an FD1793-compatible floppy controller;
- drive/side control and fast status logic.

Generic FD1793 command timing and status semantics should come from the
controller datasheet. The following is the HBF-specific connection.

### HBF memory window

When HBF is the selected EXT card:

```text
0xC000-0xCFFF  selected 4 KiB page of the 16 KiB HBF ROM
0xD000-0xDFFF  HBF private RAM
```

ROM writes are ignored. Private RAM is read/write.

### HBF I/O registers

The table gives card-local offsets. Add the slot base, normally `0x10`.

| Offset | Read | Write |
| ---: | --- | --- |
| 0 | FDC status; acknowledges/clears INTRQ as defined by card/controller | FDC command |
| 1 | FDC track | FDC track |
| 2 | FDC sector | FDC sector |
| 3 | FDC data; transfer handshake affects DRQ | FDC data |
| 4 | fast status: bit 7 DRQ, bit 0 INTRQ | HBF drive parameter register |
| 8 | unspecified | HBF ROM page selector |

### Drive parameter register, offset 4 write

```text
bit 7    side select
bit 6    motor on
bit 5    density control (DDEN)
bit 4    head-load control
bits 3-0 drive selects DS3-DS0
```

### ROM page selector, offset 8 write

```text
bits 5-4 = 00 ROM page 0
bits 5-4 = 01 ROM page 1
bits 5-4 = 10 ROM page 2
bits 5-4 = 11 ROM page 3
```

## CAS Cassette Representation

This section describes the common TVC `.cas` preservation format and the
waveform produced by the historical converter. It is a media representation,
not a register-level hardware requirement.

### Container layout

A normal CAS file has a 144-byte file header followed by payload:

```text
offset 0x000-0x07F  128-byte block/file metadata
offset 0x080-0x08F  16-byte application metadata
offset 0x090...      payload
```

The total encoded file size is derived from header words:

```text
file_size = block_count * 128 + final_block_bytes
payload_size = file_size - 144
```

Bytes are serialized least-significant bit first.

### Reference pulse lengths

The following pulse lengths reproduce the established 44.1 kHz `cas2wav`
conversion at a 3.125 MHz CPU time base:

| Signal | High half | Low half | Period |
| --- | ---: | ---: | ---: |
| pilot/pre-sound | 638 cycles | 638 cycles | 1,276 cycles |
| sync | 1,205 cycles | 1,205 cycles | 2,410 cycles |
| data bit 0 | 779 cycles | 779 cycles | 1,558 cycles |
| data bit 1 | 567 cycles | 567 cycles | 1,134 cycles |

A nominal second of silence in the historical converter is 22,052 samples,
which is slightly different from exactly 44,100 samples.

### CRC

The tape format uses a bitwise 16-bit CRC:

```text
crc = 0

for each data bit:
    carry = ((((crc >> 8) & 0xFF) XOR (bit ? 0x80 : 0x00)) & 0x80) != 0
    if carry: crc = crc XOR 0x0810
    crc = (crc << 1) & 0xFFFF
    if carry: crc = crc OR 1
```

### Header block

```text
2 nominal seconds silence
10,240 pilot pulses
1 sync pulse
0x00, excluded from CRC
CRC reset
0x6A start
0xFF header block type
0x11 non-buffered flag
0x00 write-protection flag
0x01 sector count
0x00 sector number
header-sector byte count
filename length
filename, up to 16 bytes
0x00 filler
file type
payload length, little-endian
autostart byte
10 filler bytes
version
last-sector flag
CRC, little-endian
5 pilot pulses
```

Historical converter compatibility includes a one-byte-over metadata read and
writes zero for the generated autostart field. Emulator authors seeking exact
compatibility with existing WAVs should account for that behavior.

### Data block and sectors

```text
1 nominal second silence
5,120 pilot pulses
1 sync pulse
0x00, excluded from CRC
CRC reset
0x6A start
0x00 data block type
0x11 non-buffered flag
0x00 write-protection flag
sector count
```

Each sector then contains:

```text
sector number
size byte: 0 means 256 bytes, otherwise exact final-sector size
payload bytes
filler: 0x00 after a full sector, 0xFF after a partial sector
CRC, little-endian
```

Sector 1 continues the CRC from the data-block header. CRC resets before each
later sector. The stream ends with five pilot pulses and two nominal seconds of
silence.

## Raw DSK Images

TVC-DOS disk images commonly use flat sector dumps. Geometry can be obtained
from a FAT12-compatible boot sector:

```text
offset 11-12  bytes per sector
offset 19-20  total sectors
offset 21     media descriptor (`F8` for 360 KiB, `F9` for 720 KiB)
offset 24-25  sectors per track
offset 26-27  number of heads
```

For zero-based track and side and one-based sector:

```text
byte_offset =
    (track * sectors_per_track * heads
     + side * sectors_per_track
     + sector - 1)
    * bytes_per_sector
```

The HBF/FD1793 interface still exposes controller-level transfers; parsing a
flat image is an emulator storage decision.

## System ROM Placement

The logical ROM banks are:

| Bank | Size | CPU visibility |
| --- | ---: | --- |
| SYS | 16 KiB | any page selected as SYS |
| CART | 16 KiB | page 0 or page 3 when selected |
| EXTH | 8 KiB | `0xE000-0xFFFF` when page 3 selects EXT |

Common dumped filenames used by rtvc are included only to clarify placement:

| Firmware | Placement |
| --- | --- |
| `TVC12_D4.64K` | SYS low 16 KiB image |
| `TVC12_D3.64K` | SYS upper 8 KiB overlay at bank offset `0x2000` |
| `TVC12_D7.64K` | EXTH |
| `TVC22_D6.64K` | SYS low 16 KiB image |
| `TVC22_D4.64K` | SYS upper 8 KiB overlay at bank offset `0x2000` |
| `TVC22_D7.64K` | EXTH |
| `VT-DOS12-DISK.ROM` | HBF card ROM |
| `VT-DOS12-CART.crt` | CART bank |

These names are preservation conventions, not hardware identifiers.

## Minimum Emulator Model

A practical first implementation should:

1. run a conforming Z80 core at a 3.125 MHz logical clock;
2. implement all four 16 KiB memory windows and both paging registers;
3. route the complete low-byte I/O map, including mirrors;
4. advance the MC6845 by one character clock per two CPU T-states;
5. apply the TVC VRAM address transform and three pixel serializers;
6. latch and acknowledge the active-low shared interrupt correctly;
7. scan the active-low keyboard matrix;
8. advance sound and tape state from elapsed CPU cycles;
9. provide absent expansion bus values and HBF routing when configured.

A frame-at-once renderer can boot ordinary software, but raster effects and
cursor-interrupt timing require interleaved CRTC advancement.

## Accuracy-Sensitive Behavior

- CPU-visible and displayed video banks are independent on TVC 64K+.
- CRTC ports are mirrored throughout `0x70-0x7F`.
- The CRTC cursor, not VSYNC, is the normal frame interrupt source.
- Cursor and sound timer share one active-low pending bit, latched on the
  falling edge of either source.
- Port accesses take effect inside the I/O machine cycle, not at the end of
  the instruction.
- CPU accesses to video RAM are stretched to the CRTC character clock.
- Tape input and output transitions are CPU-cycle-sensitive.
- Reading `0x5B`/`0x5F` changes sound timer phase.
- Any read or write in `0x50-0x57` toggles cassette output.
- The EXT page is split: card memory below `0xE000`, EXTH ROM above it.
- Unknown or unimplemented hardware behavior should remain visibly marked as
  uncertain instead of being inferred from a single emulator.

## External Component References

- [Zilog Z80 CPU User Manual](https://www.zilog.com/docs/z80/um0080.pdf)
- Motorola/compatible MC6845 datasheet for generic CRTC counter and register
  behavior
- Western Digital FD1793 datasheet for generic floppy-controller commands,
  timing, and status semantics
- Original TVC hardware manuals and schematics for electrical details and
  expansion signals

This reference should be updated when a TVC-specific behavior is verified or
corrected. Generic component material should remain in the component manuals.
