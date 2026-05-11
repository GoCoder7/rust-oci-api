use reqwest::Method;

use crate::client::Oci;
use crate::client::request_executor::{RequestPayload, RequestTarget};
use crate::error::Result;
use crate::services::keys::models::Key;

#[derive(Clone)]
pub struct KeysClient {
    oci_client: Oci,
    management_endpoint: String,
}

impl KeysClient {
    pub fn new(oci_client: &Oci, management_endpoint: impl Into<String>) -> Self {
        Self {
            oci_client: oci_client.clone(),
            management_endpoint: normalize_management_endpoint(&management_endpoint.into()),
        }
    }

    pub async fn get_key(&self, key_id: &str) -> Result<Key> {
        let path = format!("/20180608/keys/{key_id}");
        let response = self
            .oci_client
            .executor()
            .execute(
                Method::GET,
                RequestTarget {
                    scheme: "https",
                    host: &self.management_endpoint,
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

    pub async fn rotate_key(&self, key_id: &str) -> Result<Key> {
        let path = format!("/20180608/keys/{key_id}/actions/rotate");
        let response = self
            .oci_client
            .executor()
            .execute(
                Method::POST,
                RequestTarget {
                    scheme: "https",
                    host: &self.management_endpoint,
                    path: &path,
                },
                RequestPayload {
                    body: Some("{}".to_owned()),
                    content_type: Some("application/json"),
                    extra_headers: Vec::new(),
                },
            )
            .await?;
        response.json().await.map_err(Into::into)
    }
}

fn normalize_management_endpoint(management_endpoint: &str) -> String {
    management_endpoint
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::normalize_management_endpoint;

    #[test]
    fn test_normalize_management_endpoint_strips_scheme_and_trailing_slash() {
        assert_eq!(
            normalize_management_endpoint(
                "https://example-management.kms.ap-chuncheon-1.oci.oraclecloud.com/"
            ),
            "example-management.kms.ap-chuncheon-1.oci.oraclecloud.com"
        );
    }
}
