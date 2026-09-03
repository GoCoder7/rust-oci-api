use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;

mod api_key;
mod instance_principal;

pub use api_key::ApiKeyAuthProvider;
pub(crate) use instance_principal::{
    DEFAULT_METADATA_BASE_URL, DEFAULT_REALM_DOMAIN_COMPONENT, MetadataRegionInfo,
};
pub use instance_principal::{InstancePrincipalAuthProvider, InstancePrincipalConfig};

#[derive(Debug, Clone)]
pub struct SignRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub host: Option<&'a str>,
    pub body: Option<&'a [u8]>,
    pub content_type: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct SignedHeaders {
    pub date: String,
    pub authorization: String,
    pub content_type: Option<String>,
    pub content_length: Option<String>,
    pub x_content_sha256: Option<String>,
    pub extra_headers: Vec<(String, String)>,
}

#[async_trait]
pub trait OciAuthProvider: Send + Sync {
    async fn sign(&self, request: &SignRequest<'_>) -> Result<SignedHeaders>;
}

pub type DynOciAuthProvider = Arc<dyn OciAuthProvider>;
