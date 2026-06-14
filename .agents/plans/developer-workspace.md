# Developer Workspace Roadmap

Status: in progress

The developer workspace is split into independently shippable phases. Keep this
roadmap and the active phase plan synchronized whenever scope or acceptance
criteria change.

## Phases

1. [Docking foundation](developer-workspace-phase-1.md) - complete
2. [Integrated debugger](developer-workspace-phase-2.md) - planned
3. [BASIC and assembly editors](developer-workspace-phase-3.md) - planned

## Shared Requirements

- The default user experience remains a simple emulator screen.
- Native and full-web share the egui workspace; lightweight WASM stays small.
- Open developer panes must not prevent real-time 50 Hz emulation.
- Later phases reuse the Phase 1 workspace and persistence model.
