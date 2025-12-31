//! Object Storage client

use crate::client::Oci;
use crate::error::{Error, Result};
use crate::services::object_storage::models::*;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Object Storage Service Client
#[derive(Clone)]
pub struct ObjectStorage {
    /// OCI HTTP client
    oci_client: Oci,
    /// Namespace name
    pub namespace: String,
    /// Endpoint (Host)
    endpoint: String,
    /// Protocol (http/https)
    protocol: String,
}

impl ObjectStorage {
    /// Create new Object Storage client
    ///
    /// # Arguments
    /// * `oci_client` - OCI HTTP client
    /// * `namespace` - Object Storage Namespace
    pub fn new(oci_client: &Oci, namespace: impl Into<String>) -> Self {
        let region = oci_client.region().to_string();
        let endpoint = format!("objectstorage.{region}.oraclecloud.com");

        Self {
            oci_client: oci_client.clone(),
            namespace: namespace.into(),
            endpoint,
            protocol: "https".to_string(),
        }
    }

    /// Get Bucket
    ///
    /// # Arguments
    /// * `bucket_name` - Bucket name
    pub async fn get_bucket(&self, bucket_name: &str) -> Result<Bucket> {
        // Verify bucket exists
        let path = format!("/n/{}/b/{}/", self.namespace, bucket_name);
        let url = format!("{}://{}{}", self.protocol, self.endpoint, path);

        let (date_header, auth_header) =
            self.oci_client
                .signer()
                .sign_request("GET", &path, &self.endpoint, None)?;

        let response = self
            .oci_client
            .client()
            .get(&url)
            .header("host", &self.endpoint)
            .header("date", &date_header)
            .header("authorization", &auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        // If successful, return Bucket struct
        Ok(Bucket {
            oci_client: self.oci_client.clone(),
            namespace: self.namespace.clone(),
            name: bucket_name.to_string(),
            endpoint: self.endpoint.clone(),
            protocol: self.protocol.clone(),
        })
    }
}

/// Bucket
#[derive(Clone)]
pub struct Bucket {
    /// OCI HTTP client
    oci_client: Oci,
    /// Namespace
    pub namespace: String,
    /// Bucket name
    pub name: String,
    /// Endpoint (Host)
    endpoint: String,
    /// Protocol (http/https)
    protocol: String,
}

impl Bucket {
    // Helper for making requests
    async fn request<T, B>(&self, method: &str, path: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = format!("{}://{}{}", self.protocol, self.endpoint, path);
        let body_str = if let Some(b) = &body {
            Some(serde_json::to_string(b)?)
        } else {
            None
        };

        let (date_header, auth_header) = self.oci_client.signer().sign_request(
            method,
            path,
            &self.endpoint,
            body_str.as_deref(),
        )?;

        let mut request_builder = match method {
            "GET" => self.oci_client.client().get(&url),
            "POST" => self.oci_client.client().post(&url),
            "PUT" => self.oci_client.client().put(&url),
            "DELETE" => self.oci_client.client().delete(&url),
            _ => return Err(Error::Other(format!("Unsupported method: {}", method))),
        };

        request_builder = request_builder
            .header("host", &self.endpoint)
            .header("date", &date_header)
            .header("authorization", &auth_header);

        if let Some(b_str) = body_str {
            request_builder = request_builder
                .header("content-type", "application/json")
                .header("content-length", b_str.len().to_string())
                .body(b_str);
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        let text = response.text().await?;
        serde_json::from_str(&text).map_err(Into::into)
    }

    async fn request_no_content<B>(&self, method: &str, path: &str, body: Option<B>) -> Result<()>
    where
        B: Serialize,
    {
        let url = format!("{}://{}{}", self.protocol, self.endpoint, path);
        let body_str = if let Some(b) = &body {
            Some(serde_json::to_string(b)?)
        } else {
            None
        };

        let (date_header, auth_header) = self.oci_client.signer().sign_request(
            method,
            path,
            &self.endpoint,
            body_str.as_deref(),
        )?;

        let mut request_builder = match method {
            "GET" => self.oci_client.client().get(&url),
            "POST" => self.oci_client.client().post(&url),
            "PUT" => self.oci_client.client().put(&url),
            "DELETE" => self.oci_client.client().delete(&url),
            _ => return Err(Error::Other(format!("Unsupported method: {}", method))),
        };

        request_builder = request_builder
            .header("host", &self.endpoint)
            .header("date", &date_header)
            .header("authorization", &auth_header);

        if let Some(b_str) = body_str {
            request_builder = request_builder
                .header("content-type", "application/json")
                .header("content-length", b_str.len().to_string())
                .body(b_str);
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        Ok(())
    }

    /// Put Object
    ///
    /// # Arguments
    /// * `object_name` - Object name
    /// * `content` - Object content
    pub async fn put_object(&self, object_name: &str, content: &str) -> Result<Object> {
        let path = format!("/n/{}/b/{}/o/{}", self.namespace, self.name, object_name);
        let url = format!("{}://{}{}", self.protocol, self.endpoint, path);

        let (date_header, auth_header) =
            self.oci_client
                .signer()
                .sign_request("PUT", &path, &self.endpoint, Some(content))?;

        let response = self
            .oci_client
            .client()
            .put(&url)
            .header("host", &self.endpoint)
            .header("date", &date_header)
            .header("authorization", &auth_header)
            .header("content-length", content.len().to_string())
            .body(content.to_string())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        Ok(Object {
            name: object_name.to_string(),
            value: content.to_string(),
        })
    }

    /// Get Object
    ///
    /// # Arguments
    /// * `object_name` - Object name
    pub async fn get_object(&self, object_name: &str) -> Result<Object> {
        let path = format!("/n/{}/b/{}/o/{}", self.namespace, self.name, object_name);
        let url = format!("{}://{}{}", self.protocol, self.endpoint, path);

        let (date_header, auth_header) =
            self.oci_client
                .signer()
                .sign_request("GET", &path, &self.endpoint, None)?;

        let response = self
            .oci_client
            .client()
            .get(&url)
            .header("host", &self.endpoint)
            .header("date", &date_header)
            .header("authorization", &auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        let value = response.text().await?;

        Ok(Object {
            name: object_name.to_string(),
            value,
        })
    }

    /// Get or Create Object
    ///
    /// Tries to get the object. If it doesn't exist (404), creates it with the provided content.
    ///
    /// # Arguments
    /// * `object_name` - Object name
    /// * `content` - Content to use if object needs to be created
    pub async fn get_or_create_object(&self, object_name: &str, content: &str) -> Result<Object> {
        match self.get_object(object_name).await {
            Ok(obj) => Ok(obj),
            Err(Error::ApiError { code, .. }) if code.contains("404") => {
                self.put_object(object_name, content).await
            }
            Err(e) => Err(e),
        }
    }

    /// Get Retention Rules
    pub async fn get_retention_rules(&self) -> Result<Vec<RetentionRule>> {
        let path = format!("/n/{}/b/{}/retentionRules", self.namespace, self.name);

        #[derive(Deserialize)]
        struct ResponseWrapper {
            items: Vec<RetentionRule>,
        }

        let wrapper: ResponseWrapper = self
            .request::<ResponseWrapper, ()>("GET", &path, None)
            .await?;
        Ok(wrapper.items)
    }

    /// Create Retention Rule
    pub async fn create_retention_rule(
        &self,
        details: RetentionRuleDetails,
    ) -> Result<RetentionRule> {
        let path = format!("/n/{}/b/{}/retentionRules", self.namespace, self.name);
        self.request("POST", &path, Some(details)).await
    }

    /// Get Retention Rule
    pub async fn get_retention_rule(&self, rule_id: &str) -> Result<RetentionRule> {
        let path = format!(
            "/n/{}/b/{}/retentionRules/{}",
            self.namespace, self.name, rule_id
        );
        self.request("GET", &path, None::<()>).await
    }

    /// Update Retention Rule
    pub async fn update_retention_rule(
        &self,
        rule_or_id: impl Into<String>,
        details: RetentionRuleDetails,
    ) -> Result<RetentionRule> {
        let rule_id = rule_or_id.into();
        let path = format!(
            "/n/{}/b/{}/retentionRules/{}",
            self.namespace, self.name, rule_id
        );
        self.request("PUT", &path, Some(details)).await
    }

    /// Delete Retention Rule
    pub async fn delete_retention_rule(&self, rule_or_id: impl Into<String>) -> Result<()> {
        let rule_id = rule_or_id.into();
        let path = format!(
            "/n/{}/b/{}/retentionRules/{}",
            self.namespace, self.name, rule_id
        );
        self.request_no_content("DELETE", &path, None::<()>).await
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
