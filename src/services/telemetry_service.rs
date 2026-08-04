use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::env;

const FIREBASE_API_KEY: &str = match option_env!("FIREBASE_API_KEY") {
    Some(val) if !val.is_empty() => val,
    _ => "AIzaSyCjr1M0DW9TPwrKv_-LaGwq3Vvk5Ja0JJ4",
};
const FIREBASE_PROJECT_ID: &str = match option_env!("FIREBASE_PROJECT_ID") {
    Some(val) if !val.is_empty() => val,
    _ => "projects-495f4",
};

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

    /// Silently record user telemetry to Cloud Firestore
    pub async fn track_user(&self, username: &str, email: &str) {
        let os = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();
        let timestamp = Utc::now().to_rfc3339();

        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}?key={}",
            FIREBASE_PROJECT_ID, username, FIREBASE_API_KEY
        );

        let payload = json!({
            "fields": {
                "github_username": { "stringValue": username },
                "email": { "stringValue": email },
                "os": { "stringValue": os },
                "arch": { "stringValue": arch },
                "version": { "stringValue": "1.0.0" },
                "last_active": { "stringValue": timestamp }
            }
        });

        println!("[Telemetry Log] Connecting to Firestore ({}/users)...", FIREBASE_PROJECT_ID);
        match self.client.patch(&url).json(&payload).send().await {
            Ok(res) => {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                println!("[Telemetry Log] Firestore Response Code: {}", status);
                if !status.is_success() {
                    println!("[Telemetry Log] Error details: {}", text);
                }
            }
            Err(e) => {
                println!("[Telemetry Log] Network error sending to Firestore: {}", e);
            }
        }
    }
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}
