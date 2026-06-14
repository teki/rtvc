# Socket Debugger Interface Spec & Usage Guide

The `rtvc` emulator provides a TCP-based debugger interface on `localhost` (127.0.0.1) under native execution. It allows external scripts, test suites, or AI agents to control the emulator, query its state, and step the CPU.

The debugger works in both **headless execution mode** (`--headless`) and standard **native GUI mode**.

---

## Command Line Configuration

The socket debugger port and execution mode can be configured via CLI flags:

- `-p`, `--port <port>`: Specifies the TCP port to bind (default: `8080`).
- `-H`, `--headless`: Runs the emulator without initiating the `egui`/`eframe` GUI loop, running entirely in a background polling loop.

For example, to start the emulator with GUI and the debugger on port `8089`:
```bash
cargo run --bin rtvc -- --port 8089
```

To run a headless emulator listening on port `8080`:
```bash
cargo run --bin rtvc -- --headless --port 8080
```

---

## TCP Protocol Specification

The TCP debugger communicates using **newline-delimited JSON objects**. Every request sent by the client must consist of a single JSON object terminated by a newline (`\n`). The emulator will respond with a single JSON object terminated by a newline.

### 1. Commands

#### `status`
Queries the Z80 CPU registers, cycle counter, and current execution states.
- **Request**: `{"cmd": "status"}`
- **Response**:
  ```json
  {
    "status": "ok",
    "running": false,
    "halted": false,
    "cycles": 124508,
    "pc": 3450,
    "sp": 65535,
    "af": 65535,
    "bc": 0,
    "de": 0,
    "hl": 16384,
    "ix": 65535,
    "iy": 65535
  }
  ```

#### `stats`
Reports completed emulation frames over a rolling five-second host-time window. The initial window is shorter until the emulator has been open for five seconds; paused time is included, so the average falls when emulation cannot sustain real time or is paused.
- **Request**: `{"cmd": "stats"}`
- **Response**:
  ```json
  {
    "status": "ok",
    "running": true,
    "average_fps": 49.8,
    "window_seconds": 5.0,
    "frames": 249
  }
  ```

#### `close_app`
Closes the emulator application. GUI mode performs the normal application shutdown path, including saving application state; headless mode exits its run loop.
- **Request**: `{"cmd": "close_app"}`
- **Response**: `{"status": "ok"}`

#### `step`
Executes one or more Z80 CPU instructions. This automatically updates system timers, tape playback, sound generation, and clock cycles.
- **Request**: `{"cmd": "step", "count": 5}` (where `"count"` is an optional integer, defaulting to `1`)
- **Response**: `{"status": "ok"}`

#### `continue`
Resumes real-time CPU emulation.
- **Request**: `{"cmd": "continue"}`
- **Response**: `{"status": "ok"}`

#### `pause`
Pauses real-time CPU emulation.
- **Request**: `{"cmd": "pause"}`
- **Response**: `{"status": "ok"}`

#### `reset`
Performs a hardware reset on the CPU, MMU, and peripherals, pausing the CPU.
- **Request**: `{"cmd": "reset"}`
- **Response**: `{"status": "ok"}`

#### `breakpoint_add`
Adds an execution breakpoint at a specific 16-bit address.
- **Request**: `{"cmd": "breakpoint_add", "addr": 256}`
- **Response**: `{"status": "ok"}`

#### `breakpoint_remove`
Removes an execution breakpoint from a specific 16-bit address.
- **Request**: `{"cmd": "breakpoint_remove", "addr": 256}`
- **Response**: `{"status": "ok"}`

#### `breakpoint_list`
Lists all currently active breakpoints.
- **Request**: `{"cmd": "breakpoint_list"}`
- **Response**: `{"status": "ok", "breakpoints": [256, 1024]}`

#### `read_memory`
Reads raw or mapped memory.
- **Request**: `{"cmd": "read_memory", "addr": 0, "len": 4, "bank": "sys"}`
  - `"addr"`: Starting 16-bit address.
  - `"len"`: Number of bytes to read.
  - `"bank"`: (Optional string) Read directly from a specific physical memory bank, bypassing the active MMU page mappings. Available banks:
    - RAM banks: `"u0"`, `"u1"`, `"u2"`, `"u3"`
    - Video RAM banks: `"vid0"`, `"vid1"`, `"vid2"`, `"vid3"`
    - Boot ROM: `"sys"`
    - Cartridge ROM: `"cart"`
    - Expansion ROM: `"exth"`
- **Response**: `{"status": "ok", "data": [195, 41, 2, 0]}`

#### `disassemble`
Decodes Z80 instructions into assembly mnemonics.
- **Request**: `{"cmd": "disassemble", "addr": 0, "len": 4}`
- **Response**:
  ```json
  {
    "status": "ok",
    "instructions": [
      { "addr": 0, "bytes": [195, 41, 2], "len": 3, "text": "JP 0229H" },
      { "addr": 3, "bytes": [0], "len": 1, "text": "NOP" }
    ]
  }
  ```

#### `assemble`
Encodes one Z80 instruction without changing emulated memory. The address is
used to calculate `JR` and `DJNZ` displacements.
- **Request**: `{"cmd": "assemble", "addr": 32768, "source": "LD A,42"}`
- **Response**:
  ```json
  {
    "status": "ok",
    "addr": 32768,
    "len": 2,
    "bytes": [62, 42],
    "next_addr": 32770
  }
  ```

#### `save_snapshot` / `load_snapshot`
Saves or loads a compressed/raw emulator state snapshot.
- **Request**: `{"cmd": "save_snapshot", "path": "data/snapshots/save.rtvcsnap.zip"}`
- **Request**: `{"cmd": "load_snapshot", "path": "data/snapshots/save.rtvcsnap.zip"}`
- **Response**: `{"status": "ok"}` (or `{"status": "error", "message": "..."}`)

#### `save_screenshot`
Generates a 4:3 stretched PNG image of the current TVC display framebuffer.
- **Request**: `{"cmd": "save_screenshot", "path": "screenshot.png"}`
- **Response**: `{"status": "ok"}`

#### `key`
Simulates a host keyboard event.
- **Request**: `{"cmd": "key", "action": "press", "char": "A"}`
- **Request**: `{"cmd": "key", "action": "down", "code": 65}`
- **Request**: `{"cmd": "key", "action": "up", "code": 65}`
  - `"action"`: `"down"` (key down), `"up"` (key up), or `"press"` (character typing).
  - `"code"`: JavaScript keycode integer (required for `"down"` and `"up"` actions).
  - `"char"`: Character string to type (required for `"press"` action).
- **Response**: `{"status": "ok"}`

---

### 2. Asynchronous Event Notifications

When the emulator is in running state (`"running": true`) and hits an active breakpoint, it will transition to paused state and push an asynchronous JSON event onto the TCP stream to notify the client:

```json
{"event": "breakpoint", "pc": 256}
```

---

## Interactive Python REPL Client

An interactive CLI client interface is provided in [rtvc_debug.py](../scripts/rtvc_debug.py). It includes tab completion, history, register grid displays, and hex dumps.

### Launching the Client
To run the client and connect to the default local debugger (`127.0.0.1:8080`):
```bash
python3 scripts/rtvc_debug.py
```

To connect to a custom host or port:
```bash
python3 scripts/rtvc_debug.py --host 127.0.0.1 --port 8089
```

### REPL Commands

| Shell Command | Alias | Description |
|---|---|---|
| `status` | `s` | Print register grid (AF, BC, DE, HL, IX, IY, SP, PC), cycles, halted, and running states. |
| `stats` | `fps` | Print average emulation FPS over the recent rolling five-second window. |
| `close_app` | `close` | Close the emulator and exit the debugger shell. |
| `step [count]` | `t` | Step the Z80 CPU by `count` instructions and show register status. |
| `continue` | `c` | Resume real-time emulator execution. |
| `pause` | `p` | Pause CPU execution. |
| `reset` | | Reset the emulator hardware. |
| `bp list` | | List all active breakpoints. |
| `bp add <addr>` | | Add execution breakpoint at address (decimal or hex `0x...`). |
| `bp rm <addr>` | | Remove execution breakpoint. |
| `read <addr> <len> [bank]` | `m` | Hex dump a memory range (with ASCII viewer panel). |
| `disasm <addr> <len>` | `d` | Disassemble memory instructions. |
| `asm [addr]` | `a` | Enter interactive single-line assembler mode at `addr`, or at the current PC when omitted, and print encoded bytes. |
| `save <path>` | | Save snapshot to path. |
| `load <path>` | | Load snapshot from path. |
| `screenshot <path>` | | Save 4:3 PNG screenshot of the framebuffer. |
| `key press <char>` | | Send a character keyboard press simulation. |
| `key down <val>` | | Send a key code down simulation. |
| `key up <val>` | | Send a key code up simulation. |
| `exit` | `q` | Exit the REPL console. |
