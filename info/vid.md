# Video Controller (MC6845 CRTC) Documentation

This document provides a language-independent architectural guide for building and understanding the Video Controller for the Videoton TV Computer (TVC) emulator. It is based on the Motorola MC6845 implementation in [src/vid.rs](../src/vid.rs). For memory management details, refer to the [mmu.md](mmu.md) documentation.

## Table of Contents

- [Overview](#overview)
- [System Timings and Clocks](#system-timings-and-clocks)
- [MC6845 Registers](#mc6845-registers)
- [Video Memory Address Translation](#video-memory-address-translation)
- [Graphics and Color Modes](#graphics-and-color-modes)
- [Color Palette and RGB Format](#color-palette-and-rgb-format)
- [Cursor and Vertical Sync Interrupts](#cursor-and-vertical-sync-interrupts)
- [Rendering Architecture: Two Scheduling Modes](#rendering-architecture-two-scheduling-modes)
  - [1. Interleaved/Streaming Mode (High Accuracy)](#1-interleavedstreaming-mode-high-accuracy)
  - [2. Once-per-Frame Mode (Fast Frame)](#2-once-per-frame-mode-fast-frame)
  - [Lost Sync Presentation](#lost-sync-presentation)

---

## Overview

The Videoton TV Computer uses a **Motorola MC6845 Cathode Ray Tube Controller (CRTC)** to generate the timing signals necessary for raster display and to drive the memory addresses for drawing the screen. 

Unlike newer, integrated video chips, the MC6845 does not contain on-chip graphics memory, character generator ROMs, or pixel serialization logic. It is strictly a counter-based timing generator that outputs:
1. **Memory Addresses (MA0-MA13)** indicating which byte of video memory to read.
2. **Raster Addresses (RA0-RA4)** indicating the current scanline within the active character row.
3. **Synchronization Pulses** (HSYNC and VSYNC) to control the display monitor.
4. **Display Enable (DE)** indicating when the beam is inside the active picture area.
5. **Cursor Output (CURSOR)** indicating when the current character match corresponds to the cursor position.

The TVC wraps this chip with custom pixel decoding shift registers, color palette registers, and an interrupt generator.

---

## System Timings and Clocks

A TVC emulator must synchronize the Z80 CPU clocks with the CRTC clocking:

- **CPU Frequency**: 3,125,000 Hz (3.125 MHz)
- **Character Clock (CCLK)**: 1,562,500 Hz (1.5625 MHz)
- **Character Clock Ratio**: Exactly **2 CPU clock cycles per character clock cycle**.
- **Frame Rate**: 50 Hz (PAL standard).
- **CPU Clocks per Frame**: 3,125,000 / 50 = 62,500 CPU clocks.
- **PAL Scanlines**: Exactly 314 scanlines per frame (nominally 312, calculated from standard registers).

---

## MC6845 Registers

The Z80 CPU programs the MC6845 through the `0x70-0x7F` I/O range. The CRTC only receives address bit `A0`, so its two internal ports are mirrored throughout the range:
- **Ports `0x70`, `0x72`, ..., `0x7E`**: Address Register. Selects which internal data register (R0-R17) to access.
- **Ports `0x71`, `0x73`, ..., `0x7F`**: Data Register. Reads or writes the register selected by the address register, subject to each register's access permissions.

The emulator currently implements the canonical write ports `0x70` and `0x71`; mirrored addresses and CPU-visible CRTC reads are deferred accuracy work.

### Register Layout and TVC Defaults

| Reg | Name | Read/Write | Unit / Type | Description / TVC Default Value |
|:---:|------|:----------:|-------------|---------------------------------|
| **R0** | Horizontal Total | Write | Characters | Total characters in a line minus 1. <br> *TVC Default: 99 (100 char clocks/line = 64 µs PAL)* |
| **R1** | Horizontal Displayed | Write | Characters | Active characters per scanline. <br> *TVC Default: 64 (512 pixels wide)* |
| **R2** | Horizontal Sync Position | Write | Characters | Start of HSYNC. <br> *TVC Default: 75* |
| **R3** | Sync Widths | Write | Combined | Lower 4 bits: HSYNC width in characters (`0` is forbidden on variants with programmable HSYNC width). <br> Upper 4 bits: VSYNC width in scanlines on CRTC variants that implement it; `0` means 16 scanlines. <br> *TVC Default: `0x32` (HSYNC = 2 chars, VSYNC = 3 lines)* |
| **R4** | Vertical Total | Write | Char Rows | Total character rows per frame minus 1. <br> *TVC Default: 77* |
| **R5** | Vertical Total Adjust | Write | Scanlines | Fractional scanlines to add to the end of the frame. <br> *TVC Default: 2* |
| **R6** | Vertical Displayed | Write | Char Rows | Active character rows displayed per frame. <br> *TVC Default: 60 (240 lines active)* |
| **R7** | Vertical Sync Position | Write | Char Rows | Start of VSYNC. <br> *TVC Default: 66* |
| **R8** | Interlace & Skew | Write | Flags | Bits 0-1: Interlace mode (`00`/`10` = non-interlaced, `01` = interlace sync, `11` = interlace sync/video). <br> Bits 4-5: DE skew. <br> Bits 6-7: Cursor skew. <br> *TVC Default: 0* |
| **R9** | Max Scan Line Address | Write | Scanlines | Scanlines per character row minus 1. <br> *TVC Default: 3 (4 scanlines per row)* |
| **R10** | Cursor Start Line | Write | Scanlines | Start line of cursor inside row, and blink bits. <br> *TVC Default: 3 (No blink, start scanline 3)* |
| **R11** | Cursor End Line | Write | Scanlines | End line of cursor inside row. <br> *TVC Default: 3* |
| **R12** | Start Address High | R/W | Address bits | High byte of video start address offset. <br> *TVC Default: 0* |
| **R13** | Start Address Low | R/W | Address bits | Low byte of video start address offset. <br> *TVC Default: 0* |
| **R14** | Cursor Address High | R/W | Address bits | High byte of cursor memory address. <br> *TVC Default: `14` (`0x0E`)* |
| **R15** | Cursor Address Low | R/W | Address bits | Low byte of cursor memory address. <br> *TVC Default: `255` (`0xFF`)* |
| **R16** | Light Pen High | Read-Only | Address bits | Saved value of MA at light-pen trigger. |
| **R17** | Light Pen Low | Read-Only | Address bits | Saved value of MA at light-pen trigger. |

Using the default values, the total number of scanlines is:
$$\text{Scanlines} = (\text{R4} + 1) \times (\text{R9} + 1) + \text{R5} = (77 + 1) \times (3 + 1) + 2 = 314 \text{ scanlines}$$

---

## Video Memory Address Translation

In standard setups, the MC6845 addresses memory linearly. However, the TVC uses a custom memory address interleaving logic designed to map the raster lines efficiently. The Rust implementation lives in [src/vid.rs](../src/vid.rs).

Let:
- `ma` be the 12-bit character memory address generated by the CRTC (`MA0-MA11`).
- `rl` be the 5-bit raster line (scanline) index within the current character row (`RA0-RA4`).

The physical address inside the 16 KB Video RAM is generated by the following bitwise formula:

```text
Generated Address Bits (14 bits):
[A13 A12 A11 A10 A9  A8 ]  [A7  A6 ]  [A5  A4  A3  A2  A1  A0 ]
  \___________________/      \____/     \___________________/
      ma[6..11] << 2        rl[0..1]         ma[0..5]
```

### Reference Formula

```text
ma = ma & 0x0FFF
addr = ((rl & 0x03) << 6)    // Insert lower 2 scanline bits into bits 6-7
     | (ma & 0x003F)         // Keep lower 6 memory-address bits in bits 0-5
     | ((ma & 0x0FC0) << 2)  // Shift memory-address bits 6-11 into bits 8-13
```

---

## Graphics and Color Modes

The TVC supports three graphics modes, configured by writing to **Port `0x06`** (bits 0-1):
- **Mode 0 (`00`): 2-color mode**. High-resolution graphics. 1 byte in Video RAM = 8 pixels.
- **Mode 1 (`01`): 4-color mode**. Medium-resolution graphics. 1 byte in Video RAM = 4 pixels.
- **Mode 2/3 (`1x`): 16-color mode**. Low-resolution graphics. 1 byte in Video RAM = 2 pixels.

### Pixel Serialization Layout

#### 1. 2-Color Mode (`Mode 0`)
Each bit in the byte maps directly to a pixel, selecting between Palette Index 0 (bit is `0`) or Palette Index 1 (bit is `1`).
```text
Byte: [ b7  b6  b5  b4  b3  b2  b1  b0 ]
        |   |   |   |   |   |   |   |
Pixel: P0  P1  P2  P3  P4  P5  P6  P7
```

#### 2. 4-Color Mode (`Mode 1`)
Pixels are 2 bits each. The bits for a single pixel are split across the high and low nibbles of the byte.
- **Low bit** of pixel color: from the high nibble (`b7..b4`).
- **High bit** of pixel color: from the low nibble (`b3..b0`).

```text
Byte: [  b7    b6    b5    b4    b3    b2    b1    b0  ]
         \__   \__   \__   \__   \__   \__   \__   \__/
            |     |     |     |     |     |     |     |
Pixel 0:  Low   |     |     |   High    |     |     |   => Color index = (b3 << 1) | b7
Pixel 1:        Low   |     |         High    |     |   => Color index = (b2 << 1) | b6
Pixel 2:              Low   |               High    |   => Color index = (b1 << 1) | b5
Pixel 3:                    Low                     High  => Color index = (b0 << 1) | b4
```

#### 3. 16-Color Mode (`Mode 2/3`)
Pixels are 4 bits each. 1 byte contains 2 pixels.
- **Even bits** of the byte (`b6, b4, b2, b0`) map to the right pixel.
- **Odd bits** of the byte (`b7, b5, b3, b1`) map to the left pixel.

```text
Left Pixel Bits (Odd):  [ b7:Intensity, b5:Green, b3:Red, b1:Blue ]
Right Pixel Bits (Even): [ b6:Intensity, b4:Green, b2:Red, b0:Blue ]
```

---

## Color Palette and RGB Format

The TVC color system represents colors in an 8-bit `xIxGxRxB` layout:
- **Bit 7, 5, 3, 1**: Unused (hardwired to 0 or ignored).
- **Bit 6**: Intensity (**I**).
- **Bit 4**: Green (**G**).
- **Bit 2**: Red (**R**).
- **Bit 0**: Blue (**B**).

The 4 palette registers are mapped to I/O ports `0x60 - 0x63`. 

### Color Decoding (to RGBA)

When translating an `xIxGxRxB` color value to standard 32-bit RGBA/ARGB, use the following rules:
1. Determine the channel intensity coefficient:
   - If Intensity (**I**, bit 6) is set, `intens = 0xFF` (full brightness).
   - Otherwise, `intens = 0x7F` (half brightness).
2. If Green (**G**, bit 4) is set, the green channel value is `intens`, otherwise `0`.
3. If Red (**R**, bit 2) is set, the red channel value is `intens`, otherwise `0`.
4. If Blue (**B**, bit 0) is set, the blue channel value is `intens`, otherwise `0`.
5. Set Alpha to `0xFF`.

```javascript
function toRGBA(colorVal) {
  var intens = (colorVal & 0x40) ? 0xFF : 0x7F;
  var g = (colorVal & 0x10) ? intens : 0;
  var r = (colorVal & 0x04) ? intens : 0;
  var b = (colorVal & 0x01) ? intens : 0;
  return (0xFF << 24) | (b << 16) | (g << 8) | r;
}
```

### Border Color

The border color is configured by writing an `xIxGxRxB` value to Port `0x00`. To render the border using the standard 16-color decoding path, the TVC duplicates the odd bits (intensity, green, red, blue) into the even bits to produce a uniform color byte:

$$\text{border2} = \left((\text{color} \ \& \ \text{0xAA}) \gg 1\right) \ | \ (\text{color} \ \& \ \text{0xAA})$$

---

## Cursor and Vertical Sync Interrupts

The TVC connects the CRTC's **CURSOR** output pin directly to the CPU's interrupt line. 
- A cursor match occurs when the current video RAM read address matches the cursor address (`R14` / `R15`) and the current scanline offset matches the cursor start scanline (`R10`).
- By default, the TVC OS programs the cursor address to `0x0EFF` (character offset 3839) and the cursor scanline to `3` (the 4th scanline of the row). Because `64 * 60 = 3840`, `0x0EFF` corresponds to the very last character of the active screen.
- At this character, `genAddress(0x0EFF, 3)` translates to `0x3BFF` (the last byte of the 16 KB VRAM space).
- As a result, the cursor signal pulses high at the very last pixel of the active frame. This generates a **vertical timing interrupt (50 Hz)** used by the system for keyboard polling, cursor blinking, and music playback.

The CPU acknowledges/clears this interrupt by writing to **Port `0x07`**.

---

## Rendering Architecture: Two Scheduling Modes

Developers can choose between two TVC video scheduling modes, depending on their performance and accuracy requirements.

### 1. Interleaved/Streaming Mode (High Accuracy)

Used in high-accuracy emulators to support mid-frame effects (e.g. split-screens, scrolling changes, and raster lines).

#### Mechanics
1. **Interleaved Steps**: After the Z80 CPU executes an instruction (taking `cpuTime` cycles), the emulator immediately advances the video state machine with the interleaved streaming path in [src/vid.rs](../src/vid.rs).
2. **Stream buffer**: The CRTC maintains internal beam counters (`_char`, `_row`, `_line`). For every 2 CPU cycles (1 character clock), it pushes a 16-bit word representing that character's state to a circular stream buffer:
   - **Bit 10**: HSYNC active state.
   - **Bit 11**: VSYNC active state.
   - **Bits 8-9**: Selected mode (0 = Mode 0, 1 = Mode 1, 2 = Border).
   - **Bits 0-7**: Data (VRAM byte if inside active area; `border2` byte if outside).
3. **State Machine Renderer**: A decoupled renderer processes the circular buffer and paints the pixels. It acts like a CRT monitor, reacting to the sync pulses:
   - **Phase 0**: Wait for VSYNC to go high (start of frame).
   - **Phase 1**: Count 26 HSYNC lines (vertical back porch margin) before starting draw.
   - **Phase 100**: Wait for HSYNC trailing edge.
   - **Phase 2**: Skip 16 character clocks (horizontal back porch margin).
   - **Phase 3**: Draw 76 character clocks (608 pixels) to the current line in the framebuffer.
   - **Phase 4**: Wait for next HSYNC pulse.
4. **Cursor Interrupt Timing**: If streaming reaches the CRTC cursor position, the orchestrator services the Z80 IRQ immediately and advances the CRTC by the IRQ duration. This preserves the last-pixel timing used by games that offset drawing work from the screen interrupt.

#### Advantages
- **Cycle-accurate**: Palette changes, border colors, and scroll register offsets changed by the CPU mid-frame are rasterized on the exact line/pixel they occur.
- Cursor interrupts are triggered at the correct cycle.

---

### 2. Once-per-Frame Mode (Fast Frame)

Used in basic emulators to simplify the rendering pipeline and decrease CPU overhead.

#### Mechanics
1. **CPU Run**: The Z80 CPU runs for a full frame's worth of cycles (62,500 clocks) without advancing the screen beam character-by-character.
2. **Frame Trigger**: At the end of the frame (or when a frame-draw is requested), the video module is called once to draw the entire framebuffer.
3. **Static Draw**: The function reads the current state of Video RAM, palette registers, and CRTC registers, then draws the screen onto the 608x288 pixel framebuffer.

---

#### Design Pseudo-code

```c
// Target viewport: 608 x 288 pixels (76 x 288 characters)
void draw_frame(uint8_t* vram, uint32_t* framebuffer) {
    uint8_t R1_hd = read_crtc_reg(1);   // Active width in characters (e.g. 64)
    uint8_t R6_vd = read_crtc_reg(6);   // Active height in rows (e.g. 60)
    uint8_t R9_slr = read_crtc_reg(9);  // Scanlines per row minus 1 (e.g. 3)
    uint16_t smem = (read_crtc_reg(12) << 8) | read_crtc_reg(13);
    
    int scanlines_per_row = R9_slr + 1;
    int active_height = R6_vd * scanlines_per_row; // e.g. 240 lines
    
    // Center the active display inside the 608x288 frame
    int top_border = (288 - active_height) / 2;
    int left_border = (76 - R1_hd) / 2; 
    
    uint32_t border_color = toRGBA(read_border_register());

    for (int y = 0; y < 288; y++) {
        uint32_t* line_pixels = &framebuffer[y * 608];
        
        // 1. Vertical Border Check
        if (y < top_border || y >= (top_border + active_height)) {
            for (int x = 0; x < 608; x++) {
                line_pixels[x] = border_color;
            }
            continue;
        }
        
        // Calculate corresponding character row and scanline offset
        int row = (y - top_border) / scanlines_per_row;
        int line_offset = (y - top_border) % scanlines_per_row;
        
        // 2. Horizontal Draw
        for (int char_x = 0; char_x < 76; char_x++) {
            // Horizontal Border Check
            if (char_x < left_border || char_x >= (left_border + R1_hd)) {
                for (int p = 0; p < 8; p++) {
                    line_pixels[char_x * 8 + p] = border_color;
                }
                continue;
            }
            
            // Calculate active character coordinates
            int active_char_x = char_x - left_border;
            uint16_t ma = (smem + row * R1_hd + active_char_x) & 0x3FFF;
            uint16_t vram_addr = genAddress(ma, line_offset);
            uint8_t byte = vram[vram_addr];
            
            // Decode and write 8 pixels depending on current mode
            decode_pixels(&line_pixels[char_x * 8], byte, get_current_mode());
        }
    }
}
```

#### Advantages
- **Simple**: No stream buffer, complex state machines, or synchronization states.
- **Fast**: High performance and easy integration into basic GUI frameworks.

---

### Lost Sync Presentation

Streaming modes treat the CRTC stream as the source of truth, but presentation is bounded by host screen time. If the stream does not produce recognizable sync inside the current screen-time budget, the emulator keeps presenting the current monitor surface while it tries to relock. After several consecutive host ticks without a synchronized frame, it draws a black lost-sync background with moving white stripes and keeps running. This avoids freezing or spinning forever on misconfigured CRTC values while making the sync failure visible.

---

## Emulation Divergences and Deferred Accuracy Work

The TVC's video subsystem and emulator implementation have several functional differences compared to a standard Motorola MC6845. Some differences are intentional TVC behavior, while others are deferred hardware-accuracy work.

1. **Implemented TVC Divergence / [TODO] Vertical Sync Width (R3)**
   - `R3` behavior is CRTC-chip dependent. The Motorola MC6845 datasheet documents `R3` as HSYNC width only and describes the VSYNC pulse width as fixed at 16 scanlines, while several 6845-compatible CRTC types use bits `7-4` as programmable VSYNC width.
   - The TVC hardware manual presents `R3` as a sync-width register with low-nibble HSYNC and high-nibble VSYNC fields, but marks this area as chip-source dependent. This matches the broader 6845-compatible CRTC situation: CPC CRTC types 0, 3, and 4 implement programmable VSYNC width, while types 1 and 2 ignore the high nibble and use fixed 16-line VSYNC.
   - Nonzero values select a 1-15 scanline VSYNC pulse. The emulator implements these programmable widths: [src/vid.rs](../src/vid.rs) decodes `R3` into `vsw` and uses that value when generating the streaming VSYNC signal.
   - On CRTC variants that support programmable VSYNC width, a VSYNC width field of `0` means 16 scanlines. The emulator does not currently special-case `0` to 16.
2. **[TODO] CRTC Port Mirrors and Data Register Read Semantics**
   - TVC hardware exposes the CRTC at `0x70-0x7F`; because the CRTC only decodes `A0`, `0x70/0x72/.../0x7E` select the address register and `0x71/0x73/.../0x7F` access the selected data register.
   - The CRTC is readable by the CPU, but only according to register permissions. Both the Motorola datasheet and TVC hardware manual agree that `R0-R11` are write-only, `R14-R15` are readable/writable, and `R16-R17` are read-only light-pen latches.
   - `R12-R13` access appears to be CRTC-variant dependent: the Motorola MC6845 datasheet lists the start-address pair as write-only, while the TVC hardware manual lists it as readable/writable, matching other 6845-compatible parts such as the UMC UM6845. The TVC hardware manual also notes that schematic part markings may reflect the most common supplier and that actual fitted parts may come from another compatible supplier, but it does not name a specific CRTC supplier in the CRTC section found in `tvchardver.md`.
   - On readable high-byte registers, the upper two address bits read as `0`.
   - The emulator currently handles only writes to the canonical ports `0x70` and `0x71`; CPU reads from CRTC ports fall through to extension/default I/O handling. Internally, `Vid::get_reg()` returns the selected register from the full `R0-R17` array, which is convenient for inspection but does not model CPU-visible access restrictions.
3. **Implemented TVC Divergence / [TODO] Hardware Cursor Shape and Blink**
   - The TVC does not use the MC6845 as a character generator. Text is drawn in graphics memory by software, and the visible text cursor is also software-rendered.
   - The emulator implements the cursor output as a timing interrupt source: `R10` controls enable state and start scanline, while `R14/R15` select the cursor memory address.
   - The renderer intentionally does not draw the MC6845 hardware cursor shape or blink. `R11`/cursor end and the `R10` blink-rate bits are decoded or stored only as far as current interrupt timing needs require.
4. **Implemented TVC Policy / Skew Modes (R8)**
   - Interlace is intentionally not supported. The TVC hardware documentation states that the machine's normal video output uses non-interlaced scanning, and its composite video signal therefore does not contain the equalizing pulses or serrated vertical-sync pulses needed for odd/even field identification.
   - The UHF modulator only AM-modulates the PAL encoder's composite video onto UHF channel 36; it does not add interlace field-identification timing. Interlace-capable CRTC modes are therefore outside the intended TVC video model.
   - The emulator treats the display as non-interlaced even if `R8` selects an interlace mode.
   - Skew timing remains deferred accuracy work: `R8` cursor skew bits (`6-7`) are decoded but bypassed, and display-enable skew bits (`4-5`) are not currently decoded.
5. **[TODO] Light Pen Support (R16/R17)**
   - `R16` and `R17` exist in the internal CRTC register array, but the emulator does not implement light-pen trigger/strobe behavior or address latching.
   - TVC hardware routes the CRTC light-pen strobe input to the expansion connector. Its pulse latches the current refresh-memory address into `R16/R17`, with software expected to compensate for display and light-pen delays.
   - These registers should be treated as read-only latched light-pen address registers if light-pen support or strict MC6845 register semantics are added.
