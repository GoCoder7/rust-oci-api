---
name: release
description: Prepares and publishes a staged crates.io release for the oci-api crate with explicit approval before publish.
---

# OCI API Release Skill

## Purpose

Use this skill when the user wants a repeatable release workflow for the
`oci-api` crate.

This skill keeps the release explicit and staged:

1. require a clean worktree,
2. ask the user for a semver bump,
3. preview the next release before mutating the repository,
4. prepare the local release commit and local annotated tag,
5. ask again before publishing to crates.io and pushing to the remote.

It follows the same staged release style used in the `flow` and
`svelte-components` reference projects, but it is adapted to this Rust crate:

- `Cargo.toml`
- `README.md`
- `CHANGELOG.md`

## When to Run

- When the user asks for a new `oci-api` release.
- When the crate version should be bumped in a controlled semver step.
- When the user wants a repeatable release workflow instead of ad hoc git and
  `cargo publish` commands.

## Related Files

- `.github/skills/release/SKILL.md`
- `.github/skills/release/bin/prepare-release.sh`
- `.github/skills/release/bin/push-release.sh`
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `CHANGELOG.md`

## Workflow

### 1. Preflight

- Confirm the repository worktree is clean before any mutation.
- Do not auto-stash or auto-reset the user's work.
- Confirm that `Cargo.toml`, `README.md`, and `CHANGELOG.md` exist.
- Make sure the crates.io token is configured before the final publish step.

### 2. Collect the Release Intent

- Ask the user to choose one bump type: `major`, `minor`, or `patch`.
- Use the current branch by default unless the user explicitly wants a different
  branch.
- Use `origin` as the default remote unless the user explicitly wants another
  remote.
- Ask whether `README.md` or `CHANGELOG.md` needs content edits beyond the
  version bump. If yes, make those edits first.

### 3. Run the Release Checks

- Run the crate verification steps before any mutation:

```sh
cargo test
cargo clippy
cargo doc --no-deps
cargo publish --dry-run
```

- If any command fails, stop the release workflow.

### 4. Preview the Release Plan

- Run:

```sh
bash ./.github/skills/release/bin/prepare-release.sh --repo-root . --bump patch --dry-run
```

- Read the JSON result and summarize:
  - current branch,
  - latest stable tag,
  - current `Cargo.toml` version,
  - current README install-snippet version if present,
  - computed next version,
  - whether `Cargo.toml`, `Cargo.lock` (if tracked), and the README install
    snippet will need a version bump,
  - whether version drift exists,
  - whether the changelog already contains an entry for the target version,
  - whether the version is already prepared locally.

### 5. Prepare the Local Release

- After the user approves the plan, run:

```sh
bash ./.github/skills/release/bin/prepare-release.sh --repo-root . --bump patch
```

- This helper updates `Cargo.toml`, `Cargo.lock` (if tracked), and the README
  install-snippet version together when the next release version is not already
  prepared.
- If `CHANGELOG.md` does not yet contain the target release entry, stop and
  update it before rerunning the prepare step.
- Read the JSON result and report:
  - release commit SHA,
  - created tag,
  - previous tag,
  - previous crate version,
  - previous README install-snippet version,
  - next version,
  - whether `Cargo.lock` was updated as part of the release prepare,
  - whether a new release commit was created or the current `HEAD` was reused.

### 6. Publish Only After Explicit Approval

- Ask the user whether to publish the prepared release.
- If the user declines, stop with the local commit and local tag left in place.
- If the user approves, run:

```sh
bash ./.github/skills/release/bin/push-release.sh --repo-root . --tag <created-tag>
```

- This step publishes the crate to crates.io first, then pushes the branch tip
  and tag to the remote.

### 7. Report the Final State

- Report the branch, commit SHA, tag, and remote used.
- Make it explicit that the crate was published to crates.io before the git push.
- If the release stays local only, make that clear in the summary.

## Conventions

- Tag format: `v{MAJOR}.{MINOR}.{PATCH}`.
- Commit message format: `release: v{VERSION}`.
- `CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/) style.
- Never publish without a passing `cargo publish --dry-run` first.
- `README.md` must reflect actual feature changes, not only the install version.

## Validation

- When this skill changes, validate it against the `verify-markdown` and
  `verify-skill` checklists.
- Validate the helper scripts with `--help` and a dry-run before trusting new
  workflow changes.
- If a helper script fails partway through, report the exact mutation point and
  repository state back to the user.

## Shell Script

The deterministic release stages are implemented by the helper scripts in
`bin/`.

### Preview or prepare the local release

```sh
bash ./.github/skills/release/bin/prepare-release.sh --repo-root . --bump minor --dry-run
bash ./.github/skills/release/bin/prepare-release.sh --repo-root . --bump minor
```

### Publish the prepared release

```sh
bash ./.github/skills/release/bin/push-release.sh --repo-root . --tag v0.7.0
```
