# Coding Experiments

This directory contains small, tracked Z80 programs used to investigate TVC
hardware and emulator behavior. Keep generally useful source experiments here;
game-specific port code belongs in the standalone `tvc-ports` repository.

Write assembled CAS, TOML, BIN, screenshots, and other generated results under
the ignored `target/coding/` directory rather than committing them.

`hwscroll-chunky.asm` is a TVC-native R12/R13 hardware-scroll demo inspired by
Kevin Thacker's CPC examples
([forum thread](https://forum.cpcwiki.eu/t/hardware-scrolling-help/488)). The
CPC `scroll.zip` / `hwscroll_info.zip` downloads unpack under the gitignored
`tmp/hwscroll/` tree; they are reference material, not a drop-in TVC port.
It auto-scrolls as soon as `RUN` finishes; the border flashes each frame.
Click the Screen pane before using O/P/Q/A/Space or the cursors.

Compile numbered BASIC sources with `rtvc-basic`, and Z80 helper sources with
`rtvc-asm --format cas`:

```bash
mkdir -p target/coding
cargo run --bin rtvc-basic -- coding/crtc-register-explorer.bas -o target/coding/crtc-register-explorer.cas
```

`rtvc-tocas` compiles the same sources to sibling `.cas` files:

```bash
cargo run --bin rtvc-tocas -- coding/crtc-register-explorer.bas coding/crtc-register-explorer.asm
```
