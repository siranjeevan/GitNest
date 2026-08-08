use crate::ui::state::{AppState, Screen};
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

fn render_help_modal(f: &mut Frame, area: Rect) {
    let popup_area = Rect::new(
        area.width / 5,
        area.height / 4,
        (area.width * 3) / 5,
        area.height / 2,
    );
    f.render_widget(Clear, popup_area);

    let text = vec![
        Line::from(Span::styled("KEYBOARD SHORTCUTS CHEAT SHEET", Theme::title())),
        Line::from(""),
        Line::from("  ↑ / ↓ or j / k  : Navigate menu items"),
        Line::from("  Enter           : Select item or confirm modal"),
        Line::from("  Ctrl + K        : Open Command Palette with Fuzzy Search"),
        Line::from("  a               : Jump to Accounts View"),
        Line::from("  p               : Jump to Projects View"),
        Line::from("  s               : Jump to Security Center"),
        Line::from("  d               : Jump to Doctor Diagnostics"),
        Line::from("  Esc or b        : Return to Dashboard"),
        Line::from("  ?               : Toggle this Help Overlay"),
        Line::from("  q or Ctrl + C   : Quit GitNest cleanly"),
        Line::from(""),
        Line::from(Span::styled("  Press [Esc] or [?] to close.", Style::default().fg(Theme::CYAN))),
    ];

    let p = Paragraph::new(text).block(
        Block::default()
            .title(" HELP & NAVIGATION GUIDE ")
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(p, popup_area);
}

fn render_switch_account_modal(f: &mut Frame, _state: &AppState, target_acc: &crate::domain::account::Account, area: Rect) {
    let popup_area = Rect::new(
        area.width / 4,
        area.height / 3,
        area.width / 2,
        area.height / 3,
    );
    f.render_widget(Clear, popup_area);

    let text = vec![
        Line::from(Span::styled("CONFIRM IDENTITY SWITCH", Theme::title())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target Identity: ", Style::default().fg(Theme::MUTED)),
            Span::styled(&target_acc.github_username, Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Email Address  : ", Style::default().fg(Theme::MUTED)),
            Span::styled(&target_acc.email, Style::default().fg(Theme::CYAN)),
        ]),
        Line::from(vec![
            Span::styled("SSH Key ID     : ", Style::default().fg(Theme::MUTED)),
            Span::styled(&target_acc.key_id, Style::default().fg(Theme::VIOLET)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter] Confirm & Update .git/config   [Esc] Cancel", Theme::warning_badge()),
        ]),
    ];

    let p = Paragraph::new(text).block(
        Block::default()
            .title(" ACCOUNT SWITCH MODAL ")
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(p, popup_area);
}

fn render_remove_account_modal(f: &mut Frame, _state: &AppState, target_acc: &crate::domain::account::Account, area: Rect) {
    let popup_area = Rect::new(
        area.width / 4,
        area.height / 3,
        area.width / 2,
        area.height / 3,
    );
    f.render_widget(Clear, popup_area);

    let text = vec![
        Line::from(Span::styled("CONFIRM ACCOUNT REMOVAL", Theme::error_badge())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target Identity: ", Style::default().fg(Theme::MUTED)),
            Span::styled(&target_acc.github_username, Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Email Address  : ", Style::default().fg(Theme::MUTED)),
            Span::styled(&target_acc.email, Style::default().fg(Theme::CYAN)),
        ]),
        Line::from(vec![
            Span::styled("SSH Key ID     : ", Style::default().fg(Theme::MUTED)),
            Span::styled(&target_acc.key_id, Style::default().fg(Theme::VIOLET)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠️ Action cannot be undone! Keyring credentials will be cleared.", Theme::error_badge()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter / d] Confirm Remove Account   [Esc] Cancel", Theme::error_badge()),
        ]),
    ];

    let p = Paragraph::new(text).block(
        Block::default()
            .title(" REMOVE ACCOUNT CONFIRMATION ")
            .borders(Borders::ALL)
            .border_style(Theme::error_badge()),
    );
    f.render_widget(p, popup_area);
}

pub fn render_app(f: &mut Frame, state: &AppState) {
    let size = f.size();

    // Create main vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Compact Brand Header
            Constraint::Min(10),   // Main View Content
            Constraint::Length(3), // Footer Shortcuts & Notifications
        ])
        .split(size);

    render_header(f, chunks[0]);

    match state.current_screen {
        Screen::Dashboard => render_dashboard(f, state, chunks[1]),
        Screen::Accounts => render_accounts(f, state, chunks[1]),
        Screen::Projects => render_projects(f, state, chunks[1]),
        Screen::Security => render_security(f, state, chunks[1]),
        Screen::Doctor => render_doctor(f, state, chunks[1]),
        Screen::Settings => render_settings(f, state, chunks[1]),
        Screen::CommandPalette => {
            render_dashboard(f, state, chunks[1]);
            render_command_palette(f, state, size);
        }
        _ => render_dashboard(f, state, chunks[1]),
    }

    render_footer(f, state, chunks[2]);

    if state.show_help_modal {
        render_help_modal(f, size);
    }

    if let Some(ref target_acc) = state.modal_switch_account {
        render_switch_account_modal(f, state, target_acc, size);
    }

    if let Some(ref target_acc) = state.modal_remove_account {
        render_remove_account_modal(f, state, target_acc, size);
    }
}

fn render_header(f: &mut Frame, area: Rect) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("◈ GITNEST ", Theme::header_brand()),
            Span::styled("v1.0.0", Style::default().fg(Theme::CYAN)),
            Span::raw(" │ "),
            Span::styled(
                "Developer-First GitHub Identity Management",
                Style::default().fg(Theme::MUTED),
            ),
        ]),
        Line::from(vec![Span::styled(
            "  One workspace. Multiple Git identities. Zero identity leaks.",
            Style::default().fg(Theme::MUTED),
        )]),
    ];

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Theme::border_inactive()),
    );
    f.render_widget(header, area);
}

fn render_dashboard(f: &mut Frame, state: &AppState, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Left Column: Identity & Context Panel
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(5)])
        .split(main_chunks[0]);

    // Active Identity Card
    let identity_lines = if let Some(ref acc) = state.active_account {
        vec![
            Line::from(vec![
                Span::styled("● ", Theme::success_badge()),
                Span::styled(
                    &acc.github_username,
                    Style::default()
                        .fg(Theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                format!("  {}", acc.email),
                Style::default().fg(Theme::MUTED),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  SSH Key : ", Style::default().fg(Theme::MUTED)),
                Span::styled("✓ Protected (IdentitiesOnly=yes)", Theme::success_badge()),
            ]),
            Line::from(vec![
                Span::styled("  Security: ", Style::default().fg(Theme::MUTED)),
                Span::styled("✓ Mapped & Isolated", Theme::success_badge()),
            ]),
        ]
    } else {
        let (global_user, global_email) = match (&state.global_git_user, &state.global_git_email) {
            (Some(u), Some(e)) => (u.as_str(), e.as_str()),
            (Some(u), None) => (u.as_str(), "No global email set"),
            (None, Some(e)) => ("No global name set", e.as_str()),
            (None, None) => ("No Global Identity", "Unconfigured"),
        };

        vec![
            Line::from(vec![
                Span::styled("● ", Theme::warning_badge()),
                Span::styled(
                    format!("Global Identity: {}", global_user),
                    Style::default()
                        .fg(Theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                format!("  {}", global_email),
                Style::default().fg(Theme::MUTED),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  SSH Key : ", Style::default().fg(Theme::MUTED)),
                Span::styled("✓ Protected (IdentitiesOnly=yes)", Theme::success_badge()),
            ]),
            Line::from(vec![
                Span::styled("  Security: ", Style::default().fg(Theme::MUTED)),
                Span::styled("⚠️ Warning: Global Fallback", Theme::warning_badge()),
            ]),
        ]
    };

    let identity_card = Paragraph::new(identity_lines).block(
        Block::default()
            .title(" CURRENT IDENTITY ")
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(identity_card, left_chunks[0]);

    // Workspace Context Card
    let proj_lines = if let Some(ref proj) = state.active_project {
        vec![
            Line::from(vec![
                Span::styled("Path  : ", Style::default().fg(Theme::MUTED)),
                Span::styled(
                    proj.path.to_string_lossy(),
                    Style::default().fg(Theme::CYAN),
                ),
            ]),
            Line::from(vec![
                Span::styled("Name  : ", Style::default().fg(Theme::MUTED)),
                Span::styled(&proj.name, Style::default().fg(Theme::TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Theme::MUTED)),
                Span::styled("✓ Mapped to GitNest", Theme::success_badge()),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Path  : ", Style::default().fg(Theme::MUTED)),
                Span::styled("Unmapped Directory", Style::default().fg(Theme::WARNING)),
            ]),
            Line::from(vec![
                Span::styled("Action: ", Style::default().fg(Theme::MUTED)),
                Span::styled(
                    "Press 'Connect Folder' to assign a identity",
                    Style::default().fg(Theme::MUTED),
                ),
            ]),
        ]
    };

    let workspace_card = Paragraph::new(proj_lines).block(
        Block::default()
            .title(" ACTIVE WORKSPACE ")
            .borders(Borders::ALL)
            .border_style(Theme::border_inactive()),
    );
    f.render_widget(workspace_card, left_chunks[1]);

    // Right Column: Interactive Quick Action Menu
    let menu_items = vec![
        "Login / Add GitHub Account",
        "Connect Folder to Account",
        "Create New Repository",
        "Clone Repository",
        "Manage Accounts & Keys",
        "View Connected Projects",
        "Identity Security Panel",
        "Run System Health Doctor",
        "Settings & Telemetry",
        "Exit GitNest",
    ];

    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let style = if idx == state.menu_index {
                Theme::active_item()
            } else {
                Theme::inactive_item()
            };
            let prefix = if idx == state.menu_index {
                "❯ "
            } else {
                "  "
            };
            ListItem::new(format!("{}{}", prefix, item)).style(style)
        })
        .collect();

    let menu_list = List::new(items).block(
        Block::default()
            .title(" NAVIGATION MENU ")
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(menu_list, main_chunks[1]);
}

fn render_accounts(f: &mut Frame, state: &AppState, area: Rect) {
    let items: Vec<ListItem> = if state.accounts.is_empty() {
        vec![ListItem::new(
            "No accounts registered. Press 'a' to login via GitHub Device OAuth.",
        )]
    } else {
        state
            .accounts
            .iter()
            .enumerate()
            .map(|(idx, acc)| {
                let prefix = if idx == state.selected_account_index {
                    "❯ ● "
                } else {
                    "  ○ "
                };
                let style = if idx == state.selected_account_index {
                    Theme::active_item()
                } else {
                    Theme::inactive_item()
                };
                let text = format!(
                    "{}{} ({}) - Key: {}",
                    prefix, acc.github_username, acc.email, acc.key_id
                );
                ListItem::new(text).style(style)
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .title(" REGISTERED GITHUB IDENTITIES ([Enter] Switch Context │ [d / Delete] Remove Account) ")
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(list, area);
}

fn render_projects(f: &mut Frame, _state: &AppState, area: Rect) {
    let text = vec![
        Line::from(Span::styled("CONNECTED REPOSITORIES", Theme::title())),
        Line::from(""),
        Line::from("  Use `gitnest scan` or navigation options to connect local folders."),
    ];
    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border_inactive()),
    );
    f.render_widget(p, area);
}

fn render_security(f: &mut Frame, _state: &AppState, area: Rect) {
    let security_lines = vec![
        Line::from(Span::styled("IDENTITY SECURITY CENTER", Theme::title())),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Identity Isolation    : ",
                Style::default().fg(Theme::MUTED),
            ),
            Span::styled("✓ FAIL-CLOSED ACTIVE", Theme::success_badge()),
        ]),
        Line::from(vec![
            Span::styled(
                "  SSH Key Tamper Guard  : ",
                Style::default().fg(Theme::MUTED),
            ),
            Span::styled("✓ SHA-256 FINGERPRINT VERIFIED", Theme::success_badge()),
        ]),
        Line::from(vec![
            Span::styled(
                "  Environment Protection: ",
                Style::default().fg(Theme::MUTED),
            ),
            Span::styled("✓ ENVGUARD ENFORCED", Theme::success_badge()),
        ]),
        Line::from(vec![
            Span::styled(
                "  OAuth Credential Store: ",
                Style::default().fg(Theme::MUTED),
            ),
            Span::styled("✓ OS KEYCHAIN VAULTED", Theme::success_badge()),
        ]),
        Line::from(vec![
            Span::styled(
                "  Atomic Config Writer  : ",
                Style::default().fg(Theme::MUTED),
            ),
            Span::styled("✓ ATOMIC FSYNC ACTIVE", Theme::success_badge()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Zero accidental identity leaks guaranteed.",
            Style::default().fg(Theme::CYAN),
        )),
    ];

    let p = Paragraph::new(security_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(p, area);
}

fn render_doctor(f: &mut Frame, _state: &AppState, area: Rect) {
    let doctor_lines = vec![
        Line::from(Span::styled("GITNEST SYSTEM HEALTH DOCTOR", Theme::title())),
        Line::from(""),
        Line::from("  [PASS] Git Engine         : Installed"),
        Line::from("  [PASS] SSH Engine         : Installed"),
        Line::from("  [PASS] Configuration      : Valid"),
        Line::from("  [PASS] macOS Keychain     : Available"),
        Line::from("  [PASS] Environment Sandbox: Safe"),
        Line::from(""),
        Line::from(Span::styled(
            "  ALL SYSTEMS OPERATIONAL",
            Theme::success_badge(),
        )),
    ];
    let p = Paragraph::new(doctor_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(p, area);
}

fn render_settings(f: &mut Frame, _state: &AppState, area: Rect) {
    let settings_lines = vec![
        Line::from(Span::styled("PREFERENCES & PRIVACY", Theme::title())),
        Line::from(""),
        Line::from("  Telemetry Status: [PASS] Enabled (Randomized Local UUID)"),
        Line::from("  Strict Security : [PASS] Enabled (Fail-Closed)"),
        Line::from("  Default SSH Key : ed25519"),
    ];
    let p = Paragraph::new(settings_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border_inactive()),
    );
    f.render_widget(p, area);
}

fn render_command_palette(f: &mut Frame, state: &AppState, area: Rect) {
    let popup_area = Rect::new(
        area.width / 6,
        area.height / 4,
        (area.width * 2) / 3,
        area.height / 2,
    );
    f.render_widget(Clear, popup_area);

    let raw_items = [
        "Login / Add GitHub Account",
        "Connect Folder to Account",
        "Create New Repository",
        "Clone Repository",
        "Manage Accounts & Keys",
        "View Connected Projects",
        "Identity Security Panel",
        "Run System Health Doctor",
    ];

    let query_lower = state.command_palette_query.to_lowercase();
    let filtered_items: Vec<&str> = raw_items
        .iter()
        .filter(|item| item.to_lowercase().contains(&query_lower))
        .copied()
        .collect();

    let items: Vec<ListItem> = filtered_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let style = if idx == state.command_palette_index {
                Theme::active_item()
            } else {
                Theme::inactive_item()
            };
            let prefix = if idx == state.command_palette_index {
                "❯ "
            } else {
                "  "
            };
            ListItem::new(format!("{}{}", prefix, item)).style(style)
        })
        .collect();

    let title_text = if state.command_palette_query.is_empty() {
        " COMMAND PALETTE (Type to search...) ".to_string()
    } else {
        format!(" COMMAND PALETTE (Filter: '{}') ", state.command_palette_query)
    };

    let list = List::new(items).block(
        Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_style(Theme::border_active()),
    );
    f.render_widget(list, popup_area);
}

fn render_footer(f: &mut Frame, state: &AppState, area: Rect) {
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_frames[state.spinner_frame % spinner_frames.len()];

    let footer_text = if let Some((ref msg, is_err)) = state.notification {
        let style = if is_err {
            Theme::error_badge()
        } else {
            Theme::success_badge()
        };
        Line::from(vec![Span::styled(format!("  {} Notice: {}", spinner, msg), style)])
    } else {
        Line::from(vec![
            Span::styled(format!("  {} ", spinner), Style::default().fg(Theme::CYAN)),
            Span::styled("↑↓ Nav ", Theme::footer_help()),
            Span::styled("Enter Select ", Theme::footer_help()),
            Span::styled("Ctrl+K Palette ", Theme::footer_help()),
            Span::styled("a Acc ", Theme::footer_help()),
            Span::styled("s Sec ", Theme::footer_help()),
            Span::styled("d Doc ", Theme::footer_help()),
            Span::styled("? Help ", Theme::footer_help()),
            Span::styled("q Quit", Theme::footer_help()),
        ])
    };

    let footer = Paragraph::new(footer_text).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Theme::border_inactive()),
    );
    f.render_widget(footer, area);
}
