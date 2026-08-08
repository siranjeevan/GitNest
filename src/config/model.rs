use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub log_level: String,
    pub default_ssh_key_type: String,
    #[serde(default = "default_telemetry_enabled")]
    pub telemetry_enabled: bool,
    #[serde(default)]
    pub git: GitSettings,
    #[serde(default)]
    pub github: GitHubSettings,
    #[serde(default)]
    pub firebase: FirebaseSettings,
}

fn default_telemetry_enabled() -> bool {
    true
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseSettings {
    pub api_key: String,
    pub project_id: String,
}

impl Default for FirebaseSettings {
    fn default() -> Self {
        Self {
            api_key: "AIzaSyCjr1M0DW9TPwrKv_-LaGwq3Vvk5Ja0JJ4".to_string(),
            project_id: "projects-495f4".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            log_level: "info".to_string(),
            default_ssh_key_type: "ed25519".to_string(),
            telemetry_enabled: true,
            git: GitSettings::default(),
            github: GitHubSettings::default(),
            firebase: FirebaseSettings::default(),
        }
    }
}
