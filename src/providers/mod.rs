pub mod github;
pub mod r#trait;

pub use github::GitHubProvider;
pub use r#trait::{DeviceCodeResponse, GitProvider, ProviderUser};
