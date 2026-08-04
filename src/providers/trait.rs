use crate::domain::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderUser {
    pub username: String,
    pub name: Option<String>,
    pub email: String,
}

#[async_trait]
pub trait GitProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    async fn request_device_code(&self) -> Result<DeviceCodeResponse>;
    async fn poll_for_token(&self, device_code: &str, interval: u64) -> Result<String>;
    async fn fetch_user_info(&self, token: &str) -> Result<ProviderUser>;
    async fn upload_ssh_key(&self, token: &str, title: &str, pub_key: &str) -> Result<()>;
    async fn create_repository(&self, token: &str, name: &str, private: bool) -> Result<String>;
}
