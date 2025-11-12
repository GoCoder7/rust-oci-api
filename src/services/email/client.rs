//! Email client

use crate::client::OciClient;
use crate::error::{OciError, Result};
use crate::services::email::models::*;

/// Email client
pub struct EmailClient {
    /// OCI HTTP client
    oci_client: OciClient,

    /// Submit endpoint (loaded from email configuration)
    submit_endpoint: String,
}

impl EmailClient {
    /// Create new Email client
    ///
    /// Loads email configuration and caches the submit endpoint.
    ///
    /// # Arguments
    /// * `oci_client` - OCI HTTP client
    pub async fn new(oci_client: OciClient) -> Result<Self> {
        let compartment_id = oci_client.compartment_id().to_string();
        let region = oci_client.region().to_string();

        // Get email configuration
        let config =
            Self::get_email_configuration_internal(&oci_client, &compartment_id, &region).await?;

        Ok(Self {
            oci_client,
            submit_endpoint: config.http_submit_endpoint,
        })
    }

    /// Get Email Configuration (internal helper)
    async fn get_email_configuration_internal(
        oci_client: &OciClient,
        compartment_id: &str,
        region: &str,
    ) -> Result<EmailConfiguration> {
        // Build path with query string
        let path = format!("/20170907/configuration?compartmentId={}", compartment_id);
        let host = format!("ctrl.email.{}.oci.oraclecloud.com", region);
        let url = format!("https://{}{}", host, path);

        // Sign request
        let (date_header, auth_header) = oci_client
            .signer()
            .sign_request("GET", &path, &host, None)?;

        // Build and execute request
        let response = oci_client
            .client()
            .get(&url)
            .header("host", &host)
            .header("date", &date_header)
            .header("authorization", &auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(OciError::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

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
    /// The compartment_id from OciClient will be automatically set in the sender.
    pub async fn send(&self, mut email: Email) -> Result<SubmitEmailResponse> {
        // Get compartment_id from OciClient
        let compartment_id = self.oci_client.compartment_id().to_string();

        // Set compartment_id in sender if not already set
        if email.sender.compartment_id.is_empty() {
            email.sender.set_compartment_id(&compartment_id);
        }

        // Build path and URL
        let path = "/20220926/actions/submitEmail";
        let url = format!("https://{}{}", &self.submit_endpoint, path);

        // Serialize JSON body
        let body_json = serde_json::to_string(&email)?;

        // Calculate body SHA256 for x-content-sha256 header
        let body_sha256 = {
            use base64::{Engine, engine::general_purpose};
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(body_json.as_bytes());
            let result = hasher.finalize();
            general_purpose::STANDARD.encode(result)
        };

        // Sign request (with body)
        let (date_header, auth_header) = self.oci_client.signer().sign_request(
            "POST",
            path,
            &self.submit_endpoint,
            Some(&body_json),
        )?;

        // Build and execute request
        let response = self
            .oci_client
            .client()
            .post(&url)
            .header("host", &self.submit_endpoint)
            .header("date", &date_header)
            .header("authorization", &auth_header)
            .header("content-type", "application/json")
            .header("content-length", body_json.len().to_string())
            .header("x-content-sha256", &body_sha256)
            .body(body_json)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(OciError::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        let submit_response: SubmitEmailResponse = response.json().await?;
        Ok(submit_response)
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
            query_params.push(format!("lifecycleState={}", state));
        }

        if let Some(email) = email_address {
            query_params.push(format!("emailAddress={}", email));
        }

        let query_string = query_params.join("&");
        let path = format!("/20170907/senders?{}", query_string);
        let host = format!(
            "ctrl.email.{}.oci.oraclecloud.com",
            self.oci_client.region()
        );
        let url = format!("https://{}{}", host, path);

        // Sign request
        let (date_header, auth_header) = self
            .oci_client
            .signer()
            .sign_request("GET", &path, &host, None)?;

        // Build and execute request
        let response = self
            .oci_client
            .client()
            .get(&url)
            .header("host", &host)
            .header("date", &date_header)
            .header("authorization", &auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(OciError::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        let senders: Vec<SenderSummary> = response.json().await?;
        Ok(senders)
    }
}
