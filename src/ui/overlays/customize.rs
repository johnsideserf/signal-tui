//! Customize sub-overlay: Theme / Keybindings / Settings profile.
//!
//! Three-row launcher that picks which secondary overlay to open.
//! Reachable from the Settings overlay's "Customize..." row.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::centered_popup;
use crate::app::App;

pub(in crate::ui) fn draw_customize(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let items = ["Theme", "Keybindings", "Settings profile"];
    let (popup_area, block) = centered_popup(frame, area, 30, 5, " Customize ", theme);

    let mut lines: Vec<Line> = Vec::new();
    for (i, label) in items.iter().enumerate() {
        let is_selected = i == app.customize_index;
        let style = if is_selected {
            Style::default()
                .bg(theme.bg_selected)
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_secondary)
        };
        lines.push(Line::from(Span::styled(format!("  {label}"), style)));
    }

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}
