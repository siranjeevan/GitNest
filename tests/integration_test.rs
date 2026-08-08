use gitnest::config::ConfigManager;
use gitnest::domain::account::Account;
use gitnest::services::{AccountService, ProjectService, SshService};
use gitnest::storage::{JsonAccountRepository, JsonProjectRepository};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_revised_service_layer_lifecycle() {
    let tmp = tempdir().unwrap();
    let config_mgr = ConfigManager::with_custom_root(tmp.path().to_path_buf());
    config_mgr.init().unwrap();

    let account_repo = Arc::new(Mutex::new(JsonAccountRepository::new(
        config_mgr.accounts_path(),
    )));
    let project_repo = Arc::new(Mutex::new(JsonProjectRepository::new(
        config_mgr.projects_path(),
    )));

    let account_service = AccountService::new(account_repo);
    let project_service = ProjectService::new(project_repo);
    let ssh_service = SshService::new(config_mgr.ssh_dir());

    // 1. Generate SSH Key with key_id
    let key_id = "octocat_key";
    let key_path = ssh_service
        .generate_keypair(key_id, "octocat@github.com")
        .unwrap();

    assert!(key_path.exists());
    assert_eq!(ssh_service.resolve_key_path(key_id), key_path);

    // 2. Add Account using key_id
    let acc = Account::new(
        "Octocat Dev",
        "octocat@github.com",
        "octocat",
        "github",
        key_id,
    );
    account_service.add_account(acc.clone()).await.unwrap();

    let accounts = account_service.list_accounts().await.unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].key_id, key_id);

    // 3. Subdirectory Root Detection & Project Mapping
    let root_proj_dir = tmp.path().join("my-git-repo");
    let git_dir = root_proj_dir.join(".git");
    let deep_nested_dir = root_proj_dir.join("src").join("components").join("button");

    std::fs::create_dir_all(&git_dir).unwrap();
    std::fs::create_dir_all(&deep_nested_dir).unwrap();

    // Verify upward root lookup from deep_nested_dir
    let detected_root = project_service.detect_repo_root(&deep_nested_dir).unwrap();
    assert_eq!(detected_root, Some(root_proj_dir.clone()));

    // Map project from deep nested dir
    project_service
        .map_project(&deep_nested_dir, &acc.id)
        .await
        .unwrap();

    // Query project from deep nested dir
    let found_proj = project_service
        .find_project(&deep_nested_dir)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found_proj.account_id, acc.id);
    assert_eq!(
        gitnest::utils::normalize_path(&found_proj.path),
        gitnest::utils::normalize_path(&root_proj_dir)
    );
}
