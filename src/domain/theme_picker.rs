use crate::theme::Theme;

/// State for the theme picker overlay.
#[derive(Default)]
pub struct ThemePickerState {
    /// Theme picker overlay visible
    pub show: bool,
    /// Cursor position in theme picker
    pub index: usize,
    /// All available themes (built-in + custom)
    pub available_themes: Vec<Theme>,
}
