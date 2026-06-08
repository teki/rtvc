# rtvc

Language: [English](README.md) | [Magyar](README.hu.md)

`rtvc` is an open source, cross-platform emulator for the Videoton TV Computer
(TVC), a Hungarian 8-bit home and school computer from the 1980s.

The emulator is still in active development. It can already run TVC 64K and
64K+ machines with keyboard input, video, sound, cassette loading, HBF/VT-DOS
disk images, snapshots, and a native desktop UI.

Try the browser demo: [teki.one/rtvc](http://teki.one/rtvc/)

## About the TVC

The Videoton TV Computer, usually shortened to TVC, was a Hungarian 8-bit home
and school computer produced by Videoton in the second half of the 1980s. It
used a Z80 CPU, built-in BASIC, cassette storage, graphics modes instead of a
separate text-only display mode, and optional expansions such as floppy disk
support.

More historical background is available on the
[VIDEOTON TVC website](http://tvc.hu/html/tvc_attekintes.html) (Hungarian).

## Features

- TVC 64K and 64K+ machine variants.
- ROM 1.2 and ROM 2.2 machine selections, with optional VT-DOS/HBF extension.
- Z80 CPU emulation with FUSE and ZEX validation test harnesses.
- MC6845-based video output with fast-frame and interleaved rendering modes.
- Native keyboard, video, and sound through the desktop UI.
- CAS cassette loading and DSK disk image support.
- Snapshot save/load using `.rtvcsnap` and `.rtvcsnap.zip`.
- Static web snapshot player and full browser-based egui web application.
- TCP socket debugger for native GUI and headless use.

## Download

Download the latest release from the
[GitHub Releases page](https://github.com/teki/rtvc/releases).

Release archives are available for:

- Windows x64
- macOS x64
- macOS Apple Silicon

Extract the archive and run:

- `rtvc.exe` on Windows
- `RTVC.app` on macOS

The release packages include the emulator, ROM files, bundled programs, and a
static `web/` snapshot player.

### macOS First Launch

The macOS app is ad hoc signed, not notarized. If macOS blocks it after
download, remove the browser quarantine flag from the extracted app:

```bash
xattr -dr com.apple.quarantine RTVC.app
```

You can also download the archive from Terminal, which usually avoids the
browser quarantine flag:

```bash
curl -L https://github.com/teki/rtvc/releases/latest/download/rtvc-macos-arm64.zip | ditto -x -k - $HOME/Downloads/rtvc
```

## Using the Emulator

The native app provides menus for selecting the machine type, loading cassette
or disk images, saving and loading snapshots, saving screenshots, and showing
the I/O log.

Supported user files:

| File type | Purpose |
| --- | --- |
| `.cas` | TVC cassette image. |
| `.dsk` | Floppy disk image for HBF/VT-DOS. |
| `.zip` | Program archive containing a `.cas` or `.dsk` file. |
| `.rtvcsnap` | Raw rtvc snapshot. |
| `.rtvcsnap.zip` | Compressed rtvc snapshot. |

Snapshots are the easiest way to preserve the current machine state. The native
app can save compressed `.rtvcsnap.zip` files, load `.rtvcsnap` or
`.rtvcsnap.zip` files, and start directly from a snapshot path.

## Run From Source

Install a recent Rust toolchain, then run:

```bash
cargo run --bin rtvc
```

Start from a snapshot:

```bash
cargo run --bin rtvc -- snapshots/load_tape.rtvcsnap.zip
```

Load media on startup:

```bash
# Mount a floppy disk image
cargo run --bin rtvc -- -d path/to/disk.dsk

# Mount a cassette tape for standard loading
cargo run --bin rtvc -- -t path/to/tape.cas

# Inject a cassette tape directly into memory
cargo run --bin rtvc -- -i path/to/tape.cas
```

When running from source, place ROM files in `roms/`. Optional program archives
and media files can go in `progs/`.

## Web Player

The release archive includes a small static web snapshot player. To use it,
copy a compressed snapshot into `web/snapshot.rtvcsnap.zip`, serve the `web/`
directory, and open it in a browser:

```bash
cd web
python -m http.server 8000
```

Developers can build the full egui web application with:

```bash
cargo install wasm-bindgen-cli --version 0.2.122
cargo xtask bundle-web-full
cd dist/rtvc-web-full
python -m http.server 8000
```

The full web build can open local CAS, DSK, ZIP, and snapshot files. Small
preferences use `localStorage`; recent tape and disk bytes use IndexedDB.

## Developer Notes

Useful commands:

```bash
cargo build
cargo run --bin fuse_test
cargo run --bin perf_test
```

The socket debugger is available in both native GUI and headless modes:

```bash
# Native UI with debugger on port 8089
cargo run --bin rtvc -- -p 8089

# Headless emulator with debugger on port 8080
cargo run --bin rtvc -- -H -p 8080
```

For the full development workflow, see
[.agents/skills/development/SKILL.md](.agents/skills/development/SKILL.md).

## Documentation

- [Project overview](info/project_overview.md)
- [Snapshot format and web bundles](info/snapshot.md)
- [TVC machine core](info/tvc.md)
- [Z80 CPU](info/z80.md)
- [Z80 opcode reference](info/z80opcodes.md)
- [Memory management unit](info/mmu.md)
- [Video controller](info/vid.md)
- [Sound](info/sound.md)
- [Keyboard matrix](info/key.md)
- [Cassette support](info/cas.md)
- [HBF floppy card and FD1793 controller](info/hbf.md)
- [Socket debugger](info/dbg.md)

## Contributing

Issues and pull requests are welcome. Emulator accuracy reports are most useful
when they include a small reproduction: the machine type, media file, snapshot,
command typed on the TVC, and any relevant port or interrupt logs.

Please keep emulator behavior changes covered by focused tests where practical,
and update the documentation in `info/` when changing core architecture,
snapshot format, media handling, or build workflows.

## Acknowledgements

`rtvc` was ported from the earlier JavaScript implementation
[teki/jstvc](https://github.com/teki/jstvc). The CPU test flow uses public Z80
validation material such as FUSE and ZEX test programs. The project also relies
on historical TVC hardware information and preservation material.

## License

The emulator code is licensed under the [MIT License](LICENSE).

ROMs, cassette/disk images, snapshots, screenshots, manuals, and other
historical or third-party machine materials are included for preservation,
compatibility testing, or convenience where present. They are not covered by the
MIT license unless explicitly stated.
