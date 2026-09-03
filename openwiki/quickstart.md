---
title: Quickstart
description: Fast entry points for configuring oci-api, choosing the right service page, and finding validation or release guidance.
type: Getting started guide
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

# Quickstart

Start with `Oci` from the crate root, then choose the service client you need.

## 1. Configure the crate

Install `oci-api` `0.9.0` with Tokio. The crate defaults to `native-tls`, and you can switch to `rustls-tls` when you want to avoid the native-tls/OpenSSL path.

Then decide how the runtime will authenticate:

- use [Authentication](./authentication.md) for API key vs. Instance Principal behavior
- use `Oci::from_env()` for environment-driven startup
- use `Oci::builder()` when you need explicit programmatic configuration

## 2. Pick the service page

- use [Object Storage](./services/object-storage.md) for byte-oriented object uploads, downloads, checksums, and deletes
- use [Email Delivery](./services/email-delivery.md) for configuration lookup, mail submission, and sender listing
- use [Vault and Keys](./services/vault-and-keys.md) for secret-bundle reads, secret decoding, key lookup, and key rotation

## 3. Validate against real OCI

Before relying on a live flow, read [Testing](./testing.md). The repository keeps live OCI coverage in ignored integration tests and documents the required environment variables for both auth modes and Object Storage namespace/bucket setup.

## 4. Release carefully

When the public API changes, use [Release Workflow](./release-workflow.md). The repository already carries a staged release skill that keeps `Cargo.toml`, `Cargo.lock`, `README.md`, and `CHANGELOG.md` in sync before publish.
