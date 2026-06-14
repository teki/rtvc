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

Pane rendering is presentation-only. It does not add emulator ticks or change
the 50 Hz scheduler. The screen reuses the current texture between generated
TVC frames, and IO Log reads the existing capped 200-entry buffer.
