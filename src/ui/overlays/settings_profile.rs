//! Settings profile manager + Save-As sub-overlay.
//!
//! Lists named profiles with a marker on the active one. Footer hints
//! adapt to context: builtin profiles only allow Load; user profiles
//! also allow `s` save in place, `S` save-as, and `d` delete. The
//! save-as flow opens a separate text-input overlay
//! (`draw_settings_profile_save_as`).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::centered_popup;
use crate::app::App;

pub(in crate::ui) fn draw_settings_profile_manager(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // If save-as input is active, draw that sub-overlay instead
    if app.settings_profiles.save_as {
        draw_settings_profile_save_as(frame, app, area);
        return;
    }

    let max_visible = 10usize.min(app.settings_profiles.available.len());
    let pref_height = max_visible as u16 + 5; // borders + footer

    let (popup_area, block) =
        centered_popup(frame, area, 42, pref_height, " Settings Profiles ", theme);

    let inner_height = popup_area.height.saturating_sub(2) as usize;
    let footer_lines = 2;
    let visible_rows = inner_height.saturating_sub(footer_lines);

    let scroll_offset = if app.settings_profiles.index >= visible_rows {
        app.settings_profiles.index - visible_rows + 1
    } else {
        0
    };

    // Determine if current settings differ from loaded profile
    let has_changes = !app
        .settings_profiles
        .available
        .iter()
        .any(|p| p.name == app.settings_profiles.name && p.matches_app(app));

    let mut lines: Vec<Line> = Vec::new();
    let end = (scroll_offset + visible_rows).min(app.settings_profiles.available.len());
    for i in scroll_offset..end {
        let profile = &app.settings_profiles.available[i];
        let is_selected = i == app.settings_profiles.index;
        let is_active = profile.name == app.settings_profiles.name;

        let marker = if is_active { ">" } else { " " };
        let row_style = if is_selected {
            Style::default()
                .bg(theme.bg_selected)
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg)
        };
        let marker_style = if is_selected {
            Style::default().bg(theme.bg_selected).fg(if is_active {
                theme.accent
            } else {
                theme.fg_muted
            })
        } else {
            Style::default().fg(if is_active {
                theme.accent
            } else {
                theme.fg_muted
            })
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {marker} "), marker_style),
            Span::styled(profile.name.clone(), row_style),
        ]));
    }

    while lines.len() < visible_rows {
        lines.push(Line::from(""));
    }

    // Build contextual footer hints
    let selected_profile = app
        .settings_profiles
        .available
        .get(app.settings_profiles.index);
    let is_builtin = selected_profile
        .map(|p| crate::settings_profile::is_builtin(&p.name))
        .unwrap_or(true);

    let mut hints = vec!["j/k nav", "Enter load", "Esc close"];
    if has_changes {
        if !is_builtin {
            hints.push("s save");
        }
        hints.push("S save as");
    }
    if !is_builtin {
        hints.push("d delete");
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", hints.join("  ")),
        Style::default().fg(theme.fg_muted),
    )));

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}

fn draw_settings_profile_save_as(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let (popup_area, block) = centered_popup(frame, area, 40, 7, " Save Profile As ", theme);

    let cursor_char = if app.settings_profiles.save_as_input.is_empty() {
        "_"
    } else {
        ""
    };
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name: ", Style::default().fg(theme.fg_secondary)),
            Span::styled(
                format!("{}{cursor_char}", app.settings_profiles.save_as_input),
                Style::default()
                    .fg(theme.fg)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter save  |  Esc cancel",
            Style::default().fg(theme.fg_muted),
        )),
    ];

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}
