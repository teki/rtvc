# Cassette Tape (.cas) Support

This document describes how Cassette Tape (`.cas`) files are supported in `rtvc`.

---

## High-Speed Direct RAM Injection

The emulator can use a **high-speed direct RAM injection** mechanism (HLE loading). This bypasses the slow tape read routines of the original machine, loading programs instantly.

### .cas File Structure
A Videoton TV Computer cassette image (`.cas`) is composed of a **144-byte header** followed by the raw program bytes:
1. **First 128 bytes**: Contains file and block metadata, including:
   - File flags/identifiers (e.g., `0x11, 0x00`).
   - The total block count (`blockcnt`).
   - The size in bytes of the last block (`blockbyt`).
2. **Next 16 bytes**: Contains metadata describing the application/payload size (`applen`).
3. **Payload (from offset 144)**: The actual program or BASIC data.

### Loader Implementation Details
The high-speed load path is handled by the media-loading code in [src/tvc.rs](../src/tvc.rs), with native and WASM entry points in [src/emu.rs](../src/emu.rs) and [src/wasm.rs](../src/wasm.rs):

```text
if extension == ".cas":
    save current MMU map
    switch MMU to all-RAM map 0xB0
    copy CAS payload bytes after the 144-byte header to RAM at 0x19EF
    restore previous MMU map
```

1. **Save Map**: The emulator queries the MMU for the current memory map configuration using `getMap()`.
2. **Switch MMU to All-RAM**: The MMU map value is set to `0xB0` through [src/mmu.rs](../src/mmu.rs). This configuration maps RAM Pages 0, 1, 2, and 3 directly across the entire 64KB Z80 address space.
3. **Direct Write**: The emulator skips the first 144 bytes of the `.cas` header. It then writes the remaining bytes byte-by-byte into the TVC's RAM starting at address **`6639` (`0x19EF`)**. Address `6639` is the default start address of the TVC BASIC program area (`TXTTAB`).
4. **Restore Map**: The emulator restores the MMU map to its original saved state.

### Program Execution
Once a `.cas` file is loaded into RAM, the user is notified with the tip `run + [enter]`.

Typing `run` causes the TVC BASIC interpreter to execute the BASIC code starting at address `6639`. A standard assembly/machine code game or application compiled for the TVC usually prepends a single tokenized line of BASIC at `6639`:
```basic
10 PRINT USR(6912)
```
When executed, this jumps directly to the entry point of the machine code program loaded at address `6912` (`0x1B00`).

---

## Hardware Tape Interface (Physical Bit-Banging)

On the original Videoton TV-Computer hardware, the tape loading and saving mechanisms are almost entirely software-driven ("bit-banging"). The hardware provides very little abstraction, meaning the CPU must manually construct and decode the audio frequencies.

### 1. Tape Input (Loading)
The audio signal coming from the cassette player is amplified, converted into a TTL digital signal (a square wave), and passed directly to the CPU as a single status bit.

* **I/O Port**: `59H` (or `5DH`) - Interrupt (IT) Status Word.
* **Bit Position**: Bit 5 (`ECIN` signal).
* **Operation**: To load data, the TVC software constantly polls Bit 5 of port `59H` in a tight loop and measures the time (in Z80 cycles) between state transitions (the frequency of the wave). Depending on the density of the transitions, the software determines whether the incoming wave represents a logical 0 or a logical 1 (using Frequency Shift Keying / FSK).

### 2. Tape Output (Saving)
To save data, the software must generate the frequency-modulated signal itself by toggling a flip-flop.

* **I/O Port**: `50H` to `57H` (Any access in this range generates the `NMFCK` signal).
* **Operation**: Any `OUT` (write) or `IN` (read) command sent to an address in the `50H`–`57H` range toggles the state of the audio output flip-flop to the opposite state. The software handles the timing of these accesses to create the faster or slower frequencies required for the 0s and 1s.

### 3. Motor Control
The tape recorder's motor is controlled by a 2-bit register driving high-current transistors.

* **I/O Port**: `05H` (Write only).
* **Bit Position**: Bits 7 and 6.
* **Operation**: Determines if the cassette motor is running. A low bit means the corresponding motor output is off; a high bit means on. The emulated tape transport position advances only while one of these motor bits is on.

---

## Physical Tape Bitstream Emulation

For a cycle-accurate, low-level tape simulation, a `.cas` file can be converted into a digital bitstream read by the Z80 CPU through Bit 5 of Port `0x59`.

### The Tape Signal Model
Rather than generating an intermediate heavy `WAV` audio file, the emulator can model the tape as a sequence of **half-period pulse intervals**, where each interval has:
1. A **signal level** (`0` for low, `1` for high, or `0.5` for silence/middle).
2. A **duration** in Z80 clock cycles.

#### Timing Calculations
* **Z80 Clock Frequency**: 3,125,000 Hz.
* **Standard Audio Sample Rate**: 44,100 Hz.
* **Cycles per Sample**: `3,125,000 / 44,100 ≈ 70.861678` Z80 cycles.

Based on the original TVC loader timings (derived from the `cas2wav` utility), the cycles for each pulse state are:

| Signal Type | Phase Samples (44.1 kHz) | High Phase (Z80 Cycles) | Low Phase (Z80 Cycles) | Total Period (Z80 Cycles) | Description |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **Silence** | 22,052 per nominal second | N/A | N/A | `22,052 * 3,125,000 / 44,100` / nominal sec | flatline mid-level signal matching the legacy converter |
| **Pre-sound** | 9 | `638` | `638` | `1,276` | pilot/preamble tone |
| **Sync** | 17 | `1205` | `1205` | `2,410` | block synchronization |
| **Bit 0** | 11 | `779` | `779` | `1,558` | data zero bit |
| **Bit 1** | 8 | `567` | `567` | `1,134` | data one bit |

---

## TVC Custom Checksum (CRC)

The TVC ROM uses a custom 16-bit CRC algorithm computed bit-by-bit.

### Bit CRC Update Algorithm
```text
crc = 0

update_crc(bit):
  bh = high byte of crc
  al = 0x80 when bit is 1, otherwise 0x00
  carry = ((al xor bh) bit 7) != 0
  if carry:
    crc = crc xor 0x0810
  crc = (crc << 1) & 0xffff
  if carry:
    crc = (crc | 1) & 0xffff
```

---

## Serialization & Tape Blocks

Bytes are written to the bitstream **Least-Significant Bit first (LSB-first)**:
```text
write_byte(byte, calculate_crc):
  for bit_index in 0..8:
    bit = (byte >> bit_index) & 1
    write_bit(bit)
    if calculate_crc:
      update_crc(bit)
```

### 1. Header Block Structure
* **Silence**: 2 nominal seconds (44,104 samples in legacy `cas2wav` output).
* **Pre-sound Preamble**: 10,240 pulses.
* **Sync Pulse**: 1 pulse.
* **Data Bytes**:
  1. Write `0x00` (CRC is **not** calculated for this byte).
  2. Reset CRC (`crc = 0`).
  3. Write `0x6A` (Start byte; CRC starts here).
  4. Write `0xFF` (Block ID/type: Head block).
  5. Write `0x11` (Non-buffered file flag).
  6. Write `0x00` (Non-writeprotected).
  7. Write `0x01` (Sector count in head block).
  8. Write `0x00` (Sector number = 0).
  9. Write `bihs` (Bytes in sector: `1 + filenameLength + 16`).
  10. Write `filenameLength` (max 16).
  11. Write filename characters (padded/up to `filenameLength` bytes).
  12. Write `0x00` (fill byte).
  13. Write `typecas` (file type byte from CAS offset `0x81`, matching the legacy converter's one-byte-over seek).
  14. Write word `lof` (file payload size: total CAS size minus `144`).
  15. Write `0x00` (not autostarted).
  16. Write 10 bytes of `0x00` (filler).
  17. Write `0x00` (version number = 0).
  18. Write `0x00` (not last sector).
  19. Write word `crc` (current CRC, low byte first).
* **Preamble After**: 5 pre-sound pulses.

### 2. Data Block Structure
* **Silence**: 1 nominal second (22,052 samples in legacy `cas2wav` output).
* **Pre-sound Preamble**: 5,120 pulses.
* **Sync Pulse**: 1 pulse.
* **Data Block Head**:
  1. Write `0x00` (CRC is **not** calculated for this byte).
  2. Reset CRC (`crc = 0`).
  3. Write `0x6A` (Start byte).
  4. Write `0x00` (Block ID/type: Data block).
  5. Write `0x11` (Non-buffered file flag).
  6. Write `0x00` (Non-writeprotected).
  7. Write `sectorCount` (number of sectors: `Math.floor(payloadSize / 256) + 1`).
  * *Note: No CRC is written for the Data Block Head itself. It transitions immediately into the first sector.*

### 3. Data Sectors
Sectors are written sequentially (from sector number `1` up to `sectorCount`):

* **CRC Management**:
  * For Sector `1`: The CRC is **not** reset; it continues from the Data Block Head.
  * For Sectors `2` and onwards: The CRC **is** reset (`crc = 0`) at the start of the sector.
* **Sector Data**:
  1. Write `secnum` (starts at `1`).
  2. Write `size` (`0` for a full 256-byte sector, or `payloadSize % 256` for the last partial sector).
  3. Write sector payload bytes:
     - If `size === 0`: Write 256 bytes, followed by `0x00` (filler byte).
     - If `size > 0`: Write `size` bytes, followed by `0xFF` (filler byte).
  4. Write word `crc` (current CRC, low byte first).

After the final sector is written:
* **Preamble After**: 5 pre-sound pulses.
* **Silence**: 2 nominal seconds (44,104 samples in legacy `cas2wav` output).

---

## Integrating Into the Emulator

The native code currently converts CAS files through [src/cas.rs](../src/cas.rs) and [src/cas2wav.rs](../src/cas2wav.rs). Low-level tape playback can use the same structure in memory: parse the `.cas` header, generate an ordered list of signal intervals in Z80 cycles, and have the TVC bus sample the current level when the ROM reads the tape input port.

Condensed interval-generation flow:

1. Validate byte `0x00` is `0x11`.
2. Compute `payload_size = ((cas[2] + cas[3] * 256) * 128 + (cas[4] + cas[5] * 256)) - 144`.
3. Emit the header block: 2 seconds silence, 10,240 pre-sound pulses, one sync pulse, the header bytes, header CRC, and 5 trailing pre-sound pulses.
4. Emit the data block head: 1 second silence, 5,120 pre-sound pulses, one sync pulse, marker bytes, and sector count.
5. Emit each sector with its sector number, size byte, payload bytes, filler byte, and CRC.
6. Finish with 5 pre-sound pulses and 2 seconds silence.

### Emulator I/O Port Implementation

To support low-level cassette emulation, implement the following actions for the Z80 I/O instructions:

#### 1. Read Port `0x59` (IT Status / Tape Input)
Return the current high/low phase of the emulated tape wave on Bit 5.
```text
if port == 0x59:
  tape_bit = tape signal level when motor and playback are active, otherwise 0
  return (tape_bit << 5) | 0x40 | pending_interrupt_bits
```

#### 2. Access Ports `0x50` - `0x57` (Tape Output)
Any read (`IN`) or write (`OUT`) to this port range toggles the audio output wave state.
```text
if 0x50 <= port <= 0x57:
  flip tape output state
  record transition time for tape output capture
```

#### 3. Write Port `0x05` (Motor Control)
Use Bits 7 and 6 to determine if the virtual cassette motor is running.
```text
if port == 0x05:
  tape_motor_on = (value & 0xc0) != 0
  advance tape transport cycles only while the motor is on
```
