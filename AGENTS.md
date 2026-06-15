# AGENTS.md — rtvc Documentation Index

This document serves as the entry point for coding agents and developers to find architectural knowledge, project information, and development guides for the Videoton TV Computer (TVC) emulator.

## Directory Map

Refer to the following resources for detailed documentation on the system:

- [TVC Technical Reference](info/tvc.md) — Detailed, implementation-neutral TVC hardware reference for emulator authors: timing, memory, I/O, interrupts, video, keyboard, sound, cassette, and expansion devices.
- [rtvc Implementation and Usage Reference](info/rtvc.md) — Rust architecture, emulation choices, media handling, snapshots, debugger, UI, persistence, and build targets.
- [User README](README.md) — User-facing run, snapshot, and web bundle commands.
- [Release Notes](CHANGES.md) — Concise release notes starting with the next release; older releases are intentionally not backfilled.
- [Open Issues and Planned Work](TODO.md) — Known issues and implementation ideas that are not yet complete.
- [Development and Testing Skill](.agents/skills/development/SKILL.md) — Essential commands for compiling, running, testing, and benchmarking the emulator.
- [Release Skill](.agents/skills/release/SKILL.md) — Release workflow for version bumps, concise `CHANGES.md` updates, manual review, commits, tags, and pushes.

## Documentation Maintenance Policy

To prevent documentation rot and ensure all agents have access to accurate information, adhere to the following rules:

1. **Keep the Hardware Spec Updated**: When verified TVC behavior changes or new hardware details are established, update [info/tvc.md](info/tvc.md). Keep it implementation-neutral and do not reproduce generic Z80, MC6845, or FD1793 specifications.
2. **Keep Commands and Workflows Updated**: If compilation workflows, test cases, or benchmarks change (e.g., adding a new benchmark tool or migrating test files), immediately update [.agents/skills/development/SKILL.md](.agents/skills/development/SKILL.md).
3. **Keep rtvc Details Updated**: If changing architecture, dependencies, build targets, emulation policy, media handling, snapshots, debugger behavior, UI strategy, browser storage, or WASM boundaries, update [info/rtvc.md](info/rtvc.md).
4. **Maintain the Boundary**: Put observable TVC hardware behavior in `info/tvc.md`; put repository-specific implementation and usage behavior in `info/rtvc.md`.
5. **Avoid Topic Fragmentation**: Extend one of the two references rather than creating a new English subsystem document unless the material cannot reasonably fit either scope.
6. **Use Clickable Links**: When referencing codebase files or documentation, always use clickable Markdown links with the relative path (e.g., `[main.rs](src/main.rs)`) to enable easy navigation.

## Git and Command Execution Policy

To maintain control over repository history and remote operations, all agents must adhere to the following rules:

1. **No Automatic Pushing**: Never execute `git push` or perform any remote branch publishing/deletion commands without explicit, written confirmation and approval from the user.
2. **No Automatic Staging or Committing**: Never execute `git add`, `git commit`, or similar staging/commit commands without explicit, written confirmation and approval from the user, unless executing a documented repository skill (such as the Release Skill) that explicitly grants permission for these operations as part of its workflow.
