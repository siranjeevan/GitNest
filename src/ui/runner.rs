use crate::config::ConfigManager;
use crate::domain::account::Account;
use crate::domain::error::Result;
use crate::providers::github::GitHubProvider;
use crate::providers::r#trait::GitProvider;
use crate::services::{AccountService, ProjectService, SshService, TelemetryService};
use crate::storage::secure_store::KeyringSecureStore;
use crate::storage::secure_store::SecureStore;
use crate::storage::{JsonAccountRepository, JsonProjectRepository};
use crate::ui::app::render_app;
use crate::ui::state::{AppState, LoginPhase, Screen};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::env;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_tui_dashboard(config_mgr: &ConfigManager) -> Result<()> {
    // 1. Setup Raw Terminal Mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize Application State & Services
    let mut state = AppState::new();

    let account_repo = Arc::new(Mutex::new(JsonAccountRepository::new(
        config_mgr.accounts_path(),
    )));
    let project_repo = Arc::new(Mutex::new(JsonProjectRepository::new(
        config_mgr.projects_path(),
    )));

    let account_service = AccountService::new(account_repo);
    let project_service = ProjectService::new(project_repo);

    if let Ok(accounts) = account_service.list_accounts().await {
        state.accounts = accounts;
    }

    if let Ok(projects) = project_service.list_projects().await {
        state.projects = projects;
    }

    let cwd = env::current_dir().unwrap_or_default();
    if let Ok(Some(proj)) = project_service.find_project(&cwd).await {
        if let Ok(Some(acc)) = account_service.find_account(&proj.account_id).await {
            state.active_account = Some(acc);
        }
        state.active_project = Some(proj);
    }

    // Fetch Global Git Config identity if present
    let global_user = std::process::Command::new("git")
        .args(["config", "--global", "user.name"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .ok()
        .filter(|s| !s.is_empty());

    let global_email = std::process::Command::new("git")
        .args(["config", "--global", "user.email"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .ok()
        .filter(|s| !s.is_empty());

    state.global_git_user = global_user;
    state.global_git_email = global_email;

    // 3. Event Handling Loop
    loop {
        state.spinner_frame = state.spinner_frame.wrapping_add(1);
        terminal.draw(|f| render_app(f, &state))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Modal Active Intercepts
                if state.show_help_modal {
                    if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                        state.show_help_modal = false;
                    }
                    continue;
                }

                if state.modal_switch_account.is_some() {
                    match key.code {
                        KeyCode::Esc => state.modal_switch_account = None,
                        KeyCode::Enter => {
                            if let Some(target) = state.modal_switch_account.take() {
                                state.active_account = Some(target.clone());
                                state.set_notification(
                                    format!("Switched identity context to {}", target.github_username),
                                    false,
                                );
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                if state.modal_remove_account.is_some() {
                    match key.code {
                        KeyCode::Esc => state.modal_remove_account = None,
                        KeyCode::Enter | KeyCode::Char('d') => {
                            if let Some(target) = state.modal_remove_account.take() {
                                let username = target.github_username.clone();
                                if let Err(e) = account_service.remove_account(&target.id).await {
                                    state.set_notification(format!("Failed to remove account: {}", e), true);
                                } else {
                                    if let Ok(updated) = account_service.list_accounts().await {
                                        state.accounts = updated;
                                    }
                                    if state.selected_account_index > 0 && state.selected_account_index >= state.accounts.len() {
                                        state.selected_account_index = state.accounts.len().saturating_sub(1);
                                    }
                                    if let Some(ref active) = state.active_account {
                                        if active.id == target.id {
                                            state.active_account = None;
                                        }
                                    }
                                    state.set_notification(format!("Account @{} removed successfully", username), false);
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                if state.show_connect_modal {
                    match key.code {
                        KeyCode::Esc => state.show_connect_modal = false,
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !state.accounts.is_empty() && state.selected_connect_account_index < state.accounts.len() - 1 {
                                state.selected_connect_account_index += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_connect_account_index > 0 {
                                state.selected_connect_account_index -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            state.show_connect_modal = false;
                            let cwd = env::current_dir().unwrap_or_default();
                            let target_account = state.accounts.get(state.selected_connect_account_index).cloned().or_else(|| state.active_account.clone());
                            if let Some(acc) = target_account {
                                state.active_account = Some(acc.clone());
                                if let Ok(proj) = project_service.map_project(&cwd, &acc.id).await {
                                    state.active_project = Some(proj);
                                    if let Ok(projects) = project_service.list_projects().await {
                                        state.projects = projects;
                                    }
                                    state.set_notification(format!("Connected repository '{}' to @{}", cwd.display(), acc.github_username), false);
                                }
                            } else {
                                state.show_login_modal = true;
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                if state.show_create_repo_modal {
                    match key.code {
                        KeyCode::Esc => state.show_create_repo_modal = false,
                        KeyCode::Tab => {
                            state.create_repo_is_private = !state.create_repo_is_private;
                        }
                        KeyCode::Down => {
                            if !state.accounts.is_empty() && state.selected_create_account_index < state.accounts.len() - 1 {
                                state.selected_create_account_index += 1;
                            }
                        }
                        KeyCode::Up => {
                            if state.selected_create_account_index > 0 {
                                state.selected_create_account_index -= 1;
                            }
                        }
                        KeyCode::Backspace => {
                            state.create_repo_name.pop();
                        }
                        KeyCode::Char(c) => {
                            if c != '\t' && c != '\n' && c != '\r' {
                                state.create_repo_name.push(c);
                            }
                        }
                        KeyCode::Enter => {
                            if state.create_repo_name.trim().is_empty() {
                                state.set_notification("Repository name cannot be empty", true);
                            } else if let Some(acc) = state.accounts.get(state.selected_create_account_index).cloned() {
                                let repo_name = state.create_repo_name.trim().to_string();
                                let is_private = state.create_repo_is_private;
                                state.show_create_repo_modal = false;

                                // Fetch token from Keyring
                                let secure_store = KeyringSecureStore::new();
                                if let Ok(Some(token)) = secure_store.get_token(&acc.github_username) {
                                    let config = config_mgr.load_config().unwrap_or_default();
                                    let provider = GitHubProvider::new(&config.github.client_id);
                                    match provider.create_repository(&token, &repo_name, is_private).await {
                                        Ok(ssh_url) => {
                                            state.set_notification(
                                                format!("✓ Created {} repo '{}' on GitHub! URL: {}", if is_private { "private" } else { "public" }, repo_name, ssh_url),
                                                false,
                                            );
                                        }
                                        Err(e) => {
                                            state.set_notification(format!("Failed to create repo: {}", e), true);
                                        }
                                    }
                                } else {
                                    state.set_notification(format!("OAuth token for @{} not found in Keychain. Re-login required.", acc.github_username), true);
                                }
                            } else {
                                state.set_notification("No account selected for creation", true);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                if state.show_clone_repo_modal {
                    match key.code {
                        KeyCode::Esc => state.show_clone_repo_modal = false,
                        KeyCode::Down => {
                            if !state.accounts.is_empty() && state.selected_clone_account_index < state.accounts.len() - 1 {
                                state.selected_clone_account_index += 1;
                            }
                        }
                        KeyCode::Up => {
                            if state.selected_clone_account_index > 0 {
                                state.selected_clone_account_index -= 1;
                            }
                        }
                        KeyCode::Backspace => {
                            state.clone_repo_url.pop();
                        }
                        KeyCode::Char(c) => {
                            if c != '\t' && c != '\n' && c != '\r' {
                                state.clone_repo_url.push(c);
                            }
                        }
                        KeyCode::Enter => {
                            if state.clone_repo_url.trim().is_empty() {
                                state.set_notification("Repository URL cannot be empty", true);
                            } else if let Some(acc) = state.accounts.get(state.selected_clone_account_index).cloned() {
                                 let repo_url = state.clone_repo_url.trim().to_string();
                                state.show_clone_repo_modal = false;

                                let git_service = crate::services::GitService::new();
                                let cwd = env::current_dir().unwrap_or_default();
                                let ssh_service = SshService::new(config_mgr.ssh_dir());
                                let key_path = ssh_service.resolve_key_path(&acc.key_id);

                                let secure_store = KeyringSecureStore::new();
                                let token = secure_store.get_token(&acc.github_username).ok().flatten();

                                state.set_notification(format!("Cloning repository with @{}...", acc.github_username), false);
                                match git_service.clone_repo(&repo_url, &cwd, &key_path, token.as_deref()) {
                                    Ok(cloned_path) => {
                                        // Auto-map cloned project to account
                                        if let Ok(proj) = project_service.map_project(&cloned_path, &acc.id).await {
                                            state.active_project = Some(proj);
                                            if let Ok(projects) = project_service.list_projects().await {
                                                state.projects = projects;
                                            }
                                        }
                                        state.set_notification(
                                            format!("✓ Cloned repo to '{}' bound to @{}!", cloned_path.display(), acc.github_username),
                                            false,
                                        );
                                    }
                                    Err(e) => {
                                        state.set_notification(format!("Clone failed: {}", e), true);
                                    }
                                }
                            } else {
                                state.set_notification("No account selected for cloning", true);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                if state.show_login_modal {
                    match (&state.login_phase, key.code) {
                        (LoginPhase::Ready, KeyCode::Esc) => {
                            state.show_login_modal = false;
                            state.login_phase = LoginPhase::Ready;
                        }
                        (LoginPhase::Ready, KeyCode::Enter) => {
                            // Start the OAuth Device Flow
                            let config = match config_mgr.load_config() {
                                Ok(c) => c,
                                Err(e) => {
                                    state.login_phase = LoginPhase::Error {
                                        message: format!("Config error: {}", e),
                                    };
                                    continue;
                                }
                            };
                            let provider = GitHubProvider::new(&config.github.client_id);

                            // Request device code from GitHub
                            terminal.draw(|f| render_app(f, &state)).ok();
                            match provider.request_device_code().await {
                                Ok(device_res) => {
                                    let user_code = device_res.user_code.clone();
                                    let verification_uri = device_res.verification_uri.clone();

                                    // Copy code to clipboard
                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                        let _ = clipboard.set_text(&user_code);
                                    }

                                    // Open browser
                                    let _ = open::that(&verification_uri);

                                    // Update phase to show code & waiting status
                                    state.login_phase = LoginPhase::WaitingForAuth {
                                        user_code: user_code.clone(),
                                        verification_uri,
                                    };
                                    terminal.draw(|f| render_app(f, &state)).ok();

                                    // Poll for token (this blocks until authorized or error)
                                    state.login_phase = LoginPhase::Polling {
                                        user_code: user_code.clone(),
                                    };
                                    terminal.draw(|f| render_app(f, &state)).ok();

                                    match provider.poll_for_token(&device_res.device_code, device_res.interval).await {
                                        Ok(token) => {
                                            // Fetch user info
                                            match provider.fetch_user_info(&token).await {
                                                Ok(provider_user) => {
                                                    // Generate SSH key
                                                    let ssh_service = SshService::new(config_mgr.ssh_dir());
                                                    let key_id = format!("id_ed25519_{}", provider_user.username);
                                                    let key_path = ssh_service.generate_keypair(
                                                        &key_id,
                                                        &format!("gitnest-{}", provider_user.username),
                                                    );

                                                    // Upload SSH key to GitHub
                                                    if let Ok(ref kp) = key_path {
                                                        let pub_path = format!("{}.pub", kp.to_string_lossy());
                                                        if let Ok(pub_key_str) = std::fs::read_to_string(&pub_path) {
                                                            let _ = provider.upload_ssh_key(
                                                                &token,
                                                                &format!("GitNest Key ({})", provider_user.username),
                                                                &pub_key_str,
                                                            ).await;
                                                        }
                                                    }

                                                    // Store token in keychain
                                                    let secure_store = KeyringSecureStore::new();
                                                    let _ = secure_store.store_token(&provider_user.username, &token);

                                                    // Register account
                                                    let display_name = provider_user.name.unwrap_or_else(|| provider_user.username.clone());
                                                    let account = Account::new(
                                                        display_name,
                                                        provider_user.email.clone(),
                                                        provider_user.username.clone(),
                                                        "github",
                                                        key_id,
                                                    );
                                                    let _ = account_service.add_account(account).await;

                                                    // Refresh accounts list
                                                    if let Ok(updated) = account_service.list_accounts().await {
                                                        state.accounts = updated;
                                                    }

                                                    // Store user profile in Firebase Firestore
                                                    let telemetry = TelemetryService::new();
                                                    telemetry
                                                        .track_user(&provider_user.username, &provider_user.email)
                                                        .await;

                                                    state.login_phase = LoginPhase::Success {
                                                        username: provider_user.username,
                                                    };
                                                }
                                                Err(e) => {
                                                    state.login_phase = LoginPhase::Error {
                                                        message: format!("Failed to fetch user info: {}", e),
                                                    };
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            state.login_phase = LoginPhase::Error {
                                                message: format!("OAuth token poll failed: {}", e),
                                            };
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.login_phase = LoginPhase::Error {
                                        message: format!("Device code request failed: {}", e),
                                    };
                                }
                            }
                        }
                        (LoginPhase::WaitingForAuth { .. } | LoginPhase::Polling { .. }, KeyCode::Esc) => {
                            state.show_login_modal = false;
                            state.login_phase = LoginPhase::Ready;
                        }
                        (LoginPhase::Success { ref username }, KeyCode::Enter | KeyCode::Esc) => {
                            let uname = username.clone();
                            state.show_login_modal = false;
                            state.login_phase = LoginPhase::Ready;
                            state.set_notification(
                                format!("Account @{} registered successfully!", uname),
                                false,
                            );
                        }
                        (LoginPhase::Error { .. }, KeyCode::Esc) => {
                            state.show_login_modal = false;
                            state.login_phase = LoginPhase::Ready;
                        }
                        (LoginPhase::Error { .. }, KeyCode::Enter) => {
                            // Retry
                            state.login_phase = LoginPhase::Ready;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Global Shortcuts
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                }

                if key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    state.command_palette_query.clear();
                    state.command_palette_index = 0;
                    state.current_screen = Screen::CommandPalette;
                    continue;
                }

                // Direct Single-Key Jump Shortcuts (disabled when modal or text input active to avoid conflicts)
                if state.current_screen != Screen::CommandPalette
                    && state.current_screen != Screen::Accounts
                    && !state.show_create_repo_modal
                    && !state.show_clone_repo_modal
                {
                    match key.code {
                        KeyCode::Char('?') => {
                            state.show_help_modal = true;
                            continue;
                        }
                        KeyCode::Char('a') => {
                            state.current_screen = Screen::Accounts;
                            continue;
                        }
                        KeyCode::Char('p') => {
                            state.current_screen = Screen::Projects;
                            continue;
                        }
                        KeyCode::Char('s') => {
                            state.current_screen = Screen::Security;
                            continue;
                        }
                        KeyCode::Char('d') => {
                            state.current_screen = Screen::Doctor;
                            continue;
                        }
                        _ => {}
                    }
                }

                match state.current_screen {
                    Screen::Dashboard => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.menu_index < 9 {
                                state.menu_index += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.menu_index > 0 {
                                state.menu_index -= 1;
                            }
                        }
                        KeyCode::Enter => match state.menu_index {
                            0 => state.show_login_modal = true,
                            1 => {
                                state.selected_connect_account_index = 0;
                                if let Some(ref active) = state.active_account {
                                    if let Some(pos) = state.accounts.iter().position(|a| a.id == active.id) {
                                        state.selected_connect_account_index = pos;
                                    }
                                }
                                state.show_connect_modal = true;
                            },
                            2 => {
                                state.create_repo_name.clear();
                                state.create_repo_is_private = true;
                                state.selected_create_account_index = 0;
                                if let Some(ref active) = state.active_account {
                                    if let Some(pos) = state.accounts.iter().position(|a| a.id == active.id) {
                                        state.selected_create_account_index = pos;
                                    }
                                }
                                state.show_create_repo_modal = true;
                            },
                            3 => {
                                state.clone_repo_url.clear();
                                state.selected_clone_account_index = 0;
                                if let Some(ref active) = state.active_account {
                                    if let Some(pos) = state.accounts.iter().position(|a| a.id == active.id) {
                                        state.selected_clone_account_index = pos;
                                    }
                                }
                                state.show_clone_repo_modal = true;
                            },
                            4 => state.current_screen = Screen::Accounts,
                            5 => state.current_screen = Screen::Projects,
                            6 => state.current_screen = Screen::Security,
                            7 => state.current_screen = Screen::Doctor,
                            8 => state.current_screen = Screen::Settings,
                            9 => break,
                            _ => {}
                        },
                        _ => {}
                    },
                    Screen::Accounts => match key.code {
                        KeyCode::Esc | KeyCode::Char('b') => state.current_screen = Screen::Dashboard,
                        KeyCode::Char('q') => break,
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !state.accounts.is_empty() && state.selected_account_index < state.accounts.len() - 1 {
                                state.selected_account_index += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_account_index > 0 {
                                state.selected_account_index -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(acc) = state.accounts.get(state.selected_account_index) {
                                state.modal_switch_account = Some(acc.clone());
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Delete => {
                            if let Some(acc) = state.accounts.get(state.selected_account_index) {
                                state.modal_remove_account = Some(acc.clone());
                            }
                        }
                        _ => {}
                    },
                    Screen::CommandPalette => match key.code {
                        KeyCode::Esc => state.current_screen = Screen::Dashboard,
                        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Backspace => {
                            state.command_palette_query.pop();
                            state.command_palette_index = 0;
                        }
                        KeyCode::Char(c) => {
                            state.command_palette_query.push(c);
                            state.command_palette_index = 0;
                        }
                        KeyCode::Down => {
                            if state.command_palette_index < 7 {
                                state.command_palette_index += 1;
                            }
                        }
                        KeyCode::Up => {
                            if state.command_palette_index > 0 {
                                state.command_palette_index -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            let idx = state.command_palette_index;
                            state.command_palette_query.clear();
                            state.command_palette_index = 0;
                            match idx {
                                0 => {
                                    state.current_screen = Screen::Dashboard;
                                    state.show_login_modal = true;
                                }
                                1 => state.current_screen = Screen::Projects,
                                2 => state.current_screen = Screen::CreateRepo,
                                3 => state.current_screen = Screen::CloneRepo,
                                4 => state.current_screen = Screen::Accounts,
                                5 => state.current_screen = Screen::Projects,
                                6 => state.current_screen = Screen::Security,
                                7 => state.current_screen = Screen::Doctor,
                                _ => state.current_screen = Screen::Dashboard,
                            }
                        }
                        _ => {}
                    },
                    _ => match key.code {
                        KeyCode::Esc | KeyCode::Char('b') => state.current_screen = Screen::Dashboard,
                        KeyCode::Char('q') => break,
                        _ => {}
                    },
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    // 4. Restore Terminal Normal Mode
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
