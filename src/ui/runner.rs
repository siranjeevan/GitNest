use crate::config::ConfigManager;
use crate::domain::error::Result;
use crate::services::{AccountService, ProjectService};
use crate::storage::{JsonAccountRepository, JsonProjectRepository};
use crate::ui::app::render_app;
use crate::ui::state::{AppState, Screen};
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

                if state.show_login_modal {
                    match key.code {
                        KeyCode::Esc => state.show_login_modal = false,
                        KeyCode::Enter => {
                            state.show_login_modal = false;
                            state.set_notification(
                                "Run `gitnest auth login` in terminal to execute GitHub OAuth Device login",
                                false,
                            );
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

                // Direct Single-Key Jump Shortcuts (disabled in Palette & Accounts view for key conflicts)
                if state.current_screen != Screen::CommandPalette && state.current_screen != Screen::Accounts {
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
                            1 => state.current_screen = Screen::Projects,
                            2 => state.current_screen = Screen::CreateRepo,
                            3 => state.current_screen = Screen::CloneRepo,
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
