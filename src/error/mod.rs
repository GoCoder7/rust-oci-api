//! OCI API error type definitions
//!
//! This module defines all possible errors that can occur when using OCI API.

use thiserror::Error;

/// OCI API error type
#[derive(Debug, Error)]
pub enum OciError {
    /// Configuration file related error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Environment variable related error
    #[error("Environment variable error: {0}")]
    EnvError(String),

    /// Authentication related error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Private key file related error
    #[error("Private key error: {0}")]
    KeyError(String),

    /// HTTP request/response error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// API response error (error returned by OCI API)
    #[error("API error (code: {code}): {message}")]
    ApiError {
        /// Error code
        code: String,
        /// Error message
        message: String,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// INI file parsing error
    #[error("INI file parsing error: {0}")]
    IniError(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, OciError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let error = OciError::ConfigError("test message".to_string());
        assert_eq!(error.to_string(), "Configuration error: test message");
    }

    #[test]
    fn test_env_error() {
        let error = OciError::EnvError("OCI_USER_ID is not set".to_string());
        assert_eq!(
            error.to_string(),
            "Environment variable error: OCI_USER_ID is not set"
        );
    }

    #[test]
    fn test_auth_error() {
        let error = OciError::AuthError("Failed to sign request".to_string());
        assert_eq!(
            error.to_string(),
            "Authentication error: Failed to sign request"
        );
    }

    #[test]
    fn test_key_error() {
        let error = OciError::KeyError("Private key file not found".to_string());
        assert_eq!(
            error.to_string(),
            "Private key error: Private key file not found"
        );
    }

    #[test]
    fn test_api_error() {
        let error = OciError::ApiError {
            code: "404".to_string(),
            message: "Resource not found".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "API error (code: 404): Resource not found"
        );
    }

    #[test]
    fn test_ini_error() {
        let error = OciError::IniError("Failed to parse INI file".to_string());
        assert_eq!(
            error.to_string(),
            "INI file parsing error: Failed to parse INI file"
        );
    }

    #[test]
    fn test_other_error() {
        let error = OciError::Other("Something went wrong".to_string());
        assert_eq!(error.to_string(), "Other error: Something went wrong");
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: OciError = io_error.into();
        assert!(matches!(error, OciError::IoError(_)));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error: OciError = json_error.into();
        assert!(matches!(error, OciError::JsonError(_)));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        fn returns_error() -> Result<i32> {
            Err(OciError::ConfigError("test".to_string()))
        }

        assert_eq!(returns_result().unwrap(), 42);
        assert!(returns_error().is_err());
    }
}
