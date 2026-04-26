//! Autocomplete popup rendered above the composer.
//!
//! Three modes (`AutocompleteMode`): slash-command listing,
//! `@mention` member picker, and `/join` recipient picker. The popup
//! is sized to the longest candidate and clamped to the available
//! terminal area; it floats above `input_area` and is cleared first
//! so the chat content underneath does not bleed through.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::{App, AutocompleteMode};
use crate::input::COMMANDS;

pub(super) fn draw_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let theme = &app.theme;
    let terminal_width = frame.area().width;
    let mut lines: Vec<Line> = Vec::new();
    let mut max_content_width: usize = 0;

    match app.autocomplete.mode {
        AutocompleteMode::Command => {
            for (i, &cmd_idx) in app.autocomplete.command_candidates.iter().enumerate() {
                let cmd = &COMMANDS[cmd_idx];
                let args_part = if cmd.args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", cmd.args)
                };
                let left = format!("  {}{}", cmd.name, args_part);
                let right = format!("  {}", cmd.description);
                let total_len = left.len() + right.len() + 2;
                if total_len > max_content_width {
                    max_content_width = total_len;
                }

                let is_selected = i == app.autocomplete.index;
                let style = if is_selected {
                    Style::default()
                        .bg(theme.bg_selected)
                        .fg(theme.fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg_secondary)
                };
                let desc_style = if is_selected {
                    Style::default().bg(theme.bg_selected).fg(theme.accent)
                } else {
                    Style::default().fg(theme.fg_muted)
                };

                lines.push(Line::from(vec![
                    Span::styled(left, style),
                    Span::styled(right, desc_style),
                ]));
            }
        }
        AutocompleteMode::Mention => {
            for (i, (phone, name, _uuid)) in app.autocomplete.mention_candidates.iter().enumerate()
            {
                let left = format!("  @{name}");
                let right = format!("  {phone}");
                let total_len = left.len() + right.len() + 2;
                if total_len > max_content_width {
                    max_content_width = total_len;
                }

                let is_selected = i == app.autocomplete.index;
                let style = if is_selected {
                    Style::default()
                        .bg(theme.bg_selected)
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.accent)
                };
                let phone_style = if is_selected {
                    Style::default().bg(theme.bg_selected).fg(theme.fg_muted)
                } else {
                    Style::default().fg(theme.fg_muted)
                };

                lines.push(Line::from(vec![
                    Span::styled(left, style),
                    Span::styled(right, phone_style),
                ]));
            }
        }
        AutocompleteMode::Join => {
            for (i, (display, _value)) in app.autocomplete.join_candidates.iter().enumerate() {
                let left = format!("  {display}");
                let total_len = left.len() + 2;
                if total_len > max_content_width {
                    max_content_width = total_len;
                }

                let is_selected = i == app.autocomplete.index;
                let style = if is_selected {
                    Style::default()
                        .bg(theme.bg_selected)
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.success)
                };

                lines.push(Line::from(vec![Span::styled(left, style)]));
            }
        }
    }

    let count = lines.len();

    // Size the popup, clamping to available space
    let terminal_height = frame.area().height;
    let popup_width = (max_content_width as u16 + 2)
        .min(terminal_width.saturating_sub(2))
        .max(20);
    let popup_height = ((count as u16) + 2).min(input_area.y).min(terminal_height); // +2 for border
    if popup_height < 3 {
        return; // not enough space to render anything useful
    }

    // Position above the input box, left-aligned with it
    let x = input_area.x;
    let y = input_area.y.saturating_sub(popup_height);

    let area = Rect::new(
        x,
        y,
        popup_width.min(terminal_width.saturating_sub(x)),
        popup_height,
    );
    lines.truncate((popup_height.saturating_sub(2)) as usize);

    // Clear the area behind the popup so chat text doesn't leak through
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg));

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, area);
}
