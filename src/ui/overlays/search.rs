//! Message search overlay + match-highlighting helpers.
//!
//! Drives the `/search <query>` overlay: shows up to `SEARCH_MAX_VISIBLE`
//! results with `[conv]` prefix when searching across all conversations,
//! truncated sender, and a body snippet centered around the first match.
//! `n`/`N` cycles between results inside the overlay.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::{SEARCH_MAX_VISIBLE, SEARCH_POPUP_WIDTH, centered_popup, truncate};
use crate::app::App;
use crate::theme::Theme;

pub(in crate::ui) fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let max_visible = SEARCH_MAX_VISIBLE.min(app.search.results.len().max(1));
    let pref_height = max_visible as u16 + 5; // +3 border/title +2 footer

    let title = if app.search.query.is_empty() {
        " Search ".to_string()
    } else {
        format!(" Search [{}] ", app.search.query)
    };

    let (popup_area, block) =
        centered_popup(frame, area, SEARCH_POPUP_WIDTH, pref_height, &title, theme);

    let inner_height = popup_area.height.saturating_sub(2) as usize; // minus borders
    let footer_lines = 2; // footer + empty line
    let visible_rows = inner_height.saturating_sub(footer_lines);

    // Scroll the list so the selected item is always visible
    let scroll_offset = if app.search.index >= visible_rows {
        app.search.index - visible_rows + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    let inner_w = popup_area.width.saturating_sub(2) as usize;

    if app.search.results.is_empty() {
        let msg = if app.search.query.is_empty() {
            "  Type to search..."
        } else {
            "  No results found"
        };
        lines.push(Line::from(Span::styled(
            msg,
            Style::default().fg(theme.fg_muted),
        )));
    } else {
        let end = (scroll_offset + visible_rows).min(app.search.results.len());

        for (i, result) in app.search.results[scroll_offset..end].iter().enumerate() {
            let actual_index = scroll_offset + i;
            let is_selected = actual_index == app.search.index;

            // Format: [conv_name] sender: body_snippet
            let conv_prefix = if app.active_conversation.is_some() {
                String::new()
            } else {
                format!("[{}] ", truncate(&result.conv_name, 12))
            };

            let sender_display = truncate(&result.sender, 10);
            let prefix = format!("  {conv_prefix}{sender_display}: ");
            let body_max = inner_w.saturating_sub(prefix.len());
            // Show a snippet of the body around the match
            let body_snippet = search_snippet(&result.body, &app.search.query, body_max);

            let prefix_style = if is_selected {
                Style::default().bg(theme.bg_selected).fg(theme.accent)
            } else {
                Style::default().fg(theme.accent)
            };
            let body_style = if is_selected {
                Style::default().bg(theme.bg_selected).fg(theme.fg)
            } else {
                Style::default().fg(theme.fg_secondary)
            };

            // Build spans with highlighted match
            let mut spans = vec![Span::styled(prefix, prefix_style)];
            spans.extend(highlight_match_spans(
                &body_snippet,
                &app.search.query,
                body_style,
                is_selected,
                theme,
            ));

            lines.push(Line::from(spans));
        }
    }

    // Pad to fill visible_rows so footer is always at the bottom
    while lines.len() < visible_rows {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    let count_text = if app.search.results.is_empty() {
        String::new()
    } else {
        format!("  {}/{}", app.search.index + 1, app.search.results.len())
    };
    lines.push(Line::from(vec![
        Span::styled(count_text, Style::default().fg(theme.warning)),
        Span::styled(
            "  j/k nav | Enter jump | n/N cycle | Esc close",
            Style::default().fg(theme.fg_muted),
        ),
    ]));

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}

/// Extract a snippet of text centered around the first match of `query`.
pub(in crate::ui) fn search_snippet(body: &str, query: &str, max_len: usize) -> String {
    // Search results display on a single line; collapse newlines.
    let body = body.replace('\n', " ");
    let body = body.as_str();
    let char_count = body.chars().count();
    if char_count <= max_len {
        return body.to_string();
    }

    // Find the match position in char indices (case-insensitive)
    let body_lower = body.to_lowercase();
    let query_lower = query.to_lowercase();
    let match_byte_pos = body_lower.find(&query_lower).unwrap_or(0);
    // Convert byte position in lowered text to char index
    let match_char_pos = body_lower[..match_byte_pos].chars().count();

    // Center the snippet around the match
    let half = max_len / 2;
    let start = match_char_pos.saturating_sub(half);
    let end = (start + max_len).min(char_count);
    let start = if end == char_count {
        end.saturating_sub(max_len)
    } else {
        start
    };

    let snippet: String = body.chars().skip(start).take(end - start).collect();
    let mut result = snippet;
    if start > 0 {
        result = format!("…{}", result.chars().skip(1).collect::<String>());
    }
    if end < char_count {
        let trimmed: String = result
            .chars()
            .take(result.chars().count().saturating_sub(1))
            .collect();
        result = format!("{trimmed}…");
    }
    result
}

/// Build spans with the matching portions highlighted.
/// Uses character-level case-insensitive matching to avoid byte-boundary issues.
fn highlight_match_spans<'a>(
    text: &str,
    query: &str,
    base_style: Style,
    is_selected: bool,
    theme: &Theme,
) -> Vec<Span<'a>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    let match_style = if is_selected {
        Style::default()
            .bg(theme.bg_selected)
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD)
    };

    // Find all match positions using the lowercased strings
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    let query_len = query_lower.len();

    // Collect match byte ranges in the lowered text, then map back to original
    // For ASCII and most Unicode, to_lowercase preserves byte length per-character.
    // Use the lowered text offsets directly since they share the same structure.
    let mut match_ranges: Vec<(usize, usize)> = Vec::new();
    let mut search_pos = 0;
    while search_pos < text_lower.len() {
        if let Some(m) = text_lower[search_pos..].find(&query_lower) {
            let start = search_pos + m;
            let end = start + query_len;
            match_ranges.push((start, end));
            search_pos = end;
        } else {
            break;
        }
    }

    if match_ranges.is_empty() {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    // Build a char-index mapping: for each char, record its byte start in original and lowered
    let orig_chars: Vec<(usize, char)> = text.char_indices().collect();
    let lower_chars: Vec<(usize, char)> = text_lower.char_indices().collect();

    // Build byte-position mapping from lowered → original using char alignment
    // Both should have the same number of chars
    let char_count = orig_chars.len().min(lower_chars.len());

    // Convert match_ranges from lowered byte positions to original byte positions
    let mut orig_ranges: Vec<(usize, usize)> = Vec::new();
    for &(low_start, low_end) in &match_ranges {
        let start_char = lower_chars.iter().position(|&(pos, _)| pos == low_start);
        let end_char = lower_chars
            .iter()
            .position(|&(pos, _)| pos == low_end)
            .unwrap_or(char_count);
        if let Some(sc) = start_char {
            let orig_start = orig_chars[sc].0;
            let orig_end = if end_char < orig_chars.len() {
                orig_chars[end_char].0
            } else {
                text.len()
            };
            orig_ranges.push((orig_start, orig_end));
        }
    }

    // Build spans from original ranges
    let mut spans = Vec::new();
    let mut pos = 0;
    for (start, end) in orig_ranges {
        if start > pos {
            spans.push(Span::styled(text[pos..start].to_string(), base_style));
        }
        spans.push(Span::styled(text[start..end].to_string(), match_style));
        pos = end;
    }
    if pos < text.len() {
        spans.push(Span::styled(text[pos..].to_string(), base_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), base_style));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_snippet_short_passthrough() {
        let body = "short text";
        assert_eq!(search_snippet(body, "short", 100), body);
    }

    #[test]
    fn search_snippet_centers_on_match() {
        let body = "a".repeat(100) + "NEEDLE" + &"b".repeat(100);
        let snippet = search_snippet(&body, "NEEDLE", 30);
        assert!(
            snippet.chars().count() <= 30,
            "snippet too long ({} chars): {snippet}",
            snippet.chars().count()
        );
        assert!(
            snippet.contains("NEEDLE"),
            "expected query in snippet: {snippet}"
        );
    }
}
