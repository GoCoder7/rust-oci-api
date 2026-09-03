---
title: Vault and Keys
description: The current Vault Secrets and Keys client surfaces, including endpoint conventions and phase-1 scope.
type: Technical reference
verified:
  - by: openwiki/0.4.0
    at: 2026-09-03T14:32:33.099Z
sources:
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-4567ad98d3244e05f3e8619b
    resource: repo://src/services/keys/client.rs
  - id: openwiki-source-cb6c4ca12d1c30812c4bb74e
    resource: repo://src/services/vault/client.rs
  - id: openwiki-source-16f105b0eb69144c5f1eb769
    resource: repo://src/services/vault/models.rs
generated: {by: "agent", at: "2026-09-03T14:32:33.099Z"}
---

# Vault and Keys

The crate currently exposes read-focused Vault Secrets APIs and a small Keys surface through the main `Oci` client. Both services use the shared request executor and realm-aware host construction.

## Vault Secrets

`VaultSecretsClient::new()` derives the host as `secrets.vaults.{region}.oci.{realm_domain}`. The client currently supports three secret bundle reads:

- `get_secret_bundle(secret_id)`
- `get_secret_bundle_by_stage(secret_id, stage)`
- `get_secret_bundle_by_version(secret_id, version_number)`

`SecretBundleContent` exposes both `decoded_bytes()` and `decoded_string()` so callers can either consume the raw secret bytes or decode UTF-8 text.

## Keys

`KeysClient::new()` normalizes the supplied KMS management endpoint by stripping the scheme and trailing slash. The current Keys surface supports `get_key(key_id)` and `rotate_key(key_id)`, with the rotate action posted as JSON to `/20180608/keys/{key_id}/actions/rotate`.

## Current scope

The README explicitly keeps Vault in a phase-1 scope of current, staged, and versioned secret bundle reads, and keeps Keys in a phase-1 scope of key lookup plus rotate. This wiki page should stay aligned with that intentionally limited surface until the repository grows more Vault or KMS functionality.
