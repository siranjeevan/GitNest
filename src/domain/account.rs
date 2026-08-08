use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub github_username: String,
    #[serde(default)]
    pub github_user_id: Option<u64>,
    pub provider: String,
    pub key_id: String,
    #[serde(default)]
    pub ssh_key_fingerprint: Option<String>,
    #[serde(default)]
    pub github_ssh_key_id: Option<u64>,
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub last_used_at: DateTime<Utc>,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "active".to_string()
}

impl Account {
    pub fn new(
        name: impl Into<String>,
        email: impl Into<String>,
        github_username: impl Into<String>,
        provider: impl Into<String>,
        key_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            email: email.into(),
            github_username: github_username.into(),
            github_user_id: None,
            provider: provider.into(),
            key_id: key_id.into(),
            ssh_key_fingerprint: None,
            github_ssh_key_id: None,
            created_at: now,
            last_used_at: now,
            status: "active".to_string(),
        }
    }

    pub fn with_github_user_id(mut self, user_id: u64) -> Self {
        self.github_user_id = Some(user_id);
        self
    }
}
