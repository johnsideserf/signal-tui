/// Metadata for a slash command (used for autocomplete + help)
pub struct CommandInfo {
    pub name: &'static str,
    pub alias: &'static str,
    pub args: &'static str,
    pub description: &'static str,
}

pub const COMMANDS: &[CommandInfo] = &[
    CommandInfo { name: "/join",     alias: "/j",  args: "<name>",  description: "Switch to a conversation" },
    CommandInfo { name: "/part",     alias: "/p",  args: "",        description: "Leave current conversation" },
    CommandInfo { name: "/sidebar",  alias: "/sb", args: "",        description: "Toggle sidebar" },
    CommandInfo { name: "/bell",     alias: "",    args: "[type]",  description: "Toggle notifications (direct/group)" },
    CommandInfo { name: "/mute",     alias: "",    args: "",        description: "Mute/unmute current chat" },
    CommandInfo { name: "/contacts", alias: "/c",  args: "",        description: "Browse contacts" },
    CommandInfo { name: "/settings", alias: "",    args: "",        description: "Open settings" },
    CommandInfo { name: "/help",     alias: "/h",  args: "",        description: "Show help" },
    CommandInfo { name: "/quit",     alias: "/q",  args: "",        description: "Exit signal-tui" },
];

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
    /// Toggle terminal bell notifications (None = both, Some("direct"/"group") = specific)
    ToggleBell(Option<String>),
    /// Mute/unmute the current conversation
    ToggleMute,
    /// Show help text
    Help,
    /// Open settings overlay
    Settings,
    /// Open contacts overlay
    Contacts,
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
        "/bell" | "/notify" => {
            if arg.is_empty() {
                InputAction::ToggleBell(None)
            } else {
                InputAction::ToggleBell(Some(arg))
            }
        }
        "/mute" => InputAction::ToggleMute,
        "/contacts" | "/c" => InputAction::Contacts,
        "/settings" => InputAction::Settings,
        "/help" | "/h" => InputAction::Help,
        _ => InputAction::Unknown(format!("Unknown command: {cmd}")),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text() {
        let InputAction::SendText(s) = parse_input("hello world") else { panic!("expected SendText") };
        assert_eq!(s, "hello world");
    }

    #[test]
    fn empty_input() {
        let InputAction::SendText(s) = parse_input("") else { panic!("expected SendText") };
        assert!(s.is_empty());
    }

    #[test]
    fn whitespace_only() {
        let InputAction::SendText(s) = parse_input("   ") else { panic!("expected SendText") };
        assert!(s.is_empty());
    }

    #[test]
    fn trimmed_text() {
        let InputAction::SendText(s) = parse_input("  hello  ") else { panic!("expected SendText") };
        assert_eq!(s, "hello");
    }

    #[test]
    fn join_with_arg() {
        let InputAction::Join(s) = parse_input("/join Alice") else { panic!("expected Join") };
        assert_eq!(s, "Alice");
    }

    #[test]
    fn join_alias() {
        let InputAction::Join(s) = parse_input("/j +1234567890") else { panic!("expected Join") };
        assert_eq!(s, "+1234567890");
    }

    #[test]
    fn join_without_arg() {
        let InputAction::Unknown(s) = parse_input("/join") else { panic!("expected Unknown") };
        assert!(s.contains("requires"));
    }

    #[test]
    fn part_command() {
        assert!(matches!(parse_input("/part"), InputAction::Part));
    }

    #[test]
    fn part_alias() {
        assert!(matches!(parse_input("/p"), InputAction::Part));
    }

    #[test]
    fn quit_command() {
        assert!(matches!(parse_input("/quit"), InputAction::Quit));
    }

    #[test]
    fn quit_alias() {
        assert!(matches!(parse_input("/q"), InputAction::Quit));
    }

    #[test]
    fn sidebar_command() {
        assert!(matches!(parse_input("/sidebar"), InputAction::ToggleSidebar));
    }

    #[test]
    fn sidebar_alias() {
        assert!(matches!(parse_input("/sb"), InputAction::ToggleSidebar));
    }

    #[test]
    fn bell_no_arg() {
        let InputAction::ToggleBell(None) = parse_input("/bell") else { panic!("expected ToggleBell(None)") };
    }

    #[test]
    fn bell_with_arg() {
        let InputAction::ToggleBell(Some(s)) = parse_input("/bell direct") else { panic!("expected ToggleBell(Some)") };
        assert_eq!(s, "direct");
    }

    #[test]
    fn notify_alias() {
        let InputAction::ToggleBell(Some(s)) = parse_input("/notify group") else { panic!("expected ToggleBell(Some)") };
        assert_eq!(s, "group");
    }

    #[test]
    fn mute_command() {
        assert!(matches!(parse_input("/mute"), InputAction::ToggleMute));
    }

    #[test]
    fn settings_command() {
        assert!(matches!(parse_input("/settings"), InputAction::Settings));
    }

    #[test]
    fn contacts_command() {
        assert!(matches!(parse_input("/contacts"), InputAction::Contacts));
    }

    #[test]
    fn contacts_alias() {
        assert!(matches!(parse_input("/c"), InputAction::Contacts));
    }

    #[test]
    fn help_command() {
        assert!(matches!(parse_input("/help"), InputAction::Help));
    }

    #[test]
    fn help_alias() {
        assert!(matches!(parse_input("/h"), InputAction::Help));
    }

    #[test]
    fn unknown_command() {
        let InputAction::Unknown(s) = parse_input("/foo") else { panic!("expected Unknown") };
        assert!(s.contains("/foo"));
    }
}
