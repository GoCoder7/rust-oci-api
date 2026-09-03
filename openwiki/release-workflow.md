---
title: Release Workflow
description: The staged release process for oci-api, including helper scripts, semver bump decisions, and publish approval gates.
type: Process guide
verified:
  - by: openwiki/0.4.0
    at: 2026-09-03T14:32:33.099Z
sources:
  - id: openwiki-source-0aed2178f4f62eed99f7130e
    resource: repo://.github/skills/release/SKILL.md
  - id: openwiki-source-651d1fb6c9e49916a916ab51
    resource: repo://Cargo.toml
  - id: openwiki-source-ca6cb4b1a14fd7969dfae3ec
    resource: repo://CHANGELOG.md
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
generated: {by: "agent", at: "2026-09-03T14:32:33.099Z"}
---

# Release Workflow

This repository carries a dedicated `release` skill for publishing the crate. The workflow is intentionally staged: keep the worktree clean, choose a semver bump, preview the release plan, prepare the local release commit and tag, then ask again before publishing to crates.io and pushing to the remote.

## Release inputs

The release flow treats `Cargo.toml`, `Cargo.lock`, `README.md`, and `CHANGELOG.md` as the files that must stay synchronized. The helper scripts under `.github/skills/release/bin/` are the deterministic entry points for previewing and finalizing a release.

## Validation expectations

Before mutating the repository for a release, the workflow expects the crate checks to pass. The documented release checks are `cargo test`, `cargo clippy`, `cargo doc --no-deps`, and `cargo publish --dry-run`.

## Version synchronization

The current development version is `0.9.0`, the README install snippet now reflects `0.9.0`, and the changelog already contains an unreleased `0.9.0` section that captures the binary-safe Object Storage and TLS-backend work. That means the main release-facing files are already aligned around the next breaking release line.

## Publishing behavior

Publishing remains an explicit user-approved step. The release skill leaves room for a local-only prepared tag and commit when the user declines publishing, and it documents that crates.io publication happens before the git push when publish is approved.
