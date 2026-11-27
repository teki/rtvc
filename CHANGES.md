# Changes

Release notes start here. Older release history is intentionally not backfilled.

## Unreleased

- Linux x64 and arm64 release archives (`rtvc-linux-<arch>.tar.gz`) with the
  emulator, command-line tools, ROMs, programs, documentation and web app.

## v0.8.12 - 2026-11-27

- Dust Off and other games that split the screen with raster interrupts keep
  a steady picture. The TVC cursor interrupt now fires on every cursor raster
  from R10 to R11, not only on R10. The TV model ignores VSYNC pulses that
  come too early, as a real TV's vertical hold does, and free-runs when a game
  stops VSYNC between screens, instead of rolling or showing lost-sync
  stripes.
- No more tearing in scrolling games: the TVC picture is shown only once the
  TV has finished drawing it. Before, a host frame could show the top of a new
  picture over the bottom of the previous one, with the split line slowly
  drifting.
- Each machine runs at its real frame rate: a TVC step is one TV picture
  (49.76 Hz for a standard field) instead of a fixed 1/50 s, and the desktop,
  headless and web hosts pace by the time each step covered, so every host
  frame has a new picture.
- Files without an extension (download links such as
  `?run=https://example.com/game-cas`) are recognised by content as CAS,
  DSK or ZIP.
- TCP debugger: `run_frames` runs a given number of frames at full speed and
  replies when they are done (`frames` in `rtvc_debug.py`); headless mode no
  longer runs slow on macOS (it ran at about 39 FPS). `video_state` reports
  the CRTC registers, the TV model's sync lock and recent VSYNC intervals,
  and `stats` adds synced and unsynced frame counts. `instruction_trace_start`
  takes `"beam": true` to record the CRTC beam and selected register per
  instruction; `instruction_trace_list` pages with `after_sequence` and
  filters to port writes with `port_writes_only` (`trace start beam`,
  `trace list ports`). Large replies no longer hang the client.
- rtvc-asm: macros, structs, local labels, full expressions, INCLUDE/INCBIN/
  IMPORT, PHASE, BREAK, and runnable CAS output at any ORG.
- The Gamebase details pane shows a Web link that opens and starts the
  selected game in the web emulator at teki.one/rtvc.
- Web on phones: a keyboard button on touch screens opens the phone's
  on-screen keyboard and types into the TVC, including Backspace, Return, and
  Android word suggestions. The TVC screen moves above the keyboard while it
  is open.
- The lightweight web player runs at the TVC's own frame rate on any
  display. Before, 60–120 Hz screens ran games too fast. It also types letters
  and punctuation from a physical keyboard, which it previously ignored.
- The full web build no longer slows the TVC down when the browser repaints
  unevenly: missed frames are caught up (phones dropped to about 45 FPS).
- The status bar starts with Reset, a Run/Pause button, and FPS (with repaint
  rate and emulation time), so they show on a portrait phone. The tape light,
  name, and progress are merged into one entry.
- The full web build starts in Fast frame on phones and tablets, unless a
  video model was chosen before; Interleaved is too slow for many phones.


## v0.8.11 - 2026-11-05

- The web build can ship tapes and disks: CAS, DSK, or ZIP files in
  `docs/bundled/` appear in the web Tape and Disk menus. Laser Squad is the
  first bundled disk.
- Web links can start a game: `?run=<file>` opens a bundled name, relative
  path, or URL the same way as dropping the file on the page.
- The assembler repeats blocks with `REPT count[, counter]` … `ENDR`
  (nestable, counts may use symbols defined above) and checks layout with
  `ASSERT condition[, "message"]`, which reports both sides on failure.
- The assembler accepts the undocumented Z80 forms the disassembler prints:
  `IXH`/`IXL`/`IYH`/`IYL` operands and indexed rotate, shift, `RES`, and `SET`
  instructions that copy the result to a register.
- Several emulator instances can run at once: if the debugger port is busy,
  the next free port is used and printed.
- BASIC 1.2 ROM listings: code hidden in `DB` lines or decoded off its real
  instruction boundaries is now proper instructions, operands use labels and
  named U0 variables, and inline `RST` operands, inline strings, and BASIC
  function records are decoded in comments. The interrupt path, the RST 30H
  dispatcher, the kernel functions, the keyboard, printer, sound, and video
  drivers, the cassette tape layer, and the BASIC runtime core are commented
  instruction by instruction, and
  the jump tables name their targets. The listings record a ROM bug: the
  block I/O helpers crash instead of returning error FAH for a block above
  HI-MEM.

## v0.8.10 - 2026-10-15

- Warp: a status-bar checkbox runs the emulator at full speed while a tape is
  loading, with audio muted, and stops by itself when
  the load ends. The TCP debugger (`warp`, and `warp on`/`off`/`wait` in
  `rtvc_debug.py`) and headless mode support it too.
- Tape progress counts playing time, so a long silence at the start of a tape
  no longer sits at 0%.
- Snapshots switch to the machine they were saved on; TVC snapshot files are
  unchanged.

## v0.8.9 - 2026-10-14

- Drag a CAS, DSK, or ZIP onto the emulator window (desktop or web) to boot
  straight into TVC 1.2 VT-DOS and run it. Dropped snapshots load directly.
- Web builds link with an 8 MiB stack, so fresh full web builds no longer
  start to a blank page.
- F9 or Insert presses the TVC Ins key (matrix row 8, C0). The keyboard matrix
  is documented as 10 rows (0-9); rows 8 and 9 are the joystick rows. The Help
  window lists the host keyboard mapping.
- Tab and the arrow keys are no longer taken by egui keyboard focus navigation
  while the TVC screen has the keyboard, so Tab works as joystick fire.

## v0.8.8 - 2026-10-12

- TVC presets can be overridden with mixed memory, system ROM, a `.crt`
  cartridge, and HBF expansion-slot choices. The active mix is saved with
  the machine id.
- The VT-DOS cartridge can boot with the HBF card. Maskable interrupts now
  follow Z80 mode 2 and the one-instruction delay after `EI`, and a 64K+
  machine scans out the video bank selected by port `0x0C`.
- Each expansion slot takes its own card, including several HBF floppy cards.
- New host file card: a device that loads and saves programs and data
  files in a host folder (or browser storage on the web) instead of tape.
- Raster-timed effects now land where real hardware puts them: port accesses
  happen inside the instruction's I/O cycle, the cursor interrupt fires on the
  falling edge of the cursor pulse, and CPU accesses to video RAM get wait
  states.
- The debugger shows BASIC 1.2 ROM symbols at the address where the ROM is
  currently mapped, and the VT-DOS disk ROM has a commented listing per page.

## v0.8.7 - 2026-09-18

- Cassette inject binds BASIC `TEXT`/`CHAIN`/`TOP`, and `rtvc -i` types `RUN`
  when the CAS autostart byte is set.
- `rtvc-tocas` flattens `rtvc-asm-v1` TOML into one zero-padded CAS image from
  `19EFH` when the first segment is a BASIC stub.
- Failed snapshot loads and tape injects now exit with an error instead of
  continuing.

## v0.8.6 - 2026-09-05

- Added `rtvc-basic` to compile numbered TVC BASIC source into CAS cassette
  images.
- Added `rtvc-tocas` to bulk convert `.bas` and `.asm` sources into sibling `.cas`
  files.
- Expanded the TVC debugger with bounded instruction tracing, richer
  automation, mapped-memory diagnostics, and rewindable frame history.
- Improved TVC video emulation and diagnostics, and reorganized ROM,
  snapshot, and local-state resources.


## v0.8.5 - 2026-06-20

- Added ZX Spectrum 48K emulation through the new `zx82` frontend.
- Added writable and formattable disk-image support.
- Added developer-mode and debugger UI improvements, expanded diagnostics, and
  ROM-aware disassembly tooling.
- Moved snapshots into the application data directory and improved ROM lookup.
- Expanded the TVC, BASIC, VT-DOS, and developer documentation.
- Expanded Windows and macOS releases into containing directories with disk,
  assembler, disassembler, and CAS-to-WAV command-line tools plus English and
  Hungarian documentation.

## v0.8.0 - 2026-06-13

- Added a simple built-in Z80 assembler.
- Added fastboot support to skip long RAM tests on startup.
- Improved the Gamebase dialog with autostart support, Escape-to-close, and case/Hungarian-accent-insensitive name filtering.
- Updated the web UI and reorganized the native menu.
- Decreased snapshot size and added application icons.
- Centered the emulator display while preserving responsive 4:3 scaling.

## v0.7.0 - 2026-06-09

- Build lightweight and full web bundles entirely with the optimized Cargo release profile.
- Added a searchable native/web Gamebase dialog with image tiles, metadata, extra screenshots, and on-demand CAS/DSK ZIP loading.
- Persisted native Gamebase media beside `rtvc.toml`, deduplicated recent-media names, and added File > Quit to the native app.
- Added debugger statistics and application-close commands, and improved debugger UI update responsiveness.
- Added tape playback progress to the emulator UI.
- Fixed browser audio initialization and Windows audio devices that use unsigned 8-bit PCM.

## v0.6.0 - 2026-06-09

- Added a full browser-based emulator UI with local CAS, DSK, ZIP, and snapshot file loading.

## v0.5.0 - 2026-06-04

- Added a TCP socket debugger command interface, supporting both native GUI and headless modes.
- Created a companion interactive Python REPL debugger client (`scripts/rtvc_debug.py`).
- Added command-line options (`-d`/`--disk`, `-t`/`--tape`, `-i`/`--inject`) to directly mount disk images, mount cassette tapes, or inject cassette tapes on startup.
- Added native GUI open-file dialogs for loading disk and cassette media.
- Fixed 16-bit address wrapping edge cases in the Z80 CPU instruction emulator.

## v0.4.1 - 2026-06-04

- Packaged macOS releases as an ad hoc signed `RTVC.app` bundle.
- Documented the macOS first-launch Control-click/right-click Open flow.
