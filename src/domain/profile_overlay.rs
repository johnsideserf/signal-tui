/// State for the profile editor overlay.
#[derive(Default)]
pub struct ProfileOverlayState {
    /// Profile editor overlay visible
    pub show: bool,
    /// Cursor position in profile editor
    pub index: usize,
    /// Whether currently editing a profile field
    pub editing: bool,
    /// Profile fields: [given_name, family_name, about, about_emoji]
    pub fields: [String; 4],
    /// Temp buffer while editing a profile field
    pub edit_buffer: String,
}
