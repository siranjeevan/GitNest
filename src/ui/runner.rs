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

    // 3. Event Handling Loop
    loop {
        terminal.draw(|f| render_app(f, &state))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Global Shortcuts
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                }

                if key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    state.current_screen = Screen::CommandPalette;
                    continue;
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
                            0 => state.current_screen = Screen::Accounts,
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
                    Screen::CommandPalette => match key.code {
                        KeyCode::Esc => state.current_screen = Screen::Dashboard,
                        KeyCode::Char('q') => break,
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
