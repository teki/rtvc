# Video Controller (MC6845 CRTC) Documentation

This document provides a language-independent architectural guide for building and understanding the Video Controller for the Videoton TV Computer (TVC) emulator. It is based on the Motorola MC6845 implementation in [src/vid.js](file:///Users/teki/dev/jstvc/src/vid.js) but abstracts away JavaScript-specific details to serve as a reference for any implementation. For memory management details, refer to the [mmu.md](file:///Users/teki/dev/jstvc/docs/mmu.md) documentation.

## Table of Contents

- [Overview](#overview)
- [System Timings and Clocks](#system-timings-and-clocks)
- [MC6845 Registers](#mc6845-registers)
- [Video Memory Address Translation](#video-memory-address-translation)
- [Graphics and Color Modes](#graphics-and-color-modes)
- [Color Palette and RGB Format](#color-palette-and-rgb-format)
- [Cursor and Vertical Sync Interrupts](#cursor-and-vertical-sync-interrupts)
- [Rendering Architecture: Two Implementation Modes](#rendering-architecture-two-implementation-modes)
  - [1. Interleaved/Streaming Mode (High Accuracy)](#1-interleavedstreaming-mode-high-accuracy)
  - [2. Once-per-Frame Mode (Simple/Basic)](#2-once-per-frame-mode-simplebasic)

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

The Z80 CPU programs the MC6845 via two I/O ports:
- **Port `0x70` (Write-Only)**: Address Register. Selects which internal data register (R0-R17) to access.
- **Port `0x71` (Write-Only)**: Data Register. Writes a byte to the currently selected register.

### Register Layout and TVC Defaults

| Reg | Name | Read/Write | Unit / Type | Description / TVC Default Value |
|:---:|------|:----------:|-------------|---------------------------------|
| **R0** | Horizontal Total | Write | Characters | Total characters in a line minus 1. <br> *TVC Default: 99 (100 char clocks/line = 64 µs PAL)* |
| **R1** | Horizontal Displayed | Write | Characters | Active characters per scanline. <br> *TVC Default: 64 (512 pixels wide)* |
| **R2** | Horizontal Sync Position | Write | Characters | Start of HSYNC. <br> *TVC Default: 75* |
| **R3** | Sync Widths | Write | Combined | Lower 4 bits: HSYNC width in characters. <br> Upper 4 bits: VSYNC width in scanlines. <br> *TVC Default: `0x32` (HSYNC = 2 chars, VSYNC = 3 lines)* |
| **R4** | Vertical Total | Write | Char Rows | Total character rows per frame minus 1. <br> *TVC Default: 77* |
| **R5** | Vertical Total Adjust | Write | Scanlines | Fractional scanlines to add to the end of the frame. <br> *TVC Default: 2* |
| **R6** | Vertical Displayed | Write | Char Rows | Active character rows displayed per frame. <br> *TVC Default: 60 (240 lines active)* |
| **R7** | Vertical Sync Position | Write | Char Rows | Start of VSYNC. <br> *TVC Default: 66* |
| **R8** | Interlace & Skew | Write | Flags | Bits 0-1: Interlace Mode (0 = Progressive). <br> Bits 4-5: DE Skew. <br> Bits 6-7: Cursor Skew. <br> *TVC Default: 0* |
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

In standard setups, the MC6845 addresses memory linearly. However, the TVC uses a custom memory address interleaving logic designed to map the raster lines efficiently, implemented by [genAddress](file:///Users/teki/dev/jstvc/src/vid.js#L158).

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

### Reference Implementation Function

```javascript
function genAddress(ma, rl) {
  ma = ma & 0xFFF; // 12-bit limit
  return ((rl & 0x03) << 6)         // Insert lower 2 bits of scanline into bits 6-7
       | (ma & 0x3F)                // Keep lower 6 bits of memory address in bits 0-5
       | ((ma & 0x3FC0) << 2);      // Shift bits 6-13 of memory address to bits 8-15
}
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

When translating an `xIxGxRxB` color value to standard 32-bit RGBA/ARGB (modeled by [toRGBA](file:///Users/teki/dev/jstvc/src/vid.js#L8)), use the following rules:
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

## Rendering Architecture: Two Implementation Modes

Developers can implement the TVC video emulation in two ways, depending on their performance and accuracy requirements.

### 1. Interleaved/Streaming Mode (High Accuracy)

Used in high-accuracy emulators to support mid-frame effects (e.g. split-screens, scrolling changes, and raster lines).

#### Mechanics
1. **Interleaved Steps**: After the Z80 CPU executes an instruction (taking `cpuTime` cycles), the emulator immediately advances the video state machine by calling [streamSome](file:///Users/teki/dev/jstvc/src/vid.js#L207).
2. **Stream buffer**: The CRTC maintains internal beam counters (`_char`, `_row`, `_line`). For every 2 CPU cycles (1 character clock), it pushes a 16-bit word representing that character's state to a circular stream buffer:
   - **Bit 10**: HSYNC active state.
   - **Bit 11**: VSYNC active state.
   - **Bits 8-9**: Selected mode (0 = Mode 0, 1 = Mode 1, 2 = Border).
   - **Bits 0-7**: Data (VRAM byte if inside active area; `border2` byte if outside).
3. **State Machine Renderer**: A decoupled [renderStream](file:///Users/teki/dev/jstvc/src/vid.js#L340) function processes the circular buffer and paints the pixels. It acts like a CRT monitor, reacting to the sync pulses:
   - **Phase 0**: Wait for VSYNC to go high (start of frame).
   - **Phase 1**: Count 26 HSYNC lines (vertical back porch margin) before starting draw.
   - **Phase 100**: Wait for HSYNC trailing edge.
   - **Phase 2**: Skip 16 character clocks (horizontal back porch margin).
   - **Phase 3**: Draw 76 character clocks (608 pixels) to the current line in the framebuffer.
   - **Phase 4**: Wait for next HSYNC pulse.

#### Advantages
- **Cycle-accurate**: Palette changes, border colors, and scroll register offsets changed by the CPU mid-frame are rasterized on the exact line/pixel they occur.
- Cursor interrupts are triggered at the correct cycle.

---

### 2. Once-per-Frame Mode (Simple/Basic)

Used in basic emulators to simplify the rendering pipeline and decrease CPU overhead.

#### Mechanics
1. **CPU Run**: The Z80 CPU runs for a full frame's worth of cycles (62,500 clocks) without advancing the screen beam character-by-character.
2. **Frame Trigger**: At the end of the frame (or when a frame-draw is requested), the video module is called once to draw the entire framebuffer.
3. **Static Draw**: The function reads the current state of Video RAM, palette registers, and CRTC registers, then draws the screen onto the 608x288 pixel framebuffer.

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

## Emulation Divergences and TODOs

The TVC's video subsystem and emulator implementation have several functional differences compared to the standard Motorola MC6845 specification described in the [6845.md datasheet reference](file:///Users/teki/dev/pdfconv/6845/6845.md). These are documented below as emulation TODOs:

1. **[TODO] Programmable vs Fixed Vertical Sync Width (R3)**: 
   - Standard MC6845 specifies a fixed VSYNC width of 16 scanlines.
   - The TVC hardware (and the current [streamSome](file:///Users/teki/dev/jstvc/src/vid.js#L207) code) allows the VSYNC width to be programmed in the upper 4 bits of `R3` (`vvvv`).
2. **[TODO] Register Read/Write Access Violations**: 
   - Standard MC6845 specifies that data registers `R0-R11` and `R12-R13` are write-only.
   - The emulator currently permits internal reads for `R12-R13` (via the `getReg` function in [src/vid.js](file:///Users/teki/dev/jstvc/src/vid.js)), which is technically a datasheet violation, though the Z80 CPU cannot execute reads on these registers due to TVC port mapping constraints.
3. **[TODO] Hardware Cursor Blinking (R10/R11)**:
   - The TVC does not use the MC6845 in character mode (it operates exclusively in graphics mode). The text cursor is drawn entirely in software by the OS or application. 
   - Consequently, the MC6845's hardware cursor blinking (controlled by `R10` and `R11` scanline ranges and blink rates) is unused and ignored in the renderer.
4. **[TODO] Interlace and Skew Modes (R8)**:
   - Only progressive scan mode (no interlace) and zero-skew display enable/cursor are supported. 
   - The parameters configured in register `R8` are read but bypassed during emulation.
5. **[TODO] Light Pen Support (R16/R17)**:
   - Light pen address latching and strobe registers (`R16` and `R17`) are currently commented out and unimplemented.

