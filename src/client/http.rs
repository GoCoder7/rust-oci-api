//! OCI HTTP client
//!
//! OCI API HTTP client with custom request signing

use crate::auth::OciConfig;
use crate::client::signer::OciSigner;
use crate::error::Result;
use reqwest::Client;

/// OCI HTTP client
pub struct OciClient {
    /// HTTP client
    client: Client,

    /// OCI configuration
    config: OciConfig,

    /// Request signer
    signer: OciSigner,
}

impl OciClient {
    /// Create new OCI client
    pub fn new(config: &OciConfig) -> Result<Self> {
        let client = Client::builder().build()?;
        let signer = OciSigner::new(config)?;

        Ok(Self {
            client,
            config: config.clone(),
            signer,
        })
    }

    /// Get OCI configuration
    pub fn config(&self) -> &OciConfig {
        &self.config
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
        &self.config.region
    }

    /// Return compartment ID (defaults to tenancy_id if not set)
    pub fn compartment_id(&self) -> &str {
        self.config
            .compartment_id
            .as_ref()
            .unwrap_or(&self.config.tenancy_id)
    }
}
