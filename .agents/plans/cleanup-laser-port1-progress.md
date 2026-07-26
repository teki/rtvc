# `laser-port1` Cleanup Progress

Last updated: 2026-07-26

## Status

Implementation and validation are complete on isolated branch
`codex/cleanup-laser-port1`. This record is included in the repository-cleanup
commit. The source `laser-port1` branch has not been merged, rebased, or
pushed, and both pre-existing dirty worktrees remain untouched.

## Completed

- [x] Re-audited local/remote refs, worktrees, and the source branch merge base.
- [x] Rechecked the standalone `tvc-ports` Laser Squad build and artifact
  ownership.
- [x] Migrated `rtvc-tap2toml` as a supported Rust CLI, including release
  packaging and documentation.
- [x] Migrated frame history, instruction tracing, and deterministic timed-key
  debugger input without any Laser Squad port sources.
- [x] Moved retained ROM analysis beside the ROM binaries.
- [x] Reproduced the `bbb` worktree's uncommitted ROM-comment improvements at
  the new path without modifying that worktree.
- [x] Moved the stable boot fixture to `snapshots/`.
- [x] Moved the generic interrupt/raster source to
  `coding/tvc-interrupt-raster-probe.asm`.
- [x] Preserved the OpenCode helper skill and agent.
- [x] Removed tracked local configuration defaults and ignored their runtime
  filenames.
- [x] Removed Laser Squad material and duplicated porting knowledge from the
  cleaned repository.

## Validation Completed

- `cargo fmt --check`
- `cargo check --bin rtvc-tap2toml`
- `cargo build --no-default-features --features cli-tools --bins`
- standalone `tvc-ports` Laser Squad clean-build verification
- focused frame-history, instruction-trace, timed-key, and TCP debugger tests
- native `cargo check`
- full-web WASM `cargo check`
- Python debugger client bytecode compilation
- coding-probe assembly to ignored `target/coding/`
- complete `cargo test --lib` (121 passed)
- native and CLI-tool binary checks
- full-web WASM check
- `xtask` check
- repository path and artifact ownership audit

## Next Steps

1. Fast-forward local `master` to the reviewed cleanup result.
2. Preserve the dirty `laser-port1` worktree state before returning the main
   checkout to `master`.

No remote write or historical branch deletion is part of this work.
