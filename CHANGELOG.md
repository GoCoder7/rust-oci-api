# Changelog

All notable changes to this project will be documented in this file.

## [0.7.0] - 2026-05-13

### Added

- Envless Instance Principal bootstrap on OCI-hosted runtimes by auto-discovering region and tenancy information from OCI metadata and the instance identity certificate.
- Optional OCI-hosted validation probes for Vault / Keys, Email Delivery, and Object Storage while verifying Instance Principal flows against real OCI services.

### Changed

- Instance Principal signing and endpoint handling now cover PKCS#1 keys, normalized KMS management endpoints, and realm-aware service host construction.
- README now focuses on consumer usage and separates release tooling into a staged, approval-gated workflow.

## [0.6.0] - 2025-04-01

### Added

- `EmailSender` trait for email sending abstraction — enables dependency injection and mock implementations for testing
- `async_trait` re-export from crate root for convenience when implementing `EmailSender`
- `sender_trait` module in `email` service
- Documentation and examples for trait-based testing patterns in README and doc comments

### Changed

- `EmailDelivery::send()` now delegates to internal `send_impl()` helper, shared with `EmailSender` trait implementation
- No breaking changes — existing `EmailDelivery::send()` calls continue to work without modification

## [0.5.0] - 2025-03-18

- Initial public release with Email Delivery and Object Storage support
