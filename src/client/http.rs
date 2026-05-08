//! OCI HTTP client
//!
//! OCI API HTTP client with custom request signing

use crate::auth::config_loader::ConfigLoader;
use crate::auth::key_loader::KeyLoader;
use crate::auth::providers::{
    ApiKeyAuthProvider, DynOciAuthProvider, InstancePrincipalAuthProvider, InstancePrincipalConfig,
};
use crate::client::request_executor::RequestExecutor;
use crate::client::signer::OciSigner;
use crate::error::{Error, Result};
use crate::services::email::EmailDelivery;
use crate::services::keys::KeysClient;
use crate::services::object_storage::ObjectStorage;
use crate::services::vault::VaultSecretsClient;
use reqwest::Client;
use std::env;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    ApiKey,
    InstancePrincipal,
}

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

    /// Authentication mode
    auth_mode: AuthMode,
    /// API key signer for compatibility
    signer: Option<OciSigner>,
    /// Authentication provider
    auth_provider: DynOciAuthProvider,
}

impl Default for Oci {
    fn default() -> Self {
        Self::from_env().expect("Failed to create OCI client from environment")
    }
}

impl Oci {
    /// Create new OCI client from environment variables
    pub fn from_env() -> Result<Self> {
        let auth_mode = match env::var("OCI_AUTH_MODE")
            .unwrap_or_else(|_| "api_key".to_owned())
            .as_str()
        {
            "api_key" => AuthMode::ApiKey,
            "instance_principal" => AuthMode::InstancePrincipal,
            other => {
                return Err(Error::EnvError(format!(
                    "OCI_AUTH_MODE must be 'api_key' or 'instance_principal', got '{other}'"
                )));
            }
        };

        match auth_mode {
            AuthMode::ApiKey => Self::from_api_key_env(),
            AuthMode::InstancePrincipal => Self::from_instance_principal_env(),
        }
    }

    fn from_api_key_env() -> Result<Self> {
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
                Error::EnvError(
                    "OCI_USER_ID must be set (either directly or via OCI_CONFIG)".to_string(),
                )
            })?;

        let tenancy_id = env::var("OCI_TENANCY_ID")
            .ok()
            .or_else(|| partial_config.as_ref().and_then(|c| c.tenancy_id.clone()))
            .ok_or_else(|| {
                Error::EnvError(
                    "OCI_TENANCY_ID must be set (either directly or via OCI_CONFIG)".to_string(),
                )
            })?;

        let region = env::var("OCI_REGION")
            .ok()
            .or_else(|| partial_config.as_ref().and_then(|c| c.region.clone()))
            .ok_or_else(|| {
                Error::EnvError(
                    "OCI_REGION must be set (either directly or via OCI_CONFIG)".to_string(),
                )
            })?;

        let fingerprint = env::var("OCI_FINGERPRINT")
            .ok()
            .or_else(|| partial_config.as_ref().and_then(|c| c.fingerprint.clone()))
            .ok_or_else(|| {
                Error::EnvError(
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
            return Err(Error::EnvError(
                "OCI_PRIVATE_KEY must be set (or key_file must be in OCI_CONFIG)".to_string(),
            ));
        };

        // Step 4: Optional compartment ID
        let compartment_id = env::var("OCI_COMPARTMENT_ID").ok();

        Self::builder()
            .auth_mode(AuthMode::ApiKey)
            .user_id(user_id)
            .tenancy_id(tenancy_id)
            .region(region)
            .fingerprint(fingerprint)
            .private_key(private_key)?
            .compartment_id_opt(compartment_id)
            .build()
    }

    fn from_instance_principal_env() -> Result<Self> {
        let region = env::var("OCI_REGION").map_err(|_| {
            Error::EnvError(
                "OCI_REGION must be set when OCI_AUTH_MODE=instance_principal".to_owned(),
            )
        })?;
        let tenancy_id = env::var("OCI_TENANCY_ID").map_err(|_| {
            Error::EnvError(
                "OCI_TENANCY_ID must be set when OCI_AUTH_MODE=instance_principal".to_owned(),
            )
        })?;
        let metadata_base_url = env::var("OCI_METADATA_BASE_URL").ok();
        let compartment_id = env::var("OCI_COMPARTMENT_ID").ok();

        let mut builder = Self::builder()
            .auth_mode(AuthMode::InstancePrincipal)
            .region(region)
            .tenancy_id(tenancy_id)
            .compartment_id_opt(compartment_id);
        if let Some(metadata_base_url) = metadata_base_url {
            builder = builder.metadata_base_url(metadata_base_url);
        }
        builder.build()
    }

    /// Start builder pattern
    pub fn builder() -> OciBuilder {
        OciBuilder::default()
    }

    /// Get request signer
    pub fn signer(&self) -> &OciSigner {
        self.signer
            .as_ref()
            .expect("Oci::signer() is only available in api_key mode")
    }

    /// Return HTTP client reference
    pub fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn executor(&self) -> RequestExecutor {
        RequestExecutor::new(self.client.clone(), Arc::clone(&self.auth_provider))
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
        self.compartment_id.as_ref().unwrap_or(&self.tenancy_id)
    }

    pub fn auth_mode(&self) -> AuthMode {
        self.auth_mode
    }

    /// Create Email Delivery client
    pub async fn email_delivery(&self) -> Result<EmailDelivery> {
        EmailDelivery::new(self.clone()).await
    }

    /// Create Object Storage client
    pub fn object_storage(&self, namespace: impl Into<String>) -> ObjectStorage {
        ObjectStorage::new(self, namespace)
    }

    /// Create Vault Secrets client
    pub fn vault(&self) -> VaultSecretsClient {
        VaultSecretsClient::new(self)
    }

    /// Create Keys client
    pub fn keys(&self, management_endpoint: impl Into<String>) -> KeysClient {
        KeysClient::new(self, management_endpoint)
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
    auth_mode: AuthMode,
    metadata_base_url: Option<String>,
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

    pub fn auth_mode(mut self, auth_mode: AuthMode) -> Self {
        self.auth_mode = auth_mode;
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

    pub fn metadata_base_url(mut self, metadata_base_url: impl Into<String>) -> Self {
        self.metadata_base_url = Some(metadata_base_url.into());
        self
    }

    pub fn build(self) -> Result<Oci> {
        let tenancy_id = self
            .tenancy_id
            .ok_or_else(|| Error::ConfigError("tenancy_id is not set".to_string()))?;
        let region = self
            .region
            .ok_or_else(|| Error::ConfigError("region is not set".to_string()))?;
        let client = Client::builder().build()?;

        let (signer, auth_provider) = match self.auth_mode {
            AuthMode::ApiKey => {
                let user_id = self
                    .user_id
                    .ok_or_else(|| Error::ConfigError("user_id is not set".to_owned()))?;
                let fingerprint = self
                    .fingerprint
                    .ok_or_else(|| Error::ConfigError("fingerprint is not set".to_owned()))?;
                let private_key = self
                    .private_key
                    .ok_or_else(|| Error::ConfigError("private_key is not set".to_owned()))?;
                let signer = OciSigner::new(&user_id, &tenancy_id, &fingerprint, &private_key)?;
                let provider =
                    Arc::new(ApiKeyAuthProvider::new(signer.clone())) as DynOciAuthProvider;
                (Some(signer), provider)
            }
            AuthMode::InstancePrincipal => {
                let config = if let Some(metadata_base_url) = self.metadata_base_url {
                    InstancePrincipalConfig::new(region.clone(), tenancy_id.clone())
                        .metadata_base_url(metadata_base_url)
                } else {
                    InstancePrincipalConfig::new(region.clone(), tenancy_id.clone())
                };
                let provider = Arc::new(InstancePrincipalAuthProvider::new(client.clone(), config))
                    as DynOciAuthProvider;
                (None, provider)
            }
        };

        Ok(Oci {
            client,
            region,
            tenancy_id,
            compartment_id: self.compartment_id,
            signer,
            auth_mode: self.auth_mode,
            auth_provider,
        })
    }
}

impl Default for AuthMode {
    fn default() -> Self {
        Self::ApiKey
    }
}
