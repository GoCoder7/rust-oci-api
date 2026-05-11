//! OCI HTTP client
//!
//! OCI API HTTP client with custom request signing

use crate::auth::config_loader::ConfigLoader;
use crate::auth::key_loader::KeyLoader;
use crate::auth::providers::{
    ApiKeyAuthProvider, DEFAULT_METADATA_BASE_URL, DEFAULT_REALM_DOMAIN_COMPONENT,
    DynOciAuthProvider, InstancePrincipalAuthProvider, InstancePrincipalConfig,
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

    /// Realm domain component
    realm_domain_component: String,

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
        let metadata_client = reqwest::blocking::Client::new();
        let metadata_base_url = env::var("OCI_METADATA_BASE_URL").ok();
        let metadata_region_info = InstancePrincipalAuthProvider::metadata_region_info_blocking(
            &metadata_client,
            metadata_base_url
                .as_deref()
                .unwrap_or(DEFAULT_METADATA_BASE_URL),
        )
        .ok();
        let region = env::var("OCI_REGION")
            .ok()
            .or_else(|| {
                metadata_region_info
                    .as_ref()
                    .map(|region_info| region_info.region_identifier.clone())
            })
            .ok_or_else(|| {
                Error::EnvError(
                    "OCI_REGION must be set or discoverable from OCI metadata when OCI_AUTH_MODE=instance_principal"
                        .to_owned(),
                )
            })?;
        let tenancy_id = env::var("OCI_TENANCY_ID")
            .ok()
            .or_else(|| {
                InstancePrincipalAuthProvider::tenancy_id_from_metadata_certificate_blocking(
                    &metadata_client,
                    metadata_base_url
                        .as_deref()
                        .unwrap_or(DEFAULT_METADATA_BASE_URL),
                )
                .ok()
            })
            .ok_or_else(|| {
                Error::EnvError(
                    "OCI_TENANCY_ID must be set or discoverable from OCI metadata when OCI_AUTH_MODE=instance_principal"
                        .to_owned(),
                )
            })?;
        let compartment_id = env::var("OCI_COMPARTMENT_ID").ok();
        let realm_domain_component = metadata_region_info
            .as_ref()
            .map(|region_info| region_info.realm_domain_component.clone())
            .unwrap_or_else(|| DEFAULT_REALM_DOMAIN_COMPONENT.to_owned());

        let mut builder = Self::builder()
            .auth_mode(AuthMode::InstancePrincipal)
            .region(region)
            .realm_domain_component(realm_domain_component)
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

    /// Return realm domain component
    pub fn realm_domain(&self) -> &str {
        &self.realm_domain_component
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
    realm_domain_component: Option<String>,
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

    pub fn realm_domain_component(mut self, realm_domain_component: impl Into<String>) -> Self {
        self.realm_domain_component = Some(realm_domain_component.into());
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
        let realm_domain_component = self
            .realm_domain_component
            .unwrap_or_else(|| DEFAULT_REALM_DOMAIN_COMPONENT.to_owned());
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
                        .realm_domain_component(realm_domain_component.clone())
                        .metadata_base_url(metadata_base_url)
                } else {
                    InstancePrincipalConfig::new(region.clone(), tenancy_id.clone())
                        .realm_domain_component(realm_domain_component.clone())
                };
                let provider = Arc::new(InstancePrincipalAuthProvider::new(client.clone(), config))
                    as DynOciAuthProvider;
                (None, provider)
            }
        };

        Ok(Oci {
            client,
            region,
            realm_domain_component,
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

#[cfg(test)]
mod tests {
    use super::*;

    use mockito::Server;
    use serial_test::serial;

    const TENANT_CERT_PEM: &str = "-----BEGIN CERTIFICATE-----\n\
MIIDXzCCAkegAwIBAgIUONFqOCNE1N3Aps1ZQaPpY7SQzngwDQYJKoZIhvcNAQEL\n\
BQAwPzEuMCwGA1UECgwlb3BjLXRlbmFudDpvY2lkMS50ZW5hbnR5Lm9jMS4uZXhh\n\
bXBsZTENMAsGA1UEAwwEdGVzdDAeFw0yNjA1MTEwNjQ1NTFaFw0yNjA1MTIwNjQ1\n\
NTFaMD8xLjAsBgNVBAoMJW9wYy10ZW5hbnQ6b2NpZDEudGVuYW5jeS5vYzEuLmV4\n\
YW1wbGUxDTALBgNVBAMMBHRlc3QwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEK\n\
AoIBAQDMblfnza9gqREWumv1mTJbR939nQIYZUynTxusVBXciNRjKaqB0jFSUFg9\n\
E2pwtr7G/zr6rpIum9yaRT3O/hhIACP7CJvOoIPTV8qDmNcRnlT78nWBN8jnma1A\n\
T9AZhtR14BJVe03eSSHBTnIDNNDQZu1+p6hUiGPVG1xe/F3/HOwbUrxzsChDnliZ\n\
C46FL0JMIu/uH/Q/iSg0wYsJQKzE+iIvLo5edTeaTvdaTth8XLmltWM2DEwC/fyU\n\
D2lxoOmvBhCVl1OCvT3Db0hMXRVV79BAXNS+qUyKbWnAgkiAMDGmEtYzizAoqCl4\n\
GpDeqNfSI/xo8Zt1RqU1PgleQslDAgMBAAGjUzBRMB0GA1UdDgQWBBRnTn//hXKL\n\
fWGEt7RY27CGihg+DjAfBgNVHSMEGDAWgBRnTn//hXKLfWGEt7RY27CGihg+DjAP\n\
BgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBAQAwRR1OsfwCP1UF4PWK\n\
jQLcBHrwEL7q9/HG47G6IsD4YN365ZPKzv7cOVzL7sPXVs18f3XDZwVNhwMiP2lo\n\
ShLlHDIog2ZMD0kppoZlwf1EdbVVOr30qtHaRpd1/YHY1omuUCdis51iJzO/wMwL\n\
m3yCFx7OCb46vCHwWc+CwiF9I9HKFMJyVpmhsEw91EPH3JaHWW1wn/RSIXuWpX0Q\n\
t+CmwNhI9TC99JL2cfr5lFUjA8nQ5Xx68L9gyfQZ2aicx5XD+s+nt0mgc06oOWv3\n\
ubYEGH/Vy8oK3rEoKdcNVdZUTgA0Fs2g+ItlrBFsJl5A1/TP3f0fbV6j9eY2SpdB\n\
Eo34\n\
-----END CERTIFICATE-----\n";

    struct EnvGuard {
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            Self {
                saved: keys.iter().map(|key| (*key, env::var(key).ok())).collect(),
            }
        }

        fn set(&self, key: &'static str, value: Option<&str>) {
            unsafe {
                match value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.saved {
                unsafe {
                    match value {
                        Some(value) => env::set_var(key, value),
                        None => env::remove_var(key),
                    }
                }
            }
        }
    }

    #[test]
    #[serial]
    fn from_instance_principal_env_uses_metadata_region_info_when_bootstrap_envs_are_missing() {
        let mut server = Server::new();
        let _region_info = server
            .mock("GET", "/opc/v2/instance/regionInfo")
            .match_header("authorization", "Bearer Oracle")
            .with_status(200)
            .with_body(
                r#"{"realmKey":"oc1","realmDomainComponent":"oraclecloud.com","regionKey":"PHX","regionIdentifier":"us-phoenix-1"}"#,
            )
            .create();
        let _leaf_cert = server
            .mock("GET", "/opc/v2/identity/cert.pem")
            .match_header("authorization", "Bearer Oracle")
            .with_status(200)
            .with_body(TENANT_CERT_PEM)
            .create();

        let guard = EnvGuard::new(&[
            "OCI_AUTH_MODE",
            "OCI_REGION",
            "OCI_TENANCY_ID",
            "OCI_METADATA_BASE_URL",
            "OCI_COMPARTMENT_ID",
        ]);
        guard.set("OCI_AUTH_MODE", Some("instance_principal"));
        guard.set("OCI_REGION", None);
        guard.set("OCI_TENANCY_ID", None);
        guard.set(
            "OCI_METADATA_BASE_URL",
            Some(&format!("{}/opc/v2", server.url())),
        );
        guard.set("OCI_COMPARTMENT_ID", None);

        let oci = Oci::from_env().unwrap();

        assert_eq!(oci.region(), "us-phoenix-1");
        assert_eq!(oci.realm_domain(), "oraclecloud.com");
        assert_eq!(oci.tenancy_id(), "ocid1.tenancy.oc1..example");
    }
}
