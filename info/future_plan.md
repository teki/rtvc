# Future Plan — Build Targets and UI Strategy

This document records intended direction for future changes so implementation work preserves the planned native and WebAssembly build tiers.

## Goals

- Keep a normal local desktop build using the native egui UI.
- Keep a very lightweight WebAssembly build that excludes egui/eframe and uses a small `wasm-bindgen` API for a browser canvas UI.
- Support a full WebAssembly build that uses egui on the web and browser-local file storage.
- Keep both video models in [src/vid.rs](../src/vid.rs); choose the default with build features on web and with a runtime setting on native.
- Use [info/snapshot.md](snapshot.md) as the source of truth for snapshot state and web bundle commands.

## Build Tiers

### Local Native

- Default build path: `cargo run --bin rtvc`.
- Uses the `native` Cargo feature by default.
- Enables egui/eframe, native audio through `cpal`, and filesystem helpers such as zipped disk loading.
- Video model must remain a runtime setting in the UI, not a native build feature.

### Lightweight Web

- Intended command shape:
  ```bash
  cargo build --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown
  ```
- Must avoid egui, eframe, native filesystem assumptions, and large UI dependencies.
- Uses [src/wasm.rs](../src/wasm.rs) as the public browser-facing API.
- JavaScript owns browser UI, canvas presentation, Web Audio playback, keyboard event wiring, and file picker plumbing.
- Default video model is `VidModel::FastFrame` for WASM constructors.
- Snapshot upload bundles are produced with `cargo bundle-web <snapshot>`.
- Snapshot player skeletons without an embedded snapshot are produced with `cargo xtask bundle-web-skeleton [out-dir]`.

### Full Web

- Build check:
  ```bash
  cargo check --lib --no-default-features --features wasm-full --target wasm32-unknown-unknown
  ```
- Static bundle:
  ```bash
  cargo xtask bundle-web-full [out-dir]
  ```
- Release archives and the public `docs/` demo use this full web build.
- Uses the native egui/eframe application structure with browser-specific audio, storage, file dialogs, downloads, and keyboard plumbing.
- Uses an `AudioWorklet` for PCM playback. The browser audio context and worklet are initialized from a user gesture.
- Stores small configuration values in `localStorage`.
- Stores recent tape and disk bytes in IndexedDB, limited to five entries per media kind. Storage failures must be visible in the UI.
- Uses raw DOM keyboard events because eframe 0.31 does not expose physical keys on web. `KeyboardEvent.code` identifies the host key, `KeyboardEvent.key` supplies the layout-aware character, and `getModifierState("AltGraph")` distinguishes AltGr.
- Keeps byte-backed mounted media in `Emu` so changing machine type can restore browser-mounted tape and disk content.
- Must not force the lightweight web build to include egui/eframe, zip, IndexedDB helpers, or full-web UI code.
- Shares the same emulator core and runtime `VidModel` options as native and lightweight web.

## Video Model Policy

- Keep the runtime `VidModel` choices in [src/vid.rs](../src/vid.rs) unless the file becomes genuinely hard to maintain.
- `VidModel::FastFrame` is the small, direct framebuffer renderer: run one screen-time CPU budget, then render a full screen.
- `VidModel::Interleaved` is the streaming CRTC-timing renderer based on `stream_some()` and `render_stream()` after each CPU instruction.
- Interleaved mode must remain bounded by host screen time. If the CRTC stream does not produce sync for several consecutive host ticks, present the lost-sync black background with moving white stripes instead of waiting for a completed CRTC frame.
- Native builds expose video selection as a runtime setting.
- Web builds keep compatibility feature flags for existing commands:
  - `web-vid-simple`
  - `web-vid-realistic`
- These web video feature flags must remain mutually exclusive. WASM constructors still default to `VidModel::FastFrame`; callers can switch to interleaved mode through the runtime API.

## Feature Hygiene

- Keep default features suitable for local development.
- Require `--no-default-features` for web builds so accidental native dependencies are visible.
- Add browser-only dependencies behind explicit web/full-web features.
- Do not put native filesystem dependencies into the lightweight web tier.
- When adding dependencies, verify the lightweight web dependency tree still excludes cpal, egui, eframe, and zip unless the build tier intentionally changes.

## Validation Checklist

Run these before finishing work that affects build features, video, UI, or storage:

```bash
cargo check
cargo check --bins
cargo check --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown
cargo check --lib --no-default-features --features wasm,web-vid-realistic --target wasm32-unknown-unknown
cargo check --lib --no-default-features --features wasm-full --target wasm32-unknown-unknown
cargo check --manifest-path xtask/Cargo.toml
cargo tree --no-default-features --features wasm,web-vid-simple -e normal --target wasm32-unknown-unknown
```

The lightweight web tree should contain `wasm-bindgen` but not cpal, egui, eframe, or zip.
