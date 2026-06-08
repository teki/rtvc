# Changes

Release notes start here. Older release history is intentionally not backfilled.

## Unreleased


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
