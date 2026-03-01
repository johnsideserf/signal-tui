# signal-tui Roadmap

## Completed

- [x] Send and receive plain text messages (1:1 and group)
- [x] Receive file attachments (displayed as `[attachment: filename]`)
- [x] Typing indicators
- [x] SQLite-backed message persistence with WAL mode
- [x] Unread message counts with persistent read-marker
- [x] Vim-style modal editing (Normal/Insert modes)
- [x] Responsive layout with auto-hiding sidebar
- [x] First-run setup wizard with QR device linking
- [x] TUI error screens instead of stderr crashes
- [x] Commands: `/join`, `/part`, `/quit`, `/sidebar`, `/help`

- [x] Load contacts & groups on startup (name resolution + groups in sidebar)
- [x] Echo outgoing messages from other devices via sync messages
- [x] Contact name resolution from address book
- [x] Sync request at startup to refresh data from primary device
- [x] Inline image preview for attachments (halfblock rendering)

## Up Next

- [x] **New message notifications**
  - Terminal bell + unread count in terminal title
  - Separate toggles for direct and group messages (config + `/bell` command)
  - Per-conversation `/mute` with DB persistence
  - Setup wizard preferences step

- [x] **Delivery/read receipt display**
  - Status symbols on outgoing messages (Sending → Sent → Delivered → Read → Viewed)
  - Configurable via /settings (receipts, colors, Nerd Font icons)
  - Optional `--debug` flag for protocol diagnostics

- [ ] **Send attachments**
  - Only receiving works today
  - Add `/send-file <path>` command

## Future

- [ ] Message search
- [ ] Multi-line message input (Shift+Enter for newlines)
- [ ] Message history pagination (scroll-up to load older messages)
- [ ] Correct group typing indicators (per-sender-to-group mapping)
- [ ] **Message reactions (emoji reactions)**
  - Parse `dataMessage.reaction` from signal-cli (emoji, targetAuthor, targetTimestamp, isRemove)
  - Display reactions below the target message as compact emoji badges (e.g. `👍 2  ❤️ 1`)
  - Aggregate duplicate emoji from different senders into counts
  - Handle reaction removal (`isRemove: true`) by decrementing or removing the badge
  - Store reactions in DB (new `reactions` table keyed by message + sender)
  - Re-render reaction badges on startup from DB
  - Stretch: allow sending reactions via a command (e.g. `/react 👍`)
- [ ] Message deletion and editing
- [ ] Group management (create, add/remove members, member list)
- [ ] Scroll position memory per conversation
- [ ] Configurable keybindings
