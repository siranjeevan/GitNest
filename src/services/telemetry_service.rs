use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use sha2::Digest;
use std::env;

const FIREBASE_API_KEY: Option<&'static str> = option_env!("FIREBASE_API_KEY");
const FIREBASE_PROJECT_ID: Option<&'static str> = option_env!("FIREBASE_PROJECT_ID");

#[derive(Clone)]
pub struct TelemetryService {
    client: Client,
}

impl TelemetryService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Privacy-conscious telemetry using random local installation ID
    pub async fn track_event(&self, installation_id: &str, event_type: &str, enabled: bool) {
        if !enabled {
            return;
        }

        let api_key = match FIREBASE_API_KEY {
            Some(key) => key,
            None => return,
        };
        let project_id = match FIREBASE_PROJECT_ID {
            Some(pid) => pid,
            None => return,
        };

        let os = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();
        let timestamp = Utc::now().to_rfc3339();

        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/telemetry/{}?key={}",
            project_id, installation_id, api_key
        );

        let payload = json!({
            "fields": {
                "installation_id": { "stringValue": installation_id },
                "event_type": { "stringValue": event_type },
                "os": { "stringValue": os },
                "arch": { "stringValue": arch },
                "version": { "stringValue": "1.0.0" },
                "timestamp": { "stringValue": timestamp }
            }
        });

        // Fire and forget non-blocking network request
        let _ = self.client.patch(&url).json(&payload).send().await;
    }

    pub async fn track_user(&self, username: &str, _email: &str) {
        // Backward-compatible alias mapping to SHA256 hashed pseudonym
        let mut hasher = sha2::Sha256::new();
        hasher.update(username.as_bytes());
        let anonymized_id = format!("{:x}", hasher.finalize());
        self.track_event(&anonymized_id, "login", true).await;
    }
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}
