# Changelog

## v0.3.3

### Bug fixes

- **Settings persistence** -- changes made in `/settings` are now saved
  to the config file and persist between sessions (fixes #40)
- **Input box scrolling** -- long messages no longer disappear when
  typing past the edge of the input box; text now scrolls horizontally
  to keep the cursor visible (fixes #39)
- **Image preview refresh** -- toggling "Inline image previews" in
  `/settings` now immediately re-renders or clears previews on existing
  messages (fixes #41)

### Settings

- **Tab to toggle** -- Tab key now toggles settings items in the
  `/settings` overlay, alongside Space and Enter

---

## v0.3.2

### Read receipts and delivery status

- **Message status indicators** -- outgoing messages now show delivery
  lifecycle symbols: `◌` Sending → `○` Sent → `✓` Delivered → `●` Read
  → `◉` Viewed
- **Real-time updates** -- status symbols update live as recipients
  receive and read your messages
- **Group receipt support** -- delivery and read receipts work correctly
  in group conversations
- **Race condition handling** -- receipts that arrive before the server
  confirms the send are buffered and replayed automatically
- **Persistent status** -- message status is stored in the database and
  restored on reload (stale "Sending" messages are promoted to "Sent")
- **Nerd Font icons** -- optional Nerd Font glyphs available via
  `/settings` > "Nerd Font icons"
- **Configurable** -- three new settings toggles: "Read receipts" (on/off),
  "Receipt colors" (colored/monochrome), "Nerd Font icons" (unicode/nerd)

### Debug logging

- **`--debug` flag** -- opt-in protocol logging to `signal-tui-debug.log`
  for diagnosing signal-cli communication issues

### Database

- **Migration v3** -- adds `status` and `timestamp_ms` columns to the
  messages table (automatic on first run)

---

## v0.3.1

### Image attachments

- **Embedded file links** -- attachment URIs are now hidden behind clickable
  bracket text (e.g. `[image: photo.jpg]`) instead of showing the raw
  `file:///` path
- **Double extension fix** -- filenames like `photo.jpg.jpg` are stripped to
  `photo.jpg` when signal-cli duplicates the extension
- **Improved halfblock previews** -- increased height cap from 20 to 30
  cell-rows for better inline image quality
- **Native image protocols** -- experimental support for Kitty and iTerm2
  inline image rendering, off by default. Enable via `/settings` >
  "Native images (experimental)"
- **Pre-resized encoding** -- native protocol images are resized and cached
  as PNG before sending to the terminal, avoiding multi-megabyte raw file
  transfers every frame

### Attachment lookup

- **MSYS/WSL path fix** -- `find_signal_cli_attachment` now checks both
  platform-native data dirs (`AppData/Roaming`) and POSIX-style
  (`~/.local/share`) where signal-cli stores files under MSYS or WSL.
  Fixes outgoing images sent from Signal desktop not displaying in the TUI.

### Platform

- **Windows Ctrl+C fix** -- suppress the `STATUS_CONTROL_C_EXIT` error on
  exit by disabling the default Windows console handler (crossterm already
  captures Ctrl+C as a key event in raw mode)

### Documentation

- mdBook documentation site with custom mIRC/Win95 light theme and dark mode
  toggle

---

## v0.3.0

Initial public release.

- Terminal Signal client wrapping signal-cli via JSON-RPC
- Vim-style modal input (Normal/Insert modes)
- Sidebar with conversation list, unread counts, typing indicators
- Inline halfblock image previews
- OSC 8 clickable hyperlinks
- SQLite persistence with WAL mode
- Incognito mode (`--incognito`)
- Demo mode (`--demo`)
- First-run setup wizard with QR device linking
- Slash commands: `/join`, `/part`, `/quit`, `/sidebar`, `/help`, `/settings`,
  `/mute`, `/notify`, `/bell`
- Input history (Up/Down recall)
- Autocomplete popup for commands and @mentions
- Configurable notifications (direct/group) with terminal bell
- Cross-platform: Linux, macOS, Windows
