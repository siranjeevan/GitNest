use crate::config::model::Config;
use crate::domain::error::{GitNestError, Result};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigManager {
    root_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let base_dirs = BaseDirs::new().ok_or_else(|| {
            GitNestError::InvalidPath("Could not locate home directory".to_string())
        })?;
        let root_dir = base_dirs.home_dir().join(".gitnest");
        Ok(Self { root_dir })
    }

    pub fn with_custom_root(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }

    pub fn config_path(&self) -> PathBuf {
        self.root_dir.join("config.toml")
    }

    pub fn accounts_path(&self) -> PathBuf {
        self.root_dir.join("accounts.json")
    }

    pub fn projects_path(&self) -> PathBuf {
        self.root_dir.join("projects.json")
    }

    pub fn ssh_dir(&self) -> PathBuf {
        self.root_dir.join("ssh")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root_dir.join("logs")
    }

    pub fn installation_id_path(&self) -> PathBuf {
        self.root_dir.join("installation_id")
    }

    pub fn is_initialized(&self) -> bool {
        self.root_dir.exists() && self.config_path().exists()
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.root_dir)?;
        fs::create_dir_all(self.ssh_dir())?;
        fs::create_dir_all(self.logs_dir())?;

        // Restrict Unix permissions on ssh directory to 0700
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(self.ssh_dir(), fs::Permissions::from_mode(0o700))?;
        }

        let config_path = self.config_path();
        if !config_path.exists() {
            let default_config = Config::default();
            let toml_str = toml::to_string_pretty(&default_config)?;
            fs::write(config_path, toml_str)?;
        }

        let accounts_path = self.accounts_path();
        if !accounts_path.exists() {
            fs::write(accounts_path, r#"{"accounts":[]}"#)?;
        }

        let projects_path = self.projects_path();
        if !projects_path.exists() {
            fs::write(projects_path, r#"{"projects":[]}"#)?;
        }

        let inst_id_path = self.installation_id_path();
        if !inst_id_path.exists() {
            let inst_id = uuid::Uuid::new_v4().to_string();
            fs::write(inst_id_path, inst_id)?;
        }

        Ok(())
    }

    pub fn load_config(&self) -> Result<Config> {
        if !self.is_initialized() {
            return Err(GitNestError::NotInitialized);
        }
        let content = fs::read_to_string(self.config_path())?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_creates_structure() {
        let tmp = tempdir().unwrap();
        let manager = ConfigManager::with_custom_root(tmp.path().to_path_buf());

        assert!(!manager.is_initialized());
        manager.init().unwrap();
        assert!(manager.is_initialized());

        assert!(manager.config_path().exists());
        assert!(manager.accounts_path().exists());
        assert!(manager.projects_path().exists());
        assert!(manager.ssh_dir().exists());
        assert!(manager.logs_dir().exists());

        let cfg = manager.load_config().unwrap();
        assert_eq!(cfg.version, "1.0");
    }
}
