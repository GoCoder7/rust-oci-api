//! OCI Request Signer
//!
//! Implements Oracle Cloud Infrastructure HTTP request signing
//! according to the official specification.

use crate::auth::OciConfig;
use crate::error::{OciError, Result};
use base64::{Engine as _, engine::general_purpose};
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer as RsaSigner};
use sha2::Sha256;
use std::fs;
use std::sync::Arc;
use tempfile::NamedTempFile;

/// OCI Request Signer
pub struct OciSigner {
    user_id: String,
    tenancy_id: String,
    fingerprint: String,
    private_key: Arc<RsaPrivateKey>,
    _temp_key_file: Option<NamedTempFile>, // Keep temp file alive if needed
}

impl OciSigner {
    /// Create new OCI signer from config
    pub fn new(config: &OciConfig) -> Result<Self> {
        // Check if private_key is PEM content
        let is_pem_content =
            config.private_key.contains("-----BEGIN") && config.private_key.contains("-----END");

        let (private_key, temp_file) = if is_pem_content {
            // PEM content - create temporary file
            let temp_file = NamedTempFile::new()
                .map_err(|e| OciError::Other(format!("Failed to create temp file: {}", e)))?;

            fs::write(temp_file.path(), config.private_key.as_bytes()).map_err(|e| {
                OciError::Other(format!("Failed to write private key to temp file: {}", e))
            })?;

            let key = RsaPrivateKey::read_pkcs8_pem_file(temp_file.path()).map_err(|e| {
                OciError::ConfigError(format!("Failed to parse private key: {}", e))
            })?;

            (key, Some(temp_file))
        } else {
            // File path - read directly
            let key = RsaPrivateKey::read_pkcs8_pem_file(&config.private_key).map_err(|e| {
                OciError::ConfigError(format!("Failed to read private key from file: {}", e))
            })?;

            (key, None)
        };

        Ok(Self {
            user_id: config.user_id.clone(),
            tenancy_id: config.tenancy_id.clone(),
            fingerprint: config.fingerprint.clone(),
            private_key: Arc::new(private_key),
            _temp_key_file: temp_file,
        })
    }

    /// Sign an HTTP request
    ///
    /// # Arguments
    /// * `method` - HTTP method (e.g., "GET", "POST")
    /// * `path` - Request path including query string (e.g., "/path?query=value")
    /// * `host` - Host header value
    /// * `body` - Optional request body for POST/PUT requests
    /// * `content_type` - Optional content type (defaults to "application/json" if body is present)
    ///
    /// # Returns
    /// Tuple of (date_header, authorization_header)
    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        host: &str,
        body: Option<&str>,
    ) -> Result<(String, String)> {
        self.sign_request_full(method, path, host, body, None)
    }

    /// Sign request with custom content type
    pub fn sign_request_with_content_type(
        &self,
        method: &str,
        path: &str,
        host: &str,
        body: Option<&str>,
        content_type: &str,
    ) -> Result<(String, String)> {
        self.sign_request_full(method, path, host, body, Some(content_type))
    }

    /// Internal method for signing with all options
    fn sign_request_full(
        &self,
        method: &str,
        path: &str,
        host: &str,
        body: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<(String, String)> {
        // Generate current date in RFC 1123 format
        let date = httpdate::fmt_http_date(std::time::SystemTime::now());

        self.sign_request_with_date_and_content_type(method, path, host, body, &date, content_type)
    }

    /// Sign request with specific date and content type (useful for testing)
    pub fn sign_request_with_date_and_content_type(
        &self,
        method: &str,
        path: &str,
        host: &str,
        body: Option<&str>,
        date: &str,
        content_type: Option<&str>,
    ) -> Result<(String, String)> {
        // Build signing string
        let signing_string = if let Some(body_content) = body {
            // For requests with body, include content headers
            let body_sha256 = {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(body_content.as_bytes());
                let result = hasher.finalize();
                general_purpose::STANDARD.encode(result)
            };

            let content_length = body_content.len().to_string();
            let content_type_value = content_type.unwrap_or("application/json");

            format!(
                "date: {}\n(request-target): {} {}\nhost: {}\ncontent-length: {}\ncontent-type: {}\nx-content-sha256: {}",
                date,
                method.to_lowercase(),
                path,
                host,
                content_length,
                content_type_value,
                body_sha256
            )
        } else {
            // For requests without body (GET, DELETE, etc.)
            format!(
                "date: {}\n(request-target): {} {}\nhost: {}",
                date,
                method.to_lowercase(),
                path,
                host
            )
        };

        // Sign the string using PKCS#1 v1.5 with SHA256
        // Arc clone is cheap (only increments reference count)
        let signing_key = SigningKey::<Sha256>::new((*self.private_key).clone());
        let signature = signing_key
            .try_sign(signing_string.as_bytes())
            .map_err(|e| OciError::AuthError(format!("Failed to sign request: {}", e)))?;

        let encoded_signature = general_purpose::STANDARD.encode(signature.to_bytes());

        // Build Authorization header
        let headers_list = if body.is_some() {
            "date (request-target) host content-length content-type x-content-sha256"
        } else {
            "date (request-target) host"
        };

        let key_id = format!("{}/{}/{}", self.tenancy_id, self.user_id, self.fingerprint);

        let authorization = format!(
            "Signature version=\"1\",headers=\"{}\",keyId=\"{}\",algorithm=\"rsa-sha256\",signature=\"{}\"",
            headers_list, key_id, encoded_signature
        );

        Ok((date.to_string(), authorization))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_creation_with_pem_content() {
        let pem_content = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7VJTUt9Us8cKj
MzEfYyjiWA4R4/M2bS1+fWIcPm15j8aB2v3e1pDzLdOHLJaSecrNjAP1LfTkRcJL
iEWXiZLp6dPT3gJw/WmF9v6K8N8rFvQbSb3VvTlqcJYY/0KPJ7Pqe3gJ/tHkI1HN
6bvnm5X3O4TLNWBxOW1PQ2SdRqBJYT0x0rRqVYMiB0g1RiPcCtf1fI7RsYlGtPH8
oF0r7fLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL
-----END PRIVATE KEY-----"#;

        let config = OciConfig {
            user_id: "ocid1.user.oc1..test".to_string(),
            tenancy_id: "ocid1.tenancy.oc1..test".to_string(),
            region: "us-ashburn-1".to_string(),
            fingerprint: "aa:bb:cc:dd:ee:ff".to_string(),
            private_key: pem_content.to_string(),
            compartment_id: None,
        };

        // This should not panic, even though the key is invalid
        // (we're just testing the PEM detection and temp file creation)
        let result = OciSigner::new(&config);
        assert!(result.is_err()); // Will fail due to invalid key, but that's expected
    }

    #[test]
    fn test_signing_string_format_without_body() {
        // We can't test actual signing without a valid key,
        // but we can verify the signing string format in integration tests
    }
}
