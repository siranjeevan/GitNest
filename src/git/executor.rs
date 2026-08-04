use crate::domain::error::{GitNestError, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub trait GitExecutor: Send + Sync {
    fn execute_with_ssh(
        &self,
        working_dir: &Path,
        ssh_key_path: &Path,
        command_args: &[&str],
    ) -> Result<i32>;
    fn set_local_config(&self, working_dir: &Path, key: &str, value: &str) -> Result<()>;
    fn get_local_config(&self, working_dir: &Path, key: &str) -> Result<Option<String>>;
    fn is_git_repository(&self, working_dir: &Path) -> Result<bool>;
}

pub struct SystemGitExecutor;

impl SystemGitExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemGitExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl GitExecutor for SystemGitExecutor {
    fn execute_with_ssh(
        &self,
        working_dir: &Path,
        ssh_key_path: &Path,
        command_args: &[&str],
    ) -> Result<i32> {
        let ssh_command = format!(
            "ssh -i \"{}\" -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new",
            ssh_key_path.to_string_lossy()
        );

        let mut child = Command::new("git")
            .args(command_args)
            .current_dir(working_dir)
            .env("GIT_SSH_COMMAND", ssh_command)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| GitNestError::GitExecutionFailed(format!("Failed to spawn git: {}", e)))?;

        let status = child
            .wait()
            .map_err(|e| GitNestError::GitExecutionFailed(format!("Git process error: {}", e)))?;

        Ok(status.code().unwrap_or(-1))
    }

    fn set_local_config(&self, working_dir: &Path, key: &str, value: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["config", "--local", key, value])
            .current_dir(working_dir)
            .output()
            .map_err(|e| GitNestError::GitExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(GitNestError::GitExecutionFailed(err.to_string()));
        }
        Ok(())
    }

    fn get_local_config(&self, working_dir: &Path, key: &str) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(["config", "--local", "--get", key])
            .current_dir(working_dir)
            .output()
            .map_err(|e| GitNestError::GitExecutionFailed(e.to_string()))?;

        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn is_git_repository(&self, working_dir: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(working_dir)
            .output();

        match output {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }
}
