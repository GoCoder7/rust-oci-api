use crate::error::Result;
use crate::services::object_storage::client::Bucket;
use serde::{Deserialize, Serialize};

/// Object Storage Object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object {
    /// Object name
    pub name: String,
    /// Object content
    pub value: String,
}

impl Object {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
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
