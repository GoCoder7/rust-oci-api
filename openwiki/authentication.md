---
title: Authentication
description: How oci-api selects between API key and Instance Principal authentication and wires request signing.
type: Technical reference
verified:
  - by: openwiki/0.4.0
    at: 2026-09-03T14:32:33.099Z
sources:
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-aba3482a522770448923fcd2
    resource: repo://src/auth/providers/api_key.rs
  - id: openwiki-source-5b4d4465bffb3bcb6598d60f
    resource: repo://src/auth/providers/instance_principal.rs
  - id: openwiki-source-7eff91cc1a8a398296aaee03
    resource: repo://src/client/http.rs
generated: {by: "agent", at: "2026-09-03T14:32:33.099Z"}
---

# Authentication

`oci-api` supports two authentication modes through `AuthMode`: `ApiKey` and `InstancePrincipal`. `Oci::from_env()` resolves which mode to use and then delegates to the corresponding environment bootstrap path.

## Mode resolution

When `OCI_AUTH_MODE` is set, the client treats that as the explicit override. When it is unset, `resolve_auth_mode_from_env()` probes OCI metadata with a short timeout and selects Instance Principal when metadata is reachable; otherwise it falls back to API key mode.

## API key flow

API key bootstrap merges partial values from `OCI_CONFIG` with per-field environment overrides for `OCI_USER_ID`, `OCI_TENANCY_ID`, `OCI_REGION`, and `OCI_FINGERPRINT`. The private key comes from `OCI_PRIVATE_KEY` when present, or from the resolved `OCI_CONFIG` entry otherwise, and `OCI_COMPARTMENT_ID` remains optional.

## Instance Principal flow

Instance Principal bootstrap can derive `OCI_REGION` from metadata region info and `OCI_TENANCY_ID` from the instance identity certificate when those values are not supplied directly. The resolved metadata also feeds the realm-domain component, and an explicit `OCI_METADATA_BASE_URL` override is forwarded into the builder configuration when needed for local mock environments.

## Provider wiring

Both auth modes ultimately feed the same signing interface. The API key path signs through `ApiKeyAuthProvider`, while the Instance Principal path resolves a cached signer state and then signs the request through `InstancePrincipalAuthProvider`.
