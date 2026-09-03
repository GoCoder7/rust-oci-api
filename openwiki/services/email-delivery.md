---
title: Email Delivery
description: How the EmailDelivery client initializes, sends mail, lists approved senders, and supports test doubles through EmailSender.
type: Technical reference
verified:
  - by: openwiki/0.4.0
    at: 2026-09-03T14:32:33.099Z
sources:
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-e3cad0c5b15b93c62c590c8e
    resource: repo://src/services/email/client.rs
  - id: openwiki-source-d9eee9a73bc8e2adc1ccaadd
    resource: repo://src/services/email/sender_trait.rs
  - id: openwiki-source-8ab30c6e68877c3bd3b92865
    resource: repo://tests/real_oci_integration_test.rs
generated: {by: "agent", at: "2026-09-03T14:32:33.099Z"}
---

# Email Delivery

`EmailDelivery` is a tenancy-scoped service client that bootstraps itself from an `Oci` instance. Construction loads the Email Delivery configuration up front and caches the HTTP submit endpoint for later sends.

## Initialization and configuration lookup

`EmailDelivery::new()` derives the tenancy ID and region from `Oci`, calls the internal configuration lookup, and stores the returned `http_submit_endpoint`. That lookup uses the control-plane host pattern `ctrl.email.{region}.oci.{realm_domain}`.

## Sending mail

`send()` delegates to `send_impl()`. Before the request goes out, the client fills `email.sender.compartment_id` from `Oci::compartment_id()` when the caller has not already supplied it. The submit call posts JSON to `/20220926/actions/submitEmail` on the cached submit endpoint.

## Listing approved senders

`list_senders()` calls the Email Delivery control plane and supports optional filters for lifecycle state and email address. The real OCI integration suite treats those reads as ignored tests that run only when credentials are present.

## Testing seam

`EmailDelivery` implements the `EmailSender` trait, so application code can depend on `dyn EmailSender` and swap in mock implementations in tests. The README demonstrates that pattern directly, and the trait itself only requires a single async `send()` method.
