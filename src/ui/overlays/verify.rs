//! Identity verification overlay.
//!
//! Two layouts: 1:1 chats show a single identity with name, trust
//! state, formatted safety number, and fingerprint. Group chats show
//! a scrollable member list with trust badges; the selected member's
//! safety number renders below. The `confirming` flag toggles a
//! "press v to confirm" prompt for the verify action.

use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::centered_popup;
use crate::app::App;
use crate::signal::types::TrustLevel;

pub(in crate::ui) fn draw_verify(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let is_group = app
        .active_conversation
        .as_ref()
        .and_then(|id| app.store.conversations.get(id))
        .map(|c| c.is_group)
        .unwrap_or(false);

    let pref_height: u16 = if is_group { 18 } else { 14 };
    let pref_width: u16 = 50;
    let (popup_area, block) = centered_popup(
        frame,
        area,
        pref_width,
        pref_height,
        " Verify Identity ",
        theme,
    );
    let inner = popup_area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let mut lines: Vec<Line> = Vec::new();

    if app.verify.identities.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No identity information available",
            Style::default().fg(theme.fg_muted),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Esc: close",
            Style::default().fg(theme.fg_muted),
        )));
    } else if is_group {
        // Group view: scrollable member list with trust badges
        let member_rows = inner.height.saturating_sub(7) as usize; // reserve for safety number + footer
        let scroll_offset = if app.verify.index >= member_rows {
            app.verify.index - member_rows + 1
        } else {
            0
        };
        let end = (scroll_offset + member_rows).min(app.verify.identities.len());

        for (i, identity) in app.verify.identities[scroll_offset..end].iter().enumerate() {
            let actual_idx = scroll_offset + i;
            let is_selected = actual_idx == app.verify.index;
            let number = identity.number.as_deref().unwrap_or("unknown");
            let name = app
                .store
                .contact_names
                .get(number)
                .cloned()
                .unwrap_or_else(|| number.to_string());
            let (badge, badge_color) = match identity.trust_level {
                TrustLevel::TrustedVerified => ("\u{2713}", theme.accent),
                TrustLevel::Untrusted => ("\u{26A0}", theme.warning),
                TrustLevel::TrustedUnverified => ("\u{2500}", theme.fg_muted),
            };
            let prefix = if is_selected { "> " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .bg(theme.bg_selected)
                    .fg(theme.fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            let badge_style = if is_selected {
                Style::default().bg(theme.bg_selected).fg(badge_color)
            } else {
                Style::default().fg(badge_color)
            };
            lines.push(Line::from(vec![
                Span::styled(prefix.to_string(), style),
                Span::styled(format!("{badge} "), badge_style),
                Span::styled(name, style),
            ]));
        }

        lines.push(Line::from(""));

        // Show selected member's safety number
        if let Some(identity) = app.verify.identities.get(app.verify.index) {
            if !identity.safety_number.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Safety Number:",
                    Style::default().fg(theme.fg_secondary),
                )));
                let sn = &identity.safety_number;
                let formatted = format_safety_number(sn);
                for row in formatted {
                    lines.push(Line::from(Span::styled(
                        format!("  {row}"),
                        Style::default().fg(theme.fg),
                    )));
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "  Safety number not available",
                    Style::default().fg(theme.fg_muted),
                )));
            }
        }

        lines.push(Line::from(""));
        if app.verify.confirming {
            lines.push(Line::from(Span::styled(
                "  Compare safety numbers, then press v to confirm",
                Style::default().fg(theme.warning),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "  j/k: navigate  v: verify  Esc: close",
                Style::default().fg(theme.fg_muted),
            )));
        }
    } else {
        // 1:1 view: single identity with full details
        let identity = &app.verify.identities[0];
        let number = identity.number.as_deref().unwrap_or("unknown");
        let name = app
            .store
            .contact_names
            .get(number)
            .cloned()
            .unwrap_or_else(|| number.to_string());

        lines.push(Line::from(Span::styled(
            format!("  {} ({})", name, number),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )));

        let (trust_label, trust_color) = match identity.trust_level {
            TrustLevel::TrustedVerified => ("\u{2713} Verified", theme.accent),
            TrustLevel::Untrusted => ("\u{26A0} Untrusted", theme.warning),
            TrustLevel::TrustedUnverified => ("\u{2500} Unverified", theme.fg_muted),
        };
        lines.push(Line::from(Span::styled(
            format!("  Trust: {trust_label}"),
            Style::default().fg(trust_color),
        )));
        lines.push(Line::from(""));

        if !identity.safety_number.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Safety Number:",
                Style::default().fg(theme.fg_secondary),
            )));
            let formatted = format_safety_number(&identity.safety_number);
            for row in formatted {
                lines.push(Line::from(Span::styled(
                    format!("  {row}"),
                    Style::default().fg(theme.fg),
                )));
            }
        } else {
            lines.push(Line::from(Span::styled(
                "  Safety number not available",
                Style::default().fg(theme.fg_muted),
            )));
        }

        lines.push(Line::from(""));
        if !identity.fingerprint.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  Fingerprint: {}", identity.fingerprint),
                Style::default().fg(theme.fg_muted),
            )));
            lines.push(Line::from(""));
        }

        if app.verify.confirming {
            lines.push(Line::from(Span::styled(
                "  Compare safety numbers, then press v to confirm",
                Style::default().fg(theme.warning),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "  v: verify key  Esc: close",
                Style::default().fg(theme.fg_muted),
            )));
        }
    }

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}

/// Format a safety number string as groups of 5 digits, 6 per line.
fn format_safety_number(sn: &str) -> Vec<String> {
    let digits: String = sn.chars().filter(|c| c.is_ascii_digit()).collect();
    let chunks: Vec<&str> = digits
        .as_bytes()
        .chunks(5)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect();
    chunks.chunks(6).map(|row| row.join(" ")).collect()
}
