---
name: openwiki-repo
description: Repo-local OpenWiki policy for rust-oci-api. Use it to keep wiki content aligned with the crate's public API, auth flow, service clients, and release workflow.
openwiki_repo_skill_status: active
---

# OpenWiki Repo Policy for rust-oci-api

This repository uses OpenWiki in `manual` mode. The main agent remains responsible for repository investigation, Markdown authorship, claim selection, and verification before page submissions.

## Scope

- Treat `src/`, `tests/`, `examples/`, `README.md`, `CHANGELOG.md`, `Cargo.toml`, and `.github/skills/release/` as the authoritative sources for wiki content.
- Cover the public API surface, authentication modes, service clients, examples, integration-test expectations, and release workflow.
- Use existing `openwiki/` pages only as prior output to revise, never as the primary source of truth.

## Exclusions

- Exclude `target/`, `temp/`, `.env*`, `.oci/`, `tests/fixtures/`, and other local-only artifacts.
- Do not treat `.github/specs/` planning artifacts as canonical product behavior unless a page explicitly documents project history or implementation planning.

## Preferred taxonomy

- Keep stable top-level pages such as `overview.md`, `authentication.md`, `testing.md`, and `release-workflow.md`.
- Group service documentation under `services/` with separate pages for `object-storage.md`, `email-delivery.md`, and `vault-and-keys.md`.
- Group contributor or runtime operation details under `operations/` when they are not part of the public API surface.

## Seed paths

- `src/lib.rs`
- `src/client/http.rs`
- `src/client/request_executor.rs`
- `src/client/signer.rs`
- `src/auth/providers/instance_principal.rs`
- `src/services/object_storage/client.rs`
- `src/services/object_storage/models.rs`
- `src/services/email/client.rs`
- `src/services/vault/client.rs`
- `src/services/keys/client.rs`
- `README.md`
- `tests/object_storage_integration_test.rs`
- `.github/skills/release/SKILL.md`

## Workflow

1. Start with `README.md` and `src/lib.rs` to anchor the crate's public surface before expanding into service-specific files.
2. Follow the client stack in `src/client/` and `src/auth/providers/` when documenting request signing, auth-mode selection, or instance principal behavior.
3. Document service pages from their implementation files under `src/services/`, then cross-check examples and ignored integration tests for user-facing behavior and prerequisites.
4. When release or contributor workflow pages are needed, use `.github/skills/release/` plus `CHANGELOG.md` and `Cargo.toml` as the primary evidence.
5. Submit only claims that can be proven from repository sources with stable `repo://...` citations.

## Review rules

1. Every claim must carry `repo://...` evidence that points to the exact source file and line span.
2. Prefer code over README prose when they differ.
3. Keep examples aligned with current public signatures, feature flags, and authentication behavior.
4. Call out ignored integration tests and required OCI environment variables when documenting real-service validation.
5. Do not submit wiki pages that describe generated output, temporary artifacts, or unmerged speculative behavior as established API.
