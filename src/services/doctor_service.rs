use crate::config::ConfigManager;
use crate::storage::account_store::load_accounts;
use crate::storage::project_store::load_projects;
use crate::storage::secure_store::{KeyringSecureStore, SecureStore};
use reqwest::Client;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct DiagnosticReport {
    pub git_installed: bool,
    pub git_version: Option<String>,
    pub ssh_installed: bool,
    pub github_reachable: bool,
    pub config_exists: bool,
    pub ssh_dir_exists: bool,
    pub keyring_available: bool,
    pub json_files_valid: bool,
    pub registered_accounts_count: usize,
    pub mapped_projects_count: usize,
}

pub struct DoctorService {
    config_mgr: ConfigManager,
}

impl DoctorService {
    pub fn new(config_mgr: ConfigManager) -> Self {
        Self { config_mgr }
    }

    pub async fn run_diagnostics(&self) -> DiagnosticReport {
        let mut report = DiagnosticReport::default();

        // 1. Git Installed Check
        if let Ok(out) = Command::new("git").arg("--version").output() {
            if out.status.success() {
                report.git_installed = true;
                report.git_version = Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }

        // 2. SSH Installed Check
        if let Ok(out) = Command::new("ssh").arg("-V").output() {
            if out.status.success() || !out.stderr.is_empty() {
                report.ssh_installed = true;
            }
        }

        // 3. GitHub Reachable Check
        let client = Client::builder().timeout(Duration::from_secs(3)).build();
        if let Ok(c) = client {
            if let Ok(res) = c.get("https://api.github.com/zen").send().await {
                report.github_reachable = res.status().is_success();
            }
        }

        // 4. Config and Directories Check
        report.config_exists = self.config_mgr.is_initialized();
        report.ssh_dir_exists = self.config_mgr.ssh_dir().exists();

        // 5. Credential Store Check
        let store = KeyringSecureStore::new();
        report.keyring_available = store.get_token("healthcheck_test_username").is_ok();

        // 6. JSON files validity
        let acc_valid = load_accounts(&self.config_mgr.accounts_path()).is_ok();
        let proj_valid = load_projects(&self.config_mgr.projects_path()).is_ok();
        report.json_files_valid = acc_valid && proj_valid;

        if let Ok(accs) = load_accounts(&self.config_mgr.accounts_path()) {
            report.registered_accounts_count = accs.len();
        }
        if let Ok(projs) = load_projects(&self.config_mgr.projects_path()) {
            report.mapped_projects_count = projs.len();
        }

        report
    }
}
