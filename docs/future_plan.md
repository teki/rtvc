# Future Plan — Build Targets and UI Strategy

This document records intended direction for future changes so implementation work preserves the planned native and WebAssembly build tiers.

## Goals

- Keep a normal local desktop build using the native egui UI.
- Keep a very lightweight WebAssembly build that excludes egui/eframe and uses a small `wasm-bindgen` API for a browser canvas UI.
- Support a future full WebAssembly build that can use egui on the web and browser-local file storage.
- Keep both video models in [src/vid.rs](../src/vid.rs); choose the default with build features on web and with a runtime setting on native.

## Build Tiers

### Local Native

- Default build path: `cargo run --bin rtvc`.
- Uses the `native` Cargo feature by default.
- Enables egui/eframe and filesystem helpers such as zipped disk loading.
- Video model must remain a runtime setting in the UI, not a native build feature.

### Lightweight Web

- Intended command shape:
  ```bash
  cargo build --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown
  ```
- Must avoid egui, eframe, native filesystem assumptions, and large UI dependencies.
- Uses [src/wasm.rs](../src/wasm.rs) as the public browser-facing API.
- JavaScript owns browser UI, canvas presentation, keyboard event wiring, and file picker plumbing.
- Default video model should be `VidModel::Simple` unless explicitly built otherwise.

### Full Web

- Future tier for an egui web application.
- Should use browser storage, likely local storage or IndexedDB, for ROM/disk persistence.
- Should not force the lightweight web build to include egui/eframe or browser storage dependencies.
- Should share the same emulator core and `VidModel` options as native and lightweight web.

## Video Model Policy

- Keep `VidModel::Simple` and `VidModel::Realistic` in [src/vid.rs](../src/vid.rs) unless the file becomes genuinely hard to maintain.
- `VidModel::Simple` is the small, direct framebuffer renderer.
- `VidModel::Realistic` is the streaming CRTC-timing renderer based on `stream_some()` and `render_stream()`.
- Native builds expose video selection as a runtime setting.
- Web builds may choose constructor defaults through Cargo features:
  - `web-vid-simple`
  - `web-vid-realistic`
- These web video feature flags must remain mutually exclusive.

## Feature Hygiene

- Keep default features suitable for local development.
- Require `--no-default-features` for web builds so accidental native dependencies are visible.
- Add browser-only dependencies behind explicit web/full-web features.
- Do not put native filesystem dependencies into the lightweight web tier.
- When adding dependencies, verify the lightweight web dependency tree still excludes egui, eframe, and zip unless the build tier intentionally changes.

## Validation Checklist

Run these before finishing work that affects build features, video, UI, or storage:

```bash
cargo check
cargo check --bins
cargo check --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown
cargo check --lib --no-default-features --features wasm,web-vid-realistic --target wasm32-unknown-unknown
cargo tree --no-default-features --features wasm,web-vid-simple -e normal --target wasm32-unknown-unknown
```

The lightweight web tree should contain `wasm-bindgen` but not egui, eframe, or zip.
