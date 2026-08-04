use crate::domain::error::{GitNestError, Result};
use crate::providers::r#trait::{DeviceCodeResponse, GitProvider, ProviderUser};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct GitHubUserRaw {
    login: String,
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubUserEmailRaw {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug, Deserialize)]
struct TokenResponseRaw {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub struct GitHubProvider {
    client_id: String,
    client: Client,
}

impl GitHubProvider {
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client: Client::builder().user_agent("GitNest-CLI").build().unwrap(),
        }
    }
}

#[async_trait]
impl GitProvider for GitHubProvider {
    fn provider_name(&self) -> &'static str {
        "github"
    }

    async fn request_device_code(&self) -> Result<DeviceCodeResponse> {
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

        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        if !status.is_success() {
            return Err(GitNestError::OAuthError(format!(
                "GitHub Device Code Request Failed ({}): {}",
                status, text
            )));
        }

        let body: DeviceCodeResponse = serde_json::from_str(&text).map_err(|e| {
            GitNestError::OAuthError(format!(
                "Failed to parse device code response: {}. Body: {}",
                e, text
            ))
        })?;

        Ok(body)
    }

    async fn poll_for_token(&self, device_code: &str, mut interval: u64) -> Result<String> {
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

            let token_res: TokenResponseRaw = res
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

    async fn fetch_user_info(&self, token: &str) -> Result<ProviderUser> {
        let res = self
            .client
            .get("https://api.github.com/user")
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        let raw_user: GitHubUserRaw = res
            .json()
            .await
            .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

        let email = if let Some(e) = raw_user.email {
            e
        } else {
            let res_emails = self
                .client
                .get("https://api.github.com/user/emails")
                .header("Authorization", format!("token {}", token))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

            let emails: Vec<GitHubUserEmailRaw> = res_emails
                .json()
                .await
                .map_err(|e| GitNestError::OAuthError(e.to_string()))?;

            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
                .ok_or_else(|| GitNestError::OAuthError("No primary verified email".to_string()))?
        };

        Ok(ProviderUser {
            username: raw_user.login,
            name: raw_user.name,
            email,
        })
    }

    async fn upload_ssh_key(&self, token: &str, title: &str, key: &str) -> Result<()> {
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
                "Failed to upload key: {}",
                res.status()
            )));
        }

        Ok(())
    }
}
