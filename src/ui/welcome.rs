//! Welcome / empty-state screen rendered into the chat area.
//!
//! Shown whenever no conversation is active. Three branches:
//! - Connection error: red header + error text + setup hint
//! - Loading: spinner with the current `startup_status` line
//! - First-run vs returning: tailored "getting started" copy plus
//!   the most useful slash commands

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub(super) fn draw_welcome(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut lines = vec![Line::from("")];

    if let Some(ref err) = app.connection_error {
        lines.push(Line::from(Span::styled(
            "  Connection Error",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {err}"),
            Style::default().fg(theme.error),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Run with --setup to reconfigure.",
            Style::default().fg(theme.fg_secondary),
        )));
    } else if app.loading {
        const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let spinner_char = SPINNER[app.spinner_tick % SPINNER.len()];
        lines.push(Line::from(Span::styled(
            "  siggy",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {spinner_char} {}", app.startup_status),
            Style::default().fg(theme.fg_muted),
        )));
    } else if app.store.conversation_order.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Welcome to siggy",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No conversations yet",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(Span::styled(
            "  Messages you send and receive will appear here.",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  /join +1234567890  message someone by phone number",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(Span::styled(
            "  /contacts          browse your synced contacts",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(Span::styled(
            "  /help              see all commands and keybindings",
            Style::default().fg(theme.fg_secondary),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Welcome to siggy",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Getting started",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  Tab / Shift+Tab    cycle through conversations",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(Span::styled(
            "  /join <contact>    open a conversation by name or number",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(Span::styled(
            "  Esc                switch to Normal mode (vim keys)",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Useful commands",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  /contacts          browse synced contacts",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(Span::styled(
            "  /settings          configure preferences",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(Span::styled(
            "  /help              all commands and keybindings",
            Style::default().fg(theme.fg_secondary),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Ctrl+\u{2190}/\u{2192} to resize sidebar",
            Style::default().fg(theme.fg_muted),
        )));
    }

    frame.render_widget(Paragraph::new(lines), area);
}
