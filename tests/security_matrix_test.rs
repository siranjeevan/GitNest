use gitnest::domain::account::Account;
use gitnest::domain::error::GitNestError;
use gitnest::security::{IdentityGuard, RemoteInfo};
use gitnest::ssh::generator::{DefaultSshKeyManager, SshKeyManager};
use tempfile::tempdir;

#[test]
fn test_remote_url_parsing_all_formats() {
    let r1 = RemoteInfo::parse_github_url("git@github.com:siranjeevan/GitNest.git").unwrap();
    assert_eq!(r1.owner, "siranjeevan");
    assert_eq!(r1.repository, "GitNest");

    let r2 =
        RemoteInfo::parse_github_url("https://github.com/siranjeevanhope3/my-repo.git").unwrap();
    assert_eq!(r2.owner, "siranjeevanhope3");
    assert_eq!(r2.repository, "my-repo");

    let r3 = RemoteInfo::parse_github_url("ssh://git@github.com/octocat/Hello-World").unwrap();
    assert_eq!(r3.owner, "octocat");
    assert_eq!(r3.repository, "Hello-World");
}

#[test]
fn test_identity_guard_blocks_mismatched_remote_owner() {
    let tmp = tempdir().unwrap();
    let ssh_dir = tmp.path().join("ssh");
    std::fs::create_dir_all(&ssh_dir).unwrap();

    let ssh_manager = DefaultSshKeyManager::new();
    let keypair = ssh_manager
        .generate_ed25519_keypair(&ssh_dir, "account_a", "account_a@github.com")
        .unwrap();

    let account_a = Account::new(
        "Account A",
        "account_a@github.com",
        "account_a",
        "github",
        "account_a",
    );
    let remote_b = "git@github.com:account_b/project-b.git";

    // Attempting to run operation with Account A on Account B's remote MUST FAIL CLOSED
    let result = IdentityGuard::validate_operation(
        tmp.path(),
        &account_a,
        &keypair.private_key_path,
        Some(remote_b),
    );
    assert!(result.is_err());

    if let Err(GitNestError::IdentityMismatch(err_msg)) = result {
        assert!(err_msg.contains("Identity Mismatch"));
        assert!(err_msg.contains("account_a"));
        assert!(err_msg.contains("account_b"));
    } else {
        panic!("Expected IdentityMismatch error variant");
    }
}

#[test]
fn test_identity_guard_allows_matching_remote_owner() {
    let tmp = tempdir().unwrap();
    let ssh_dir = tmp.path().join("ssh");
    std::fs::create_dir_all(&ssh_dir).unwrap();

    let ssh_manager = DefaultSshKeyManager::new();
    let keypair = ssh_manager
        .generate_ed25519_keypair(&ssh_dir, "account_a", "account_a@github.com")
        .unwrap();

    let account_a = Account::new(
        "Account A",
        "account_a@github.com",
        "account_a",
        "github",
        "account_a",
    );
    let remote_a = "git@github.com:account_a/project-a.git";

    let result = IdentityGuard::validate_operation(
        tmp.path(),
        &account_a,
        &keypair.private_key_path,
        Some(remote_a),
    );
    assert!(result.is_ok());
}

#[test]
fn test_key_content_swap_attack() {
    use sha2::{Digest, Sha256};

    let tmp = tempdir().unwrap();
    let ssh_dir = tmp.path().join("ssh");
    std::fs::create_dir_all(&ssh_dir).unwrap();

    let ssh_manager = DefaultSshKeyManager::new();
    let keypair_a = ssh_manager
        .generate_ed25519_keypair(&ssh_dir, "key_a", "account_a@github.com")
        .unwrap();

    let keypair_b = ssh_manager
        .generate_ed25519_keypair(&ssh_dir, "key_b", "account_b@github.com")
        .unwrap();

    // Calculate public key fingerprint of key_a
    let pub_a_content = std::fs::read_to_string(&keypair_a.public_key_path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(pub_a_content.as_bytes());
    let fp_a = format!("{:x}", hasher.finalize());

    let mut account_a = Account::new(
        "Account A",
        "account_a@github.com",
        "account_a",
        "github",
        "key_a",
    );
    account_a.ssh_key_fingerprint = Some(fp_a);

    let remote_a = "git@github.com:account_a/project-a.git";

    // 1. Initial valid key check -> ALLOW
    assert!(IdentityGuard::validate_operation(
        tmp.path(),
        &account_a,
        &keypair_a.private_key_path,
        Some(remote_a)
    )
    .is_ok());

    // 2. Perform Key Content Swap Attack: Replace key_a.pub with key_b.pub content
    let pub_b_content = std::fs::read_to_string(&keypair_b.public_key_path).unwrap();
    std::fs::write(&keypair_a.public_key_path, pub_b_content).unwrap();

    // 3. Validation must detect fingerprint mismatch and BLOCK
    let swapped_result = IdentityGuard::validate_operation(
        tmp.path(),
        &account_a,
        &keypair_a.private_key_path,
        Some(remote_a),
    );
    assert!(swapped_result.is_err());
    if let Err(GitNestError::IdentityMismatch(msg)) = swapped_result {
        assert!(msg.contains("Key Swap / Tamper Detected!"));
    } else {
        panic!("Expected Key Swap error variant");
    }

    // 4. Restore original key_a content -> ALLOW
    std::fs::write(&keypair_a.public_key_path, pub_a_content).unwrap();
    assert!(IdentityGuard::validate_operation(
        tmp.path(),
        &account_a,
        &keypair_a.private_key_path,
        Some(remote_a)
    )
    .is_ok());
}

#[test]
fn test_env_override_inspection() {
    use gitnest::security::EnvGuard;
    std::env::set_var("GIT_SSH_COMMAND", "ssh -i /tmp/malicious_key");
    let warnings = EnvGuard::inspect_environment();
    assert!(!warnings.is_empty());
    assert!(warnings[0].contains("GIT_SSH_COMMAND"));

    let block_result = EnvGuard::validate_and_sanitize();
    assert!(block_result.is_err());

    std::env::remove_var("GIT_SSH_COMMAND");
    assert!(EnvGuard::validate_and_sanitize().is_ok());
}

#[test]
fn test_atomic_json_write_integrity() {
    use gitnest::storage::atomic::atomic_write_json;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct DummyConfig {
        name: String,
        count: u32,
    }

    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("config.json");

    let data = DummyConfig {
        name: "GitNest Production Test".to_string(),
        count: 42,
    };

    // Atomic write
    atomic_write_json(&file_path, &data).unwrap();
    assert!(file_path.exists());

    // Read back and verify
    let read_content = std::fs::read_to_string(&file_path).unwrap();
    let restored: DummyConfig = serde_json::from_str(&read_content).unwrap();
    assert_eq!(data, restored);
}
