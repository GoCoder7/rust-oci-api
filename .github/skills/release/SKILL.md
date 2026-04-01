---
name: release
description: Manages crate versioning, changelog updates, and publishing to crates.io. Covers version bump, dry-run validation, git tagging, and final publish.
---

# Release Skill

Use this skill to prepare and publish a new version of the `oci-api` crate.

## When to Use

- After merging feature or fix changes that should be released.
- When bumping the crate version (major, minor, or patch).
- Before and after publishing to crates.io.

## Workflow

### 1. Determine Version Bump

- Review changes since the last release.
- Decide bump level:
  - **patch**: Bug fixes, documentation, internal refactors (no API changes).
  - **minor**: New features, additive API changes (backward compatible).
  - **major**: Breaking API changes.

### 2. Pre-Release Checks

- Run the full test suite: `cargo test`
- Run clippy: `cargo clippy`
- Build documentation: `cargo doc --no-deps`
- Confirm all checks pass with no new warnings or errors.

### 3. Update Version and Changelog

- Update `version` in `Cargo.toml`.
- Update `README.md` installation section if version string appears.
- Add a new section to `CHANGELOG.md` with the release date and changes.
- Categorize changes under `### Added`, `### Changed`, `### Fixed`, `### Removed` as applicable.

### 4. Dry-Run Publish

```bash
cargo publish --dry-run
```

- Verify the package builds and passes all pre-publish checks.
- Resolve any issues before proceeding.

### 5. Commit and Tag

```bash
git add -A
git commit -m "release: v{VERSION}"
git tag -a v{VERSION} -m "v{VERSION}"
```

### 6. Publish

```bash
cargo publish
```

- Verify the new version appears on [crates.io](https://crates.io/crates/oci-api).

### 7. Push

```bash
git push origin main
git push origin v{VERSION}
```

## Conventions

- Tag format: `v{MAJOR}.{MINOR}.{PATCH}` (e.g., `v0.6.0`).
- Commit message format: `release: v{VERSION}`.
- CHANGELOG follows [Keep a Changelog](https://keepachangelog.com/) format.
- Never publish without a passing `cargo publish --dry-run` first.
