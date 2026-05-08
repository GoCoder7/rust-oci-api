use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    pub id: String,
    pub display_name: Option<String>,
    pub lifecycle_state: Option<String>,
    pub current_key_version: Option<String>,
    pub time_created: Option<String>,
    pub vault_id: Option<String>,
}
