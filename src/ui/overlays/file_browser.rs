//! File browser overlay for picking attachments.
//!
//! Shows the current directory path at the top, the list of entries
//! (directories first, type-to-filter narrowing), and a size column
//! for files. Sets the title to the active filter when one is set.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::{FILE_BROWSER_MAX_VISIBLE, FILE_BROWSER_POPUP_WIDTH, centered_popup, truncate};
use crate::app::App;

pub(in crate::ui) fn draw_file_browser(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let visible_count = FILE_BROWSER_MAX_VISIBLE.min(if app.file_picker.filtered.is_empty() {
        1
    } else {
        app.file_picker.filtered.len()
    });
    let pref_height = visible_count as u16 + 5; // border + header + footer

    let title = if app.file_picker.filter.is_empty() {
        " Attach File ".to_string()
    } else {
        format!(" Attach File [{}] ", app.file_picker.filter)
    };

    let (popup_area, block) = centered_popup(
        frame,
        area,
        FILE_BROWSER_POPUP_WIDTH,
        pref_height,
        &title,
        theme,
    );

    let inner_height = popup_area.height.saturating_sub(2) as usize;
    let header_lines = 1; // path header
    let footer_lines = 2; // empty + key hints
    let visible_rows = inner_height.saturating_sub(header_lines + footer_lines);
    let inner_w = popup_area.width.saturating_sub(2) as usize;

    let mut lines: Vec<Line> = Vec::new();

    // Current path header
    let dir_display = app.file_picker.dir.to_string_lossy();
    let dir_truncated = truncate(&dir_display, inner_w.saturating_sub(2));
    lines.push(Line::from(Span::styled(
        format!("  {dir_truncated}"),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));

    if let Some(ref err) = app.file_picker.error {
        lines.push(Line::from(Span::styled(
            format!("  {}", truncate(err, inner_w.saturating_sub(2))),
            Style::default().fg(theme.error),
        )));
    } else if app.file_picker.filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Empty directory",
            Style::default().fg(theme.fg_muted),
        )));
    } else {
        // Scroll the list so the selected item is always visible
        let scroll_offset = if app.file_picker.index >= visible_rows {
            app.file_picker.index - visible_rows + 1
        } else {
            0
        };

        let end = (scroll_offset + visible_rows).min(app.file_picker.filtered.len());

        for (i, &entry_idx) in app.file_picker.filtered[scroll_offset..end]
            .iter()
            .enumerate()
        {
            let actual_index = scroll_offset + i;
            let is_selected = actual_index == app.file_picker.index;
            let (ref name, is_dir, size) = app.file_picker.entries[entry_idx];

            let size_str = if is_dir {
                String::new()
            } else {
                format_file_size(size)
            };

            let display_name = if is_dir {
                format!("{name}/")
            } else {
                name.clone()
            };

            // Leave room for size column
            let size_col_width = 8;
            let name_max = inner_w.saturating_sub(size_col_width + 4);
            let display_name = truncate(&display_name, name_max);

            let name_style = if is_selected {
                if is_dir {
                    Style::default()
                        .bg(theme.bg_selected)
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .bg(theme.bg_selected)
                        .fg(theme.fg)
                        .add_modifier(Modifier::BOLD)
                }
            } else if is_dir {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.fg)
            };

            let size_style = if is_selected {
                Style::default().bg(theme.bg_selected).fg(theme.fg_muted)
            } else {
                Style::default().fg(theme.fg_muted)
            };

            // Pad name to align size column
            let name_padded = format!("  {display_name:width$}", width = name_max);
            let size_padded = format!("{size_str:>width$}  ", width = size_col_width);

            lines.push(Line::from(vec![
                Span::styled(name_padded, name_style),
                Span::styled(size_padded, size_style),
            ]));
        }
    }

    // Pad to fill visible rows
    while lines.len() < header_lines + visible_rows {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  j/k nav  Enter open/select  Backspace/- up  Esc cancel",
        Style::default().fg(theme.fg_muted),
    )));

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}

/// Format a file size in human-readable form (B, K, M, G).
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{}K", bytes / 1024)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, "0B")]
    #[case(512, "512B")]
    #[case(1024, "1K")]
    #[case(1536, "1K")]
    #[case(1024 * 1024, "1.0M")]
    #[case(1024 * 1024 * 1024, "1.0G")]
    fn format_file_size_cases(#[case] bytes: u64, #[case] expected: &str) {
        assert_eq!(format_file_size(bytes), expected);
    }
}
