# Tasks

- [ ] **Dependency**: Add `md5`, `sha2`, `crc32c`, `base64` (if not present) to `Cargo.toml`. <!-- id: 4 -->
- [ ] **Test**: Create a test case in `tests/object_storage_integration_test.rs` (or a unit test) that mocks a response with various checksum headers and asserts they are present in the response object. <!-- id: 0 -->
- [ ] **Implement**: Add checksum fields (`content_md5`, `content_sha256`, `content_sha384`, `content_crc32c`) to `Object` struct in `src/services/object_storage/models.rs`. <!-- id: 1 -->
- [ ] **Implement**: Update `src/services/object_storage/client.rs` to extract these headers. <!-- id: 2 -->
- [ ] **Implement**: Add `verify_checksums` method to `Object` struct to validate integrity. <!-- id: 5 -->
- [ ] **Refactor**: Ensure the fields are public and documented. <!-- id: 3 -->
