use crate::domain::account::Account;
use crate::domain::project::Project;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Accounts,
    CreateRepo,
    CloneRepo,
    Projects,
    Security,
    Doctor,
    Settings,
    CommandPalette,
}

pub struct AppState {
    pub current_screen: Screen,
    pub menu_index: usize,
    pub selected_account_index: usize,
    pub selected_project_index: usize,
    pub command_palette_query: String,
    pub command_palette_index: usize,
    pub accounts: Vec<Account>,
    pub active_project: Option<Project>,
    pub active_account: Option<Account>,
    pub global_git_user: Option<String>,
    pub global_git_email: Option<String>,
    pub notification: Option<(String, bool)>, // (message, is_error)
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::Dashboard,
            menu_index: 0,
            selected_account_index: 0,
            selected_project_index: 0,
            command_palette_query: String::new(),
            command_palette_index: 0,
            accounts: Vec::new(),
            active_project: None,
            active_account: None,
            global_git_user: None,
            global_git_email: None,
            notification: None,
            should_quit: false,
        }
    }

    pub fn set_notification(&mut self, msg: impl Into<String>, is_error: bool) {
        self.notification = Some((msg.into(), is_error));
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
