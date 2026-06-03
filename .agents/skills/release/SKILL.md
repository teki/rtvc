---
name: release
description: "Use when preparing and publishing an rtvc release: bump Cargo package version if needed, update CHANGES.md from git history, require manual user review, then commit, tag v<version>, and push."
---

# rtvc Release

Use this skill only when the user asks to prepare, cut, publish, tag, or push an `rtvc` release.

## Workflow

1. Inspect repository state.
   - Run `git status --short`.
   - If unrelated or surprising dirty files exist, identify them and avoid overwriting user work.
   - Read `Cargo.toml`, `CHANGES.md`, and recent git history/tags.

2. Determine the release version.
   - Use a user-provided version if one is explicitly requested.
   - Otherwise compare `Cargo.toml` package `version` with the latest `vX.Y.Z` git tag.
   - If the Cargo version is missing, equal to, or lower than the latest tag, bump the patch version.
   - If the Cargo version is already newer than the latest tag, keep it.
   - Update `Cargo.toml` only when the version has not already been bumped.
   - After changing `Cargo.toml`, run `cargo check` so `Cargo.lock` is refreshed if Cargo records the package version there.

3. Update `CHANGES.md`.
   - Use `git log --oneline <latest-tag>..HEAD` when a latest release tag exists; otherwise use recent history.
   - Keep entries concise and user-facing.
   - Do not backfill older release history.
   - Replace `## Unreleased` with `## v<version> - <YYYY-MM-DD>` for the release being prepared, and create a new empty `## Unreleased` section above it.
   - The GitHub release workflow extracts the matching `## v<version>` section from `CHANGES.md` and uses it as the release page body, so the heading must match the tag exactly.

4. Stop for manual review.
   - Show the user the proposed `CHANGES.md` content or a concise diff.
   - Explicitly ask the user to manually review and edit `CHANGES.md`.
   - Do not commit, tag, or push until the user confirms the changelog is ready.

5. After user confirmation, commit the release.
   - Re-read `git status --short`.
   - Stage only release-related files, normally `Cargo.toml`, `Cargo.lock` if changed, and `CHANGES.md`.
   - Commit with message `Prepare v<version> release`.

6. Tag and push.
   - Create annotated tag `v<version>` with message `v<version>`.
   - Push the release commit and tag:
     ```bash
     git push
     git push origin v<version>
     ```

## Notes

- Never skip the manual `CHANGES.md` review gate.
- If tests or release checks are requested, run them before the commit and report failures.
- Keep release notes shorter rather than exhaustive; combine low-level commits into a small number of meaningful bullets.
