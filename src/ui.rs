use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

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

    if app.sidebar_visible {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // sidebar
                Constraint::Min(30),   // chat area
            ])
            .split(body_area);

        draw_sidebar(frame, app, horizontal[0]);
        draw_chat_area(frame, app, horizontal[1]);
    } else {
        draw_chat_area(frame, app, body_area);
    }

    draw_status_bar(frame, app, status_area);
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
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

            let prefix = if conv.is_group { "#" } else { " " };
            let unread = if conv.unread > 0 {
                format!(" ({})", conv.unread)
            } else {
                String::new()
            };

            let marker = if is_active { "> " } else { "  " };
            let label = format!("{marker}{prefix}{}{unread}", conv.name);

            let style = if is_active {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if conv.unread > 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let sidebar = List::new(items).block(
        Block::default()
            .borders(Borders::RIGHT)
            .title(" Channels ")
            .title_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(sidebar, area);
}

fn draw_chat_area(frame: &mut Frame, app: &App, area: Rect) {
    // Split into messages + input
    let chat_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // messages
            Constraint::Length(3), // input
        ])
        .split(area);

    let messages_area = chat_layout[0];
    let input_area = chat_layout[1];

    draw_messages(frame, app, messages_area);
    draw_input(frame, app, input_area);
}

fn draw_messages(frame: &mut Frame, app: &App, area: Rect) {
    let title = match &app.active_conversation {
        Some(id) => {
            let conv = &app.conversations[id];
            let prefix = if conv.is_group { " #" } else { " " };
            format!("{prefix}{} ", conv.name)
        }
        None => " signal-tui ".to_string(),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan));

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
            // Show welcome text
            let welcome = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Welcome to signal-tui",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("  Use /join <contact> to start a conversation"),
                Line::from("  Use /help for all commands"),
            ])
            .style(Style::default().fg(Color::Gray));
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

    let lines: Vec<Line> = messages[start..end]
        .iter()
        .map(|msg| {
            if msg.is_system {
                Line::from(Span::styled(
                    format!("  {}", msg.body),
                    Style::default().fg(Color::DarkGray),
                ))
            } else {
                let time = msg.format_time();
                Line::from(vec![
                    Span::styled(
                        format!("[{time}] "),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("<{}>", msg.sender),
                        Style::default()
                            .fg(if msg.sender == "you" {
                                Color::Green
                            } else {
                                Color::Cyan
                            })
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" {}", msg.body)),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let input_text = format!("> {}", app.input_buffer);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(block);

    frame.render_widget(input, area);

    // Place cursor
    let cursor_x = area.x + 3 + app.input_cursor as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {}", app.status_message))
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray),
        );
    frame.render_widget(status, area);
}
