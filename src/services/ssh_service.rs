use crate::domain::error::{GitNestError, Result};
use crate::ssh::generator::{DefaultSshKeyManager, SshKeyManager};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SshService {
    ssh_dir: PathBuf,
}

impl SshService {
    pub fn new(ssh_dir: PathBuf) -> Self {
        Self { ssh_dir }
    }

    pub fn resolve_key_path(&self, key_id: &str) -> PathBuf {
        self.ssh_dir.join(key_id)
    }

    pub fn generate_keypair(&self, key_id: &str, comment: &str) -> Result<PathBuf> {
        let generator = DefaultSshKeyManager::new();
        let keypair = generator.generate_ed25519_keypair(&self.ssh_dir, key_id, comment)?;
        Ok(keypair.private_key_path)
    }

    pub fn discover_existing_ssh_keys(&self) -> Result<Vec<PathBuf>> {
        let base = BaseDirs::new().ok_or_else(|| {
            GitNestError::InvalidPath("Could not locate user home directory".to_string())
        })?;
        let user_ssh_dir = base.home_dir().join(".ssh");
        let mut keys = Vec::new();

        if user_ssh_dir.exists() {
            for entry in fs::read_dir(user_ssh_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let fname = path.file_name().unwrap_or_default().to_string_lossy();
                    if fname.starts_with("id_") && !fname.ends_with(".pub") {
                        keys.push(path);
                    }
                }
            }
        }
        Ok(keys)
    }

    pub fn import_ssh_key(&self, source_private_key: &Path, key_id: &str) -> Result<PathBuf> {
        if !self.ssh_dir.exists() {
            fs::create_dir_all(&self.ssh_dir)?;
        }

        let target_private = self.resolve_key_path(key_id);
        fs::copy(source_private_key, &target_private)?;

        let source_pub = source_private_key.with_extension("pub");
        if source_pub.exists() {
            let target_pub = PathBuf::from(format!("{}.pub", target_private.to_string_lossy()));
            fs::copy(source_pub, target_pub)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_private, fs::Permissions::from_mode(0o600))?;
        }

        Ok(target_private)
    }
}
