# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                    # dev build
cargo build --release          # release build
cargo test                     # run all tests
cargo test app::tests          # run app module tests only
cargo test signal::client::tests  # run signal client tests only
cargo test <test_name>         # run a single test by name
```

## Architecture

Terminal Signal messenger client wrapping signal-cli via JSON-RPC. Built on Tokio async runtime with Ratatui TUI.

### Data Flow

```
Keyboard → InputAction → App state → SignalClient (mpsc) → signal-cli (JSON-RPC over stdin/stdout)
signal-cli → JsonRpcResponse → SignalEvent (mpsc) → App state → SQLite + Ratatui render
```

### Key Modules

- **main.rs** — Event loop: polls keyboard (50ms), drains signal events, renders each frame. Orchestrates setup wizard → device linking → app startup.
- **app.rs** — All application state. `App` owns conversations (HashMap + ordered Vec for sidebar), input buffer, mode (Normal/Insert). `handle_signal_event()` is the single entry point for all backend events.
- **signal/client.rs** — Spawns signal-cli child process. Two tokio tasks: stdout reader (parses JSON-RPC into `SignalEvent`), stdin writer (sends `JsonRpcRequest`). `pending_requests` map tracks RPC call IDs to correlate responses with their method.
- **signal/types.rs** — Shared types: `SignalEvent` enum, `SignalMessage`, `Contact`, `Group`, JSON-RPC structs.
- **ui.rs** — Stateless rendering. `draw()` takes `&App` and renders sidebar + chat + status bar. Sender colors are hash-based (8 colors). Groups prefixed with `#`.
- **db.rs** — SQLite with WAL mode. Three tables: `conversations`, `messages`, `read_markers`. Schema migration is version-based.
- **config.rs** — TOML config at platform-specific path. Fields: `account` (E.164 phone), `signal_cli_path`, `download_dir`.
- **input.rs** — Parses text input into `InputAction` enum. Commands: `/join`, `/part`, `/quit`, `/sidebar`, `/help`.
- **setup.rs** — Multi-step first-run wizard (signal-cli detection, phone input, QR device linking).
- **link.rs** — Device linking flow with QR code display and account registration check.

### Conversations

Keyed by phone number (1:1) or group ID (groups). `get_or_create_conversation()` is the single point for ensuring a conversation exists — it upserts to both the in-memory HashMap and SQLite. New conversations append to `conversation_order`; existing ones are no-ops.

### Signal-CLI Communication

Notifications (incoming messages, typing, receipts) arrive as JSON-RPC requests with a `method` field. RPC responses (listContacts, listGroups) arrive with a `result` field and are matched by request ID via `pending_requests`. Both flows produce `SignalEvent` variants sent through the same mpsc channel.

### Modal Input

Insert mode (default) for typing messages. Normal mode (Esc) for vim-style navigation: j/k scroll, h/l cursor, w/b word movement, i/a/I/A/o to re-enter Insert.
