---
openwiki_repo_policy_version: "1"
openwiki_repo_policy_status: active
openwiki_repo_policy_automation: manual
openwiki_repo_temp_plan: temp/openwiki/plan.json
openwiki_repo_temp_page_claims: temp/openwiki/page-claims.json
openwiki_repo_temp_viewer_export: temp/openwiki-viewer
---

# OpenWiki Instructions for oci-api

These instructions define the repo-local policy for host-driven OpenWiki runs in this repository.

## Scope

- Use `src/`, `tests/`, `examples/`, `README.md`, `CHANGELOG.md`, `Cargo.toml`, and `.github/skills/release/` as the main source material.
- Focus on public API behavior, authentication flow, service capabilities, examples, integration-test expectations, and release workflow.
- When existing `openwiki/` pages are present, use them only as prior output to revise rather than as authoritative evidence.

## Exclusions

- Exclude `target/`, `temp/`, `.env*`, `.oci/`, and `tests/fixtures/`.
- Exclude `.github/specs/` from canonical product documentation unless the page explicitly covers project history or implementation planning.
- Exclude local runtime leftovers, downloaded test assets, and any future generated viewer exports.

## Preferred taxonomy

- Keep stable top-level pages such as `overview.md`, `authentication.md`, `testing.md`, and `release-workflow.md`.
- Keep service-focused content under `services/` with separate pages for `object-storage.md`, `email-delivery.md`, and `vault-and-keys.md`.
- Keep operational or contributor-oriented runtime details under `operations/` when they do not belong in the core API overview.

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

## Review rules

1. Every submitted claim must include precise `repo://...` evidence.
2. Prefer code over README prose when they disagree.
3. Keep examples aligned with the current public API signatures, feature flags, and auth-mode behavior.
4. Mention ignored integration tests and required OCI environment variables when documenting real OCI validation.
5. Do not present temporary artifacts, generated output, or speculative future work as stable repository behavior.
