use crate::keybindings::{KeyAction, KeyCombo};

/// State for the keybindings configuration overlay.
#[derive(Default)]
pub struct KeybindingsOverlayState {
    /// Keybindings overlay visible
    pub show: bool,
    /// Cursor position in keybindings overlay
    pub index: usize,
    /// Whether capturing a new key binding
    pub capturing: bool,
    /// Conflict detected during capture
    pub conflict: Option<(KeyAction, KeyCombo)>,
    /// Profile sub-picker visible within keybindings overlay
    pub profile_picker: bool,
    /// Cursor position in profile sub-picker
    pub profile_index: usize,
    /// All available keybinding profile names
    pub available_profiles: Vec<String>,
}
