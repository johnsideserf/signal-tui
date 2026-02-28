/// Parsed user input — either a command or plain text to send
#[derive(Debug)]
pub enum InputAction {
    /// Send text to the current conversation
    SendText(String),
    /// Switch to a conversation by name/number
    Join(String),
    /// Leave current conversation (go back to no selection)
    Part,
    /// Quit the application
    Quit,
    /// Toggle sidebar visibility
    ToggleSidebar,
    /// Show help text
    Help,
    /// Unknown command
    Unknown(String),
}

/// Parse a line of input into an action
pub fn parse_input(input: &str) -> InputAction {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return InputAction::SendText(String::new());
    }

    if !trimmed.starts_with('/') {
        return InputAction::SendText(trimmed.to_string());
    }

    let mut parts = trimmed.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim().to_string();

    match cmd {
        "/join" | "/j" => {
            if arg.is_empty() {
                InputAction::Unknown("/join requires a contact or group name".to_string())
            } else {
                InputAction::Join(arg)
            }
        }
        "/part" | "/p" => InputAction::Part,
        "/quit" | "/q" => InputAction::Quit,
        "/sidebar" | "/sb" => InputAction::ToggleSidebar,
        "/help" | "/h" => InputAction::Help,
        _ => InputAction::Unknown(format!("Unknown command: {cmd}")),
    }
}

pub const HELP_TEXT: &str = "\
Commands:
  /join <name>  - Switch to a conversation (contact number or group name)
  /part         - Leave current conversation view
  /sidebar      - Toggle sidebar visibility
  /quit         - Exit signal-tui
  /help         - Show this help

Shortcuts:
  Tab           - Next conversation
  Shift+Tab     - Previous conversation
  PgUp/PgDn     - Scroll messages
  Ctrl+Left/Right - Resize sidebar
  Ctrl+C        - Quit

Vim Keybindings:
  Esc           - Switch to Normal mode
  i/a/I/A/o     - Switch to Insert mode
  j/k           - Scroll up/down (Normal)
  g/G           - Top/bottom of messages (Normal)
  Ctrl+D/U      - Half-page scroll (Normal)
  h/l           - Move cursor left/right (Normal)
  w/b           - Word forward/back (Normal)
  0/$           - Cursor to start/end of line (Normal)
  x             - Delete character (Normal)
  D             - Delete to end of line (Normal)
  /             - Start command input (Normal)";
