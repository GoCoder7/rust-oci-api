//! Email client

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Method;

use crate::client::Oci;
use crate::client::request_executor::{RequestPayload, RequestTarget};
use crate::error::Result;
use crate::services::email::models::*;
use crate::services::email::sender_trait::EmailSender;

/// Email client
#[derive(Clone)]
pub struct EmailDelivery {
    /// OCI HTTP client
    oci_client: Oci,

    /// Submit endpoint (loaded from email configuration)
    submit_endpoint: String,
}

impl EmailDelivery {
    fn control_host_for(region: &str, realm_domain: &str) -> String {
        format!("ctrl.email.{region}.oci.{realm_domain}")
    }

    /// Create new Email client
    ///
    /// Loads email configuration and caches the submit endpoint.
    ///
    /// # Arguments
    /// * `oci_client` - OCI HTTP client
    pub async fn new(oci_client: Oci) -> Result<Self> {
        // Email configuration is a tenancy-level resource, so use tenancy_id
        let tenancy_id = oci_client.tenancy_id().to_string();
        let region = oci_client.region().to_string();

        // Get email configuration
        let config =
            Self::get_email_configuration_internal(&oci_client, &tenancy_id, &region).await?;

        Ok(Self {
            oci_client,
            submit_endpoint: config.http_submit_endpoint,
        })
    }

    /// Get Email Configuration (internal helper)
    async fn get_email_configuration_internal(
        oci_client: &Oci,
        compartment_id: &str,
        region: &str,
    ) -> Result<EmailConfiguration> {
        let path = format!("/20170907/configuration?compartmentId={compartment_id}");
        let host = Self::control_host_for(region, oci_client.realm_domain());
        let response = oci_client
            .executor()
            .execute(
                Method::GET,
                RequestTarget {
                    scheme: "https",
                    host: &host,
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

    /// Get Email Configuration (public API)
    ///
    /// # Arguments
    /// * `compartment_id` - Compartment OCID (typically tenancy OCID)
    pub async fn get_email_configuration(
        &self,
        compartment_id: impl Into<String>,
    ) -> Result<EmailConfiguration> {
        let compartment_id = compartment_id.into();
        let region = self.oci_client.region().to_string();
        Self::get_email_configuration_internal(&self.oci_client, &compartment_id, &region).await
    }

    /// Send email
    ///
    /// # Arguments
    /// * `email` - Email message
    ///
    /// # Note
    /// The compartment_id from Oci will be automatically set in the sender.
    pub async fn send(&self, email: Email) -> Result<SubmitEmailResponse> {
        self.send_impl(email).await
    }

    /// 실제 전송 로직 (inherent method + trait impl 공용)
    async fn send_impl(&self, mut email: Email) -> Result<SubmitEmailResponse> {
        // Get compartment_id from Oci
        let compartment_id = self.oci_client.compartment_id().to_string();

        // Set compartment_id in sender if not already set
        if email.sender.compartment_id.is_empty() {
            email.sender.set_compartment_id(&compartment_id);
        }

        let path = "/20220926/actions/submitEmail";
        let body_json = serde_json::to_string(&email)?;
        let response = self
            .oci_client
            .executor()
            .execute(
                Method::POST,
                RequestTarget {
                    scheme: "https",
                    host: &self.submit_endpoint,
                    path,
                },
                RequestPayload {
                    body: Some(Bytes::from(body_json)),
                    content_type: Some("application/json"),
                    extra_headers: Vec::new(),
                },
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    /// List approved senders
    ///
    /// # Arguments
    /// * `compartment_id` - Compartment OCID (required)
    /// * `lifecycle_state` - Optional filter by lifecycle state
    /// * `email_address` - Optional filter by email address
    pub async fn list_senders(
        &self,
        compartment_id: impl Into<String>,
        lifecycle_state: Option<&str>,
        email_address: Option<&str>,
    ) -> Result<Vec<SenderSummary>> {
        let compartment_id = compartment_id.into();

        // Build query string
        let mut query_params = vec![format!("compartmentId={}", compartment_id)];

        if let Some(state) = lifecycle_state {
            query_params.push(format!("lifecycleState={state}"));
        }

        if let Some(email) = email_address {
            query_params.push(format!("emailAddress={email}"));
        }

        let query_string = query_params.join("&");
        let path = format!("/20170907/senders?{query_string}");
        let host = Self::control_host_for(self.oci_client.region(), self.oci_client.realm_domain());
        let response = self
            .oci_client
            .executor()
            .execute(
                Method::GET,
                RequestTarget {
                    scheme: "https",
                    host: &host,
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

#[async_trait]
impl EmailSender for EmailDelivery {
    async fn send(&self, email: Email) -> Result<SubmitEmailResponse> {
        self.send_impl(email).await
    }
}

#[cfg(test)]
mod tests {
    use super::EmailDelivery;
    use crate::client::{AuthMode, Oci};

    fn instance_principal_client(region: &str, realm_domain: &str) -> Oci {
        Oci::builder()
            .auth_mode(AuthMode::InstancePrincipal)
            .tenancy_id("ocid1.tenancy.oc1..example")
            .region(region)
            .realm_domain_component(realm_domain)
            .build()
            .unwrap()
    }

    #[test]
    fn email_control_host_uses_commercial_realm_for_instance_principal() {
        let oci = instance_principal_client("ap-chuncheon-1", "oraclecloud.com");
        assert_eq!(
            EmailDelivery::control_host_for(oci.region(), oci.realm_domain()),
            "ctrl.email.ap-chuncheon-1.oci.oraclecloud.com"
        );
    }

    #[test]
    fn email_control_host_uses_gov_realm_for_instance_principal() {
        let oci = instance_principal_client("us-langley-1", "oraclegovcloud.com");
        assert_eq!(
            EmailDelivery::control_host_for(oci.region(), oci.realm_domain()),
            "ctrl.email.us-langley-1.oci.oraclegovcloud.com"
        );
    }
}
