# Changelog

All notable changes to this project will be documented in this file.

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
