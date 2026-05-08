use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretBundleContent {
    pub content: String,
    pub content_type: String,
    pub stage: Option<String>,
    pub name: Option<String>,
}

impl SecretBundleContent {
    pub fn decoded_bytes(&self) -> Result<Vec<u8>> {
        STANDARD
            .decode(&self.content)
            .map_err(|e| Error::Other(format!("Failed to decode secret bundle content: {e}")))
    }

    pub fn decoded_string(&self) -> Result<String> {
        String::from_utf8(self.decoded_bytes()?)
            .map_err(|e| Error::Other(format!("Failed to decode secret bundle utf-8: {e}")))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretBundle {
    pub secret_id: Option<String>,
    pub time_created: Option<String>,
    pub version_number: Option<i64>,
    pub secret_bundle_content: SecretBundleContent,
    #[serde(default)]
    pub stages: Vec<String>,
}
