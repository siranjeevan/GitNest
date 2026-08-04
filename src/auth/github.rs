use crate::domain::error::{GitNestError, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUserEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

pub struct GitHubOAuth {
    client_id: String,
    client: Client,
}

impl GitHubOAuth {
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client: Client::builder().user_agent("GitNest-CLI").build().unwrap(),
        }
    }

    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("scope", "read:user user:email admin:public_key"),
        ];

        let res = self
            .client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        if !res.status().is_success() {
            return Err(GitNestError::OAuthError(format!(
                "Failed to request device code: {}",
                res.status()
            )));
        }

        let body: DeviceCodeResponse = res
            .json()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        Ok(body)
    }

    pub async fn poll_for_token(&self, device_code: &str, mut interval: u64) -> Result<String> {
        if interval == 0 {
            interval = 5;
        }

        loop {
            sleep(Duration::from_secs(interval)).await;

            let params = [
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ];

            let res = self
                .client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&params)
                .send()
                .await
                .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

            let token_res: TokenResponse = res
                .json()
                .await
                .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

            if let Some(token) = token_res.access_token {
                return Ok(token);
            }

            if let Some(err) = token_res.error {
                match err.as_str() {
                    "authorization_pending" => continue,
                    "slow_down" => {
                        interval += 5;
                        continue;
                    }
                    _ => {
                        return Err(GitNestError::OAuthError(
                            token_res
                                .error_description
                                .unwrap_or_else(|| err.to_string()),
                        ))
                    }
                }
            }
        }
    }

    pub async fn fetch_user_info(&self, token: &str) -> Result<(GitHubUser, String)> {
        let res = self
            .client
            .get("https://api.github.com/user")
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        let user: GitHubUser = res
            .json()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        let email = if let Some(e) = &user.email {
            e.clone()
        } else {
            self.fetch_primary_email(token).await?
        };

        Ok((user, email))
    }

    async fn fetch_primary_email(&self, token: &str) -> Result<String> {
        let res = self
            .client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        let emails: Vec<GitHubUserEmail> = res
            .json()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        let primary = emails
            .into_iter()
            .find(|e| e.primary && e.verified)
            .ok_or_else(|| GitNestError::OAuthError("No primary verified email found".to_string()))?;

        Ok(primary.email)
    }

    pub async fn upload_ssh_key(&self, token: &str, title: &str, key: &str) -> Result<()> {
        let payload = serde_json::json!({
            "title": title,
            "key": key,
        });

        let res = self
            .client
            .post("https://api.github.com/user/keys")
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        if !res.status().is_success() && res.status() != reqwest::StatusCode::UNPROCESSABLE_ENTITY {
            return Err(GitNestError::OAuthError(format!(
                "Failed to upload SSH key to GitHub: {}",
                res.status()
            )));
        }

        Ok(())
    }
}
