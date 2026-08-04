use crate::cli::args::{AccountCommands, Commands, ProjectCommands};
use crate::config::ConfigManager;
use crate::domain::account::Account;
use crate::domain::error::{GitNestError, Result};
use crate::providers::{GitHubProvider, GitProvider};
use crate::services::{
    AccountService, BackupService, DoctorService, GitService, ProjectService, SshService,
};
use crate::storage::secure_store::{KeyringSecureStore, SecureStore};
use crate::storage::{JsonAccountRepository, JsonProjectRepository};
use crate::utils::normalize_path;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_command(cmd: Commands, config_mgr: ConfigManager) -> Result<()> {
    let account_repo = Arc::new(Mutex::new(JsonAccountRepository::new(config_mgr.accounts_path())));
    let project_repo = Arc::new(Mutex::new(JsonProjectRepository::new(config_mgr.projects_path())));

    let account_service = AccountService::new(account_repo);
    let project_service = ProjectService::new(project_repo);
    let ssh_service = SshService::new(config_mgr.ssh_dir());
    let git_service = GitService::new();

    match cmd {
        Commands::Init => {
            config_mgr.init()?;
            println!("Successfully initialized GitNest at {:?}", config_mgr.root_dir());
            Ok(())
        }
        Commands::Login => handle_login(&config_mgr, &account_service, &ssh_service).await,
        Commands::Accounts => handle_accounts(&account_service).await,
        Commands::Account { command } => match command {
            AccountCommands::Remove { id_or_username } => {
                let removed = account_service.remove_account(&id_or_username).await?;
                println!("Removed account: {} ({})", removed.github_username, removed.id);
                Ok(())
            }
        },
        Commands::Project { command } => match command {
            ProjectCommands::Add { path, account } => {
                handle_project_add(&project_service, &account_service, path, account).await
            }
            ProjectCommands::Remove { path } => {
                let raw_path = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let removed = project_service.unmap_project(&raw_path).await?;
                println!("Removed mapping for project: {:?}", removed.path);
                Ok(())
            }
        },
        Commands::Projects => handle_projects(&project_service, &account_service).await,
        Commands::Current => handle_current(&project_service, &account_service, &ssh_service).await,
        Commands::Scan { path } => handle_scan(&project_service, &account_service, path).await,
        Commands::Doctor => handle_doctor(config_mgr).await,
        Commands::Version => handle_version(&config_mgr),
        Commands::Export { output } => {
            let out_path = output.unwrap_or_else(|| PathBuf::from("gitnest-backup.zip"));
            let backup_svc = BackupService::new(config_mgr);
            backup_svc.export_backup(&out_path)?;
            println!("Successfully exported backup to {:?}", out_path);
            Ok(())
        }
        Commands::Import { input } => {
            let backup_svc = BackupService::new(config_mgr);
            backup_svc.import_backup(&input)?;
            println!("Successfully imported backup from {:?}", input);
            Ok(())
        }
        Commands::Clone {
            url,
            target_dir,
            account,
        } => handle_clone(&project_service, &account_service, &ssh_service, &git_service, &url, target_dir, account).await,
        Commands::Push { args } => handle_push(&project_service, &account_service, &ssh_service, &git_service, &args).await,
        Commands::Pull { args } => handle_pull(&project_service, &account_service, &ssh_service, &git_service, &args).await,
        Commands::Status => handle_status(&config_mgr, &project_service, &account_service, &ssh_service, &git_service).await,
    }
}

async fn handle_login(
    config_mgr: &ConfigManager,
    account_service: &AccountService,
    ssh_service: &SshService,
) -> Result<()> {
    if !config_mgr.is_initialized() {
        return Err(GitNestError::NotInitialized);
    }

    let config = config_mgr.load_config()?;
    let provider: Box<dyn GitProvider> = Box::new(GitHubProvider::new(&config.github.client_id));

    println!("Requesting GitHub OAuth device code...");
    let device_res = provider.request_device_code().await?;

    let clipboard_status = match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(&device_res.user_code) {
            Ok(_) => " (Code copied to clipboard!)",
            Err(_) => "",
        },
        Err(_) => "",
    };

    println!("\n=======================================================");
    println!("  GitHub Verification Code: {}", device_res.user_code);
    println!("=======================================================");
    println!("Code has been automatically copied to your clipboard!{}", clipboard_status);
    println!("\nPress [ENTER] to open browser ({}) and authorize...", device_res.verification_uri);
    io::stdout().flush().ok();

    let mut user_input = String::new();
    let _ = io::stdin().read_line(&mut user_input);

    let _ = open::that(&device_res.verification_uri);

    println!("\nWaiting for GitHub authorization...");
    let token = provider
        .poll_for_token(&device_res.device_code, device_res.interval)
        .await?;

    println!("Authorization received! Fetching user identity...");
    let provider_user = provider.fetch_user_info(&token).await?;

    println!("Authenticated as GitHub user: {} ({})", provider_user.username, provider_user.email);

    // Existing SSH Key Search Option
    let existing_keys = ssh_service.discover_existing_ssh_keys()?;
    let key_id = format!("id_ed25519_{}", provider_user.username);

    let key_path = if !existing_keys.is_empty() {
        println!("\nExisting SSH Keys Found in ~/.ssh/:");
        for (idx, k) in existing_keys.iter().enumerate() {
            println!("  [{}] Import {:?}", idx + 1, k);
        }
        println!("  [{}] Generate New Ed25519 Key", existing_keys.len() + 1);
        print!("Enter choice [1-{}]: ", existing_keys.len() + 1);
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice: usize = input.trim().parse().unwrap_or(existing_keys.len() + 1);

        if choice > 0 && choice <= existing_keys.len() {
            ssh_service.import_ssh_key(&existing_keys[choice - 1], &key_id)?
        } else {
            ssh_service.generate_keypair(&key_id, &format!("gitnest-{}", provider_user.username))?
        }
    } else {
        println!("Generating dedicated Ed25519 SSH keypair...");
        ssh_service.generate_keypair(&key_id, &format!("gitnest-{}", provider_user.username))?
    };

    let pub_key_path = PathBuf::from(format!("{}.pub", key_path.to_string_lossy()));
    if pub_key_path.exists() {
        if let Ok(pub_key_str) = std::fs::read_to_string(pub_key_path) {
            println!("Registering public SSH key with GitHub...");
            let _ = provider
                .upload_ssh_key(&token, &format!("GitNest Key ({})", provider_user.username), &pub_key_str)
                .await;
        }
    }

    let secure_store = KeyringSecureStore::new();
    let _ = secure_store.store_token(&provider_user.username, &token);

    let display_name = provider_user.name.unwrap_or_else(|| provider_user.username.clone());
    let account = Account::new(display_name, provider_user.email, provider_user.username, "github", key_id);

    account_service.add_account(account).await?;
    println!("\nAccount successfully registered with GitNest!");
    Ok(())
}

async fn handle_accounts(account_service: &AccountService) -> Result<()> {
    let accounts = account_service.list_accounts().await?;
    if accounts.is_empty() {
        println!("No accounts registered. Run `gitnest login` to add an account.");
        return Ok(());
    }

    use comfy_table::presets::UTF8_FULL;
    use comfy_table::*;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ACCOUNT ID").add_attribute(Attribute::Bold),
            Cell::new("USERNAME").add_attribute(Attribute::Bold),
            Cell::new("KEY ID").add_attribute(Attribute::Bold),
            Cell::new("EMAIL").add_attribute(Attribute::Bold),
        ]);

    for acc in accounts {
        table.add_row(vec![acc.id, acc.github_username, acc.key_id, acc.email]);
    }

    println!("\nRegistered GitHub Accounts:\n{}", table);
    println!();
    Ok(())
}

async fn handle_project_add(
    project_service: &ProjectService,
    account_service: &AccountService,
    path: Option<PathBuf>,
    account_opt: Option<String>,
) -> Result<()> {
    let raw_path = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let accounts = account_service.list_accounts().await?;

    if accounts.is_empty() {
        return Err(GitNestError::AccountNotFound("No accounts found. Run `gitnest login` first.".to_string()));
    }

    let selected_account = match account_opt {
        Some(target) => accounts
            .into_iter()
            .find(|a| a.id == target || a.github_username == target)
            .ok_or_else(|| GitNestError::AccountNotFound(target))?,
        None => {
            println!("Select GitHub Account for project:");
            for (idx, acc) in accounts.iter().enumerate() {
                println!("  [{}] {} ({})", idx + 1, acc.github_username, acc.email);
            }
            print!("Enter choice [1-{}]: ", accounts.len());
            io::stdout().flush().ok();

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice: usize = input.trim().parse().map_err(|_| GitNestError::InvalidPath("Invalid choice".to_string()))?;
            accounts[choice - 1].clone()
        }
    };

    let proj = project_service.map_project(&raw_path, &selected_account.id).await?;
    println!("Project {:?} successfully mapped to account: {}", proj.path, selected_account.github_username);
    Ok(())
}

async fn handle_projects(project_service: &ProjectService, account_service: &AccountService) -> Result<()> {
    let projects = project_service.list_projects().await?;
    let accounts = account_service.list_accounts().await?;

    if projects.is_empty() {
        println!("No mapped projects. Run `gitnest project add` to map a project.");
        return Ok(());
    }

    use comfy_table::presets::UTF8_FULL;
    use comfy_table::*;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("FOLDER / PROJECT NAME").add_attribute(Attribute::Bold),
            Cell::new("CONNECTED GITHUB ACCOUNT").add_attribute(Attribute::Bold),
            Cell::new("EMAIL").add_attribute(Attribute::Bold),
            Cell::new("FOLDER PATH").add_attribute(Attribute::Bold),
        ]);

    for proj in projects {
        let (username, email) = accounts
            .iter()
            .find(|a| a.id == proj.account_id)
            .map(|a| (a.github_username.as_str(), a.email.as_str()))
            .unwrap_or(("UNKNOWN", "N/A"));
        table.add_row(vec![
            proj.name,
            username.to_string(),
            email.to_string(),
            proj.path.to_string_lossy().to_string(),
        ]);
    }

    println!("\nAll Connected Projects & GitHub Accounts:\n{}", table);
    println!();
    Ok(())
}

async fn handle_current(
    project_service: &ProjectService,
    account_service: &AccountService,
    ssh_service: &SshService,
) -> Result<()> {
    let cwd = env::current_dir()?;
    let project = project_service.find_project(&cwd).await?.ok_or_else(|| GitNestError::ProjectNotFound(cwd.clone()))?;
    let account = account_service.find_account(&project.account_id).await?.ok_or_else(|| GitNestError::AccountNotFound(project.account_id.clone()))?;

    let key_path = ssh_service.resolve_key_path(&account.key_id);

    println!("\nCurrent Subdirectory Repository Identity:");
    println!("Project Root : {:?}", project.path);
    println!("GitHub User  : {}", account.github_username);
    println!("Email        : {}", account.email);
    println!("Resolved Key : {:?}", key_path);
    println!();
    Ok(())
}

async fn handle_scan(
    project_service: &ProjectService,
    account_service: &AccountService,
    path_opt: Option<PathBuf>,
) -> Result<()> {
    let scan_root = path_opt.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let scan_root = normalize_path(&scan_root);

    println!("Scanning directory {:?} recursively for Git repositories...", scan_root);
    let found_repos = project_service.scan_directory_for_git_repos(&scan_root);

    if found_repos.is_empty() {
        println!("No Git repositories found in {:?}", scan_root);
        return Ok(());
    }

    println!("\nDiscovered {} Git Repositories:", found_repos.len());
    for (idx, repo) in found_repos.iter().enumerate() {
        println!("  [{}] {:?}", idx + 1, repo);
    }

    let accounts = account_service.list_accounts().await?;
    if accounts.is_empty() {
        println!("\nNo registered accounts found. Run `gitnest login` to map these repositories.");
        return Ok(());
    }

    println!("\nSelect GitHub Account to map ALL scanned repositories (or enter 0 to cancel):");
    for (idx, acc) in accounts.iter().enumerate() {
        println!("  [{}] {} ({})", idx + 1, acc.github_username, acc.email);
    }
    print!("Enter choice: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse().unwrap_or(0);

    if choice > 0 && choice <= accounts.len() {
        let selected_acc = &accounts[choice - 1];
        for repo_path in found_repos {
            let _ = project_service.map_project(&repo_path, &selected_acc.id).await;
        }
        println!("Successfully mapped all scanned repositories to {}", selected_acc.github_username);
    }

    Ok(())
}

async fn handle_doctor(config_mgr: ConfigManager) -> Result<()> {
    let doctor = DoctorService::new(config_mgr);
    let report = doctor.run_diagnostics().await;

    println!("\n--- GitNest Health Diagnostic Report ---");
    println!("✓ Git Installed      : {} ({})", report.git_installed, report.git_version.unwrap_or_default());
    println!("✓ SSH Installed      : {}", report.ssh_installed);
    println!("✓ GitHub Reachable   : {}", report.github_reachable);
    println!("✓ Config Exists      : {}", report.config_exists);
    println!("✓ SSH Directory      : {}", report.ssh_dir_exists);
    println!("✓ Keyring Store      : {}", report.keyring_available);
    println!("✓ JSON Storage Valid : {}", report.json_files_valid);
    println!("✓ Accounts Count     : {}", report.registered_accounts_count);
    println!("✓ Mapped Projects    : {}", report.mapped_projects_count);
    println!();
    Ok(())
}

fn handle_version(config_mgr: &ConfigManager) -> Result<()> {
    use sysinfo::System;

    let sys_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let sys_os = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let arch = System::cpu_arch().unwrap_or_else(|| std::env::consts::ARCH.to_string());

    println!("GitNest Version : 1.0.0");
    println!("Rust Engine     : rustc 1.75+");
    println!("Operating System: {} ({})", sys_name, sys_os);
    println!("Architecture    : {}", arch);
    println!("Config Directory: {:?}", config_mgr.root_dir());
    Ok(())
}

async fn handle_clone(
    project_service: &ProjectService,
    account_service: &AccountService,
    ssh_service: &SshService,
    git_service: &GitService,
    url: &str,
    target_dir: Option<PathBuf>,
    account_opt: Option<String>,
) -> Result<()> {
    let accounts = account_service.list_accounts().await?;
    if accounts.is_empty() {
        return Err(GitNestError::AccountNotFound("No accounts found. Run `gitnest login` first.".to_string()));
    }

    let selected_acc = match account_opt {
        Some(t) => accounts.into_iter().find(|a| a.id == t || a.github_username == t).ok_or_else(|| GitNestError::AccountNotFound(t))?,
        None => {
            println!("Select GitHub Account to clone with:");
            for (idx, acc) in accounts.iter().enumerate() {
                println!("  [{}] {} ({})", idx + 1, acc.github_username, acc.email);
            }
            print!("Enter choice [1-{}]: ", accounts.len());
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice: usize = input.trim().parse().unwrap_or(1);
            accounts[choice - 1].clone()
        }
    };

    let key_path = ssh_service.resolve_key_path(&selected_acc.key_id);
    let cwd = env::current_dir()?;

    let mut clone_args = vec!["clone", url];
    let dest_str;
    if let Some(ref dest) = target_dir {
        dest_str = dest.to_string_lossy().to_string();
        clone_args.push(&dest_str);
    }

    let exit_code = git_service.execute_ephemeral(&cwd, &selected_acc, &key_path, &clone_args)?;
    if exit_code == 0 {
        let cloned_path = target_dir.unwrap_or_else(|| {
            let name = url.split('/').last().unwrap_or("repo").trim_end_matches(".git");
            cwd.join(name)
        });
        project_service.map_project(&cloned_path, &selected_acc.id).await?;
        println!("Clone complete! Mapped {:?} to {}", cloned_path, selected_acc.github_username);
    } else {
        return Err(GitNestError::GitExecutionFailed(format!("Git clone exited with code {}", exit_code)));
    }
    Ok(())
}

async fn handle_push(
    project_service: &ProjectService,
    account_service: &AccountService,
    ssh_service: &SshService,
    git_service: &GitService,
    args: &[String],
) -> Result<()> {
    execute_ephemeral_git_cmd("push", project_service, account_service, ssh_service, git_service, args).await
}

async fn handle_pull(
    project_service: &ProjectService,
    account_service: &AccountService,
    ssh_service: &SshService,
    git_service: &GitService,
    args: &[String],
) -> Result<()> {
    execute_ephemeral_git_cmd("pull", project_service, account_service, ssh_service, git_service, args).await
}

async fn execute_ephemeral_git_cmd(
    cmd_name: &str,
    project_service: &ProjectService,
    account_service: &AccountService,
    ssh_service: &SshService,
    git_service: &GitService,
    extra_args: &[String],
) -> Result<()> {
    let cwd = env::current_dir()?;
    let project = project_service.find_project(&cwd).await?.ok_or_else(|| GitNestError::ProjectNotFound(cwd.clone()))?;
    let account = account_service.find_account(&project.account_id).await?.ok_or_else(|| GitNestError::AccountNotFound(project.account_id.clone()))?;

    let key_path = ssh_service.resolve_key_path(&account.key_id);
    let mut command_args = vec![cmd_name];
    for a in extra_args {
        command_args.push(a.as_str());
    }

    println!("Executing `git {}` as GitHub identity: {}...", cmd_name, account.github_username);
    let code = git_service.execute_ephemeral(&project.path, &account, &key_path, &command_args)?;

    if code != 0 {
        return Err(GitNestError::GitExecutionFailed(format!("git {} failed with exit code {}", cmd_name, code)));
    }
    Ok(())
}

async fn handle_status(
    config_mgr: &ConfigManager,
    project_service: &ProjectService,
    account_service: &AccountService,
    ssh_service: &SshService,
    git_service: &GitService,
) -> Result<()> {
    println!("\n=======================================================");
    println!("               GitNest v1.0 System Status              ");
    println!("=======================================================");
    println!("GitNest Version : 1.0.0");
    println!("Config Path     : {:?}", config_mgr.root_dir());

    let cwd = env::current_dir()?;
    if let Ok(Some(proj)) = project_service.find_project(&cwd).await {
        if let Ok(Some(acc)) = account_service.find_account(&proj.account_id).await {
            let key_path = ssh_service.resolve_key_path(&acc.key_id);
            println!("\nActive Context  : Mapped Repository");
            println!("Repository Path : {:?}", proj.path);
            println!("GitHub Username : {}", acc.github_username);
            println!("SSH Key Status  : {}", if key_path.exists() { "VALID" } else { "MISSING" });

            if let Some(remote) = git_service.get_remote_url(&proj.path) {
                println!("Remote URL      : {}", remote);
            }
            if let Some(status_str) = git_service.get_git_status_summary(&proj.path) {
                println!("Git State       : {}", status_str);
            }
        }
    } else {
        println!("\nActive Context  : Unmapped Directory ({:?})", cwd);
    }
    println!("=======================================================\n");
    Ok(())
}
