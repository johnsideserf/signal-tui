//! Message-request overlay for unaccepted conversations.
//!
//! Shown when the active conversation has not yet been accepted.
//! Displays the requester's name, phone, and message count, plus
//! the `(a)ccept / (d)elete` choice.

use ratatui::{
    Frame,
    layout::{Alignment, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::centered_popup;
use crate::app::App;

pub(in crate::ui) fn draw_message_request(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let conv_id = match app.active_conversation.as_ref() {
        Some(id) => id,
        None => return,
    };
    let conv = match app.store.conversations.get(conv_id) {
        Some(c) => c,
        None => return,
    };

    let msg_count = conv.messages.len();
    let name = &conv.name;
    let phone = &conv.id;

    let (popup_area, block) = centered_popup(frame, area, 36, 9, " Message Request ", theme);
    frame.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });
    let lines = vec![
        Line::from(Span::styled(
            name.as_str(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            phone.as_str(),
            Style::default().fg(theme.fg_muted),
        )),
        Line::from(Span::styled(
            format!(
                "{} message{}",
                msg_count,
                if msg_count == 1 { "" } else { "s" }
            ),
            Style::default().fg(theme.fg_secondary),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "(a)",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("ccept / ", Style::default().fg(theme.fg_secondary)),
            Span::styled(
                "(d)",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("elete", Style::default().fg(theme.fg_secondary)),
        ]),
        Line::from(Span::styled(
            "Esc to go back",
            Style::default().fg(theme.fg_muted),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
