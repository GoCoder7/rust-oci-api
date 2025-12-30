//! OCI HTTP client
//!
//! OCI API HTTP client with custom request signing

use crate::auth::config_loader::ConfigLoader;
use crate::auth::key_loader::KeyLoader;
use crate::client::signer::OciSigner;
use crate::error::{OciError, Result};
use crate::services::email::EmailDelivery;
use crate::services::object_storage::ObjectStorage;
use reqwest::Client;
use std::env;

/// OCI HTTP client
#[derive(Clone)]
pub struct Oci {
    /// HTTP client
    client: Client,

    /// Region
    region: String,

    /// Tenancy ID
    tenancy_id: String,

    /// Compartment ID
    compartment_id: Option<String>,

    /// Request signer
    signer: OciSigner,
}

impl Default for Oci {
    fn default() -> Self {
        Self::from_env().expect("Failed to create OCI client from environment")
    }
}

impl Oci {
    /// Create new OCI client from environment variables
    pub fn from_env() -> Result<Self> {
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
        let private_key = if let Ok(key_input) = env::var("OCI_PRIVATE_KEY") {
            KeyLoader::load(&key_input)?
        } else if let Ok(config_value) = env::var("OCI_CONFIG") {
            let full_config = ConfigLoader::load_from_env_var(&config_value, None)?;
            full_config.private_key
        } else {
            return Err(OciError::EnvError(
                "OCI_PRIVATE_KEY must be set (or key_file must be in OCI_CONFIG)".to_string(),
            ));
        };

        // Step 4: Optional compartment ID
        let compartment_id = env::var("OCI_COMPARTMENT_ID").ok();

        Self::builder()
            .user_id(user_id)
            .tenancy_id(tenancy_id)
            .region(region)
            .fingerprint(fingerprint)
            .private_key(private_key)?
            .compartment_id_opt(compartment_id)
            .build()
    }

    /// Start builder pattern
    pub fn builder() -> OciBuilder {
        OciBuilder::default()
    }

    /// Get request signer
    pub fn signer(&self) -> &OciSigner {
        &self.signer
    }

    /// Return HTTP client reference
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Return region
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Return tenancy ID
    pub fn tenancy_id(&self) -> &str {
        &self.tenancy_id
    }

    /// Return compartment ID (defaults to tenancy_id if not set)
    pub fn compartment_id(&self) -> &str {
        self.compartment_id
            .as_ref()
            .unwrap_or(&self.tenancy_id)
    }

    /// Create Email Delivery client
    pub async fn email_delivery(&self) -> Result<EmailDelivery> {
        EmailDelivery::new(self.clone()).await
    }

    /// Create Object Storage client
    pub fn object_storage(&self, namespace: impl Into<String>) -> ObjectStorage {
        ObjectStorage::new(self, namespace)
    }
}

/// OCI client builder
#[derive(Default)]
pub struct OciBuilder {
    user_id: Option<String>,
    tenancy_id: Option<String>,
    region: Option<String>,
    fingerprint: Option<String>,
    private_key: Option<String>,
    compartment_id: Option<String>,
}

impl OciBuilder {
    /// Load configuration from OCI config file
    pub fn config(mut self, path: impl AsRef<std::path::Path>) -> Result<Self> {
        let loaded = ConfigLoader::load_from_file(path.as_ref(), Some("DEFAULT"))?;

        self.user_id = Some(loaded.user_id);
        self.tenancy_id = Some(loaded.tenancy_id);
        self.region = Some(loaded.region);
        self.fingerprint = Some(loaded.fingerprint);
        self.private_key = Some(loaded.private_key);
        
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

    pub fn private_key(mut self, private_key: impl Into<String>) -> Result<Self> {
        let key_input = private_key.into();
        let loaded_key = KeyLoader::load(&key_input)?;
        self.private_key = Some(loaded_key);
        Ok(self)
    }

    pub fn compartment_id(mut self, compartment_id: impl Into<String>) -> Self {
        self.compartment_id = Some(compartment_id.into());
        self
    }

    // Internal helper for optional compartment_id
    fn compartment_id_opt(mut self, compartment_id: Option<String>) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    pub fn build(self) -> Result<Oci> {
        let user_id = self.user_id.ok_or_else(|| OciError::ConfigError("user_id is not set".to_string()))?;
        let tenancy_id = self.tenancy_id.ok_or_else(|| OciError::ConfigError("tenancy_id is not set".to_string()))?;
        let region = self.region.ok_or_else(|| OciError::ConfigError("region is not set".to_string()))?;
        let fingerprint = self.fingerprint.ok_or_else(|| OciError::ConfigError("fingerprint is not set".to_string()))?;
        let private_key = self.private_key.ok_or_else(|| OciError::ConfigError("private_key is not set".to_string()))?;

        let signer = OciSigner::new(&user_id, &tenancy_id, &fingerprint, &private_key)?;
        let client = Client::builder().build()?;

        Ok(Oci {
            client,
            region,
            tenancy_id,
            compartment_id: self.compartment_id,
            signer,
        })
    }
}
