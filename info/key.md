# Keyboard Matrix and Mapping Documentation

This document provides a language-independent architectural guide for building and understanding the keyboard subsystem of the Videoton TV Computer (TVC) emulator. It is based on the implementation in [src/key.rs](../src/key.rs).

## Table of Contents

- [Overview](#overview)
- [The Hardware Keyboard Matrix](#the-hardware-keyboard-matrix)
- [I/O Ports Interface](#io-ports-interface)
- [Dynamic Key Mapping (Auto-mapping)](#dynamic-key-mapping-auto-mapping)
- [Shift State Compensation (Modifiers)](#shift-state-compensation-modifiers)
- [Key State Lifecycle and Release](#key-state-lifecycle-and-release)

---

## Overview

The TVC keyboard is organized as a switch matrix of **11 rows by 8 columns**. Because of differences between the keyboard layout of the host system running the emulator and the target Hungarian layout of the TVC, the emulator implements a **dynamic auto-mapping system**. This system automatically maps host keyboard keystrokes to the corresponding TVC matrix coordinates upon the first press.

---

## The Hardware Keyboard Matrix

The keyboard state is maintained as an array of 11 bytes, `_state[0..10]`. Each byte represents one row of 8 keys (columns 0–7).
- A bit value of `1` indicates the key is **released** (default).
- A bit value of `0` indicates the key is **pressed**.

### Matrix Mapping Layout

Below is the layout of the matrix characters as defined by the static tables:

#### Rows 0–7 (Alphanumeric and Symbol Keys)
These are mapped sequentially from the layout tables `_ntable` (normal) and `_stable` (shifted):

| Row | Col 0 | Col 1 | Col 2 | Col 3 | Col 4 | Col 5 | Col 6 | Col 7 |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Row 0** | 5 | 3 | 2 | 0 | 6 | í | 1 | 4 |
| **Row 1** | ^ | 8 | 9 | ü | * | ó | ö | 7 |
| **Row 2** | t | e | w | ; | z | @ | q | r |
| **Row 3** | ] | i | o | ő | [ | ú | p | u |
| **Row 4** | g | d | s | \ | h | < | a | f |
| **Row 5** | _space_ | k | l | á | _space_ | ű | é | j |
| **Row 6** | b | c | x | _space_ | n | _space_ | y | v |
| **Row 7** | _space_ | , | . | _space_ | _space_ | _space_ | - | m |

*Note: Spaces in the table represent slots without direct character bindings or reserved keys.*

#### Rows 5–8 (System & Control Keys)
Statically pre-mapped keys that are not altered by the dynamic mapper:

| Key | TVC Coordinate (Row, Col) | Description |
|---|---|---|
| **Backspace** | `Row 5, Col 0` | Deletes previous character |
| **Delete** | `Row 5, Col 0` | Deletes next character |
| **Return** | `Row 5, Col 4` | Enter key |
| **Shift** | `Row 6, Col 3` | Shift modifier |
| **Lock** | `Row 6, Col 5` | Caps lock |
| **Alt** | `Row 7, Col 0` | Alt modifier |
| **Esc** | `Row 7, Col 3` | Escape key |
| **Ctrl** | `Row 7, Col 4` | Control key |
| **Space** | `Row 7, Col 5` | Spacebar |
| **Up Arrow** | `Row 8, Col 1` | Navigates up |
| **Down Arrow** | `Row 8, Col 2` | Navigates down |
| **Tab (Fire)** | `Row 8, Col 3` | Tab key / Joystick fire button |
| **Right Arrow** | `Row 8, Col 5` | Navigates right |
| **Left Arrow** | `Row 8, Col 6` | Navigates left |

---

## I/O Ports Interface

The keyboard is integrated into the emulator bus inside [src/tvc.rs](../src/tvc.rs):

1. **Row Selection (Port `0x03`)**:
   The CPU selects which row (0–10) to read by writing to Port `0x03` (masked as `val & 0x0F`), updating the keyboard row selector.
2. **Column Read (Port `0x58`)**:
   The CPU reads the column state for the selected row by reading from Port `0x58`. If no row state is configured, it returns `0xFF` (all keys released).

---

## Dynamic Key Mapping (Auto-mapping)

To support layout-independent typing (e.g., typing on a US, German, or Hungarian host keyboard and having it map correctly to the target TVC layout), the emulator dynamically maps raw keystrokes to TVC coordinate pairs. This process is divided into two distinct events supplied by the host operating system or UI toolkit:

1. **Key Down Event (Physical Key ID)**:
   When a physical key is pressed, the host environment reports a key identifier (such as a hardware scancode or virtual keycode). The emulator remembers this raw keycode as the active physical key.
2. **Text / Character Input Event (Unicode Character)**:
   If the keystroke generates a typable character, the host environment fires a text input event containing the resulting Unicode character (or character code). 
3. **Matrix Coordinates Lookup**:
   If the Unicode character is not yet mapped, the emulator searches the translation tables to bind it to the physical key recorded in step 1:
   - **Unshifted Table (`_ntable`)**: If found, maps the coordinates (`idx >> 3`, `idx & 7`). If the host Shift modifier was active, the `KSDEL` flag is set (TVC Shift must be suppressed).
   - **Shifted Table (`_stable`)**: If found, maps the coordinates (`idx >> 3`, `idx & 7`). If the host Shift modifier was inactive, the `KSADD` flag is set (TVC Shift must be forced).
4. **Registration**:
   The coordinates and modifier flags are stored in the active modifier keymap (`_keymap[mod][raw_keycode]`), and the character is marked as mapped. Subsequent key-down events for that raw keycode lookup this coordinate directly.

### Rust + egui / winit Implementation Notes

Native and full-web builds deliberately use different host event sources:

- **Native Key Down / Up**: Match `egui::Event::Key` and prefer `physical_key`, falling back to the logical `key`.
  - On press (`pressed: true`), store the `egui::Key` (or raw physical scancode) as the `last_press`. Call the emulator's equivalent of `keyDown(key)`.
  - On release (`pressed: false`), call the emulator's equivalent of `keyUp(key)`.
- **Native Character Input**: Match `egui::Event::Text` to intercept typable characters.
  - If a character is received, check if it has been mapped. If not, trigger the layout search using the active `last_press` key code, and call `keyPress(char)`.
- **Full-web Input**: Use raw DOM keyboard events because eframe 0.31 does not provide `physical_key` on web.
  - `KeyboardEvent.code` is translated to a stable legacy host-key identifier.
  - `KeyboardEvent.key` supplies the Unicode character used by dynamic mapping.
  - Repeated key-down events are ignored through `KeyboardEvent.repeat` and a held-key table keyed by `KeyboardEvent.code`.
  - `getModifierState("AltGraph")` maps right Alt/AltGr to host code `225`, separate from ordinary Alt.
  - Canvas blur, window blur, and document visibility loss reset the full TVC keyboard matrix.
  - Browser defaults are prevented only while the emulator canvas handles the key.

---

## Shift State Compensation (Modifiers)

Sometimes a character requires Shift on the host layout but is unshifted on the TVC layout (or vice-versa). The dynamic mapper uses flags to manipulate the TVC's Shift key (Row 6, Col 3) state automatically:

- **`KSADD` (Shift Add)**:
  Fires when a key requires the shifted representation on the TVC but the host key is unshifted. It programmatically holds the TVC Shift key down during the keystroke:
  ```javascript
  this.keySet(6, 3, true); // Force Shift pressed
  ```
- **`KSDEL` (Shift Delete)**:
  Fires when a key requires the unshifted representation on the TVC but the host key is shifted. It programmatically releases the TVC Shift key during the keystroke:
  ```javascript
  this.keySet(6, 3, false); // Force Shift released
  ```

---

## Key State Lifecycle and Release

The keyboard driver is driven by three platform-independent hooks:
- **Key Down (`keyDown`)**: Called when a raw physical key is pressed. Updates modifier states and stores the active scancode/keycode.
- **Text Input (`keyPress`)**: Intercepts typable Unicode characters to create new coordinate bindings.
- **Key Up (`keyUp`)**: Called when a raw physical key is released. Restores key states and handles release cleanup.

### Key Stick Prevention
If a user releases the host `Shift` key *before* releasing a character key, the modifier state changes. A naive mapping lookup during `keyup` would check `_keymap[0]` instead of `_keymap[SHIFT_ON]`, causing the release code to miss the active matrix cell and leave the key stuck "pressed" in the TVC matrix.

To prevent this, the key release logic iterates through **all modifier tables** (unshifted, shifted, alt, altgr) for the released key code, clearing the row/column bit in the matrix for every mapping found:

```javascript
if (!down) {
    Object.keys(this._keymap).forEach(function(k) {
        m = this._keymap[k][code];
        if (m) {
            this.keySet(m[0], m[1], down); // Safely release key in all mapped states
        }
    }, this);
}
```

The release path also reapplies `fixState(..., false)` for every mapping. This releases a synthesized TVC Shift (`KSADD`) and restores or releases compensated Shift state (`KSDEL`) even when the physical Shift key was released before the character key.
