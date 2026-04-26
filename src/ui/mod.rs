//! Stateless rendering layer.
//!
//! [`draw`] takes the current [`App`] and renders sidebar + chat + status
//! bar each frame. Sender colors are hash-based across an 8-color palette;
//! groups are prefixed with `#`. OSC 8 hyperlinks are injected post-render
//! to dodge ratatui width calculation bugs (see [`LinkRegion`]).

mod autocomplete;
mod chat_pane;
mod composer;
mod overlays;
mod sidebar;
mod status_bar;
mod welcome;

use autocomplete::draw_autocomplete;
use chat_pane::draw_chat_area;
use overlays::about::draw_about;
use overlays::action_menu::draw_action_menu;
use overlays::contacts::draw_contacts;
use overlays::customize::draw_customize;
use overlays::delete_confirm::draw_delete_confirm;
use overlays::emoji_picker::draw_emoji_picker;
use overlays::file_browser::draw_file_browser;
use overlays::forward::draw_forward;
use overlays::group_menu::draw_group_menu;
use overlays::help::draw_help;
use overlays::keybindings::draw_keybindings;
use overlays::message_request::draw_message_request;
use overlays::pin_duration::draw_pin_duration_picker;
use overlays::poll_vote::draw_poll_vote_overlay;
use overlays::profile::draw_profile;
use overlays::reaction_picker::draw_reaction_picker;
use overlays::search::draw_search;
use overlays::settings::draw_settings;
use overlays::settings_profile::draw_settings_profile_manager;
use overlays::theme_picker::draw_theme_picker;
use overlays::verify::draw_verify;
use sidebar::draw_sidebar;
use status_bar::draw_status_bar;

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear},
};

use crate::app::{App, OverlayKind};
use crate::signal::types::{MessageStatus, StyleType};
use crate::theme::Theme;

// Layout constants
const SIDEBAR_AUTO_HIDE_WIDTH: u16 = 60;
const MIN_CHAT_WIDTH: u16 = 30;
pub(super) const MSG_WINDOW_MULTIPLIER: usize = 10;

// Popup dimensions
pub(super) const SETTINGS_POPUP_WIDTH: u16 = 50;
pub(super) const SETTINGS_POPUP_HEIGHT: u16 = 25;
pub(super) const CONTACTS_POPUP_WIDTH: u16 = 50;
pub(super) const CONTACTS_MAX_VISIBLE: usize = 20;
pub(super) const FILE_BROWSER_POPUP_WIDTH: u16 = 60;
pub(super) const FILE_BROWSER_MAX_VISIBLE: usize = 20;
pub(super) const SEARCH_POPUP_WIDTH: u16 = 60;
pub(super) const SEARCH_MAX_VISIBLE: usize = 15;
pub(super) const GROUP_MENU_POPUP_WIDTH: u16 = 40;
pub(super) const GROUP_MEMBER_MAX_VISIBLE: usize = 15;
pub(super) const ABOUT_POPUP_WIDTH: u16 = 50;
pub(super) const PROFILE_POPUP_WIDTH: u16 = 50;
pub(super) const EMOJI_POPUP_WIDTH: u16 = 52;
pub(super) const EMOJI_POPUP_HEIGHT: u16 = 20;

/// Map a MessageStatus to its display symbol and color.
pub(crate) fn status_symbol(
    status: MessageStatus,
    nerd_fonts: bool,
    color: bool,
    theme: &Theme,
) -> (&'static str, Color) {
    let (unicode_sym, nerd_sym, colored) = match status {
        MessageStatus::Failed => ("\u{2717}", "\u{f055c}", theme.receipt_failed),
        MessageStatus::Sending => ("\u{25cc}", "\u{f0996}", theme.receipt_sending),
        MessageStatus::Sent => ("\u{25cb}", "\u{f0954}", theme.receipt_sent),
        MessageStatus::Delivered => ("\u{2713}", "\u{f012c}", theme.receipt_delivered),
        MessageStatus::Read => ("\u{25cf}", "\u{f012d}", theme.receipt_read),
        MessageStatus::Viewed => ("\u{25c9}", "\u{f0208}", theme.receipt_viewed),
    };
    let sym = if nerd_fonts { nerd_sym } else { unicode_sym };
    let fg = if color { colored } else { theme.fg_muted };
    (sym, fg)
}

/// Hash a sender name to one of ~8 distinct colors. "you" always gets sender_self.
pub(crate) fn sender_color(name: &str, theme: &Theme) -> Color {
    if name == "you" {
        return theme.sender_self;
    }
    let hash: u32 = name
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    theme.sender_palette[(hash as usize) % theme.sender_palette.len()]
}

/// Truncate a string to fit within `max_width`, appending `…` if truncated.
pub(crate) fn truncate(s: &str, max_width: usize) -> String {
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

/// Build a centered separator line: `───── label ─────`
pub(crate) fn build_separator(label: &str, width: usize, style: Style) -> Line<'static> {
    let pad_total = width.saturating_sub(label.len());
    let pad_left = pad_total / 2;
    let pad_right = pad_total - pad_left;
    Line::from(Span::styled(
        format!("{}{}{}", "─".repeat(pad_left), label, "─".repeat(pad_right)),
        style,
    ))
}

/// Create a centered popup overlay: clears the area, returns the Rect and a styled Block.
/// Preferred width/height are clamped to fit within the terminal.
pub(super) fn centered_popup(
    frame: &mut Frame,
    area: Rect,
    pref_width: u16,
    pref_height: u16,
    title: &str,
    theme: &Theme,
) -> (Rect, Block<'static>) {
    let w = pref_width.min(area.width.saturating_sub(4));
    let h = pref_height.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup_area = Rect::new(x, y, w, h);
    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(title.to_string())
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg));
    (popup_area, block)
}

/// A clickable link region detected in the rendered buffer.
pub struct LinkRegion {
    pub x: u16,
    pub y: u16,
    pub url: String,
    pub text: String,
    /// Display width in terminal columns (may differ from text.len() for Unicode).
    pub width: u16,
    /// Background color from the buffer cell, if non-default (e.g. highlight).
    pub bg: Option<Color>,
}

/// Extract a URL from link-styled text.
pub(crate) fn extract_url(text: &str) -> String {
    for scheme in &["file:///", "https://", "http://"] {
        if let Some(pos) = text.find(scheme) {
            let uri_start = &text[pos..];
            let uri_end = uri_start
                .find(|c: char| c.is_whitespace())
                .unwrap_or(uri_start.len());
            return uri_start[..uri_end].to_string();
        }
    }
    text.to_string()
}

/// Check if a cell's style matches the link style (link color fg + UNDERLINED).
fn is_link_style(style: &Style, link_color: Color) -> bool {
    style.fg == Some(link_color) && style.add_modifier.contains(Modifier::UNDERLINED)
}

/// Scan a rendered buffer area for consecutive cells with the link style,
/// and collect them into LinkRegion structs.
pub(super) fn collect_link_regions(buf: &Buffer, area: Rect, link_color: Color) -> Vec<LinkRegion> {
    let right_edge = area.x.saturating_add(area.width);
    let mut regions = Vec::new();
    let mut wrap_url: Option<String> = None;

    for y in area.y..area.y.saturating_add(area.height) {
        let mut x = area.x;
        let mut row_last_url: Option<String> = None;
        let mut row_last_reached_edge = false;

        while x < right_edge {
            let cell = match buf.cell(Position::new(x, y)) {
                Some(c) => c,
                None => {
                    x += 1;
                    continue;
                }
            };

            if !is_link_style(&cell.style(), link_color) {
                x += 1;
                continue;
            }

            // Start of a link run
            let start_x = x;
            let mut text = String::new();

            while x < right_edge {
                match buf.cell(Position::new(x, y)) {
                    Some(c) if is_link_style(&c.style(), link_color) => {
                        let sym = c.symbol();
                        if !sym.is_empty() {
                            text.push_str(sym);
                        }
                        x += 1;
                    }
                    _ => break,
                }
            }

            if text.is_empty() {
                continue;
            }

            // Determine URL: use continuation URL if this is a wrapped link
            let url = if start_x == area.x {
                if let Some(ref wu) = wrap_url {
                    wu.clone()
                } else {
                    extract_url(&text)
                }
            } else {
                extract_url(&text)
            };

            let reached_edge = x >= right_edge;
            row_last_url = Some(url.clone());
            row_last_reached_edge = reached_edge;

            // Capture background color from the first cell of the link run so
            // emit_osc8_links can preserve it (e.g. highlight bg on selection).
            let bg = buf
                .cell(Position::new(start_x, y))
                .and_then(|c| c.style().bg);
            regions.push(LinkRegion {
                x: start_x,
                y,
                url,
                text,
                width: x - start_x,
                bg,
            });
        }

        // Propagate URL for wrapped links
        wrap_url = if row_last_reached_edge {
            row_last_url
        } else {
            None
        };
    }

    regions
}

/// Split a list of body spans into sub-lists, one per output line, using `\n`
/// in any span's content as the line break. Styles are preserved when splitting
/// a span. Empty lines (consecutive `\n`) produce an empty sub-list.
pub(super) fn split_spans_by_newline(spans: Vec<Span<'static>>) -> Vec<Vec<Span<'static>>> {
    let mut lines: Vec<Vec<Span<'static>>> = vec![Vec::new()];
    for span in spans {
        if !span.content.contains('\n') {
            lines.last_mut().unwrap().push(span);
            continue;
        }
        let style = span.style;
        let content = span.content.into_owned();
        let mut parts = content.split('\n').peekable();
        while let Some(part) = parts.next() {
            if !part.is_empty() {
                lines
                    .last_mut()
                    .unwrap()
                    .push(Span::styled(part.to_string(), style));
            }
            if parts.peek().is_some() {
                lines.push(Vec::new());
            }
        }
    }
    lines
}

/// Split a message body into spans, styling any URI (https://, http://, file:///) as
/// underlined blue text. Non-URI text is rendered as plain spans.
///
/// Returns `(spans, Option<hidden_url>)`. For attachment bodies like
/// `[image: label](file:///path)`, the bracket text is the visible link and
/// the URI inside parens is returned separately (not displayed).
pub(super) fn styled_uri_spans(
    body: &str,
    mention_ranges: &[(usize, usize)],
    style_ranges: &[(usize, usize, StyleType)],
    theme: &Theme,
) -> (Vec<Span<'static>>, Option<String>) {
    let link_style = Style::default()
        .fg(theme.link)
        .add_modifier(Modifier::UNDERLINED);
    let mention_style = Style::default()
        .fg(theme.mention)
        .add_modifier(Modifier::BOLD);

    // Attachment/image patterns: extract bracket text as display, URI as hidden metadata
    if body.starts_with("[image:") || body.starts_with("[attachment:") {
        // Extract the bracket portion: [image: label] or [attachment: label]
        if let Some(bracket_end) = body.find(']') {
            let display_text = &body[..=bracket_end]; // e.g. "[image: photo.jpg]"

            // Extract URI from either new format ](file:///...) or old format ] file:///...
            let hidden_url = if let Some(uri_pos) = body.find("file:///") {
                let uri_start = &body[uri_pos..];
                // End at whitespace, closing paren, or end of string
                let uri_end = uri_start
                    .find(|c: char| c.is_whitespace() || c == ')')
                    .unwrap_or(uri_start.len());
                Some(uri_start[..uri_end].to_string())
            } else {
                None
            };

            if hidden_url.is_some() {
                return (
                    vec![Span::styled(display_text.to_string(), link_style)],
                    hidden_url,
                );
            }
        }
    }

    // Build a sorted list of styled regions: mentions and URIs
    // Each region: (byte_start, byte_end, style)
    let mut regions: Vec<(usize, usize, Style)> = Vec::new();

    // Add mention regions
    for &(start, end) in mention_ranges {
        if start < body.len() && end <= body.len() {
            regions.push((start, end, mention_style));
        }
    }

    // Find URI regions
    let mut search_pos = 0;
    while search_pos < body.len() {
        let rest = &body[search_pos..];
        let next_uri = ["https://", "http://", "file:///"]
            .iter()
            .filter_map(|scheme| rest.find(scheme).map(|pos| (pos, *scheme)))
            .min_by_key(|(pos, _)| *pos);

        match next_uri {
            Some((rel_pos, _scheme)) => {
                let abs_start = search_pos + rel_pos;
                let uri_slice = &body[abs_start..];
                let uri_len = uri_slice
                    .find(|c: char| c.is_whitespace())
                    .unwrap_or(uri_slice.len());
                let abs_end = abs_start + uri_len;
                // Only add if not overlapping a mention region
                let overlaps = regions
                    .iter()
                    .any(|(ms, me, _)| abs_start < *me && abs_end > *ms);
                if !overlaps {
                    regions.push((abs_start, abs_end, link_style));
                }
                search_pos = abs_end;
            }
            None => break,
        }
    }

    // Sort regions by start position
    regions.sort_by_key(|r| r.0);

    // If no text styles, use the simple path
    if style_ranges.is_empty() {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut pos = 0;
        for (start, end, style) in &regions {
            if *start > pos {
                spans.push(Span::raw(body[pos..*start].to_string()));
            }
            spans.push(Span::styled(body[*start..*end].to_string(), *style));
            pos = *end;
        }
        if pos < body.len() {
            spans.push(Span::raw(body[pos..].to_string()));
        }
        return (spans, None);
    }

    // With text styles: collect all boundary points and build segments where
    // the active set of styles is constant
    let mut boundaries: Vec<usize> = Vec::new();
    boundaries.push(0);
    boundaries.push(body.len());
    for &(start, end, _) in &regions {
        boundaries.push(start);
        boundaries.push(end);
    }
    for &(start, end, _) in style_ranges {
        if start <= body.len() {
            boundaries.push(start);
        }
        if end <= body.len() {
            boundaries.push(end);
        }
    }
    boundaries.sort();
    boundaries.dedup();

    let mut spans: Vec<Span<'static>> = Vec::new();
    for window in boundaries.windows(2) {
        let seg_start = window[0];
        let seg_end = window[1];
        if seg_start >= seg_end || seg_start >= body.len() {
            continue;
        }
        let seg_end = seg_end.min(body.len());

        // Determine base style from mention/URI regions
        let mut style = Style::default();
        for &(rs, re, ref_style) in &regions {
            if seg_start >= rs && seg_end <= re {
                style = ref_style;
                break;
            }
        }

        // Check for spoiler first — if any spoiler range covers this segment,
        // replace the text with block characters
        let mut is_spoiler = false;
        for &(ss, se, st) in style_ranges {
            if st == StyleType::Spoiler && seg_start >= ss && seg_end <= se {
                is_spoiler = true;
                break;
            }
        }

        let segment_text = &body[seg_start..seg_end];
        if is_spoiler {
            // Replace each character with a block character
            let block_text: String = segment_text.chars().map(|_| '\u{2588}').collect();
            let spoiler_style = style.fg(theme.fg_muted);
            spans.push(Span::styled(block_text, spoiler_style));
        } else {
            // Apply text style modifiers
            for &(ss, se, st) in style_ranges {
                if seg_start >= ss && seg_end <= se {
                    match st {
                        StyleType::Bold => style = style.add_modifier(Modifier::BOLD),
                        StyleType::Italic => style = style.add_modifier(Modifier::ITALIC),
                        StyleType::Strikethrough => {
                            style = style.add_modifier(Modifier::CROSSED_OUT)
                        }
                        StyleType::Monospace => style = style.fg(theme.fg_muted),
                        StyleType::Spoiler => {} // handled above
                    }
                }
            }

            if style == Style::default() {
                spans.push(Span::raw(segment_text.to_string()));
            } else {
                spans.push(Span::styled(segment_text.to_string(), style));
            }
        }
    }

    (spans, None)
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    app.image.link_url_map.clear();
    app.image.visible_images.clear();
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

    // Narrow terminal adaptation: auto-hide sidebar below threshold
    let sidebar_auto_hidden = terminal_width < SIDEBAR_AUTO_HIDE_WIDTH;
    let show_sidebar = app.sidebar_visible && !sidebar_auto_hidden;

    let input_area = if show_sidebar {
        let (sidebar_idx, chat_idx, constraints) = if app.sidebar_on_right {
            (
                1,
                0,
                [
                    Constraint::Min(MIN_CHAT_WIDTH),
                    Constraint::Length(app.sidebar_width),
                ],
            )
        } else {
            (
                0,
                1,
                [
                    Constraint::Length(app.sidebar_width),
                    Constraint::Min(MIN_CHAT_WIDTH),
                ],
            )
        };
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(body_area);

        draw_sidebar(frame, app, horizontal[sidebar_idx]);
        draw_chat_area(frame, app, horizontal[chat_idx])
    } else {
        app.mouse.sidebar_inner = None;
        draw_chat_area(frame, app, body_area)
    };

    draw_status_bar(frame, app, status_area, sidebar_auto_hidden);

    // Autocomplete popup (overlays everything)
    if app.is_overlay(OverlayKind::Autocomplete) {
        let has_items = !app.autocomplete.is_empty();
        if has_items {
            draw_autocomplete(frame, app, input_area);
        }
    }

    // Settings overlay (overlays everything)
    if app.is_overlay(OverlayKind::Settings) {
        draw_settings(frame, app, size);
    }

    // Customize sub-menu overlay (Theme, Keybindings, Profile)
    if app.is_overlay(OverlayKind::Customize) {
        draw_customize(frame, app, size);
    }

    // Help overlay (overlays everything)
    if app.is_overlay(OverlayKind::Help) {
        draw_help(frame, app, size);
    }

    // Contacts overlay (overlays everything)
    if app.is_overlay(OverlayKind::Contacts) {
        draw_contacts(frame, app, size);
    }

    // Verify identity overlay
    if app.is_overlay(OverlayKind::Verify) {
        draw_verify(frame, app, size);
    }

    // Search overlay
    if app.is_overlay(OverlayKind::Search) {
        draw_search(frame, app, size);
    }

    // File browser overlay
    if app.is_overlay(OverlayKind::FilePicker) {
        draw_file_browser(frame, app, size);
    }

    // Group management menu overlay
    if app.is_overlay(OverlayKind::GroupMenu) {
        draw_group_menu(frame, app, size);
    }

    // Message request overlay
    if app.is_overlay(OverlayKind::MessageRequest) {
        draw_message_request(frame, app, size);
    }

    // Action menu overlay
    if app.is_overlay(OverlayKind::ActionMenu) {
        draw_action_menu(frame, app, size);
    }

    // Reaction picker overlay
    if app.is_overlay(OverlayKind::ReactionPicker) {
        draw_reaction_picker(frame, app, size);
    }

    // Emoji picker overlay
    if app.is_overlay(OverlayKind::EmojiPicker) {
        draw_emoji_picker(frame, app, size);
    }

    // Delete confirmation overlay
    if app.is_overlay(OverlayKind::DeleteConfirm) {
        draw_delete_confirm(frame, app, size);
    }

    // Theme picker overlay
    if app.is_overlay(OverlayKind::ThemePicker) {
        draw_theme_picker(frame, app, size);
    }

    // Keybindings overlay
    if app.is_overlay(OverlayKind::Keybindings) {
        draw_keybindings(frame, app, size);
    }

    // Settings profile manager overlay
    if app.is_overlay(OverlayKind::SettingsProfiles) {
        draw_settings_profile_manager(frame, app, size);
    }

    // Pin duration picker overlay
    if app.is_overlay(OverlayKind::PinDuration) {
        draw_pin_duration_picker(frame, app, size);
    }

    // Poll vote overlay
    if app.is_overlay(OverlayKind::PollVote) {
        draw_poll_vote_overlay(frame, app, size);
    }

    // About overlay
    if app.is_overlay(OverlayKind::About) {
        draw_about(frame, app, size);
    }

    // Profile editor overlay
    if app.is_overlay(OverlayKind::Profile) {
        draw_profile(frame, app, size);
    }

    // Forward message picker overlay
    if app.is_overlay(OverlayKind::Forward) {
        draw_forward(frame, app, size);
    }

    // Collect link regions from the rendered buffer for OSC 8 injection
    let area = frame.area();
    app.image.link_regions = collect_link_regions(frame.buffer_mut(), area, app.theme.link);

    // Resolve hidden URLs for attachment links (display text has no URI scheme)
    for link in &mut app.image.link_regions {
        if !link.url.contains("://")
            && let Some(url) = app.image.link_url_map.get(&link.text)
        {
            link.url = url.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::types::MessageStatus;
    use crate::theme::default_theme;
    use rstest::rstest;

    // --- sender_color ---

    #[test]
    fn sender_color_you_returns_self() {
        let theme = default_theme();
        assert_eq!(sender_color("you", &theme), theme.sender_self);
    }

    #[test]
    fn sender_color_deterministic() {
        let theme = default_theme();
        let c1 = sender_color("Alice", &theme);
        let c2 = sender_color("Alice", &theme);
        assert_eq!(c1, c2);
    }

    #[test]
    fn sender_color_in_palette() {
        let theme = default_theme();
        let c = sender_color("Bob", &theme);
        assert!(theme.sender_palette.contains(&c));
    }

    // --- truncate ---

    #[rstest]
    #[case("hi", 10, "hi")]
    #[case("hello", 5, "hello")]
    #[case("hello world", 5, "hell\u{2026}")]
    #[case("abc", 1, "\u{2026}")]
    #[case("abc", 0, "\u{2026}")]
    #[case("", 5, "")]
    fn truncate_cases(#[case] input: &str, #[case] max: usize, #[case] expected: &str) {
        assert_eq!(truncate(input, max), expected);
    }

    // --- status_symbol ---

    #[rstest]
    #[case(MessageStatus::Failed, "\u{2717}")]
    #[case(MessageStatus::Sending, "\u{25cc}")]
    #[case(MessageStatus::Sent, "\u{25cb}")]
    #[case(MessageStatus::Delivered, "\u{2713}")]
    #[case(MessageStatus::Read, "\u{25cf}")]
    #[case(MessageStatus::Viewed, "\u{25c9}")]
    fn status_symbol_variants(#[case] status: MessageStatus, #[case] expected_sym: &str) {
        let theme = default_theme();
        let (sym, _) = status_symbol(status, false, true, &theme);
        assert_eq!(sym, expected_sym);
    }

    #[test]
    fn status_symbol_color_vs_muted() {
        let theme = default_theme();
        let (_, colored) = status_symbol(MessageStatus::Read, false, true, &theme);
        let (_, muted) = status_symbol(MessageStatus::Read, false, false, &theme);
        assert_eq!(colored, theme.receipt_read);
        assert_eq!(muted, theme.fg_muted);
    }

    // --- build_separator ---

    #[test]
    fn build_separator_pads() {
        let theme = default_theme();
        let line = build_separator(" Jan 1 ", 40, Style::default().fg(theme.fg_muted));
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert_eq!(text.chars().count(), 40);
        assert!(text.contains("Jan 1"));
    }

    // --- extract_url ---

    #[rstest]
    #[case("https://example.com", "https://example.com")]
    #[case("http://foo.bar/baz", "http://foo.bar/baz")]
    #[case("file:///tmp/a.txt", "file:///tmp/a.txt")]
    #[case("check https://x.com/path here", "https://x.com/path")]
    #[case("no-scheme.com", "no-scheme.com")]
    fn extract_url_cases(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(extract_url(input), expected);
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    use crate::app::{App, InputMode, PinPending};
    use crate::db::Database;
    use crate::domain::EmojiPickerSource;
    use crate::image_render::ImageProtocol;
    use chrono::NaiveDate;
    use ratatui::{Terminal, backend::TestBackend};

    /// Fixed date for deterministic timestamps in snapshots.
    fn fixed_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()
    }

    /// Create a fully-populated demo App with deterministic data.
    fn demo_app() -> App {
        let db = Database::open_in_memory().unwrap();
        let mut app = App::new("+15559999999".to_string(), db);
        app.connected = true;
        app.loading = false;
        app.is_demo = true;
        app.date_separators = false;
        app.image.image_protocol = ImageProtocol::Halfblock;
        app.populate_demo_data(fixed_date());
        app
    }

    /// Render the app into a TestBackend and return the buffer contents as a string.
    fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(cell.symbol());
            }
            // Trim trailing spaces for cleaner snapshots
            let trimmed = output.trim_end();
            output.truncate(trimmed.len());
            output.push('\n');
        }
        output
    }

    #[test]
    fn test_sidebar_layout() {
        let mut app = demo_app();
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_chat_messages() {
        let mut app = demo_app();
        // Alice is already the active conversation
        assert_eq!(app.active_conversation.as_deref(), Some("+15550001111"));
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn body_newlines_render_as_separate_lines() {
        use crate::conversation_store::DisplayMessage;
        let mut app = demo_app();
        let conv_id = app.active_conversation.clone().unwrap();
        if let Some(conv) = app.store.conversations.get_mut(&conv_id) {
            conv.messages.clear();
            conv.messages.push(DisplayMessage {
                sender: "Alice".to_string(),
                timestamp: chrono::Utc::now(),
                body: "line one\nline two".to_string(),
                is_system: false,
                image_lines: None,
                image_path: None,
                status: None,
                timestamp_ms: 1_700_000_000_000,
                reactions: Vec::new(),
                mention_ranges: Vec::new(),
                style_ranges: Vec::new(),
                body_raw: None,
                mentions: Vec::new(),
                quote: None,
                is_edited: false,
                is_deleted: false,
                is_pinned: false,
                sender_id: "+15550001111".to_string(),
                expires_in_seconds: 0,
                expiration_start_ms: 0,
                poll_data: None,
                poll_votes: Vec::new(),
                preview: None,
                preview_image_lines: None,
                preview_image_path: None,
            });
        }
        let output = render_to_string(&mut app, 100, 30);
        for row in output.lines() {
            assert!(
                !(row.contains("line one") && row.contains("line two")),
                "body text should split across rows; got row: {row:?}\nfull output:\n{output}"
            );
        }
        assert!(
            output.contains("line one") && output.contains("line two"),
            "expected both body lines to appear; got:\n{output}"
        );
    }

    #[test]
    fn test_normal_vs_insert_mode() {
        let mut app = demo_app();

        app.mode = InputMode::Insert;
        let insert_output = render_to_string(&mut app, 100, 30);

        app.mode = InputMode::Normal;
        let normal_output = render_to_string(&mut app, 100, 30);

        // They should differ (mode indicator in status bar)
        assert_ne!(insert_output, normal_output);
        insta::assert_snapshot!("insert_mode", insert_output);
        insta::assert_snapshot!("normal_mode", normal_output);
    }

    #[test]
    fn test_no_active_conversation() {
        let mut app = demo_app();
        app.active_conversation = None;
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_help_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::Help);
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_narrow_terminal() {
        let mut app = demo_app();
        // Below SIDEBAR_AUTO_HIDE_WIDTH (60), sidebar should auto-hide
        let output = render_to_string(&mut app, 50, 20);
        insta::assert_snapshot!(output);
    }

    // --- Phase 2: Message features ---
    // Note: quote replies, link previews, edited messages, and reactions are all
    // covered by test_chat_messages (Alice conversation contains all of these).

    #[test]
    fn test_styled_text() {
        // Bob conversation: bold and monospace styled text
        let mut app = demo_app();
        app.active_conversation = Some("+15550002222".to_string());
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_poll() {
        // Rust Devs group: poll rendering with question, options, vote counts
        let mut app = demo_app();
        app.active_conversation = Some("group_rustdevs".to_string());
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_pinned_message() {
        // Rust Devs group: "(pinned)" label on the pinned message
        let mut app = demo_app();
        app.active_conversation = Some("group_rustdevs".to_string());
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_unread_marker() {
        // Family group has 2 unread out of 5 messages, last_read_index = 3
        let mut app = demo_app();
        app.active_conversation = Some("group_family".to_string());
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    // --- Phase 3: Overlays ---

    #[test]
    fn test_settings_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::Settings);
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_about_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::About);
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    // --- Phase 4: Edge cases ---

    #[test]
    fn test_empty_conversation() {
        use crate::app::Conversation;
        let mut app = demo_app();
        let empty_id = "+15550009999".to_string();
        app.store.conversations.insert(
            empty_id.clone(),
            Conversation {
                name: "Empty".to_string(),
                id: empty_id.clone(),
                messages: Vec::new(),
                unread: 0,
                is_group: false,
                expiration_timer: 0,
                accepted: true,
            },
        );
        app.store.conversation_order.push(empty_id.clone());
        app.active_conversation = Some(empty_id);
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_message_request() {
        // Eve's conversation is unaccepted (message request)
        let mut app = demo_app();
        app.active_conversation = Some("+15550007777".to_string());
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_disappearing_messages() {
        // Dave's conversation has disappearing messages with timer icons
        let mut app = demo_app();
        app.active_conversation = Some("+15550004444".to_string());
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_sidebar_filter() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::SidebarFilter);
        app.sidebar_filter = "ali".to_string();
        app.refresh_sidebar_filter();
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_theme_picker_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::ThemePicker);
        app.theme_picker.index = 1;
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_pin_duration_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::PinDuration);
        app.pin_duration.index = 1;
        app.pin_duration.pending = Some(PinPending {
            conv_id: "+15551234567".to_string(),
            is_group: false,
            target_author: "+15551234567".to_string(),
            target_timestamp: 1000,
        });
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_action_menu_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::ActionMenu);
        app.action_menu.index = 0;
        app.scroll.focused_index = Some(0);
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_contacts_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::Contacts);
        app.contacts_overlay.index = 0;
        app.contacts_overlay.filtered = vec![
            ("+15551234567".to_string(), "Alice".to_string()),
            ("+15559876543".to_string(), "Bob".to_string()),
        ];
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_forward_overlay() {
        let mut app = demo_app();
        app.open_overlay(OverlayKind::Forward);
        app.forward.index = 0;
        app.forward.filtered = vec![
            ("+15551234567".to_string(), "Alice".to_string()),
            ("+15559876543".to_string(), "Bob".to_string()),
        ];
        app.forward.body = "Hello world".to_string();
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_emoji_picker_overlay() {
        let mut app = demo_app();
        app.emoji_picker.open(EmojiPickerSource::Input, None);
        app.open_overlay(OverlayKind::EmojiPicker);
        let output = render_to_string(&mut app, 100, 30);
        insta::assert_snapshot!(output);
    }
}
