---
name: release
description: Manages crate versioning, changelog updates, and publishing to crates.io. Enforces user approval gates before final publish.
---

# Release Skill

Use this skill to prepare and publish a new version of the `oci-api` crate.

## When to Use

- After merging feature or fix changes that should be released.
- When bumping the crate version (major, minor, or patch).
- Before and after publishing to crates.io.

## Workflow

### 1. Collect User Input

Before any automated step, the agent must ask the user for:

- **Bump level** (semver): `major`, `minor`, or `patch`.
  - **patch**: Bug fixes, documentation, internal refactors (no API changes).
  - **minor**: New features, additive API changes (backward compatible).
  - **major**: Breaking API changes.
- **Feature-related README changes**: Ask whether README content (usage examples, feature descriptions, API docs) needs updating beyond the version string. If yes, make those edits first.

Do not proceed without explicit user confirmation of the bump level.

### 2. Pre-Release Checks

- Run the full test suite: `cargo test`
- Run clippy: `cargo clippy`
- Build documentation: `cargo doc --no-deps`
- Confirm all checks pass with no new warnings or errors.

### 3. Update Documentation

- Update `README.md`:
  - Update the version string in the installation section.
  - Update any feature-related content that reflects the changes in this release (new API sections, changed usage examples, removed features).
- Update `CHANGELOG.md`:
  - Add a new section with the release date.
  - Categorize changes under `### Added`, `### Changed`, `### Fixed`, `### Removed` as applicable.
- Update `version` in `Cargo.toml`.

### 4. User Review Gate

Present the following to the user and wait for approval before proceeding:

- Computed new version
- CHANGELOG entry summary
- README diff (if feature-related changes were made)
- Pre-release check results

The agent must not run the publish script or commit until the user explicitly approves.

### 5. Publish via Script

After user approval, run the release script:

```bash
bash .github/skills/release/bin/release.sh <major|minor|patch>
```

The script handles: dry-run → commit → tag → publish → push.

If the script fails at any step, report the failure to the user and do not retry without guidance.

## Conventions

- Tag format: `v{MAJOR}.{MINOR}.{PATCH}` (e.g., `v0.6.0`).
- Commit message format: `release: v{VERSION}`.
- CHANGELOG follows [Keep a Changelog](https://keepachangelog.com/) format.
- Never publish without a passing `cargo publish --dry-run` first.
- README must reflect actual feature changes, not just version string bumps.

## Shell Script

The `bin/release.sh` script automates the final release steps from the command line. It is designed to run **after** manual documentation updates and user approval.

```bash
# Minor release (e.g., 0.6.0 → 0.7.0)
bash .github/skills/release/bin/release.sh minor

# Patch release (e.g., 0.7.0 → 0.7.1)
bash .github/skills/release/bin/release.sh patch

# Major release (e.g., 0.7.1 → 1.0.0)
bash .github/skills/release/bin/release.sh major
```

The script performs:
1. Version computation from current `Cargo.toml`
2. Pre-release checks (`cargo test`, `cargo clippy`, `cargo doc --no-deps`)
3. `Cargo.toml` and `README.md` version string bump
4. `cargo publish --dry-run` validation
5. CHANGELOG entry check (aborts if missing)
6. Git commit, annotated tag, `cargo publish`, and push

**Prerequisites (manual, before running the script):**
- `CHANGELOG.md` must contain an entry for the new version.
- `README.md` must be updated with any feature-related content changes.
- User must have approved the release via the agent's review gate.
- crates.io token must be configured in `~/.cargo/credentials.toml`.
