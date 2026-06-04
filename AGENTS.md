# AGENTS.md — rtvc Documentation Index

This document serves as the entry point for coding agents and developers to find architectural knowledge, project information, and development guides for the Videoton TV Computer (TVC) emulator.

---

## Directory Map

Refer to the following resources for detailed documentation on the system:

- [Project Overview & Architecture](info/project_overview.md) — High-level description of the project structure, Rust toolchain, and core emulator architecture.
- [User README](README.md) — User-facing run, snapshot, and web bundle commands.
- [Release Notes](CHANGES.md) — Concise release notes starting with the next release; older releases are intentionally not backfilled.
- [Future Build/UI Plan](info/future_plan.md) — Planned native, lightweight WASM, and full web build tiers, including video model and feature hygiene policy.
- [Snapshot Format and Web Bundles](info/snapshot.md) — Custom TVC snapshot format and `cargo bundle-web` upload workflow.
- [Z80 CPU Documentation](info/z80.md) — Detailed specifications, instructions, lookup tables, and execution details for the Z80 CPU emulator.
- [Memory Management Unit (MMU) Documentation](info/mmu.md) — Architectural reference for TVC bank switching, page layout, and I/O memory mapping.
- [Development and Testing Skill](.agents/skills/development/SKILL.md) — Essential commands for compiling, running, testing, and benchmarking the emulator.
- [Release Skill](.agents/skills/release/SKILL.md) — Release workflow for version bumps, concise `CHANGES.md` updates, manual review, commits, tags, and pushes.

---

## Documentation Maintenance Policy

To prevent documentation rot and ensure all agents have access to accurate information, adhere to the following rules:

1. **Keep Specs Updated**: When you implement new features, rewrite core functionality (e.g., wiring the MMU to the main loop, adding screen rendering, or input devices), update the corresponding file in `info/` or create a new topic-specific markdown file.
2. **Keep Commands and Workflows Updated**: If compilation workflows, test cases, or benchmarks change (e.g., adding a new benchmark tool or migrating test files), immediately update [.agents/skills/development/SKILL.md](.agents/skills/development/SKILL.md).
3. **Keep Crate / Dependency Specs Updated**: If updating `Cargo.toml` dependencies, required Rust edition, or adding new binary targets, update [info/project_overview.md](info/project_overview.md).
4. **Keep Build-Tier Plans Updated**: If changing native/web feature flags, video model selection, web UI strategy, egui usage, browser storage, or lightweight WASM dependency boundaries, update [info/future_plan.md](info/future_plan.md).
5. **Keep Snapshot Specs Updated**: If changing snapshot chunks, save/load coverage, or web bundle behavior, update [info/snapshot.md](info/snapshot.md).
6. **Use Clickable Links**: When referencing codebase files or documentation, always use clickable Markdown links with the relative path (e.g., `[main.rs](src/main.rs)`) to enable easy navigation.

---

## Git and Command Execution Policy

To maintain control over repository history and remote operations, all agents must adhere to the following rules:

1. **No Automatic Pushing**: Never execute `git push` or perform any remote branch publishing/deletion commands without explicit, written confirmation and approval from the user.
2. **Local Commits**: You may stage (`git add`) and commit (`git commit`) files locally to save progress, but always present the diff and ask the user before committing or pushing.
