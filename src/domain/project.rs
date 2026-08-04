use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
    pub account_id: String,
    pub mapped_at: DateTime<Utc>,
}

impl Project {
    pub fn new(path: PathBuf, name: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            path,
            name: name.into(),
            account_id: account_id.into(),
            mapped_at: Utc::now(),
        }
    }
}
