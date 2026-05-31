# Cassette Tape (.cas) Support in jstvc

This document describes how Cassette Tape (`.cas`) files are supported in the `jstvc` emulator.

---

## High-Speed Direct RAM Injection

By default, `jstvc` uses a **high-speed direct RAM injection** mechanism (HLE loading). This bypasses the slow tape read routines of the original machine, loading programs instantly.

### .cas File Structure
A Videoton TV Computer cassette image (`.cas`) is composed of a **144-byte header** followed by the raw program bytes:
1. **First 128 bytes**: Contains file and block metadata, including:
   - File flags/identifiers (e.g., `0x11, 0x00`).
   - The total block count (`blockcnt`).
   - The size in bytes of the last block (`blockbyt`).
2. **Next 16 bytes**: Contains metadata describing the application/payload size (`applen`).
3. **Payload (from offset 144)**: The actual program or BASIC data.

### Loader Implementation Details
The core loader logic is located in the [loadImg](file:///Users/teki/dev/jstvc/src/tvc.js#L86-L106) function inside [src/tvc.js](file:///Users/teki/dev/jstvc/src/tvc.js):

```javascript
TVC.prototype.loadImg = function(name, data) {
    var extension = name.slice(-4).toLowerCase();
    if (extension == ".cas") {
        var savemap = this._mmu.getMap();
        this._mmu.setMap(0xb0);
        for (var i = 144; i < data.length; i++) {
            this._mmu.w8(6639 + i - 144, data[i]);
        }
        this._mmu.setMap(savemap);
    }
    // ...
};
```

1. **Save Map**: The emulator queries the MMU for the current memory map configuration using `getMap()`.
2. **Switch MMU to All-RAM**: The MMU map value is set to `0xB0` via [setMap](file:///Users/teki/dev/jstvc/src/mmu.js#L94-L104) in [src/mmu.js](file:///Users/teki/dev/jstvc/src/mmu.js). This configuration maps RAM Pages 0, 1, 2, and 3 (`_u0`, `_u1`, `_u2`, `_u3`) directly across the entire 64KB Z80 address space.
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
* **Operation**: Determines if the cassette motor is running.

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
| **Silence** | N/A | N/A | N/A | `3,125,000` / sec | flatline mid-level signal |
| **Pre-sound** | 9 | `638` | `638` | `1,276` | pilot/preamble tone |
| **Sync** | 17 | `1205` | `1205` | `2,410` | block synchronization |
| **Bit 0** | 11 | `779` | `779` | `1,558` | data zero bit |
| **Bit 1** | 8 | `567` | `567` | `1,134` | data one bit |

---

## TVC Custom Checksum (CRC)

The TVC ROM uses a custom 16-bit CRC algorithm computed bit-by-bit.

### Bit CRC Update Algorithm (JavaScript)
```javascript
let crc = 0;

function updateCrc(bit) {
    const bh = (crc >>> 8) & 0xff;
    const al = (bit !== 0) ? 0x80 : 0x00;
    const xorAl = al ^ bh;
    const cy = (xorAl & 0x80) !== 0;

    if (cy) {
        crc ^= 0x0810; // TVC CRC-CCITT variant polynomial
    }
    crc = (crc << 1) & 0xffff;
    if (cy) {
        crc = (crc | 1) & 0xffff;
    }
}
```

---

## Serialization & Tape Blocks

Bytes are written to the bitstream **Least-Significant Bit first (LSB-first)**:
```javascript
function writeByte(b, calculateCrc = true) {
    for (let i = 0; i < 8; i++) {
        const bit = (b >>> i) & 1;
        writeBit(bit);
        if (calculateCrc) {
            updateCrc(bit);
        }
    }
}
```

### 1. Header Block Structure
* **Silence**: 2 seconds.
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
  13. Write `typecas` (file type byte from CAS offset `128`).
  14. Write word `lof` (file payload size: total CAS size minus `144`).
  15. Write `0x00` (not autostarted).
  16. Write 10 bytes of `0x00` (filler).
  17. Write `0x00` (version number = 0).
  18. Write `0x00` (not last sector).
  19. Write word `crc` (current CRC, low byte first).
* **Preamble After**: 5 pre-sound pulses.

### 2. Data Block Structure
* **Silence**: 1 second.
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
* **Silence**: 2 seconds.

---

## Integrating Into the Emulator

To run this inside the emulator, you can create a Javascript generator that parses the `.cas` file and yields the tape signal level dynamically, then hook it into the emulator's Z80 port-reading.

### JavaScript Bitstream Generator Class
```javascript
class TapeBitstreamGenerator {
    constructor(casData, filename = "PROGRAM") {
        this.data = casData;
        this.filename = filename.toUpperCase().slice(0, 16);
        this.intervals = []; // Array of { level, duration }
        
        // Timing constants in Z80 cycles
        this.CYCLES_SILENCE = 3125000;
        this.CYCLES_PRE_HIGH = 638;
        this.CYCLES_PRE_LOW = 638;
        this.CYCLES_SYNC_HIGH = 1205;
        this.CYCLES_SYNC_LOW = 1205;
        this.CYCLES_0_HIGH = 779;
        this.CYCLES_0_LOW = 779;
        this.CYCLES_1_HIGH = 567;
        this.CYCLES_1_LOW = 567;

        this.crc = 0;
        this.generate();
    }

    writeSilence(seconds) {
        this.intervals.push({ level: 0.5, duration: this.CYCLES_SILENCE * seconds });
    }

    writePre(count) {
        for (let i = 0; i < count; i++) {
            this.intervals.push({ level: 1, duration: this.CYCLES_PRE_HIGH });
            this.intervals.push({ level: 0, duration: this.CYCLES_PRE_LOW });
        }
    }

    writeSync() {
        this.intervals.push({ level: 1, duration: this.CYCLES_SYNC_HIGH });
        this.intervals.push({ level: 0, duration: this.CYCLES_SYNC_LOW });
    }

    writeBit(bit) {
        if (bit === 0) {
            this.intervals.push({ level: 1, duration: this.CYCLES_0_HIGH });
            this.intervals.push({ level: 0, duration: this.CYCLES_0_LOW });
        } else {
            this.intervals.push({ level: 1, duration: this.CYCLES_1_HIGH });
            this.intervals.push({ level: 0, duration: this.CYCLES_1_LOW });
        }
    }

    updateCrc(bit) {
        const bh = (this.crc >>> 8) & 0xff;
        const al = (bit !== 0) ? 0x80 : 0x00;
        const xorAl = al ^ bh;
        const cy = (xorAl & 0x80) !== 0;

        if (cy) {
            this.crc ^= 0x0810;
        }
        this.crc = (this.crc << 1) & 0xffff;
        if (cy) {
            this.crc = (this.crc | 1) & 0xffff;
        }
    }

    writeByte(b, calculateCrc = true) {
        for (let i = 0; i < 8; i++) {
            const bit = (b >>> i) & 1;
            this.writeBit(bit);
            if (calculateCrc) {
                this.updateCrc(bit);
            }
        }
    }

    writeWord(w, calculateCrc = true) {
        this.writeByte(w & 0xff, calculateCrc);
        this.writeByte((w >>> 8) & 0xff, calculateCrc);
    }

    generate() {
        if (this.data[0] !== 0x11) {
            throw new Error("Invalid CAS file: Missing standard 0x11 file identifier.");
        }

        const bsl = this.data[2];
        const bsh = this.data[3];
        const brl = this.data[4];
        const brh = this.data[5];
        const dfsize = (bsl + bsh * 256) * 128 + (brl + brh * 256);
        const payloadSize = dfsize - 144;

        const typecas = this.data[0x80];
        const casauto = this.data[0x83];

        const payload = this.data.slice(144, 144 + payloadSize);

        // --- 1. HEAD BLOCK ---
        this.writeSilence(2);
        this.writePre(10240);
        this.writeSync();

        this.writeByte(0x00, false);
        this.crc = 0;
        this.writeByte(0x6A);
        this.writeByte(0xFF); // head tmb
        this.writeByte(0x11); // non-buffered
        this.writeByte(0x00); // non writeprotected
        this.writeByte(0x01); // 1 sector in head
        this.writeByte(0x00); // sector number 0

        const bihs = 1 + this.filename.length + 16;
        this.writeByte(bihs);
        this.writeByte(this.filename.length);
        for (let i = 0; i < this.filename.length; i++) {
            this.writeByte(this.filename.charCodeAt(i));
        }
        this.writeByte(0x00); // fill byte
        this.writeByte(typecas);
        this.writeWord(payloadSize); // length of file
        this.writeByte(casauto);     // autostart

        for (let i = 0; i < 10; i++) {
            this.writeByte(0x00);
        }
        this.writeByte(0x00); // version number
        this.writeByte(0x00); // not last sector

        // write head CRC
        this.writeWord(this.crc, false);
        this.writePre(5);

        // --- 2. DATA BLOCK HEAD ---
        this.writeSilence(1);
        this.writePre(5120);
        this.writeSync();

        this.writeByte(0x00, false);
        this.crc = 0;
        this.writeByte(0x6A);
        this.writeByte(0x00); // data tmb
        this.writeByte(0x11); // non-buffered
        this.writeByte(0x00); // non-writeprotected

        const sectorCount = Math.floor(payloadSize / 256) + 1;
        this.writeByte(sectorCount);

        // --- 3. DATA SECTORS ---
        let payloadPtr = 0;
        for (let secnum = 1; secnum <= sectorCount; secnum++) {
            if (secnum > 1) {
                this.crc = 0;
            }

            this.writeByte(secnum);

            const isLast = (secnum === sectorCount);
            const size = isLast ? (payloadSize % 256) : 0;
            this.writeByte(size);

            if (size === 0) {
                for (let i = 0; i < 256; i++) {
                    this.writeByte(payload[payloadPtr++]);
                }
                this.writeByte(0x00); // standard sector end padding
            } else {
                for (let i = 0; i < size; i++) {
                    this.writeByte(payload[payloadPtr++]);
                }
                this.writeByte(0xff); // partial sector end padding
            }

            this.writeWord(this.crc, false);
        }

        this.writePre(5);
        this.writeSilence(2);
    }
}
```

### Emulator I/O Port Implementation

To support low-level cassette emulation, implement the following actions for the Z80 I/O instructions:

#### 1. Read Port `0x59` (IT Status / Tape Input)
Return the current high/low phase of the emulated tape wave on Bit 5.
```javascript
case 0x59:
    let tapeBit = 0;
    if (this._tapeMotorOn && this._tapePlayActive) {
        const elapsedCycles = this._clock - this._tapeStartClock;
        tapeBit = this.getTapeSignalAtCycle(elapsedCycles);
    }
    result = (tapeBit << 5) | 0x40 | this._pendIt;
    break;
```

#### 2. Access Ports `0x50` - `0x57` (Tape Output)
Any read (`IN`) or write (`OUT`) to this port range toggles the audio output wave state.
```javascript
// Inside writePort(addr, val) and readPort(addr) handlers:
if (addr >= 0x50 && addr <= 0x57) {
    this._tapeOutputFlipFlop = !this._tapeOutputFlipFlop;
    // Record transition time to generate output file/sound
}
```

#### 3. Write Port `0x05` (Motor Control)
Use Bits 7 and 6 to determine if the virtual cassette motor is running.
```javascript
case 0x05:
    // Bits 7 and 6 control tape motor
    this._tapeMotorOn = ((val & 0xc0) !== 0);
    break;
```
