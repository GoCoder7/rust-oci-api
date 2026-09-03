---
title: Object Storage
description: The Object Storage client surface, including bucket access, byte-oriented object APIs, checksum handling, and delete support.
type: Technical reference
verified:
  - by: openwiki/0.4.0
    at: 2026-09-03T14:32:33.099Z
sources:
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-54b0b91d430e23c133fcf8b2
    resource: repo://src/services/object_storage/client.rs
  - id: openwiki-source-755914665c47e3ce4c95c015
    resource: repo://src/services/object_storage/models.rs
  - id: openwiki-source-11aadf5d6b0d083cc9f7ed24
    resource: repo://src/services/object_storage/tests.rs
  - id: openwiki-source-3ef6a182b800b985b0a69843
    resource: repo://tests/object_storage_integration_test.rs
generated: {by: "agent", at: "2026-09-03T14:32:33.099Z"}
---

# Object Storage

`ObjectStorage` is a namespace-scoped client built from `Oci`. It derives the service host from the OCI region and realm domain, then exposes `get_bucket()` to produce a bucket-scoped client.

## Bucket access

`get_bucket()` performs a GET against `/n/{namespace}/b/{bucket}/` before returning the `Bucket` handle. From there, object calls reuse the shared request executor and the bucket-specific path prefix.

## Byte-oriented object API

The current object API is binary-safe. `put_object()`, `put_object_with_checksum()`, and `get_or_create_object()` accept byte-oriented input through `AsRef<[u8]>`, while `get_object()` returns the exact response bytes without UTF-8 decoding. `delete_object()` removes the object path and preserves the existing `ApiError` behavior on unsuccessful responses.

## Checksums and text conversion

`Object.value` is `Bytes`, not `String`. The object model stores the default MD5 plus one optional additional checksum (`SHA256`, `SHA384`, or `CRC32C`), exposes `try_utf8()` for callers that want text, and can verify all present checksums through `verify_checksums()`.

## Validation coverage

The unit tests now cover non-UTF-8 uploads, non-UTF-8 downloads, successful deletes, and DELETE error propagation. The ignored real OCI integration test uploads raw bytes, round-trips them through `get_object()`, and then deletes the object again.
