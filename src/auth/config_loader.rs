//! INI format OCI configuration file loader
//!
//! Reads OCI configuration from file path or INI content string.

use crate::auth::config::{OciConfig, OciConfigBuilder};
use crate::auth::key_loader::KeyLoader;
use crate::error::{OciError, Result};
use ini::{Ini, Properties};
use std::path::Path;

/// OCI configuration file loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from OCI_CONFIG environment variable value
    ///
    /// Automatically detects whether the value is a file path or INI content
    ///
    /// # Arguments
    /// * `config_value` - Value from OCI_CONFIG environment variable (file path or INI content)
    /// * `profile` - Profile name (default: "DEFAULT")
    pub fn load_from_env_var(config_value: &str, profile: Option<&str>) -> Result<OciConfig> {
        // Check if it's a file path
        let path = Path::new(config_value);
        if path.exists() {
            Self::load_from_file(path, profile)
        } else {
            // Treat as INI content
            Self::load_from_ini_content(config_value, profile)
        }
    }

    /// Load configuration from INI content string
    ///
    /// # Arguments
    /// * `ini_content` - INI format configuration string
    /// * `profile` - Profile name (default: "DEFAULT")
    pub fn load_from_ini_content(ini_content: &str, profile: Option<&str>) -> Result<OciConfig> {
        let profile_name = profile.unwrap_or("DEFAULT");

        // Parse INI content
        let ini = Ini::load_from_str(ini_content)
            .map_err(|e| OciError::IniError(format!("Failed to parse INI content: {}", e)))?;

        // Find profile section
        let section = ini.section(Some(profile_name)).ok_or_else(|| {
            OciError::ConfigError(format!(
                "Profile '{}' not found in INI content",
                profile_name
            ))
        })?;

        // Read and build config
        Self::build_config_from_section(section)
    }

    /// Load configuration from file path
    ///
    /// # Arguments
    /// * `path` - Configuration file path
    /// * `profile` - Profile name (default: "DEFAULT")
    pub fn load_from_file(path: &Path, profile: Option<&str>) -> Result<OciConfig> {
        let profile_name = profile.unwrap_or("DEFAULT");

        // Parse INI file
        let ini = Ini::load_from_file(path)
            .map_err(|e| OciError::IniError(format!("Failed to load INI file: {}", e)))?;

        // Find profile section
        let section = ini.section(Some(profile_name)).ok_or_else(|| {
            OciError::ConfigError(format!("Profile '{}' not found", profile_name))
        })?;

        // Read and build config
        Self::build_config_from_section(section)
    }

    /// Build OciConfig from INI section
    fn build_config_from_section(section: &Properties) -> Result<OciConfig> {
        // Read required fields
        let user_id = section
            .get("user")
            .ok_or_else(|| OciError::ConfigError("user field not found in config".to_string()))?
            .to_string();

        let tenancy_id = section
            .get("tenancy")
            .ok_or_else(|| OciError::ConfigError("tenancy field not found in config".to_string()))?
            .to_string();

        let region = section
            .get("region")
            .ok_or_else(|| OciError::ConfigError("region field not found in config".to_string()))?
            .to_string();

        let fingerprint = section
            .get("fingerprint")
            .ok_or_else(|| {
                OciError::ConfigError("fingerprint field not found in config".to_string())
            })?
            .to_string();

        // key_file is required for traditional config file loading
        // If key_file is missing, the caller must provide private_key separately
        let key_file = section.get("key_file").ok_or_else(|| {
            OciError::ConfigError("key_file field not found in config".to_string())
        })?;

        // Load private key from key_file path
        // Note: key_file in OCI config typically uses paths like ~/...
        // We expand ~ to home directory for convenience
        let key_path = if key_file.starts_with("~/") {
            let home = std::env::var("HOME").map_err(|_| {
                OciError::EnvError("Cannot find HOME environment variable".to_string())
            })?;
            key_file.replacen("~", &home, 1)
        } else {
            key_file.to_string()
        };

        let private_key = KeyLoader::load(&key_path)?;

        // Create OciConfig (compartment_id is None when loading from config)
        OciConfigBuilder::default()
            .user_id(user_id)
            .tenancy_id(tenancy_id)
            .region(region)
            .fingerprint(fingerprint)
            .private_key(private_key)?
            .build()
    }

    /// Load partial configuration from OCI_CONFIG environment variable
    /// Returns only the fields present in the config file
    /// Used by from_env() to get base values before applying environment variable overrides
    pub(crate) fn load_partial_from_env_var(config_value: &str) -> Result<PartialOciConfig> {
        let ini = if std::path::Path::new(config_value).exists() {
            // It's a file path
            Ini::load_from_file(config_value)
                .map_err(|e| OciError::ConfigError(format!("Failed to load config file: {}", e)))?
        } else {
            // It's INI content
            Ini::load_from_str(config_value)
                .map_err(|e| OciError::ConfigError(format!("Failed to parse INI content: {}", e)))?
        };

        let profile_name = "DEFAULT";
        let section = ini.section(Some(profile_name)).ok_or_else(|| {
            OciError::ConfigError(format!("Profile '{}' not found", profile_name))
        })?;

        // Extract only the fields that are present
        Ok(PartialOciConfig {
            user_id: section.get("user").map(|s| s.to_string()),
            tenancy_id: section.get("tenancy").map(|s| s.to_string()),
            region: section.get("region").map(|s| s.to_string()),
            fingerprint: section.get("fingerprint").map(|s| s.to_string()),
        })
    }
}

/// Partial OCI configuration with optional fields
/// Used when loading from OCI_CONFIG environment variable
#[derive(Debug, Default)]
pub(crate) struct PartialOciConfig {
    pub user_id: Option<String>,
    pub tenancy_id: Option<String>,
    pub region: Option<String>,
    pub fingerprint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_file_success() {
        // Create temporary INI file and key file
        let mut key_file = NamedTempFile::new().unwrap();
        let key_content = "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----\n";
        key_file.write_all(key_content.as_bytes()).unwrap();

        let mut ini_file = NamedTempFile::new().unwrap();
        let ini_content = format!(
            r#"
[DEFAULT]
user=ocid1.user.test
tenancy=ocid1.tenancy.test
region=ap-seoul-1
fingerprint=aa:bb:cc:dd:ee:ff
key_file={}
"#,
            key_file.path().to_str().unwrap()
        );
        ini_file.write_all(ini_content.as_bytes()).unwrap();

        let result = ConfigLoader::load_from_file(ini_file.path(), None);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.user_id, "ocid1.user.test");
        assert_eq!(config.tenancy_id, "ocid1.tenancy.test");
        assert_eq!(config.region, "ap-seoul-1");
        assert_eq!(config.fingerprint, "aa:bb:cc:dd:ee:ff");
        assert!(config.private_key.contains("BEGIN RSA PRIVATE KEY"));
    }

    #[test]
    fn test_load_from_file_missing_field() {
        let mut ini_file = NamedTempFile::new().unwrap();
        let ini_content = r#"
[DEFAULT]
user=ocid1.user.test
tenancy=ocid1.tenancy.test
region=ap-seoul-1
"#;
        ini_file.write_all(ini_content.as_bytes()).unwrap();

        let result = ConfigLoader::load_from_file(ini_file.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_file_profile_not_found() {
        let mut ini_file = NamedTempFile::new().unwrap();
        let ini_content = r#"
[DEFAULT]
user=ocid1.user.test
"#;
        ini_file.write_all(ini_content.as_bytes()).unwrap();

        let result = ConfigLoader::load_from_file(ini_file.path(), Some("NONEXISTENT"));
        assert!(result.is_err());
        match result.unwrap_err() {
            OciError::ConfigError(msg) => assert!(msg.contains("NONEXISTENT")),
            _ => panic!("Expected ConfigError"),
        }
    }
}
