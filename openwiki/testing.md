---
title: Testing
description: How the repository splits offline configuration tests from ignored live OCI integration tests and what each suite validates.
type: Process guide
verified:
  - by: openwiki/0.4.0
    at: 2026-09-03T14:32:33.099Z
sources:
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-773e96b86ab97bbf31aae604
    resource: repo://tests/auth_integration_test.rs
  - id: openwiki-source-3ef6a182b800b985b0a69843
    resource: repo://tests/object_storage_integration_test.rs
  - id: openwiki-source-8ab30c6e68877c3bd3b92865
    resource: repo://tests/real_oci_integration_test.rs
generated: {by: "agent", at: "2026-09-03T14:32:33.099Z"}
---

# Testing

The repository uses a mix of offline tests and ignored live OCI integration tests.

## Offline configuration coverage

`tests/auth_integration_test.rs` exercises the builder and config-loader workflow without making live OCI calls. It covers full builder construction, multi-profile config loading, path handling, missing-field validation, and region handling for API-key configuration.

## Ignored live OCI tests

The live suites are ignored by default and require explicit `cargo test --test ... -- --ignored` execution. Both Object Storage and the broader real-OCI suite accept either explicit API-key credentials or Instance Principal on an OCI runtime where IMDS is reachable.

## Object Storage live coverage

The Object Storage integration test requires `TEST_NAMESPACE` and `TEST_BUCKET`. It gets the bucket, uploads raw bytes, downloads the same bytes back, and deletes the object at the end of the test.

## Broader real OCI coverage

`real_oci_integration_test.rs` validates client creation from the environment and then exercises Email Delivery reads such as configuration lookup, endpoint caching, and sender listing. The full send flow is intentionally guarded because it depends on approved senders and optional recipient-related environment variables.
