//! OCI configuration struct definition
//!
//! This module defines the configuration structure required for OCI API authentication.

use crate::error::{OciError, Result};

/// OCI configuration
#[derive(Debug, Clone)]
pub struct OciConfig {
    /// User ID (OCID format)
    pub user_id: String,

    /// Tenancy ID (OCID format)
    pub tenancy_id: String,

    /// Region (e.g., ap-seoul-1)
    pub region: String,

    /// Private key fingerprint
    pub fingerprint: String,

    /// Private key content (PEM format)
    pub private_key: String,

    /// Compartment ID (OCID format, optional - defaults to tenancy_id if not set)
    pub compartment_id: Option<String>,
}

impl OciConfig {
    /// Load configuration from environment variables
    ///
    /// # Priority (highest to lowest):
    /// 1. Individual environment variables (OCI_USER_ID, etc.) - override everything
    /// 2. OCI_CONFIG content (if set) - provides base values  
    /// 3. Error if required fields are missing
    ///
    /// # Environment Variables
    ///
    /// ## Base configuration (lower priority):
    /// - `OCI_CONFIG`: INI content string or file path to OCI config file
    ///
    /// ## Override configuration (higher priority):
    /// - `OCI_USER_ID`: User ID (overrides value from OCI_CONFIG)
    /// - `OCI_TENANCY_ID`: Tenancy ID (overrides value from OCI_CONFIG)
    /// - `OCI_REGION`: Region (overrides value from OCI_CONFIG)
    /// - `OCI_FINGERPRINT`: Private key fingerprint (overrides value from OCI_CONFIG)
    /// - `OCI_PRIVATE_KEY`: Private key file path or PEM content (overrides key_file from OCI_CONFIG)
    /// - `OCI_COMPARTMENT_ID`: Compartment ID (optional, defaults to tenancy_id)
    ///
    /// # Private Key Loading
    ///
    /// Private key is loaded in the following priority:
    /// 1. `OCI_PRIVATE_KEY` environment variable (if set) - file path or PEM content
    /// 2. `key_file` field from `OCI_CONFIG` (if OCI_CONFIG is set and contains key_file)
    /// 3. Error if neither is available
    pub fn from_env() -> Result<Self> {
        use crate::auth::config_loader::ConfigLoader;
        use crate::auth::key_loader::KeyLoader;
        use std::env;

        // Step 1: Load partial configuration from OCI_CONFIG if available
        let partial_config = if let Ok(config_value) = env::var("OCI_CONFIG") {
            Some(ConfigLoader::load_partial_from_env_var(&config_value)?)
        } else {
            None
        };

        // Step 2: Merge with individual environment variables (highest priority)
        let user_id = env::var("OCI_USER_ID")
            .ok()
            .or_else(|| partial_config.as_ref().and_then(|c| c.user_id.clone()))
            .ok_or_else(|| {
                OciError::EnvError(
                    "OCI_USER_ID must be set (either directly or via OCI_CONFIG)".to_string(),
                )
            })?;

        let tenancy_id = env::var("OCI_TENANCY_ID")
            .ok()
            .or_else(|| partial_config.as_ref().and_then(|c| c.tenancy_id.clone()))
            .ok_or_else(|| {
                OciError::EnvError(
                    "OCI_TENANCY_ID must be set (either directly or via OCI_CONFIG)".to_string(),
                )
            })?;

        let region = env::var("OCI_REGION")
            .ok()
            .or_else(|| partial_config.as_ref().and_then(|c| c.region.clone()))
            .ok_or_else(|| {
                OciError::EnvError(
                    "OCI_REGION must be set (either directly or via OCI_CONFIG)".to_string(),
                )
            })?;

        let fingerprint = env::var("OCI_FINGERPRINT")
            .ok()
            .or_else(|| partial_config.as_ref().and_then(|c| c.fingerprint.clone()))
            .ok_or_else(|| {
                OciError::EnvError(
                    "OCI_FINGERPRINT must be set (either directly or via OCI_CONFIG)".to_string(),
                )
            })?;

        // Step 3: Load private key
        // Priority: OCI_PRIVATE_KEY env var > key_file from OCI_CONFIG
        let private_key = if let Ok(key_input) = env::var("OCI_PRIVATE_KEY") {
            // OCI_PRIVATE_KEY provided - use it (file path or PEM content)
            KeyLoader::load(&key_input)?
        } else if let Ok(config_value) = env::var("OCI_CONFIG") {
            // Fall back to loading from config file (which includes key_file)
            let full_config = ConfigLoader::load_from_env_var(&config_value, None)?;
            full_config.private_key
        } else {
            return Err(OciError::EnvError(
                "OCI_PRIVATE_KEY must be set (or key_file must be in OCI_CONFIG)".to_string(),
            ));
        };

        // Step 4: Optional compartment ID (defaults to tenancy_id)
        let compartment_id = env::var("OCI_COMPARTMENT_ID").ok();

        Ok(Self {
            user_id,
            tenancy_id,
            region,
            fingerprint,
            private_key,
            compartment_id,
        })
    }

    /// Get region
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Start builder pattern
    pub fn builder() -> OciConfigBuilder {
        OciConfigBuilder::default()
    }
}

/// OCI configuration builder
#[derive(Default)]
pub struct OciConfigBuilder {
    user_id: Option<String>,
    tenancy_id: Option<String>,
    region: Option<String>,
    fingerprint: Option<String>,
    private_key: Option<String>,
    compartment_id: Option<String>,
}

impl OciConfigBuilder {
    /// Load configuration from OCI config file
    ///
    /// Always uses the "DEFAULT" profile.
    ///
    /// # Arguments
    /// - `path`: File path to OCI config file (e.g., `~/.oci/config`)
    ///
    /// # Example
    /// ```no_run
    /// # use oci_api::auth::OciConfig;
    /// // Load from file
    /// let config = OciConfig::builder()
    ///     .config("/path/to/.oci/config")?
    ///     .private_key("/path/to/key.pem")?  // Optional override
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn config(mut self, path: impl AsRef<std::path::Path>) -> Result<Self> {
        use crate::auth::config_loader::ConfigLoader;

        let loaded = ConfigLoader::load_from_file(path.as_ref(), Some("DEFAULT"))?;

        // Set all fields from loaded config (will be overridden by individual setters if called)
        self.user_id = Some(loaded.user_id);
        self.tenancy_id = Some(loaded.tenancy_id);
        self.region = Some(loaded.region);
        self.fingerprint = Some(loaded.fingerprint);
        self.private_key = Some(loaded.private_key);
        // Don't set compartment_id from config - only if explicitly set by user

        Ok(self)
    }

    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn tenancy_id(mut self, tenancy_id: impl Into<String>) -> Self {
        self.tenancy_id = Some(tenancy_id.into());
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        self.fingerprint = Some(fingerprint.into());
        self
    }

    /// Set private key (file path or PEM content)
    ///
    /// Automatically detects whether the input is a file path or PEM content.
    ///
    /// # Example
    /// ```no_run
    /// # use oci_api::auth::OciConfig;
    /// // From file path
    /// let config = OciConfig::builder()
    ///     .config("/path/to/.oci/config")?
    ///     .private_key("/path/to/key.pem")?
    ///     .build()?;
    ///
    /// // From PEM content
    /// let config = OciConfig::builder()
    ///     .config("/path/to/.oci/config")?
    ///     .private_key("-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----")?
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn private_key(mut self, private_key: impl Into<String>) -> Result<Self> {
        use crate::auth::key_loader::KeyLoader;

        let key_input = private_key.into();
        let loaded_key = KeyLoader::load(&key_input)?;
        self.private_key = Some(loaded_key);

        Ok(self)
    }

    pub fn compartment_id(mut self, compartment_id: impl Into<String>) -> Self {
        self.compartment_id = Some(compartment_id.into());
        self
    }

    pub fn build(self) -> Result<OciConfig> {
        Ok(OciConfig {
            user_id: self
                .user_id
                .ok_or_else(|| OciError::ConfigError("user_id is not set".to_string()))?,
            tenancy_id: self
                .tenancy_id
                .ok_or_else(|| OciError::ConfigError("tenancy_id is not set".to_string()))?,
            region: self
                .region
                .ok_or_else(|| OciError::ConfigError("region is not set".to_string()))?,
            fingerprint: self
                .fingerprint
                .ok_or_else(|| OciError::ConfigError("fingerprint is not set".to_string()))?,
            private_key: self
                .private_key
                .ok_or_else(|| OciError::ConfigError("private_key is not set".to_string()))?,
            compartment_id: self.compartment_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_all_fields() {
        let user_id = "ocid1.user.test";
        let tenancy_id = "ocid1.tenancy.test";
        let region = "ap-seoul-1";
        let fingerprint = "aa:bb:cc:dd";
        let config = OciConfig::builder()
            .user_id(user_id)
            .tenancy_id(tenancy_id)
            .region(region)
            .fingerprint(fingerprint)
            .private_key("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----")
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(config.user_id, user_id);
        assert_eq!(config.tenancy_id, tenancy_id);
        assert_eq!(config.region, region);
        assert_eq!(config.fingerprint, fingerprint);
        assert!(config.private_key.contains("BEGIN RSA PRIVATE KEY"));
    }

    #[test]
    fn test_builder_missing_user_id() {
        let result = OciConfig::builder()
            .tenancy_id("ocid1.tenancy.test")
            .region("ap-seoul-1")
            .fingerprint("aa:bb:cc:dd")
            .private_key("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----")
            .unwrap()
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::ConfigError(msg) => assert!(msg.contains("user_id")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_builder_missing_tenancy_id() {
        let result = OciConfig::builder()
            .user_id("ocid1.user.test")
            .region("ap-seoul-1")
            .fingerprint("aa:bb:cc:dd")
            .private_key("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----")
            .unwrap()
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::ConfigError(msg) => assert!(msg.contains("tenancy_id")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_builder_missing_region() {
        let result = OciConfig::builder()
            .user_id("ocid1.user.test")
            .tenancy_id("ocid1.tenancy.test")
            .fingerprint("aa:bb:cc:dd")
            .private_key("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----")
            .unwrap()
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::ConfigError(msg) => assert!(msg.contains("region")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_builder_missing_fingerprint() {
        let result = OciConfig::builder()
            .user_id("ocid1.user.test")
            .tenancy_id("ocid1.tenancy.test")
            .region("ap-seoul-1")
            .private_key("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----")
            .unwrap()
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::ConfigError(msg) => assert!(msg.contains("fingerprint")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_builder_missing_private_key() {
        let result = OciConfig::builder()
            .user_id("ocid1.user.test")
            .tenancy_id("ocid1.tenancy.test")
            .region("ap-seoul-1")
            .fingerprint("aa:bb:cc:dd")
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::ConfigError(msg) => assert!(msg.contains("private_key")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_from_env_missing_user_id() {
        unsafe {
            std::env::remove_var("OCI_CONFIG");
            std::env::remove_var("OCI_USER_ID");
            std::env::remove_var("OCI_TENANCY_ID");
            std::env::remove_var("OCI_REGION");
            std::env::remove_var("OCI_FINGERPRINT");
            std::env::remove_var("OCI_PRIVATE_KEY");
        }

        let result = OciConfig::from_env();
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                OciError::EnvError(msg) => {
                    assert!(msg.contains("OCI_USER_ID") || msg.contains("OCI_CONFIG"));
                }
                _ => panic!("Expected EnvError, got: {:?}", e),
            }
        }
    }

    #[test]
    fn test_env_override_with_oci_config() {
        unsafe {
            // Clear all variables first
            std::env::remove_var("OCI_CONFIG");
            std::env::remove_var("OCI_USER_ID");
            std::env::remove_var("OCI_TENANCY_ID");
            std::env::remove_var("OCI_REGION");
            std::env::remove_var("OCI_FINGERPRINT");
            std::env::remove_var("OCI_PRIVATE_KEY");
        }

        unsafe {
            // Override specific values with individual environment variables FIRST
            std::env::set_var("OCI_USER_ID", "ocid1.user.from_env");
            std::env::set_var("OCI_REGION", "ap-seoul-1");
            std::env::set_var(
                "OCI_PRIVATE_KEY",
                "-----BEGIN PRIVATE KEY-----\ntest_key\n-----END PRIVATE KEY-----",
            );

            // THEN Setup OCI_CONFIG with base values
            std::env::set_var(
                "OCI_CONFIG",
                r#"
[DEFAULT]
user=ocid1.user.from_config
tenancy=ocid1.tenancy.from_config
region=us-phoenix-1
fingerprint=aa:bb:cc:dd:ee:ff
"#,
            );
        }

        let config = OciConfig::from_env().expect("Failed to load config");

        // Individual environment variables should override OCI_CONFIG values
        assert_eq!(config.user_id, "ocid1.user.from_env");
        assert_eq!(config.region, "ap-seoul-1");
        assert_eq!(
            config.private_key,
            "-----BEGIN PRIVATE KEY-----\ntest_key\n-----END PRIVATE KEY-----"
        );

        // Non-overridden values should come from OCI_CONFIG
        assert_eq!(config.tenancy_id, "ocid1.tenancy.from_config");
        assert_eq!(config.fingerprint, "aa:bb:cc:dd:ee:ff");

        unsafe {
            std::env::remove_var("OCI_CONFIG");
            std::env::remove_var("OCI_USER_ID");
            std::env::remove_var("OCI_REGION");
            std::env::remove_var("OCI_PRIVATE_KEY");
        }
    }

    #[test]
    fn test_oci_private_key_not_in_config() {
        unsafe {
            // OCI_CONFIG should NOT contain private_key field
            std::env::set_var(
                "OCI_CONFIG",
                r#"
[DEFAULT]
user=ocid1.user.test
tenancy=ocid1.tenancy.test
region=us-phoenix-1
fingerprint=aa:bb:cc:dd:ee:ff
private_key=this_should_be_ignored
"#,
            );

            // OCI_PRIVATE_KEY is always required as separate environment variable
            std::env::set_var(
                "OCI_PRIVATE_KEY",
                "-----BEGIN PRIVATE KEY-----\ntest_key\n-----END PRIVATE KEY-----",
            );
        }

        let config = OciConfig::from_env().expect("Failed to load config");

        // Should use OCI_PRIVATE_KEY, not the private_key field in OCI_CONFIG
        assert_eq!(
            config.private_key,
            "-----BEGIN PRIVATE KEY-----\ntest_key\n-----END PRIVATE KEY-----"
        );

        unsafe {
            std::env::remove_var("OCI_CONFIG");
            std::env::remove_var("OCI_PRIVATE_KEY");
        }
    }
}
