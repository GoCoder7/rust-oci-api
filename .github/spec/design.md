# Design

## Architecture
- **Clean Architecture**: The changes will be in the `services/object_storage` module, specifically in `models.rs` for the response structure and `client.rs` for the logic if needed, or just exposing the data for the caller to verify.
- **Response Object**: Update the `GetObject` response struct to include `content_md5` and `opc_multipart_md5` fields.

## Data Flow
1. Client calls `get_object`.
2. `oci-api` sends HTTP request.
3. OCI responds with headers including `Content-MD5`.
4. `oci-api` parses headers and populates the response struct.
5. Client (or internal helper) calculates MD5 of the body and compares.

## Library/Tools
- **MD5**: Add `md5` crate to `Cargo.toml`.
- **SHA2**: Add `sha2` crate to `Cargo.toml` (for SHA256, SHA384).
- **CRC32C**: Add `crc32c` crate to `Cargo.toml`.
- **Base64**: The headers are base64 encoded.

## Implementation Details
- **File**: `src/services/object_storage/models.rs`
    - Struct: `GetObjectResponse` (or similar)
    - Fields:
        - `pub content_md5: Option<String>`
        - `pub content_sha256: Option<String>`
        - `pub content_sha384: Option<String>`
        - `pub content_crc32c: Option<String>`
- **File**: `src/services/object_storage/client.rs`
    - Update the mapping from `reqwest::Response` to `GetObjectResponse`.
    - Headers to look for:
        - `content-md5`
        - `opc-multipart-md5`
        - `opc-content-sha256` (assumed)
        - `opc-content-sha384` (assumed)
        - `opc-content-crc32c` (assumed)

## Verification Logic
- Implement a `verify_checksums(&self) -> Result<()>` method on the `Object` struct (or similar).
- This method will calculate the checksum of the `value` (content) and compare it with the available headers.
- If any checksum is present and doesn't match, return an error.

