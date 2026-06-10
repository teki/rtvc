# Changes

Release notes start here. Older release history is intentionally not backfilled.

## Unreleased

- Improved the Gamebase dialog with Escape-to-close and case- and Hungarian-accent-insensitive name filtering.
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
