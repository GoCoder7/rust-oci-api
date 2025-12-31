use crate::error::{Error, Result};
use crate::services::object_storage::client::Bucket;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256, Sha384};

/// Checksum type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Checksum {
    pub algorithm: ChecksumAlgorithm,
    pub value: String,
}

/// Checksum Algorithm
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChecksumAlgorithm {
    SHA256,
    SHA384,
    CRC32C,
}

/// Object Storage Object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object {
    /// Object name
    pub name: String,
    /// Object content
    pub value: String,
    /// MD5 Checksum (Default)
    pub md5: String,
    /// Additional Checksum (Optional)
    pub checksum: Option<Checksum>,
}

impl Object {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        let digest = md5::compute(value.as_bytes());
        let md5 = BASE64.encode(*digest);

        Self {
            name: name.into(),
            value,
            md5,
            checksum: None,
        }
    }

    /// Verify checksums
    pub fn verify_checksums(&self) -> Result<()> {
        let data = self.value.as_bytes();

        // Verify MD5 (Default)
        let digest = md5::compute(data);
        let calculated = BASE64.encode(*digest);
        if calculated != self.md5 {
            return Err(Error::Other(format!(
                "MD5 checksum mismatch: expected {}, calculated {}",
                self.md5, calculated
            )));
        }

        // Verify Additional Checksum
        if let Some(checksum) = &self.checksum {
            match checksum.algorithm {
                ChecksumAlgorithm::SHA256 => {
                    let mut hasher = Sha256::new();
                    hasher.update(data);
                    let result = hasher.finalize();
                    let calculated = BASE64.encode(result);
                    if calculated != checksum.value {
                        return Err(Error::Other(format!(
                            "SHA256 checksum mismatch: expected {}, calculated {}",
                            checksum.value, calculated
                        )));
                    }
                }
                ChecksumAlgorithm::SHA384 => {
                    let mut hasher = Sha384::new();
                    hasher.update(data);
                    let result = hasher.finalize();
                    let calculated = BASE64.encode(result);
                    if calculated != checksum.value {
                        return Err(Error::Other(format!(
                            "SHA384 checksum mismatch: expected {}, calculated {}",
                            checksum.value, calculated
                        )));
                    }
                }
                ChecksumAlgorithm::CRC32C => {
                    let calculated_crc = crc32c::crc32c(data);
                    let bytes = calculated_crc.to_be_bytes();
                    let calculated = BASE64.encode(bytes);

                    if calculated != checksum.value {
                        return Err(Error::Other(format!(
                            "CRC32C checksum mismatch: expected {}, calculated {}",
                            checksum.value, calculated
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Retention Rule Duration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionDuration {
    /// Time amount
    pub time_amount: u64,
    /// Time unit (YEARS, DAYS)
    pub time_unit: RetentionTimeUnit,
}

/// Retention Time Unit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetentionTimeUnit {
    Years,
    Days,
}

/// Retention Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionRule {
    /// Retention Rule ID
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<RetentionDuration>,
    /// Time rule locked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_rule_locked: Option<String>,
    /// Time created
    pub time_created: String,
    /// Time modified
    pub time_modified: String,
    /// ETag
    pub etag: String,
}

impl From<RetentionRule> for String {
    fn from(rule: RetentionRule) -> Self {
        rule.id
    }
}

impl From<&RetentionRule> for String {
    fn from(rule: &RetentionRule) -> Self {
        rule.id.clone()
    }
}

/// Retention Rule Details
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RetentionRuleDetails {
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<RetentionDuration>,
    /// Time rule locked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_rule_locked: Option<String>,
}

impl RetentionRule {
    /// Delete this retention rule
    pub async fn delete(&self, bucket: &Bucket) -> Result<()> {
        bucket.delete_retention_rule(&self.id).await
    }

    /// Update this retention rule
    pub async fn update(
        &self,
        bucket: &Bucket,
        details: RetentionRuleDetails,
    ) -> Result<RetentionRule> {
        bucket.update_retention_rule(&self.id, details).await
    }
}
