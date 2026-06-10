# AGENTS.md — rtvc Documentation Index

This document serves as the entry point for coding agents and developers to find architectural knowledge, project information, and development guides for the Videoton TV Computer (TVC) emulator.

## Directory Map

Refer to the following resources for detailed documentation on the system:

- [Project Overview & Architecture](info/project_overview.md) — High-level description of the project structure, supported build targets, Rust toolchain, and core emulator architecture.
- [User README](README.md) — User-facing run, snapshot, and web bundle commands.
- [Release Notes](CHANGES.md) — Concise release notes starting with the next release; older releases are intentionally not backfilled.
- [Open Issues and Planned Work](TODO.md) — Known issues and implementation ideas that are not yet complete.
- [Snapshot Format and Web Bundles](info/snapshot.md) — Custom TVC snapshot format and `cargo bundle-web` upload workflow.
- [TVC Machine Core](info/tvc.md) — Machine orchestration, timing, I/O ports, interrupts, ROM loading, and media integration.
- [Z80 CPU Documentation](info/z80.md) — Detailed specifications, instructions, lookup tables, and execution details for the Z80 CPU emulator.
- [Z80 Opcode Reference](info/z80opcodes.md) — Maintained opcode, timing, flag, and instruction-effect reference.
- [Memory Management Unit (MMU) Documentation](info/mmu.md) — Architectural reference for TVC bank switching, page layout, and I/O memory mapping.
- [Video Controller Documentation](info/vid.md) — MC6845 registers, rendering modes, timing, and deferred accuracy work.
- [Sound Documentation](info/sound.md) — Programmable sound generator, timer interrupt, and PCM output model.
- [Keyboard Documentation](info/key.md) — TVC keyboard matrix and host-key mapping.
- [Cassette Documentation](info/cas.md) — CAS structure, tape signal emulation, loading, and injection behavior.
- [HBF Floppy Documentation](info/hbf.md) — HBF expansion card, FD1793 controller, and disk image handling.
- [Socket Debugger Documentation](info/dbg.md) — Protocol specification for the TCP socket debugger and Python REPL client.
- [Development and Testing Skill](.agents/skills/development/SKILL.md) — Essential commands for compiling, running, testing, and benchmarking the emulator.
- [Release Skill](.agents/skills/release/SKILL.md) — Release workflow for version bumps, concise `CHANGES.md` updates, manual review, commits, tags, and pushes.

## Documentation Maintenance Policy

To prevent documentation rot and ensure all agents have access to accurate information, adhere to the following rules:

1. **Keep Specs Updated**: When you implement new features, rewrite core functionality (e.g., wiring the MMU to the main loop, adding screen rendering, or input devices), update the corresponding file in `info/` or create a new topic-specific markdown file.
2. **Keep Commands and Workflows Updated**: If compilation workflows, test cases, or benchmarks change (e.g., adding a new benchmark tool or migrating test files), immediately update [.agents/skills/development/SKILL.md](.agents/skills/development/SKILL.md).
3. **Keep Architecture and Build Targets Updated**: If changing `Cargo.toml` dependencies, the Rust edition, binary targets, native/web feature flags, UI strategy, browser storage, or lightweight WASM dependency boundaries, update [info/project_overview.md](info/project_overview.md).
4. **Keep Subsystem Details Local**: If changing video models, keyboard behavior, sound, media devices, or another subsystem, update its topic-specific document rather than duplicating the details in the project overview.
5. **Keep Snapshot Specs Updated**: If changing snapshot chunks, save/load coverage, or web bundle behavior, update [info/snapshot.md](info/snapshot.md).
6. **Use Clickable Links**: When referencing codebase files or documentation, always use clickable Markdown links with the relative path (e.g., `[main.rs](src/main.rs)`) to enable easy navigation.

## Git and Command Execution Policy

To maintain control over repository history and remote operations, all agents must adhere to the following rules:

1. **No Automatic Pushing**: Never execute `git push` or perform any remote branch publishing/deletion commands without explicit, written confirmation and approval from the user.
2. **No Automatic Staging or Committing**: Never execute `git add`, `git commit`, or similar staging/commit commands without explicit, written confirmation and approval from the user, unless executing a documented repository skill (such as the Release Skill) that explicitly grants permission for these operations as part of its workflow.
