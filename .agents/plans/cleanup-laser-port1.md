# Clean Up the `laser-port1` Porting Effort

## Goal

Move reusable emulator and tool changes from the local `laser-port1` branch
onto current `master`, while keeping all Laser Squad port sources and derived
artifacts in the standalone `tvc-ports` repository.

The cleanup must leave `rtvc` as an emulator/toolchain repository, preserve
the useful ROM-analysis and coding experiments outside the old `data/`
catch-all, preserve unrelated local work in both existing worktrees, and never
publish `laser-port1` to GitHub.

## Non-negotiable Safety Rules

1. Do not merge, rebase, or push `laser-port1`.
2. Do not run `git push --all`, `git push --mirror`, or any command that can
   publish every local branch.
3. Do not add a remote-tracking configuration for `laser-port1`.
4. Do not use `codex/laser-port1-on-master` as an integration source. It is an
   older, highly divergent experiment based on `d238a34`, not the current
   branch.
5. Do not work in either existing dirty worktree. Create a separate clean
   worktree and a new `codex/` branch from local `master`.
6. Preserve all uncommitted user files. Do not use `git reset --hard`,
   `git clean`, or a forced checkout.
7. Do not delete either historical local branch until the migrated result has
   passed validation and the user explicitly approves branch deletion.
8. No remote write is part of this plan. If the cleaned result is later
   published, push only an explicit reviewed ref such as
   `master:master`—never the source branch.

## Verified Starting State

Record these again immediately before implementation because the repository
may have moved since this plan was written.

As inspected on 2026-07-26:

- local `master` and `origin/master` both point to `a8e2993`;
- `laser-port1` points to `f1c3800`, has no upstream, and is twelve commits
  ahead of `master`;
- `master` is the merge base, so the useful branch work can be selected
  without bringing a separate line of master changes across;
- `master` is not checked out in any worktree and is available as the cleanup
  base;
- the secondary worktree at `$HOME/dev/rtvc-master` is now on local branch
  `bbb` at `a8e2993` and contains unrelated local work:
  - modified `data/rom_comments_1_2.json`;
  - untracked `data/TVC12_SYS.rom.asm`; and
  - untracked `roms/TVC12_SYS.64K`;
- the `laser-port1` worktree contains:
  - a modified generated Laser Squad DSK;
  - three untracked diagnostic snapshots;
  - a modified plans index; and
  - an untracked split-repository plan.

The existing worktrees must remain untouched while integration occurs.

## Canonical Port Copy

The standalone repository at `$HOME/dev/tvc-ports` is now authoritative for
the Laser Squad port. Its initial commit is `2d7ad34`.

The following evidence was checked before writing this plan:

- the ROM listing, loader, three authored game sources, BASIC note, and all
  tracked canonical/loader-wait disassembly outputs match byte-for-byte;
- the bridge and disk-write probe differ only by their newer standalone paths
  and descriptions, so the `tvc-ports` versions are canonical;
- `data/porting/output/lasersqd_tvc.dsk` matches the standalone generated DSK:
  `8d0634e74a759f03b7b44e1bab031bdccd3cbd390aee221110311b7c4e4d736b`;
- the `baddraw`, `nogreen`, and `problem` snapshots match the ignored
  standalone copies by SHA-256.

Recheck the standalone repository status and these mappings before removing
anything local. The original TAP, generated TAP TOML, generated DSK, and
diagnostic snapshots are deliberately ignored rather than committed there.

The annotated `48.rom.asm` is intentionally useful in both repositories:
`tvc-ports` keeps it as porting knowledge, while `rtvc` will keep a copy beside
the ROM binaries for emulator and ROM-analysis work.

## Change Classification

### Migrate to `master`

#### `rtvc-tap2toml`

The reusable tool was introduced by `aadc63e`:

- `src/bin/rtvc_tap2toml.rs`;
- the `Cargo.toml` binary/dependency changes;
- the matching `Cargo.lock` update; and
- removal of the superseded `scripts/tap_to_toml.py`.

Integrate the tool as a normal supported CLI, not as a Laser Squad helper:

- add it to the English and Hungarian command tables;
- document it in `info/rtvc.md` and the development skill;
- build and package it in Windows and macOS release workflows;
- add it to `scripts/package-macos-app.sh`; and
- verify the `cli-tools` no-default-features build.

Do not copy the entire commit blindly; review the final branch file and fix any
issues found while integrating it.

#### Frame history, instruction trace, and timed key input

The reusable debugger implementation is primarily in `5260d5b` and `3b5063f`.
Carry forward the non-port paths:

- frame-history and instruction-trace models;
- narrow TVC/Zx82 execution hooks and focused tests;
- debugger UI, workspace, TCP protocol, and `scripts/rtvc_debug.py`;
- `key_press` frame-duration input;
- shared snapshot restore integration;
- `info/rtvc.md`, the development skill, TODO entry, and debugger plans.

Do not carry the four `data/porting/` edits contained in `3b5063f`.

The frame-history and instruction-trace progress files still record manual UI
smoke tests as unfinished. Retain them until those checks are actually run;
do not mark the features complete merely because they compile.

#### Local configuration policy

Carry forward only the reusable part of `8cfcc55`:

- ignore `rtvc.toml` and `rtvc-workspace.json`; and
- remove their tracked defaults from the cleaned branch while keeping the
  documented runtime lookup behavior.

The `/data/porting/work/` ignore rule is obsolete and must not survive the
port removal.

#### Generic CAS-injection diagnostic

`data/porting/test.asm` is a generic interrupt/raster probe used by
`info/developer.md`, not Laser Squad source. Move the assembly source to the
new repository-level coding experiment area:

```text
coding/tvc-interrupt-raster-probe.asm
```

Do not retain the generated `test.cas`. Update the developer note to assemble
the probe into an ignored `target/coding/` path before injecting it into
the clean boot snapshot.

Keep the emulator-specific CAS-injection startup finding. Remove or replace
duplicated TVC software-porting guidance that is now authoritative in
`tvc-ports/knowledge/`.

Add `coding/README.md` explaining that this directory holds small tracked
emulator/machine-code experiments, while assembled CAS, TOML, BIN, screenshots,
and other generated results belong under ignored `target/coding/`.

#### Retire the `data/` catch-all

Preserve the incomplete but useful ROM-analysis work by moving it beside the
corresponding ROM binaries:

| Current path | New path |
| --- | --- |
| `data/48.rom.asm` | `roms/48.rom.asm` |
| `data/TVC12_D3.64K.asm` | `roms/TVC12_D3.64K.asm` |
| `data/TVC12_D4.64K.asm` | `roms/TVC12_D4.64K.asm` |
| `data/TVC12_D7.64K.asm` | `roms/TVC12_D7.64K.asm` |
| `data/rom_comments_1_2.json` | `roms/rom_comments_1_2.json` |
| `data/rom_symbols_1_2.json` | `roms/rom_symbols_1_2.json` |

These are works in progress, not authoritative ROM source. Preserve their
content and history; update their generated headers, assembler examples,
debugger `include_str!`, and all English/Hungarian documentation links to the
new paths.

The dirty `bbb` worktree contains an uncommitted edit to
`data/rom_comments_1_2.json`. Before moving the tracked master version, record
and preserve that diff. Do not silently overwrite it or assume it should be
dropped. Either:

1. have the user commit the `bbb` ROM-analysis work first and then port that
   content to the new path; or
2. copy its exact diff into a separately reviewable change at
   `roms/rom_comments_1_2.json` while leaving the `bbb` worktree untouched.

The untracked `data/TVC12_SYS.rom.asm` and `roms/TVC12_SYS.64K` in `bbb` are
not part of this cleanup unless the user expands the scope.

Move the stable boot fixture out of `data/`:

```text
data/snapshots/boot12dos.rtvcsnap.zip
    -> snapshots/boot12dos.rtvcsnap.zip
```

Update the compile-time `include_bytes!`, native/Hungarian READMEs, hardware
and implementation references, development commands, and tests. The three
Laser Squad diagnostic snapshots remain port-local ignored files in
`tvc-ports` and must not move into the new root `snapshots/`.

After the ROM analysis, boot snapshot, and coding probe moves—and after all
Laser Squad material is excluded—the tracked `data/` directory should be
empty and can disappear.

#### Preserve the OpenCode helpers

Carry forward:

- `.agents/skills/opencode-orchestrator/`; and
- `.opencode/agents/codex-implementer.md`.

They were introduced during the port effort but are intentionally retained as
general agent tooling that may be used later. Keep them independent of the
Laser Squad porting skill and document an entry point only if current agent
instructions need one.

#### Preserve the native Gamebase cache policy

`rtvc-media/` is not source data. It is the ignored native Gamebase cache
created beside the active `rtvc.toml`; it contains downloaded/extracted local
CAS and DSK media. Keep `/rtvc-media` ignored and keep the documented cache
behavior. Do not move it into `roms/`, `coding/`, or Git, and do not delete the
user's current cache as part of repository cleanup.

### Keep only in `tvc-ports`

Do not copy any of the following onto the cleanup branch:

- `data/porting/LASERSQD.TAP` and generated TAP TOML;
- the obsolete `.z80` input and snapshot bridge JSON;
- Laser Squad loader, overlay, build, notes, and disassembly files;
- generated CAS, DSK, TOML, Z80, report, work, and output files;
- `data/porting/tvc_disk_write_probe.asm`;
- the three Laser Squad diagnostic snapshots;
- Laser Squad-specific parts of the ZX porting skill; or
- `progs/mralex.cas` and `progs/mralex.dsk`.

`data/porting/test.asm` is the one deliberate exception to removing
`data/porting/`: its source moves to
`coding/tvc-interrupt-raster-probe.asm`, while `test.cas` is discarded as
generated output.

The existing `mralex` archive/screenshots on `master` are independent existing
media. The raw CAS/DSK files added on `laser-port1` were temporary test inputs
and have no non-port references.

### Audit rather than assume

- Reconcile `.agents/plans/README.md` by content. Keep active emulator plans;
  do not copy the obsolete split-repository plan back to `master`.
- Review every file outside `data/porting/` in
  `git diff --name-status master...laser-port1`. Any unclassified file blocks
  cleanup until its purpose is understood.

## Implementation Sequence

### Phase 1 — Freeze evidence without changing branches

1. Record:

   ```sh
   git status --short --branch
   git worktree list --porcelain
   git branch -vv
   git rev-list --left-right --count master...laser-port1
   git log --left-right --cherry-pick --oneline master...laser-port1
   ```

2. Confirm local `master` still matches the intended integration base.
3. Record both existing worktree statuses separately.
4. Confirm `laser-port1` still has no upstream and that no remote branch with
   that name is visible.
5. Re-run the source, DSK, disassembly, and snapshot comparisons against
   `$HOME/dev/tvc-ports`.
6. Copy nothing and delete nothing if any supposed canonical destination is
   missing or has unexplained content differences.

### Phase 2 — Create an isolated cleanup worktree

From the main repository, create a new branch and worktree from local
`master`, using an unused path:

```sh
git worktree add -b codex/cleanup-laser-port1 \
  "$HOME/dev/rtvc-laser-cleanup" master
```

Do not switch either existing worktree. Copy this plan into the cleanup branch
as the first documentation change and create
`cleanup-laser-port1-progress.md` only when implementation begins.

`master` is free to seed this worktree; there is no need to disturb the main
`laser-port1` checkout or the secondary `bbb` checkout.

### Phase 3 — Migrate the TAP conversion tool

1. Bring over the final `rtvc-tap2toml` implementation and scoped Cargo
   changes by path, not by merging `laser-port1`.
2. Remove the old Python converter only after the Rust tool covers its intended
   supported workflow.
3. Add user, implementation, development, Hungarian, and release-package
   documentation.
4. Add the binary to both release jobs and both platform package layouts.
5. Keep this as one reviewable commit independent of debugger and cleanup
   changes.

Focused validation:

```sh
cargo fmt --check
cargo check --bin rtvc-tap2toml
cargo check --no-default-features --features cli-tools --bin rtvc-tap2toml
```

Then run the standalone Laser Squad `verify.py` with the new tool on `PATH` or
through `RTVC_BIN_DIR`; this provides an end-to-end TAP output/hash check
without committing commercial media to `rtvc`.

### Phase 4 — Migrate debugger functionality

1. Apply only the non-port paths from the final `laser-port1` tree.
2. Review each generic machine/MMU hook against current `master`; preserve the
   narrow integration points and avoid port-specific branches.
3. Retain the frame-history and trace plans/progress records.
4. Update `TODO.md`, `info/rtvc.md`, the development skill, and the plans index
   to match actual implemented behavior.
5. Keep this as a second reviewable commit.

Focused validation:

```sh
cargo fmt --check
cargo test frame_history --lib
cargo test instruction_trace --lib
cargo test timed_key_press --lib
cargo test tcp_debugger --lib
cargo check
cargo check --lib --no-default-features --features wasm-full \
  --target wasm32-unknown-unknown
```

Perform the remaining hands-on checks with a generic boot snapshot:

- record, stop, move backward/forward, select a thumbnail, return live;
- resume from an older frame and confirm future-history truncation;
- save a selected frame and load it in a fresh process;
- start/stop/list an instruction trace through UI and TCP; and
- confirm reset/state load clears trace and timed-key state.

Update the two progress files with the actual result. Optional convenience
ideas remain backlog items rather than blockers.

### Phase 5 — Apply repository cleanup and directory moves

1. Remove tracked `rtvc.toml` and `rtvc-workspace.json`; retain their ignore
   rules and documentation.
2. Move the six ROM-analysis files into `roms/`, preserving and separately
   reviewing the outstanding `bbb` comments diff.
3. Move the stable boot snapshot into root `snapshots/` and update all
   compile-time and documentation consumers.
4. Move the generic interrupt/raster source to
   `coding/tvc-interrupt-raster-probe.asm`, add the coding-area README, and
   update `info/developer.md`.
5. Retain the OpenCode orchestrator/agent files.
6. Keep `rtvc-media/` ignored and untouched.
7. Ensure no Laser Squad data, diagnostic snapshots, port plans, or commercial
   media were introduced into the cleanup branch.
8. Remove stale `data/` and `data/porting` references outside historical
   discussion in this cleanup plan.
9. Confirm no tracked or untracked repository input still requires `data/`,
   then allow the empty directory to disappear.
10. Reconcile the plans index and create a third focused cleanup commit.

Because the cleanup branch starts from `master`, the port files should be
absent by construction. If they appear, stop and determine which overly broad
restore or cherry-pick introduced them.

### Phase 6 — Validate the cleaned branch

Run functionality-focused checks:

```sh
cargo fmt --check
cargo check
cargo test --lib
cargo check --bins
cargo check --no-default-features --features cli-tools --bins
cargo check --lib --no-default-features --features wasm-full \
  --target wasm32-unknown-unknown
cargo check --manifest-path xtask/Cargo.toml
```

Also:

1. run `$HOME/dev/tvc-ports/ports/laser-squad/verify.py` against the cleaned
   toolchain;
2. inspect `git diff master...codex/cleanup-laser-port1`;
3. confirm no staged or committed path matches the port-removal denylist;
4. confirm all documentation links and release package tool lists agree; and
5. confirm `coding/`, `roms/`, and `snapshots/` contain the intended moved
   sources/fixture and no generated port output;
6. confirm `rtvc-media/` is still ignored and its local contents are unchanged;
7. confirm no non-historical source or documentation reference begins with
   `data/`; and
8. confirm the existing dirty `bbb` and `laser-port1` worktrees are
   unchanged.

### Phase 7 — Integrate locally without publishing `laser-port1`

1. Ask the user to review the three cleanup commits.
2. Leave the unrelated dirty `bbb` worktree unchanged; it does not need to be
   resolved before advancing the free `master` ref.
3. If `master` has not moved, fast-forward it to the reviewed cleanup branch.
   If it has moved, rebase the cleanup branch—not `laser-port1`—and rerun the
   focused validation.
4. Perform the fast-forward from the clean cleanup worktree, for example by
   switching that worktree to `master` and merging with `--ff-only`.
5. Do not push as part of cleanup. A later user-authorized push must name only
   the reviewed cleanup/master ref explicitly.

### Phase 8 — Retire local historical branches

Only after the cleaned master contains all selected functionality:

1. verify the current `laser-port1` DSK and snapshots still match the
   standalone local copies;
2. preserve any remaining unique uncommitted file outside Git or move it to
   Trash;
3. optionally create a local Git bundle if the user wants an offline history
   archive;
4. release `master` from the temporary cleanup worktree—switch it back to the
   cleanup branch or remove the clean worktree—before switching
   `$HOME/dev/rtvc` from `laser-port1` to `master`;
5. request explicit approval before force-deleting `laser-port1`, because a
   selective migration will not make Git consider the whole branch merged;
6. audit and separately request approval before deleting
   `codex/laser-port1-on-master`; and
7. verify the remote still has no `laser-port1` branch.

Never push either historical branch as a substitute for a local bundle.

## Acceptance Criteria

- Reviewed `master` contains `rtvc-tap2toml`, frame history, instruction trace,
  and timed key input with their relevant tests and documentation.
- Release builds and packages include `rtvc-tap2toml`.
- `rtvc.toml` and `rtvc-workspace.json` are local ignored state, not tracked
  repository defaults.
- `data/` is gone: ROM-analysis work is under `roms/`, the boot fixture is
  under root `snapshots/`, and the generic CAS-injection probe source is under
  `coding/`.
- The incomplete ROM assembly/listing work and symbol/comment databases remain
  available beside their ROM binaries.
- The generic CAS-injection probe is regenerated rather than committed as CAS.
- The OpenCode orchestrator and implementer agent remain available.
- `rtvc-media/` remains an ignored local Gamebase cache and is neither moved
  nor deleted.
- No Laser Squad TAP, snapshot, DSK, disassembly, patch, loader, build script,
  port note, or port-only agent setup exists on cleaned `master`.
- `tvc-ports` still performs a clean verified Laser Squad build using the
  cleaned `rtvc` toolchain.
- Unrelated dirty files in the secondary `bbb` worktree remain unchanged.
- No remote write occurs during cleanup, and GitHub never receives
  `laser-port1`.
- Historical local branches are retained until separate explicit deletion
  approval.

## Deliberate Non-goals

- Reworking Laser Squad in `rtvc`.
- Rewriting the already verified standalone port.
- Publishing game media or generated DSK files.
- Folding the old `codex/laser-port1-on-master` experiment into current code.
- Expanding frame history or instruction trace beyond their existing plans.
- Pushing any branch or opening a pull request.
