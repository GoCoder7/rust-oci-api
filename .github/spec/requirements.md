# Requirements

## User Story
As a user of the `oci-api` library, I want to verify the integrity of objects downloaded from Object Storage so that I can ensure the data has not been corrupted during transfer.

## Acceptance Criteria
1. **Retrieve Checksum**: The `GetObject` API response must include the checksum provided by OCI.
    - Mandatory: `Content-MD5`
    - Optional: `opc-content-sha256`, `opc-content-sha384`, `opc-content-crc32c` (or appropriate headers)
2. **Verify Integrity**: Provide a mechanism to compare the received object's data against the checksums from the response.
3. **Support Standard MD5**: Support the standard `Content-MD5` header.
4. **Support Multipart MD5**: Support `opc-multipart-md5` if applicable.
5. **Support Additional Algorithms**: SHA256, SHA384, CRC32C.
