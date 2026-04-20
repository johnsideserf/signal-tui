/// State for the contacts list overlay.
#[derive(Default)]
pub struct ContactsOverlayState {
    /// Cursor position in contacts list
    pub index: usize,
    /// Type-to-filter text for contacts overlay
    pub filter: String,
    /// Filtered list of (phone_number, display_name)
    pub filtered: Vec<(String, String)>,
}
