//! Keybindings configuration overlay + nested profile picker.
//!
//! Lists all rebindable actions grouped by `BindingMode` (Global /
//! Normal / Insert), with the Profile row at the top. Selecting a row
//! and pressing Enter enters capture mode (`[Press key...]`) until a
//! key is recorded. The nested `draw_keybindings_profile_picker`
//! shows when `keybindings_overlay.profile_picker` is active and lets
//! the user switch between named keybinding profiles.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::centered_popup;
use crate::app::App;
use crate::keybindings::{self, BindingMode, KeyAction};

pub(in crate::ui) fn draw_keybindings(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // If the profile picker sub-overlay is open, draw it instead
    if app.keybindings_overlay.profile_picker {
        draw_keybindings_profile_picker(frame, app, area);
        return;
    }

    let total_rows = app.keybindings_overlay_total();
    let max_visible = 24usize.min(total_rows);
    let pref_height = max_visible as u16 + 4; // borders + footer
    let pref_width = 52;

    let (popup_area, block) =
        centered_popup(frame, area, pref_width, pref_height, " Keybindings ", theme);

    let inner_height = popup_area.height.saturating_sub(2) as usize;
    let footer_lines = 2;
    let visible_rows = inner_height.saturating_sub(footer_lines);

    let scroll_offset = if app.keybindings_overlay.index >= visible_rows {
        app.keybindings_overlay.index - visible_rows + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    let key_col_width = 26;
    let val_col_width = 20;

    let end = (scroll_offset + visible_rows).min(total_rows);
    for row in scroll_offset..end {
        let is_selected = row == app.keybindings_overlay.index;
        let (mode, action): (BindingMode, Option<KeyAction>) = app.keybindings_overlay_item(row);

        if row == 0 {
            // Profile row
            let style = if is_selected {
                Style::default()
                    .bg(theme.bg_selected)
                    .fg(theme.fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg_secondary)
            };
            let val_style = if is_selected {
                Style::default().bg(theme.bg_selected).fg(theme.accent)
            } else {
                Style::default().fg(theme.accent)
            };
            lines.push(Line::from(vec![
                Span::styled("  Profile: ", style),
                Span::styled(app.keybindings.profile_name.clone(), val_style),
            ]));
        } else if action.is_none() {
            // Section header
            let label = match mode {
                BindingMode::Global => "Global",
                BindingMode::Normal => "Normal Mode",
                BindingMode::Insert => "Insert Mode",
            };
            let header_style = Style::default()
                .fg(theme.accent_secondary)
                .add_modifier(Modifier::BOLD);
            lines.push(Line::from(Span::styled(
                format!("  -- {label} --"),
                header_style,
            )));
        } else {
            // Action row
            let action = action.unwrap();
            let label = keybindings::action_label(action);
            let key_display = if is_selected && app.keybindings_overlay.capturing {
                "[Press key...]".to_string()
            } else {
                // Multi-key sequences not in the binding map
                match action {
                    KeyAction::ScrollToTop => "gg".to_string(),
                    KeyAction::DeleteMessage => "dd".to_string(),
                    _ => app.keybindings.display_key(action),
                }
            };

            let row_style = if is_selected {
                Style::default()
                    .bg(theme.bg_selected)
                    .fg(theme.fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg_secondary)
            };
            let key_style = if is_selected {
                Style::default().bg(theme.bg_selected).fg(theme.accent)
            } else {
                Style::default().fg(theme.accent)
            };

            let padded_label = format!("{label:width$}", width = key_col_width);
            lines.push(Line::from(vec![
                Span::styled(format!("  {padded_label}"), row_style),
                Span::styled(
                    format!("{key_display:>width$}", width = val_col_width),
                    key_style,
                ),
            ]));
        }
    }

    // Pad
    while lines.len() < visible_rows {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Enter rebind | Backspace reset | Esc close",
        Style::default().fg(theme.fg_muted),
    )));

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}

fn draw_keybindings_profile_picker(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let max_visible = 8usize.min(app.keybindings_overlay.available_profiles.len());
    let pref_height = max_visible as u16 + 5;

    let (popup_area, block) =
        centered_popup(frame, area, 36, pref_height, " Keybinding Profile ", theme);

    let inner_height = popup_area.height.saturating_sub(2) as usize;
    let footer_lines = 2;
    let visible_rows = inner_height.saturating_sub(footer_lines);

    let scroll_offset = if app.keybindings_overlay.profile_index >= visible_rows {
        app.keybindings_overlay.profile_index - visible_rows + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    let end = (scroll_offset + visible_rows).min(app.keybindings_overlay.available_profiles.len());
    for i in scroll_offset..end {
        let is_selected = i == app.keybindings_overlay.profile_index;
        let is_active =
            app.keybindings_overlay.available_profiles[i] == app.keybindings.profile_name;
        let marker = if is_active { "[*]" } else { "[ ]" };

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
                theme.success
            } else {
                theme.fg_muted
            })
        } else {
            Style::default().fg(if is_active {
                theme.success
            } else {
                theme.fg_muted
            })
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {marker} "), marker_style),
            Span::styled(
                app.keybindings_overlay.available_profiles[i].clone(),
                row_style,
            ),
        ]));
    }

    while lines.len() < visible_rows {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  j/k navigate  |  Enter apply  |  Esc cancel",
        Style::default().fg(theme.fg_muted),
    )));

    let popup = Paragraph::new(lines).block(block);
    frame.render_widget(popup, popup_area);
}
