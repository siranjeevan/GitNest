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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginPhase {
    /// Initial prompt: Press Enter to start
    Ready,
    /// Device code generated, browser opening, waiting for user to authorize
    WaitingForAuth {
        user_code: String,
        verification_uri: String,
    },
    /// Polling GitHub for token
    Polling {
        user_code: String,
    },
    /// Successfully authenticated
    Success {
        username: String,
    },
    /// Error occurred
    Error {
        message: String,
    },
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
    pub modal_switch_account: Option<Account>,
    pub modal_remove_account: Option<Account>,
    pub show_help_modal: bool,
    pub show_login_modal: bool,
    pub login_phase: LoginPhase,
    pub spinner_frame: usize,
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
            modal_switch_account: None,
            modal_remove_account: None,
            show_help_modal: false,
            show_login_modal: false,
            login_phase: LoginPhase::Ready,
            spinner_frame: 0,
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
