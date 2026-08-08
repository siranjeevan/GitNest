use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    // Primary Color Palette
    pub const INDIGO: Color = Color::Rgb(99, 102, 241); // #6366F1
    pub const VIOLET: Color = Color::Rgb(139, 92, 246); // #8B5CF6
    pub const CYAN: Color = Color::Rgb(34, 211, 238); // #22D3EE
    pub const SUCCESS: Color = Color::Rgb(34, 197, 94); // #22C55E
    pub const WARNING: Color = Color::Rgb(245, 158, 11); // #F59E0B
    pub const ERROR: Color = Color::Rgb(239, 68, 68); // #EF4444
    pub const TEXT: Color = Color::Rgb(248, 250, 252); // #F8FAFC
    pub const MUTED: Color = Color::Rgb(148, 163, 184); // #94A3B8
    pub const BORDER: Color = Color::Rgb(51, 65, 85); // #334155
    pub const BG: Color = Color::Rgb(11, 15, 25); // #0B0F19

    // Helper Styles
    pub fn title() -> Style {
        Style::default()
            .fg(Self::INDIGO)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_brand() -> Style {
        Style::default()
            .fg(Self::VIOLET)
            .add_modifier(Modifier::BOLD)
    }

    pub fn active_item() -> Style {
        Style::default()
            .fg(Self::TEXT)
            .bg(Color::Rgb(30, 41, 59))
            .add_modifier(Modifier::BOLD)
    }

    pub fn inactive_item() -> Style {
        Style::default().fg(Self::MUTED)
    }

    pub fn border_active() -> Style {
        Style::default().fg(Self::INDIGO)
    }

    pub fn border_inactive() -> Style {
        Style::default().fg(Self::BORDER)
    }

    pub fn success_badge() -> Style {
        Style::default()
            .fg(Self::SUCCESS)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning_badge() -> Style {
        Style::default()
            .fg(Self::WARNING)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error_badge() -> Style {
        Style::default()
            .fg(Self::ERROR)
            .add_modifier(Modifier::BOLD)
    }

    pub fn footer_help() -> Style {
        Style::default().fg(Self::MUTED)
    }
}
