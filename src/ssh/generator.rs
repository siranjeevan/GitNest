use crate::domain::error::{GitNestError, Result};
use rand::rngs::OsRng;
use ssh_key::LineEnding;
use ssh_key::{private::Ed25519Keypair, PrivateKey};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GeneratedSshKeyPair {
    pub private_key_path: PathBuf,
    pub public_key_path: PathBuf,
    pub public_key_openssh: String,
}

pub trait SshKeyManager: Send + Sync {
    fn generate_ed25519_keypair(
        &self,
        ssh_dir: &Path,
        identity_name: &str,
        comment: &str,
    ) -> Result<GeneratedSshKeyPair>;
}

pub struct DefaultSshKeyManager;

impl DefaultSshKeyManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultSshKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SshKeyManager for DefaultSshKeyManager {
    fn generate_ed25519_keypair(
        &self,
        ssh_dir: &Path,
        identity_name: &str,
        comment: &str,
    ) -> Result<GeneratedSshKeyPair> {
        if !ssh_dir.exists() {
            fs::create_dir_all(ssh_dir)?;
        }

        let keypair = Ed25519Keypair::random(&mut OsRng);
        let private_key = PrivateKey::new(ssh_key::private::KeypairData::Ed25519(keypair), comment)
            .map_err(|e| GitNestError::SshKeyError(e.to_string()))?;

        let private_key_pem = private_key
            .to_openssh(LineEnding::LF)
            .map_err(|e| GitNestError::SshKeyError(e.to_string()))?;

        let public_key = private_key.public_key();
        let public_key_openssh = public_key
            .to_openssh()
            .map_err(|e| GitNestError::SshKeyError(e.to_string()))?;

        let file_prefix = identity_name;
        let private_key_path = ssh_dir.join(file_prefix);
        let public_key_path = ssh_dir.join(format!("{}.pub", file_prefix));

        fs::write(&private_key_path, private_key_pem.as_str())?;
        fs::write(&public_key_path, &public_key_openssh)?;

        // Set strict file permissions (0600) on private key for Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&private_key_path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(GeneratedSshKeyPair {
            private_key_path,
            public_key_path,
            public_key_openssh,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_ssh_key() {
        let tmp = tempdir().unwrap();
        let manager = DefaultSshKeyManager::new();
        let result = manager
            .generate_ed25519_keypair(tmp.path(), "octocat", "octocat@github.com")
            .unwrap();

        assert!(result.private_key_path.exists());
        assert!(result.public_key_path.exists());
        assert!(result.public_key_openssh.starts_with("ssh-ed25519 "));
    }
}
