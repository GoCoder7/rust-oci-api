use reqwest::Method;

use crate::client::Oci;
use crate::client::request_executor::{RequestPayload, RequestTarget};
use crate::error::Result;
use crate::services::vault::models::SecretBundle;

#[derive(Clone)]
pub struct VaultSecretsClient {
    oci_client: Oci,
    endpoint: String,
}

impl VaultSecretsClient {
    pub fn new(oci_client: &Oci) -> Self {
        let endpoint = format!(
            "secrets.vaults.{}.oci.{}",
            oci_client.region(),
            oci_client.realm_domain()
        );
        Self {
            oci_client: oci_client.clone(),
            endpoint,
        }
    }

    pub async fn get_secret_bundle(&self, secret_id: &str) -> Result<SecretBundle> {
        let path = format!("/20190301/secretbundles/{secret_id}");
        let response = self
            .oci_client
            .executor()
            .execute(
                Method::GET,
                RequestTarget {
                    scheme: "https",
                    host: &self.endpoint,
                    path: &path,
                },
                RequestPayload {
                    body: None,
                    content_type: None,
                    extra_headers: Vec::new(),
                },
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_secret_bundle_by_stage(
        &self,
        secret_id: &str,
        stage: &str,
    ) -> Result<SecretBundle> {
        let path = format!("/20190301/secretbundles/{secret_id}?stage={stage}");
        let response = self
            .oci_client
            .executor()
            .execute(
                Method::GET,
                RequestTarget {
                    scheme: "https",
                    host: &self.endpoint,
                    path: &path,
                },
                RequestPayload {
                    body: None,
                    content_type: None,
                    extra_headers: Vec::new(),
                },
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_secret_bundle_by_version(
        &self,
        secret_id: &str,
        version_number: i64,
    ) -> Result<SecretBundle> {
        let path = format!("/20190301/secretbundles/{secret_id}?versionNumber={version_number}");
        let response = self
            .oci_client
            .executor()
            .execute(
                Method::GET,
                RequestTarget {
                    scheme: "https",
                    host: &self.endpoint,
                    path: &path,
                },
                RequestPayload {
                    body: None,
                    content_type: None,
                    extra_headers: Vec::new(),
                },
            )
            .await?;
        response.json().await.map_err(Into::into)
    }
}
