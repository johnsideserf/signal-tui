# Roadmap

## Completed

- [x] Send and receive plain text messages (1:1 and group)
- [x] Receive file attachments (displayed as `[attachment: filename]`)
- [x] Typing indicators
- [x] SQLite-backed message persistence with WAL mode
- [x] Unread message counts with persistent read markers
- [x] Vim-style modal editing (Normal / Insert modes)
- [x] Responsive layout with auto-hiding sidebar
- [x] First-run setup wizard with QR device linking
- [x] TUI error screens instead of stderr crashes
- [x] Commands: `/join`, `/part`, `/quit`, `/sidebar`, `/help`
- [x] Load contacts and groups on startup (name resolution, groups in sidebar)
- [x] Echo outgoing messages from other devices via sync messages
- [x] Contact name resolution from address book
- [x] Sync request at startup to refresh data from primary device
- [x] Inline image preview for attachments (halfblock rendering)
- [x] New message notifications (terminal bell, per-type toggles, per-chat mute)
- [x] Command autocomplete with Tab completion
- [x] Settings overlay
- [x] Input history (Up/Down to recall previous messages)
- [x] Incognito mode (`--incognito`)
- [x] Demo mode (`--demo`)

## Up next

- [ ] **Delivery/read receipt display** -- receipts are already parsed but
  silently discarded. Show checkmark indicators next to messages.
- [ ] **Send attachments** -- only receiving works today. Add a `/send-file <path>`
  command.

## Future

- [ ] Message search
- [ ] Multi-line message input (Shift+Enter for newlines)
- [ ] Message history pagination (scroll-up to load older messages)
- [ ] Correct group typing indicators (per-sender-to-group mapping)
- [ ] Message reactions (emoji badges with counts)
- [ ] Message deletion and editing
- [ ] Group management (create, add/remove members, member list)
- [ ] Scroll position memory per conversation
- [ ] Configurable keybindings
