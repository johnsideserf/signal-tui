# Features

## Messaging

Send and receive 1:1 and group messages. Messages sent from your phone (or other
linked devices) sync into the TUI automatically.

## Attachments

- **Images** -- rendered inline as halfblock art when `inline_images = true`
- **Other files** -- shown as `[attachment: filename]` with the download path

Received attachments are saved to the `download_dir` configured in your config file
(default: `~/signal-downloads/`).

## Clickable links

URLs and file paths in messages are rendered as
[OSC 8 hyperlinks](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda).
In supported terminals (Windows Terminal, iTerm2, Kitty, etc.), you can click them
to open in your browser.

## Typing indicators

When someone is typing, their name appears below the chat area. Contact name
resolution is used where available.

## Persistence

All conversations, messages, and read markers are stored in a SQLite database with
WAL (Write-Ahead Logging) mode for safe concurrent access. Data survives app restarts.

The database is stored alongside the config file:
- **Linux / macOS:** `~/.config/signal-tui/signal-tui.db`
- **Windows:** `%APPDATA%\signal-tui\signal-tui.db`

## Unread tracking

The sidebar shows unread counts next to each conversation. When you open a
conversation, a "new messages" separator line marks where you left off. Read
markers persist across restarts.

## Notifications

Terminal bell notifications fire when new messages arrive in background
conversations. Configure them per type:

- `notify_direct` -- 1:1 messages (default: on)
- `notify_group` -- group messages (default: on)
- `/mute` -- per-conversation mute (persists in the database)
- `/bell` -- toggle notification types at runtime

## Contact resolution

On startup, signal-tui requests your contact list and group list from signal-cli.
Names from your Signal address book are used throughout the sidebar, chat area,
and typing indicators.

## Responsive layout

The sidebar auto-hides on narrow terminals (less than 60 columns). Use
`Ctrl+Left` / `Ctrl+Right` to resize it, or `/sidebar` to toggle it.

## Incognito mode

```sh
signal-tui --incognito
```

Uses an in-memory database instead of on-disk SQLite. No messages, conversations,
or read markers are written to disk. The status bar shows a bold magenta
**incognito** indicator. When you exit, everything is gone.

## Demo mode

```sh
signal-tui --demo
```

Launches with dummy conversations and messages. No signal-cli process is spawned.
Useful for testing the UI, exploring keybindings, and taking screenshots.
