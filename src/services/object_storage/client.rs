//! Object Storage client

use crate::client::Oci;
use crate::client::request_executor::{RequestPayload, RequestTarget};
use crate::error::{Error, Result};
use crate::services::object_storage::models::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Method;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest as ShaDigest, Sha256, Sha384};

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
        let endpoint = format!("objectstorage.{region}.{}", oci_client.realm_domain());

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
        let path = format!("/n/{}/b/{}/", self.namespace, bucket_name);
        self.oci_client
            .executor()
            .execute(
                Method::GET,
                RequestTarget {
                    scheme: &self.protocol,
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
    async fn execute(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
        content_type: Option<&str>,
        extra_headers: Vec<(String, String)>,
    ) -> Result<reqwest::Response> {
        self.oci_client
            .executor()
            .execute(
                method,
                RequestTarget {
                    scheme: &self.protocol,
                    host: &self.endpoint,
                    path,
                },
                RequestPayload {
                    body,
                    content_type,
                    extra_headers,
                },
            )
            .await
    }

    // Helper for making requests
    async fn request<T, B>(&self, method: &str, path: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let body_str = if let Some(b) = &body {
            Some(serde_json::to_string(b)?)
        } else {
            None
        };
        let method = parse_method(method)?;
        let response = self
            .execute(method, path, body_str, Some("application/json"), Vec::new())
            .await?;
        let text = response.text().await?;
        serde_json::from_str(&text).map_err(Into::into)
    }

    async fn request_no_content<B>(&self, method: &str, path: &str, body: Option<B>) -> Result<()>
    where
        B: Serialize,
    {
        let body_str = if let Some(b) = &body {
            Some(serde_json::to_string(b)?)
        } else {
            None
        };
        let method = parse_method(method)?;
        self.execute(method, path, body_str, Some("application/json"), Vec::new())
            .await?;
        Ok(())
    }

    /// Put Object
    ///
    /// # Arguments
    /// * `object_name` - Object name
    /// * `content` - Object content
    pub async fn put_object(&self, object_name: &str, content: &str) -> Result<Object> {
        self.put_object_internal(object_name, content, None).await
    }

    /// Put Object with Checksum
    ///
    /// # Arguments
    /// * `object_name` - Object name
    /// * `content` - Object content
    /// * `algorithm` - Checksum algorithm to use
    pub async fn put_object_with_checksum(
        &self,
        object_name: &str,
        content: &str,
        algorithm: ChecksumAlgorithm,
    ) -> Result<Object> {
        self.put_object_internal(object_name, content, Some(algorithm))
            .await
    }

    async fn put_object_internal(
        &self,
        object_name: &str,
        content: &str,
        algorithm: Option<ChecksumAlgorithm>,
    ) -> Result<Object> {
        let path = format!("/n/{}/b/{}/o/{}", self.namespace, self.name, object_name);
        let mut extra_headers = Vec::new();

        if let Some(algo) = algorithm {
            let data = content.as_bytes();
            match algo {
                ChecksumAlgorithm::SHA256 => {
                    let mut hasher = Sha256::new();
                    hasher.update(data);
                    let result = hasher.finalize();
                    let b64 = BASE64.encode(result);
                    extra_headers.push(("opc-checksum-algorithm".to_owned(), "SHA256".to_owned()));
                    extra_headers.push(("opc-content-sha256".to_owned(), b64));
                }
                ChecksumAlgorithm::SHA384 => {
                    let mut hasher = Sha384::new();
                    hasher.update(data);
                    let result = hasher.finalize();
                    let b64 = BASE64.encode(result);
                    extra_headers.push(("opc-checksum-algorithm".to_owned(), "SHA384".to_owned()));
                    extra_headers.push(("opc-content-sha384".to_owned(), b64));
                }
                ChecksumAlgorithm::CRC32C => {
                    let crc = crc32c::crc32c(data);
                    let bytes = crc.to_be_bytes();
                    let b64 = BASE64.encode(bytes);
                    extra_headers.push(("opc-checksum-algorithm".to_owned(), "CRC32C".to_owned()));
                    extra_headers.push(("opc-content-crc32c".to_owned(), b64));
                }
            }
        }

        let response = self
            .execute(
                Method::PUT,
                &path,
                Some(content.to_owned()),
                Some("application/octet-stream"),
                extra_headers,
            )
            .await?;
        let headers = response.headers();
        let md5 = headers
            .get("opc-content-md5")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| Error::Other("Missing required header: opc-content-md5".to_string()))?
            .to_string();

        let mut checksum = None;

        if let Some(val) = headers
            .get("opc-content-sha256")
            .and_then(|h| h.to_str().ok())
        {
            checksum = Some(Checksum {
                algorithm: ChecksumAlgorithm::SHA256,
                value: val.to_string(),
            });
        } else if let Some(val) = headers
            .get("opc-content-sha384")
            .and_then(|h| h.to_str().ok())
        {
            checksum = Some(Checksum {
                algorithm: ChecksumAlgorithm::SHA384,
                value: val.to_string(),
            });
        } else if let Some(val) = headers
            .get("opc-content-crc32c")
            .and_then(|h| h.to_str().ok())
        {
            checksum = Some(Checksum {
                algorithm: ChecksumAlgorithm::CRC32C,
                value: val.to_string(),
            });
        }

        Ok(Object {
            name: object_name.to_string(),
            value: content.to_string(),
            md5,
            checksum,
        })
    }

    /// Get Object
    ///
    /// # Arguments
    /// * `object_name` - Object name
    pub async fn get_object(&self, object_name: &str) -> Result<Object> {
        let path = format!("/n/{}/b/{}/o/{}", self.namespace, self.name, object_name);
        let response = self
            .execute(Method::GET, &path, None, None, Vec::new())
            .await?;

        let headers = response.headers();
        let md5 = headers
            .get("content-md5")
            .or_else(|| headers.get("opc-multipart-md5"))
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| Error::Other("Missing required header: content-md5".to_string()))?
            .to_string();

        let mut checksum = None;

        if let Some(val) = headers
            .get("opc-content-sha256")
            .and_then(|h| h.to_str().ok())
        {
            checksum = Some(Checksum {
                algorithm: ChecksumAlgorithm::SHA256,
                value: val.to_string(),
            });
        } else if let Some(val) = headers
            .get("opc-content-sha384")
            .and_then(|h| h.to_str().ok())
        {
            checksum = Some(Checksum {
                algorithm: ChecksumAlgorithm::SHA384,
                value: val.to_string(),
            });
        } else if let Some(val) = headers
            .get("opc-content-crc32c")
            .and_then(|h| h.to_str().ok())
        {
            checksum = Some(Checksum {
                algorithm: ChecksumAlgorithm::CRC32C,
                value: val.to_string(),
            });
        }

        let value = response.text().await?;

        Ok(Object {
            name: object_name.to_string(),
            value,
            md5,
            checksum,
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

fn parse_method(method: &str) -> Result<Method> {
    Method::from_bytes(method.as_bytes())
        .map_err(|e| Error::Other(format!("Unsupported method {method}: {e}")))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
