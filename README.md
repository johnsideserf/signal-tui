# signal-tui

A terminal-based Signal client with an IRC aesthetic. Wraps [signal-cli](https://github.com/AsamK/signal-cli) via JSON-RPC for the messaging backend.

```
┌─Channels──┬─ #group-chat ──────────────────────┐
│ > alice    │ [12:34] <bob> hey everyone          │
│   bob      │ [12:35] <alice> what's up           │
│   #work    │ [12:36] <bob> check this out        │
│   #friends │ [12:36] <bob> [attachment: pic.jpg]  │
│            │                                      │
│            ├──────────────────────────────────────┤
│            │ > your message here_                 │
├────────────┴──────────────────────────────────────┤
│ connected | #group-chat | 4 members               │
└───────────────────────────────────────────────────┘
```

## Prerequisites

- [signal-cli](https://github.com/AsamK/signal-cli) installed and registered/linked to a Signal account
- Rust toolchain (1.70+)

## Building

```sh
cargo build --release
```

## Usage

```sh
# Use account from config file
signal-tui

# Specify account on command line
signal-tui -a +15551234567

# Custom config path
signal-tui -c /path/to/config.toml
```

## Configuration

Config is loaded from `~/.config/signal-tui/config.toml`:

```toml
account = "+15551234567"
signal_cli_path = "signal-cli"
download_dir = "/home/user/signal-downloads"
```

All fields are optional. `signal_cli_path` defaults to `"signal-cli"` (found via PATH), and `download_dir` defaults to `~/signal-downloads/`.

## Commands

| Command | Description |
|---|---|
| `/join <name>` | Switch to a conversation (contact number or group name) |
| `/part` | Leave current conversation view |
| `/sidebar` | Toggle sidebar visibility |
| `/quit` | Exit signal-tui |
| `/help` | Show help |

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `Tab` | Next conversation |
| `Shift+Tab` | Previous conversation |
| `PgUp` / `PgDn` | Scroll messages |
| `Ctrl+C` | Quit |

## Architecture

```
┌──────────┐   mpsc channels   ┌──────────────┐
│  TUI     │ <───────────────> │  Signal      │
│  (main   │   SignalEvent     │  Backend     │
│  thread) │   UserCommand     │  (tokio task)│
└──────────┘                   └──────┬───────┘
                                      │
                               stdin/stdout
                                      │
                               ┌──────▼───────┐
                               │  signal-cli  │
                               │  (child proc)│
                               └──────────────┘
```

## License

MIT
