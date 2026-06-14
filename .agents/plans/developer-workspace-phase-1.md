# Phase 1: Docking Foundation

Status: complete

## Deliverables

- Optional `egui_dock` workspace for native and full-web.
- Screen and existing IO Log panes.
- Simple first-launch mode and remembered developer layout.
- Explicit Screen keyboard capture with Escape release.
- Versioned workspace persistence separate from `rtvc.toml`.

## Acceptance

- Simple mode preserves the current screen presentation and keyboard behavior.
- Screen is non-closeable; IO Log can be closed, reopened, docked, and floated.
- Developer mode and layout survive restart.
- Invalid persisted state falls back to the default developer layout.
- Existing 200-entry log behavior remains unchanged.
- Native, full-web, and both lightweight WASM checks pass.
- IO Log can remain visible while the emulator sustains 50 FPS.

## Implementation Record

- Added `egui_dock` 0.16 to native and full-web only.
- Added [src/workspace.rs](../../src/workspace.rs) for modes, tabs, rendering,
  keyboard capture, JSON persistence, and layout recovery.
- Added Simple/Developer view controls, pane reopen/reset actions, and separate
  native/full-web workspace storage.
- Added serialization, version rejection, fallback, pane reopening, input
  policy, and native restart tests.
- Added [info/ui.md](../../info/ui.md) and updated architecture, user, machine,
  and development documentation.

## Validation Record

- `cargo fmt --check`
- `cargo check` and `cargo check --bins`
- `cargo test --lib` (62 tests)
- Full-web WASM check
- Both lightweight WASM compatibility checks
- `xtask` check
- Native debugger stats held 50.0 FPS over a five-second window.
- Full-web manual run held 50-51 FPS with IO Log visible, no browser warnings
  or audio-status errors, and restored Developer mode/layout after reload.
- Manual checks covered Screen capture, Escape release, IO Log close/reopen,
  and Reset Workspace.
