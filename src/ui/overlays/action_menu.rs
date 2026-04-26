//! Per-message action menu overlay.
//!
//! Lists the actions available on the focused message (reply, react,
//! edit, delete, copy, forward, etc.) with a highlighted cursor row,
//! Nerd Font icons when enabled, and right-aligned key hints.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::centered_popup;
use crate::app::App;

pub(in crate::ui) fn draw_action_menu(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let items = app.action_menu_items();
    if items.is_empty() {
        return;
    }

    let popup_width: u16 = 30;
    let popup_height = items.len() as u16 + 4;

    let (popup_area, block) =
        centered_popup(frame, area, popup_width, popup_height, " Actions ", theme);

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let content_width = inner.width as usize;

    let mut lines: Vec<Line> = Vec::new();
    for (i, action) in items.iter().enumerate() {
        let is_selected = i == app.action_menu.index;
        let icon = if app.nerd_fonts {
            format!("{} ", action.nerd_icon)
        } else {
            String::new()
        };

        let label_part = format!("  {icon}{}", action.label);
        let hint_width = action.key_hint.len();
        let pad = content_width.saturating_sub(label_part.chars().count() + hint_width + 2);
        let padding = " ".repeat(pad);

        let row_style = if is_selected {
            Style::default().bg(theme.bg_selected)
        } else {
            Style::default()
        };
        let hint_style = if is_selected {
            Style::default()
                .bg(theme.bg_selected)
                .fg(theme.fg_muted)
                .add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(theme.fg_muted)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{label_part}{padding}"), row_style),
            Span::styled(format!("{} ", action.key_hint), hint_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Esc to close",
        Style::default().fg(theme.fg_muted),
    )));

    let popup = Paragraph::new(lines);
    frame.render_widget(popup, inner);
}
