use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub github_username: String,
    pub provider: String,
    pub key_id: String,
    pub created_at: DateTime<Utc>,
}

impl Account {
    pub fn new(
        name: impl Into<String>,
        email: impl Into<String>,
        github_username: impl Into<String>,
        provider: impl Into<String>,
        key_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            email: email.into(),
            github_username: github_username.into(),
            provider: provider.into(),
            key_id: key_id.into(),
            created_at: Utc::now(),
        }
    }
}
