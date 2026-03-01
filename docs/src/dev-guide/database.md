# Database Schema

signal-tui uses SQLite with WAL (Write-Ahead Logging) mode for safe concurrent
reads/writes. The database file is stored alongside the config file.

## Tables

### `schema_version`

Tracks the current migration version.

```sql
CREATE TABLE schema_version (
    version INTEGER NOT NULL
);
```

### `conversations`

One row per conversation (1:1 or group).

```sql
CREATE TABLE conversations (
    id         TEXT PRIMARY KEY,      -- phone number or group ID
    name       TEXT NOT NULL,         -- display name
    is_group   INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    muted      INTEGER NOT NULL DEFAULT 0   -- added in migration v2
);
```

The `id` is a phone number (E.164 format) for 1:1 conversations or a
base64-encoded group ID for groups.

### `messages`

All messages, ordered by insertion rowid.

```sql
CREATE TABLE messages (
    rowid           INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL REFERENCES conversations(id),
    sender          TEXT NOT NULL,       -- sender phone or empty for system
    timestamp       TEXT NOT NULL,       -- RFC 3339 timestamp
    body            TEXT NOT NULL,       -- message text
    is_system       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_messages_conv_ts ON messages(conversation_id, timestamp);
```

System messages (`is_system = 1`) are used for join/leave notifications and
are excluded from unread counts.

### `read_markers`

Tracks the last-read message per conversation for unread counting.

```sql
CREATE TABLE read_markers (
    conversation_id TEXT PRIMARY KEY REFERENCES conversations(id),
    last_read_rowid INTEGER NOT NULL DEFAULT 0
);
```

Unread count = messages with `rowid > last_read_rowid` and `is_system = 0`.

## Migrations

Migrations are version-based and run sequentially in `Database::migrate()`:

| Version | Changes |
|---|---|
| 1 | Initial schema: `conversations`, `messages`, `read_markers` tables |
| 2 | Add `muted` column to `conversations` |

Each migration is wrapped in a transaction. The `schema_version` table tracks
the current version.

## WAL mode

WAL mode is enabled on every connection:

```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
```

WAL allows concurrent readers while a writer is active, preventing database
locks during normal operation.

## In-memory mode

When running with `--incognito`, `Database::open_in_memory()` is used instead
of `Database::open()`. The same schema and migrations apply, but everything
lives in memory and is lost on exit.
