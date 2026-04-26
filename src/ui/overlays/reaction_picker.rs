//! Quick-reaction picker overlay.
//!
//! Horizontal strip of `QUICK_REACTIONS` emoji with the selected
//! entry bracketed and bolded. Pressing `e` from the picker opens
//! the full emoji search overlay.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::centered_popup;
use crate::app::{App, QUICK_REACTIONS};

pub(in crate::ui) fn draw_reaction_picker(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let emoji_count = QUICK_REACTIONS.len();
    let popup_width = (emoji_count * 4 + 4) as u16;
    let popup_height = 3u16;

    let (popup_area, block) = centered_popup(
        frame,
        area,
        popup_width,
        popup_height,
        " React (e: search all) ",
        theme,
    );

    let mut spans = vec![Span::raw(" ".to_string())];
    for (i, emoji) in QUICK_REACTIONS.iter().enumerate() {
        let style = if i == app.reactions.picker_index {
            Style::default()
                .bg(theme.bg_selected)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if i == app.reactions.picker_index {
            "["
        } else {
            " "
        };
        let suffix = if i == app.reactions.picker_index {
            "]"
        } else {
            " "
        };
        spans.push(Span::styled(format!("{prefix}{emoji}{suffix}"), style));
    }

    let line = Line::from(spans);
    let popup = Paragraph::new(vec![line]).block(block);
    frame.render_widget(popup, popup_area);
}
