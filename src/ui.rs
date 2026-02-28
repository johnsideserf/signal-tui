use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InputMode};

/// Hash a sender name to one of ~8 distinct colors. "you" always gets Green.
fn sender_color(name: &str) -> Color {
    if name == "you" {
        return Color::Green;
    }
    let hash: u32 = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    const COLORS: [Color; 8] = [
        Color::Cyan,
        Color::Magenta,
        Color::Yellow,
        Color::Blue,
        Color::LightRed,
        Color::LightGreen,
        Color::LightCyan,
        Color::LightMagenta,
    ];
    COLORS[(hash as usize) % COLORS.len()]
}

/// Truncate a string to fit within `max_width`, appending `…` if truncated.
fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width <= 1 {
        "…".to_string()
    } else {
        let mut truncated: String = s.chars().take(max_width - 1).collect();
        truncated.push('…');
        truncated
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let terminal_width = size.width;

    // Main vertical layout: body + status bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // body
            Constraint::Length(1), // status bar
        ])
        .split(size);

    let body_area = outer[0];
    let status_area = outer[1];

    // Narrow terminal adaptation: auto-hide sidebar when width < 60
    let sidebar_auto_hidden = terminal_width < 60;
    let show_sidebar = app.sidebar_visible && !sidebar_auto_hidden;

    if show_sidebar {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(app.sidebar_width),
                Constraint::Min(30),
            ])
            .split(body_area);

        draw_sidebar(frame, app, horizontal[0]);
        draw_chat_area(frame, app, horizontal[1]);
    } else {
        draw_chat_area(frame, app, body_area);
    }

    draw_status_bar(frame, app, status_area, sidebar_auto_hidden);
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let max_name_width = (area.width as usize).saturating_sub(5); // "• # " + margin

    let items: Vec<ListItem> = app
        .conversation_order
        .iter()
        .map(|id| {
            let conv = &app.conversations[id];
            let is_active = app
                .active_conversation
                .as_ref()
                .map(|a| a == id)
                .unwrap_or(false);

            let has_unread = conv.unread > 0;
            let name = truncate(&conv.name, max_name_width);

            let mut spans = Vec::new();

            // Active marker or padding
            if is_active {
                spans.push(Span::styled(
                    "▸ ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw("  "));
            }

            // Unread dot
            if has_unread && !is_active {
                spans.push(Span::styled("• ", Style::default().fg(Color::Yellow)));
            } else {
                spans.push(Span::raw("  "));
            }

            // Group prefix (dimmed #)
            if conv.is_group {
                spans.push(Span::styled(
                    "#",
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Conversation name
            let name_style = if is_active {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if has_unread {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            spans.push(Span::styled(name, name_style));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let sidebar = List::new(items).block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_type(BorderType::Rounded)
            .title(" Chats ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    );

    frame.render_widget(sidebar, area);
}

fn draw_chat_area(frame: &mut Frame, app: &App, area: Rect) {
    // Check if there's an active typing indicator for current conversation
    let has_typing = app.active_conversation.as_ref().map_or(false, |conv_id| {
        app.typing_indicators.keys().any(|sender| {
            // For 1:1 conversations, sender == conv_id
            // For groups, we'd need a mapping, but for now show any active indicator
            sender == conv_id || app.conversations.get(conv_id).map_or(false, |c| c.is_group)
        })
    });

    let typing_height = if has_typing { 1 } else { 0 };

    let chat_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),              // messages
            Constraint::Length(typing_height), // typing indicator
            Constraint::Length(3),            // input
        ])
        .split(area);

    let messages_area = chat_layout[0];
    let typing_area = chat_layout[1];
    let input_area = chat_layout[2];

    draw_messages(frame, app, messages_area);

    if has_typing {
        draw_typing_indicator(frame, app, typing_area);
    }

    draw_input(frame, app, input_area);
}

fn draw_messages(frame: &mut Frame, app: &App, area: Rect) {
    let (title_left, title_right) = match &app.active_conversation {
        Some(id) => {
            let conv = &app.conversations[id];
            let prefix = if conv.is_group { " #" } else { " " };
            let left = format!("{prefix}{} ", conv.name);

            // Scroll indicator in title
            let right = if app.scroll_offset > 0 {
                format!(" ↑ {} more ", app.scroll_offset)
            } else {
                String::new()
            };
            (left, right)
        }
        None => (" signal-tui ".to_string(), String::new()),
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title_left)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    if !title_right.is_empty() {
        block = block
            .title_bottom(Line::from(title_right).alignment(Alignment::Right))
            .title_style(Style::default().fg(Color::Cyan));
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let messages = match &app.active_conversation {
        Some(id) => {
            if let Some(conv) = app.conversations.get(id) {
                &conv.messages
            } else {
                return;
            }
        }
        None => {
            let mut lines = vec![
                Line::from(""),
            ];

            // Show connection error prominently if present
            if let Some(ref err) = app.connection_error {
                lines.push(Line::from(Span::styled(
                    "  Connection Error",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  {err}"),
                    Style::default().fg(Color::Red),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Run with --setup to reconfigure.",
                    Style::default().fg(Color::Gray),
                )));
            } else if app.conversation_order.is_empty() {
                // No conversations yet
                lines.push(Line::from(Span::styled(
                    "  Welcome to signal-tui",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  No conversations yet",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(Span::styled(
                    "  Messages you send and receive will appear here.",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Use /join +1234567890 to message someone",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(Span::styled(
                    "  Use /help for all commands",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                // Has conversations but none selected
                lines.push(Line::from(Span::styled(
                    "  Welcome to signal-tui",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Use /join <contact> to start a conversation",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(Span::styled(
                    "  Use /help for all commands",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Ctrl+←/→ to resize sidebar",
                    Style::default().fg(Color::DarkGray),
                )));
            }

            let welcome = Paragraph::new(lines);
            frame.render_widget(welcome, inner);
            return;
        }
    };

    let available_height = inner.height as usize;
    let total = messages.len();

    // Calculate visible window
    let end = if app.scroll_offset >= total {
        0
    } else {
        total - app.scroll_offset
    };
    let start = end.saturating_sub(available_height);
    let visible = &messages[start..end];

    // Compute max sender name width for alignment
    let max_sender_width = visible
        .iter()
        .filter(|m| !m.is_system)
        .map(|m| m.sender.len())
        .max()
        .unwrap_or(0);

    // Get last_read_index for unread marker
    let conv_id = app.active_conversation.as_ref().unwrap();
    let last_read = app.last_read_index.get(conv_id).copied().unwrap_or(0);

    let inner_width = inner.width as usize;

    let mut lines: Vec<Line> = Vec::new();
    let mut prev_date: Option<String> = None;

    for (i, msg) in visible.iter().enumerate() {
        let msg_index = start + i;

        // Date separator: detect day boundary
        let local = msg.timestamp.with_timezone(&chrono::Local);
        let date_str = local.format("%b %d, %Y").to_string();
        if prev_date.as_ref() != Some(&date_str) {
            if prev_date.is_some() {
                // Insert date separator line
                let label = format!(" {} ", date_str);
                let pad_total = inner_width.saturating_sub(label.len());
                let pad_left = pad_total / 2;
                let pad_right = pad_total - pad_left;
                let sep = format!(
                    "{}{}{}",
                    "─".repeat(pad_left),
                    label,
                    "─".repeat(pad_right)
                );
                lines.push(Line::from(Span::styled(
                    sep,
                    Style::default().fg(Color::DarkGray),
                )));
            }
            prev_date = Some(date_str);
        }

        // Unread marker: between last_read - 1 and last_read
        if msg_index == last_read && last_read > 0 && last_read < total {
            let label = " new messages ";
            let pad_total = inner_width.saturating_sub(label.len());
            let pad_left = pad_total / 2;
            let pad_right = pad_total - pad_left;
            let sep = format!(
                "{}{}{}",
                "─".repeat(pad_left),
                label,
                "─".repeat(pad_right)
            );
            lines.push(Line::from(Span::styled(
                sep,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        if msg.is_system {
            lines.push(Line::from(Span::styled(
                format!("  {}", msg.body),
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            let time = msg.format_time();
            let sender_padded = format!("{:>width$}", msg.sender, width = max_sender_width);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}] ", time),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("<{}>", sender_padded),
                    Style::default()
                        .fg(sender_color(&msg.sender))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {}", msg.body)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn draw_typing_indicator(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref conv_id) = app.active_conversation {
        // Collect names of people typing in this conversation
        let typers: Vec<String> = app
            .typing_indicators
            .keys()
            .filter(|sender| {
                *sender == conv_id
                    || app
                        .conversations
                        .get(conv_id)
                        .map_or(false, |c| c.is_group)
            })
            .map(|s| {
                // Try to get a short display name
                if let Some(conv) = app.conversations.get(s) {
                    conv.name.clone()
                } else {
                    s.clone()
                }
            })
            .collect();

        if !typers.is_empty() {
            let text = if typers.len() == 1 {
                format!("  {} is typing...", typers[0])
            } else {
                format!("  {} are typing...", typers.join(", "))
            };
            let indicator = Paragraph::new(Span::styled(
                text,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ));
            frame.render_widget(indicator, area);
        }
    }
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let border_color = match app.mode {
        InputMode::Insert => Color::Cyan,
        InputMode::Normal => Color::Yellow,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    if app.input_buffer.is_empty() {
        let placeholder = match app.mode {
            InputMode::Normal => "  Press i to type, / for commands",
            InputMode::Insert => "  Type a message...",
        };
        let input = Paragraph::new(Span::styled(
            placeholder,
            Style::default().fg(Color::DarkGray),
        ))
        .block(block);
        frame.render_widget(input, area);
    } else {
        let input_text = format!("> {}", app.input_buffer);
        let input = Paragraph::new(input_text)
            .style(Style::default().fg(Color::White))
            .block(block);
        frame.render_widget(input, area);
    }

    // Place cursor (only visible in Insert mode)
    if app.mode == InputMode::Insert {
        let cursor_x = area.x + 3 + app.input_cursor as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect, sidebar_auto_hidden: bool) {
    let mut segments: Vec<Span> = Vec::new();

    // Mode indicator
    match app.mode {
        InputMode::Normal => {
            segments.push(Span::styled(
                " [NORMAL] ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ));
        }
        InputMode::Insert => {
            segments.push(Span::styled(
                " [INSERT] ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ));
        }
    }
    segments.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));

    // Connection status dot
    if let Some(ref err) = app.connection_error {
        segments.push(Span::styled(" ● ", Style::default().fg(Color::Red)));
        let display: String = err.chars().take(30).collect();
        segments.push(Span::styled(
            format!("error: {display}"),
            Style::default().fg(Color::Red),
        ));
    } else if app.connected {
        segments.push(Span::styled(" ● ", Style::default().fg(Color::Green)));
        segments.push(Span::styled("connected", Style::default().fg(Color::White)));
    } else {
        segments.push(Span::styled(" ● ", Style::default().fg(Color::Red)));
        segments.push(Span::styled("disconnected", Style::default().fg(Color::White)));
    }

    // Pipe separator
    segments.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));

    // Current conversation
    if let Some(ref id) = app.active_conversation {
        if let Some(conv) = app.conversations.get(id) {
            let prefix = if conv.is_group { "#" } else { "" };
            segments.push(Span::styled(
                format!("{}{}", prefix, conv.name),
                Style::default().fg(Color::Cyan),
            ));
        }
    } else {
        segments.push(Span::styled(
            "no conversation",
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Pipe separator + conversation count
    if !app.conversation_order.is_empty() {
        segments.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        segments.push(Span::styled(
            format!("{} chats", app.conversation_order.len()),
            Style::default().fg(Color::Gray),
        ));
    }

    // Scroll offset indicator
    if app.scroll_offset > 0 {
        segments.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        segments.push(Span::styled(
            format!("↑{}", app.scroll_offset),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Auto-hidden sidebar indicator
    if sidebar_auto_hidden && app.sidebar_visible {
        segments.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        segments.push(Span::styled(
            "[+]",
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Pad the rest with background
    let status = Paragraph::new(Line::from(segments)).style(
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray),
    );
    frame.render_widget(status, area);
}
