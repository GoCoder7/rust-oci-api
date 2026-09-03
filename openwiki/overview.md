---
title: Overview
description: High-level map of the oci-api crate, its exported entry points, supported services, and feature defaults.
type: Overview
verified:
  - by: openwiki/0.4.0
    at: 2026-09-03T14:32:33.099Z
sources:
  - id: openwiki-source-651d1fb6c9e49916a916ab51
    resource: repo://Cargo.toml
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-7eff91cc1a8a398296aaee03
    resource: repo://src/client/http.rs
  - id: openwiki-source-ed8bf05e307c6278442542c2
    resource: repo://src/lib.rs
generated: {by: "agent", at: "2026-09-03T14:32:33.099Z"}
---

# Overview

`oci-api` is a Rust client library for Oracle Cloud Infrastructure APIs. Its current public surface centers on `Oci`, `AuthMode`, `OciBuilder`, `Result`, `Error`, and a crate-level `Bytes` re-export for byte-oriented Object Storage payloads.

## Supported services

The repository currently documents and ships client surfaces for Email Delivery, Object Storage, Vault Secrets, and Keys. These live under `src/services/` and are re-exported from the crate root for convenient imports.

## Client entry points

`Oci` is the central runtime entry point. From a configured client you can create:

- `email_delivery()` for Email Delivery
- `object_storage(namespace)` for Object Storage
- `vault()` for Vault Secrets
- `keys(management_endpoint)` for Keys

## Build and feature defaults

The crate currently targets version `0.9.0`. It uses Tokio as the async runtime, defaults to a `native-tls` feature set, and exposes a `rustls-tls` alternative for consumers that want to avoid the native-tls/OpenSSL path.

## Where to read next

- See [Authentication](./authentication.md) for auth-mode selection and environment bootstrap.
- See [Object Storage](./services/object-storage.md) for the binary-safe object API.
- See [Testing](./testing.md) for ignored integration tests and live OCI prerequisites.
- See [Release Workflow](./release-workflow.md) for the staged release process.
