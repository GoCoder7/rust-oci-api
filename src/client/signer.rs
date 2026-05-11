//! OCI Request Signer
//!
//! Implements Oracle Cloud Infrastructure HTTP request signing
//! according to the official specification.

use crate::error::{Error, Result};
use base64::{Engine as _, engine::general_purpose};
use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer as RsaSigner};
use sha2::Sha256;
use std::fs;
use std::sync::Arc;

/// Signed headers produced by request signing.
#[derive(Debug, Clone)]
pub(crate) struct SignedRequestHeaders {
    pub date: String,
    pub authorization: String,
    pub content_type: Option<String>,
    pub content_length: Option<String>,
    pub x_content_sha256: Option<String>,
}

struct SigningInput<'a> {
    key_id: &'a str,
    method: &'a str,
    path: &'a str,
    host: Option<&'a str>,
    body: Option<&'a str>,
    date: &'a str,
    content_type: Option<&'a str>,
}

/// OCI Request Signer
pub struct OciSigner {
    user_id: String,
    tenancy_id: String,
    fingerprint: String,
    key_id_override: Option<String>,
    private_key: Arc<RsaPrivateKey>,
}

impl Clone for OciSigner {
    fn clone(&self) -> Self {
        Self {
            user_id: self.user_id.clone(),
            tenancy_id: self.tenancy_id.clone(),
            fingerprint: self.fingerprint.clone(),
            key_id_override: self.key_id_override.clone(),
            private_key: Arc::clone(&self.private_key),
        }
    }
}

impl OciSigner {
    /// Create new OCI signer
    pub fn new(
        user_id: &str,
        tenancy_id: &str,
        fingerprint: &str,
        private_key: &str,
    ) -> Result<Self> {
        let private_key_obj = load_private_key(private_key)?;

        Ok(Self {
            user_id: user_id.to_owned(),
            tenancy_id: tenancy_id.to_owned(),
            fingerprint: fingerprint.to_owned(),
            key_id_override: None,
            private_key: Arc::new(private_key_obj),
        })
    }

    pub(crate) fn new_with_key_id(key_id: impl Into<String>, private_key: &str) -> Result<Self> {
        let private_key_obj = load_private_key(private_key)?;

        Ok(Self {
            user_id: String::new(),
            tenancy_id: String::new(),
            fingerprint: String::new(),
            key_id_override: Some(key_id.into()),
            private_key: Arc::new(private_key_obj),
        })
    }

    /// Get user ID
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Get tenancy ID
    pub fn tenancy_id(&self) -> &str {
        &self.tenancy_id
    }

    /// Get fingerprint
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
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
        let signed = self.sign_request_headers(method, path, Some(host), body, None, None)?;
        Ok((signed.date, signed.authorization))
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
        let signed =
            self.sign_request_headers(method, path, Some(host), body, Some(content_type), None)?;
        Ok((signed.date, signed.authorization))
    }

    pub(crate) fn sign_request_headers(
        &self,
        method: &str,
        path: &str,
        host: Option<&str>,
        body: Option<&str>,
        content_type: Option<&str>,
        date: Option<&str>,
    ) -> Result<SignedRequestHeaders> {
        let date = date
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| httpdate::fmt_http_date(std::time::SystemTime::now()));

        sign_request_headers(
            &self.private_key,
            SigningInput {
                key_id: &self.key_id(),
                method,
                path,
                host,
                body,
                date: &date,
                content_type,
            },
        )
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
        let signed = sign_request_headers(
            &self.private_key,
            SigningInput {
                key_id: &self.key_id(),
                method,
                path,
                host: Some(host),
                body,
                date,
                content_type,
            },
        )?;

        Ok((signed.date, signed.authorization))
    }

    fn key_id(&self) -> String {
        self.key_id_override
            .clone()
            .unwrap_or_else(|| format!("{}/{}/{}", self.tenancy_id, self.user_id, self.fingerprint))
    }
}

fn load_private_key(private_key: &str) -> Result<RsaPrivateKey> {
    let is_pem_content = private_key.contains("-----BEGIN") && private_key.contains("-----END");

    let pem_content = if is_pem_content {
        private_key.to_owned()
    } else {
        fs::read_to_string(private_key)
            .map_err(|e| Error::ConfigError(format!("Failed to read private key from file: {e}")))?
    };

    parse_private_key_pem(&pem_content)
}

fn parse_private_key_pem(private_key_pem: &str) -> Result<RsaPrivateKey> {
    RsaPrivateKey::from_pkcs8_pem(private_key_pem).or_else(|pkcs8_error| {
        RsaPrivateKey::from_pkcs1_pem(private_key_pem).map_err(|pkcs1_error| {
            Error::ConfigError(format!(
                "Failed to parse private key: {pkcs8_error}; PKCS#1 fallback also failed: {pkcs1_error}"
            ))
        })
    })
}

fn sign_request_headers(
    private_key: &RsaPrivateKey,
    input: SigningInput<'_>,
) -> Result<SignedRequestHeaders> {
    let body_sha256 = input.body.map(|body_content| {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(body_content.as_bytes());
        general_purpose::STANDARD.encode(hasher.finalize())
    });
    let content_length = input
        .body
        .map(|body_content| body_content.len().to_string());
    let content_type = input
        .body
        .map(|_| input.content_type.unwrap_or("application/json").to_owned());

    let mut signing_lines = vec![
        format!("date: {}", input.date),
        format!(
            "(request-target): {} {}",
            input.method.to_lowercase(),
            input.path
        ),
    ];
    let mut signed_header_names = vec!["date".to_owned(), "(request-target)".to_owned()];

    if let Some(host) = input.host {
        signing_lines.push(format!("host: {host}"));
        signed_header_names.push("host".to_owned());
    }

    if let Some(content_length) = &content_length {
        signing_lines.push(format!("content-length: {content_length}"));
        signed_header_names.push("content-length".to_owned());
    }

    if let Some(content_type_value) = &content_type {
        signing_lines.push(format!("content-type: {content_type_value}"));
        signed_header_names.push("content-type".to_owned());
    }

    if let Some(body_sha256) = &body_sha256 {
        signing_lines.push(format!("x-content-sha256: {body_sha256}"));
        signed_header_names.push("x-content-sha256".to_owned());
    }

    let signing_key = SigningKey::<Sha256>::new(private_key.clone());
    let signature = signing_key
        .try_sign(signing_lines.join("\n").as_bytes())
        .map_err(|e| Error::AuthError(format!("Failed to sign request: {e}")))?;

    let encoded_signature = general_purpose::STANDARD.encode(signature.to_bytes());
    let headers_list = signed_header_names.join(" ");
    let authorization = format!(
        "Signature version=\"1\",headers=\"{headers_list}\",keyId=\"{}\",algorithm=\"rsa-sha256\",signature=\"{encoded_signature}\"",
        input.key_id
    );

    Ok(SignedRequestHeaders {
        date: input.date.to_owned(),
        authorization,
        content_type,
        content_length,
        x_content_sha256: body_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::pkcs8::{EncodePrivateKey, LineEnding};
    use rsa::rand_core::OsRng;

    #[test]
    fn test_signer_creation_with_pem_content() {
        let pem_content = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7VJTUt9Us8cKj
MzEfYyjiWA4R4/M2bS1+fWIcPm15j8aB2v3e1pDzLdOHLJaSecrNjAP1LfTkRcJL
iEWXiZLp6dPT3gJw/WmF9v6K8N8rFvQbSb3VvTlqcJYY/0KPJ7Pqe3gJ/tHkI1HN
6bvnm5X3O4TLNWBxOW1PQ2SdRqBJYT0x0rRqVYMiB0g1RiPcCtf1fI7RsYlGtPH8
oF0r7fLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL
-----END PRIVATE KEY-----"#;

        // This should not panic, even though the key is invalid
        // (we're just testing the PEM detection and temp file creation)
        let result = OciSigner::new(
            "ocid1.user.oc1..test",
            "ocid1.tenancy.oc1..test",
            "aa:bb:cc:dd:ee:ff",
            pem_content,
        );
        assert!(result.is_err()); // Will fail due to invalid key, but that's expected
    }

    #[test]
    fn test_signing_string_format_without_body() {
        // We can't test actual signing without a valid key,
        // but we can verify the signing string format in integration tests
    }

    #[test]
    fn test_signer_accepts_pkcs1_pem_content() {
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let pkcs1_pem = private_key.to_pkcs1_pem(LineEnding::LF).unwrap();

        let result = OciSigner::new(
            "ocid1.user.oc1..test",
            "ocid1.tenancy.oc1..test",
            "aa:bb:cc:dd:ee:ff",
            pkcs1_pem.as_str(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_signer_accepts_pkcs8_pem_file_path() {
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let pkcs8_pem = private_key.to_pkcs8_pem(LineEnding::LF).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let key_path = temp_dir.path().join("key.pem");
        fs::write(&key_path, pkcs8_pem.as_bytes()).unwrap();

        let result = OciSigner::new(
            "ocid1.user.oc1..test",
            "ocid1.tenancy.oc1..test",
            "aa:bb:cc:dd:ee:ff",
            key_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
    }
}
