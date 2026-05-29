# rtvc

`rtvc` is a Rust emulator for the Videoton TV Computer (TVC).

## Run the Native Emulator

```bash
cargo run --bin rtvc
```

Place ROM files in `roms/` before running:

- `TVC12_D3.64K`
- `TVC12_D4.64K`
- `TVC12_D7.64K`

Optional program zip files can go in `progs/`.

## Native Snapshots

The native GUI has snapshot buttons:

- `Save Snapshot` writes a compressed `.rtvcsnap.zip` file by default.
- `Load Snapshot` reads either `.rtvcsnap.zip` or raw `.rtvcsnap` files.
- `Save Screenshot` writes the current TVC framebuffer as a 4:3 PNG (`768x576`).

Compressed snapshots are ordinary zip files containing one `snapshot.rtvcsnap` entry.

## Build a Web Snapshot Bundle

Install the matching `wasm-bindgen` CLI once:

```bash
cargo install wasm-bindgen-cli --version 0.2.122
```

Bundle a snapshot for static web hosting:

```bash
cargo bundle-web path/to/game.rtvcsnap.zip
```

Equivalent form:

```bash
cargo xtask bundle-web path/to/game.rtvcsnap.zip
```

The command writes a static bundle to:

```text
dist/<snapshot-name>-web/
```

If the input snapshot is zipped, the generated bundle keeps it zipped to reduce upload size. The browser loader decompresses it before passing state to the lightweight WASM emulator.

Serve the generated directory with any static web server.

## Useful Checks

```bash
cargo check
cargo check --bins
cargo check --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown
```

## Developer Docs

- [Project overview](docs/project_overview.md)
- [Future build/UI plan](docs/future_plan.md)
- [Snapshot format](docs/snapshot.md)
- [Development workflow](.agents/skills/development/SKILL.md)
