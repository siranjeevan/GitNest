use crate::domain::account::Account;
use crate::domain::error::{GitNestError, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct GitService;

impl GitService {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_ephemeral(
        &self,
        working_dir: &Path,
        account: &Account,
        ssh_key_path: &Path,
        command_args: &[&str],
    ) -> Result<i32> {
        let ssh_command = format!(
            "ssh -i \"{}\" -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new",
            ssh_key_path.to_string_lossy()
        );

        let child = Command::new("git")
            .args(command_args)
            .current_dir(working_dir)
            .env("GIT_SSH_COMMAND", ssh_command)
            .env("GIT_AUTHOR_NAME", &account.name)
            .env("GIT_AUTHOR_EMAIL", &account.email)
            .env("GIT_COMMITTER_NAME", &account.name)
            .env("GIT_COMMITTER_EMAIL", &account.email)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GitNestError::GitExecutionFailed(format!("Failed to spawn git: {}", e)))?;

        let output = child
            .wait_with_output()
            .map_err(|e| GitNestError::GitExecutionFailed(format!("Git process error: {}", e)))?;

        Ok(output.status.code().unwrap_or(-1))
    }

    pub fn get_remote_url(&self, working_dir: &Path) -> Option<String> {
        let output = Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .current_dir(working_dir)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    pub fn get_git_status_summary(&self, working_dir: &Path) -> Option<String> {
        let output = Command::new("git")
            .args(["status", "--short", "--branch"])
            .current_dir(working_dir)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    pub fn is_git_repository(&self, working_dir: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(working_dir)
            .output();

        match output {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }

    pub fn clone_repo(
        &self,
        repo_url: &str,
        target_dir: &Path,
        ssh_key_path: &Path,
        token: Option<&str>,
    ) -> Result<std::path::PathBuf> {
        let mut cmd = Command::new("git");
        cmd.arg("clone");

        // If HTTPS URL and token is provided, embed token into URL for authentication
        let auth_url = if repo_url.starts_with("https://") {
            if let Some(t) = token {
                let parts: Vec<&str> = repo_url.splitn(2, "https://").collect();
                if parts.len() == 2 {
                    format!("https://x-access-token:{}@{}", t, parts[1])
                } else {
                    repo_url.to_string()
                }
            } else {
                repo_url.to_string()
            }
        } else {
            repo_url.to_string()
        };

        cmd.arg(&auth_url);
        cmd.current_dir(target_dir);

        // If SSH URL or SSH key exists, pass GIT_SSH_COMMAND
        if ssh_key_path.exists() {
            let ssh_command = format!(
                "ssh -i \"{}\" -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new",
                ssh_key_path.to_string_lossy()
            );
            cmd.env("GIT_SSH_COMMAND", ssh_command);
        }

        let output = cmd.output().map_err(|e| {
            GitNestError::GitExecutionFailed(format!("Failed to spawn git clone: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Sanitize token from stderr message if present
            let sanitized_err = if let Some(t) = token {
                stderr.replace(t, "********")
            } else {
                stderr.to_string()
            };
            return Err(GitNestError::GitExecutionFailed(format!(
                "git clone failed: {}",
                sanitized_err.trim()
            )));
        }

        // Extract folder name from URL (e.g., git@github.com:user/repo.git -> repo)
        let repo_name = repo_url
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .split('/')
            .next_back()
            .unwrap_or("cloned-repo");

        Ok(target_dir.join(repo_name))
    }
}

impl Default for GitService {
    fn default() -> Self {
        Self::new()
    }
}
