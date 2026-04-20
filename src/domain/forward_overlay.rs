/// State for the forward message picker overlay.
#[derive(Default)]
pub struct ForwardOverlayState {
    /// Cursor position in forward picker
    pub index: usize,
    /// Type-to-filter text for forward picker
    pub filter: String,
    /// Filtered list of (conv_id, display_name)
    pub filtered: Vec<(String, String)>,
    /// Body of the message being forwarded
    pub body: String,
}
