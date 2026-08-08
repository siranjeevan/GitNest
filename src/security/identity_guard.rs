use crate::domain::account::Account;
use crate::domain::error::{GitNestError, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteInfo {
    pub host: String,
    pub owner: String,
    pub repository: String,
}

impl RemoteInfo {
    pub fn parse_github_url(url: &str) -> Option<Self> {
        let trimmed = url.trim();
        let cleaned = trimmed.trim_end_matches(".git");

        // Format 1: git@github.com:owner/repo
        if let Some(path) = cleaned.strip_prefix("git@github.com:") {
            let mut parts = path.split('/');
            let owner = parts.next()?;
            let repo = parts.next()?;
            return Some(Self {
                host: "github.com".to_string(),
                owner: owner.to_string(),
                repository: repo.to_string(),
            });
        }

        // Format 2: ssh://git@github.com/owner/repo
        if let Some(path) = cleaned.strip_prefix("ssh://git@github.com/") {
            let mut parts = path.split('/');
            let owner = parts.next()?;
            let repo = parts.next()?;
            return Some(Self {
                host: "github.com".to_string(),
                owner: owner.to_string(),
                repository: repo.to_string(),
            });
        }

        // Format 3: https://github.com/owner/repo
        if let Some(path) = cleaned.strip_prefix("https://github.com/") {
            let mut parts = path.split('/');
            let owner = parts.next()?;
            let repo = parts.next()?;
            return Some(Self {
                host: "github.com".to_string(),
                owner: owner.to_string(),
                repository: repo.to_string(),
            });
        }

        None
    }
}

pub struct IdentityGuard;

impl IdentityGuard {
    /// Validates complete alignment between Mapped Account, Remote Owner, Local Git Identity, and SSH Key
    pub fn validate_operation(
        working_dir: &Path,
        account: &Account,
        ssh_key_path: &Path,
        remote_url_opt: Option<&str>,
    ) -> Result<()> {
        // 1. Verify SSH Private Key existence & non-empty
        if !ssh_key_path.exists() {
            return Err(GitNestError::IdentityMismatch(format!(
                "BLOCKED: Dedicated SSH key does not exist at {:?}",
                ssh_key_path
            )));
        }

        // 1b. Verify SSH key content consistency & public key alignment
        let pub_key_path = ssh_key_path.with_extension("pub");
        if pub_key_path.exists() {
            let pub_content = std::fs::read_to_string(&pub_key_path).unwrap_or_default();
            if let Some(ref expected_fp) = account.ssh_key_fingerprint {
                // Calculate fingerprint of public key
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(pub_content.as_bytes());
                let actual_fp = format!("{:x}", hasher.finalize());

                if actual_fp != *expected_fp {
                    return Err(GitNestError::IdentityMismatch(format!(
                        "BLOCKED: Key Swap / Tamper Detected! SSH key fingerprint mismatch for account '{}'. Expected: {}, Actual: {}",
                        account.github_username, expected_fp, actual_fp
                    )));
                }
            }
        }

        // 2. Validate Remote Owner alignment if remote URL is provided
        if let Some(url) = remote_url_opt {
            if let Some(remote_info) = RemoteInfo::parse_github_url(url) {
                if !remote_info
                    .owner
                    .eq_ignore_ascii_case(&account.github_username)
                {
                    return Err(GitNestError::IdentityMismatch(format!(
                        "BLOCKED: Identity Mismatch! Mapped account is '{}', but remote URL owner is '{}' (URL: {}).\n\
                        GitNest prevents pushing/pulling to mismatched GitHub owners to avoid identity leakage.",
                        account.github_username, remote_info.owner, url
                    )));
                }
            }
        }

        // 3. Verify Local Git user.name and user.email if inside a Git repository
        let local_username = Command::new("git")
            .args(["config", "--local", "--get", "user.name"])
            .current_dir(working_dir)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let local_email = Command::new("git")
            .args(["config", "--local", "--get", "user.email"])
            .current_dir(working_dir)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        if !local_username.is_empty()
            && !local_username.eq_ignore_ascii_case(&account.github_username)
        {
            return Err(GitNestError::IdentityMismatch(format!(
                "BLOCKED: Repository-local git user.name '{}' does not match mapped account '{}'.",
                local_username, account.github_username
            )));
        }

        if !local_email.is_empty() && !local_email.eq_ignore_ascii_case(&account.email) {
            return Err(GitNestError::IdentityMismatch(format!(
                "BLOCKED: Repository-local git user.email '{}' does not match mapped account email '{}'.",
                local_email, account.email
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_url_parsing() {
        let r1 = RemoteInfo::parse_github_url("git@github.com:siranjeevan/GitNest.git").unwrap();
        assert_eq!(r1.owner, "siranjeevan");
        assert_eq!(r1.repository, "GitNest");

        let r2 = RemoteInfo::parse_github_url("https://github.com/siranjeevanhope3/my-repo.git")
            .unwrap();
        assert_eq!(r2.owner, "siranjeevanhope3");
        assert_eq!(r2.repository, "my-repo");

        let r3 = RemoteInfo::parse_github_url("ssh://git@github.com/octocat/Hello-World").unwrap();
        assert_eq!(r3.owner, "octocat");
        assert_eq!(r3.repository, "Hello-World");
    }
}
