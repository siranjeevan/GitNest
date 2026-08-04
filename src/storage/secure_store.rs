use crate::domain::error::{GitNestError, Result};
use keyring::Entry;

pub trait SecureStore: Send + Sync {
    fn store_token(&self, username: &str, token: &str) -> Result<()>;
    fn get_token(&self, username: &str) -> Result<Option<String>>;
    fn delete_token(&self, username: &str) -> Result<()>;
}

pub struct KeyringSecureStore {
    service_name: String,
}

impl KeyringSecureStore {
    pub fn new() -> Self {
        Self {
            service_name: "gitnest".to_string(),
        }
    }
}

impl Default for KeyringSecureStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureStore for KeyringSecureStore {
    fn store_token(&self, username: &str, token: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, username)
            .map_err(|e| GitNestError::CredentialError(e.to_string()))?;
        entry
            .set_password(token)
            .map_err(|e| GitNestError::CredentialError(e.to_string()))?;
        Ok(())
    }

    fn get_token(&self, username: &str) -> Result<Option<String>> {
        let entry = Entry::new(&self.service_name, username)
            .map_err(|e| GitNestError::CredentialError(e.to_string()))?;
        match entry.get_password() {
            Ok(pass) => Ok(Some(pass)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(GitNestError::CredentialError(e.to_string())),
        }
    }

    fn delete_token(&self, username: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, username)
            .map_err(|e| GitNestError::CredentialError(e.to_string()))?;
        match entry.delete_password() {
            Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(GitNestError::CredentialError(e.to_string())),
        }
    }
}
