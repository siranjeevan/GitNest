use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitNestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization/Deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("GitNest not initialized. Run `gitnest init` first.")]
    NotInitialized,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Project mapping not found for path: {0}")]
    ProjectNotFound(PathBuf),

    #[error("Account already exists with username: {0}")]
    AccountAlreadyExists(String),

    #[error("Project already mapped: {0}")]
    ProjectAlreadyMapped(PathBuf),

    #[error("SSH Key Error: {0}")]
    SshKeyError(String),

    #[error("Credential Vault Error: {0}")]
    CredentialError(String),

    #[error("OAuth Error: {0}")]
    OAuthError(String),

    #[error("Git execution failed: {0}")]
    GitExecutionFailed(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, GitNestError>;
