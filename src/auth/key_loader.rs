//! Private key file loader
//!
//! Reads RSA private key files in PEM format.

use crate::error::{OciError, Result};
use std::fs;
use std::path::Path;

/// Private key loader
pub struct KeyLoader;

impl KeyLoader {
    /// Load private key from input (automatically detects file path vs PEM content)
    ///
    /// # Arguments
    /// * `input` - Either a file path or PEM content string
    ///
    /// # Returns
    /// Private key string in PEM format
    ///
    /// # Examples
    /// ```no_run
    /// # use oci_api::auth::KeyLoader;
    /// // Load from file
    /// let key = KeyLoader::load("~/.oci/key.pem").unwrap();
    ///
    /// // Load from PEM content
    /// let key = KeyLoader::load("-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----").unwrap();
    /// ```
    pub fn load(input: &str) -> Result<String> {
        let trimmed = input.trim();

        // Check if it's PEM content (starts with -----BEGIN)
        if trimmed.starts_with("-----BEGIN") {
            // Validate and return PEM content
            Self::validate_pem(trimmed)?;
            return Ok(trimmed.to_string());
        }

        // Otherwise, treat as file path
        Self::load_from_file(input)
    }

    /// Load private key from file
    ///
    /// # Arguments
    /// * `path` - Private key file path
    ///
    /// # Returns
    /// Private key string in PEM format
    pub fn load_from_file(path: &str) -> Result<String> {
        let key_path = Path::new(path);

        if !key_path.exists() {
            return Err(OciError::KeyError(format!(
                "Private key file not found: {}",
                path
            )));
        }

        let content = fs::read_to_string(key_path)
            .map_err(|e| OciError::KeyError(format!("Failed to read private key file: {}", e)))?;

        // Validate PEM format
        Self::validate_pem(&content)?;

        Ok(content)
    }

    /// Validate PEM format
    ///
    /// Check if `-----BEGIN` and `-----END` exist
    fn validate_pem(content: &str) -> Result<()> {
        if !content.contains("-----BEGIN") || !content.contains("-----END") {
            return Err(OciError::KeyError("Not a valid PEM format".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_pem_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let pem_content =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----\n";
        temp_file.write_all(pem_content.as_bytes()).unwrap();

        let result = KeyLoader::load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), pem_content);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = KeyLoader::load_from_file("/nonexistent/path/to/key.pem");
        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::KeyError(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected KeyError"),
        }
    }

    #[test]
    fn test_validate_pem_valid() {
        let valid_pem = "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----";
        assert!(KeyLoader::validate_pem(valid_pem).is_ok());
    }

    #[test]
    fn test_validate_pem_missing_begin() {
        let invalid_pem = "some content\n-----END RSA PRIVATE KEY-----";
        let result = KeyLoader::validate_pem(invalid_pem);
        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::KeyError(msg) => assert!(msg.contains("valid PEM")),
            _ => panic!("Expected KeyError"),
        }
    }

    #[test]
    fn test_validate_pem_missing_end() {
        let invalid_pem = "-----BEGIN RSA PRIVATE KEY-----\nsome content";
        let result = KeyLoader::validate_pem(invalid_pem);
        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::KeyError(msg) => assert!(msg.contains("valid PEM")),
            _ => panic!("Expected KeyError"),
        }
    }

    #[test]
    fn test_validate_pem_empty() {
        let result = KeyLoader::validate_pem("");
        assert!(result.is_err());
    }
}
