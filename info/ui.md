# UI and Developer Workspace

The native and full-web frontends share [src/ui.rs](../src/ui.rs) and
[src/workspace.rs](../src/workspace.rs). `EmuApp` remains responsible for
emulator scheduling, audio draining, menus, the status bar, file operations,
and Gamebase. The workspace module owns view mode, pane identity, dock
rendering, keyboard capture, and layout persistence.

## Modes

- **Simple** is the first-launch default and renders only the PAL 4:3 TVC
  screen. Keyboard behavior matches the original UI.
- **Developer** uses `egui_dock`. Its default layout places the non-closeable
  Screen pane above the IO Log pane.

Use **View > Developer Workspace** to change modes. In Developer mode,
**View > Panes > IO Log** closes or reopens the log and **Reset Workspace**
restores the default layout. IO Log may also be tabbed, resized, docked, or
floated through normal dock interactions.

## Integrated Debugger

**View > Debugger Layout** opens the Phase 2 debugging arrangement. Individual
panes are also available under **View > Panes**:

- **CPU** shows registers, flags, interrupt state, clock, the four current MMU
  page mappings, and the current BASIC 1.2 symbol. Standard-machine video RAM
  is shown as `V`; Plus video banks are shown as `V0` through `V3`. Hover the
  mapping to see the raw paging register. It provides Run/Pause, Step, Step 10,
  Run to IRQ, and Reset. Run to IRQ stops when the Z80 accepts an interrupt and
  times out after two TVC frames if interrupts remain disabled.
- **Disassembly** follows PC by default or accepts a hexadecimal address. It
  shows instruction bytes and metadata, annotates mapped BASIC 1.2 ROM
  routines, and toggles breakpoints from the left marker.
- **Memory** renders a bounded read-only hex/ASCII view of mapped CPU memory or
  a selected raw RAM/video/ROM bank.
- **Breakpoints** adds, removes, clears, and navigates execution breakpoints.
- **ROM Symbols** searches the curated BASIC 1.2 database and navigates to
  disassembly, raw memory, or a breakpoint. BASIC 2.2 remains unavailable until
  its database is created.
- **Events** keeps a capped structured history of debugger controls,
  breakpoint hits, and optional ROM landmark trace events.

The dock and TCP debuggers share core stepping and breakpoint behavior, but the
dock UI acts directly on the emulator and is also available in full web.

## Keyboard Capture

Developer panes must not receive accidental TVC input. Click inside Screen to
capture the TVC keyboard. Escape, focus loss, hiding Screen behind another tab,
or clicking elsewhere releases all held TVC keys and modifier state. Simple
mode continues to route keyboard input directly to the machine.

## Persistence

Workspace state is versioned and separate from emulator preferences:

- Native: `rtvc-workspace.json` beside the active `rtvc.toml`.
- Full web: `rtvc_workspace_v1` in browser `localStorage`.
- Lightweight WASM: no egui workspace dependency or storage.

A valid saved document restores both Developer mode and its dock layout.
Invalid or incompatible data starts with the default Developer layout without
changing emulator preferences. Workspace changes are saved after mode/menu
actions, dock pointer interactions, and normal application exit.

## Performance Boundary

Pane rendering does not add emulator ticks or change the 50 Hz scheduler. The
screen reuses the current texture between generated TVC frames, IO Log reads
the existing capped 200-entry buffer, and debugger memory/disassembly ranges
are bounded. ROM tracing is installed only while Events is visible and its
trace toggle is enabled; the core skips trace lookup entirely otherwise.
