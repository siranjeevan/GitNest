use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub log_level: String,
    pub default_ssh_key_type: String,
    #[serde(default)]
    pub git: GitSettings,
    #[serde(default)]
    pub github: GitHubSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSettings {
    pub auto_configure_user: bool,
    pub auto_configure_ssh: bool,
}

impl Default for GitSettings {
    fn default() -> Self {
        Self {
            auto_configure_user: true,
            auto_configure_ssh: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSettings {
    pub client_id: String,
}

impl Default for GitHubSettings {
    fn default() -> Self {
        Self {
            client_id: "178c6fc778ccc68e1d6a".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            log_level: "info".to_string(),
            default_ssh_key_type: "ed25519".to_string(),
            git: GitSettings::default(),
            github: GitHubSettings::default(),
        }
    }
}
