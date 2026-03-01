# Testing

## Running tests

Run the full test suite:

```sh
cargo test
```

Run tests for a specific module:

```sh
cargo test app::tests          # App module tests
cargo test signal::client::tests  # Signal client tests
cargo test db::tests           # Database tests
cargo test input::tests        # Input parsing tests
```

Run a single test by name:

```sh
cargo test test_name
```

## Test modules

Tests are defined as `#[cfg(test)] mod tests` blocks within each source file.

### `db.rs` tests

Database tests use `Database::open_in_memory()` for isolated, fast test instances.
Coverage includes:

- Schema migration and table creation
- Conversation upsert and loading
- Name updates on conflict
- Message insertion and retrieval (ordering)
- Unread count with read markers
- System message exclusion from unread counts
- Conversation ordering by most recent message
- Mute flag round-trip
- Last message rowid tracking

### `input.rs` tests

Input parser tests cover:

- Plain text passthrough
- Empty and whitespace-only input
- All commands and their aliases (`/join`, `/j`, `/part`, `/p`, etc.)
- Commands with and without arguments
- Unknown command handling

### `app.rs` tests

Application state tests cover signal event handling, conversation management,
and mode transitions.

### `signal/client.rs` tests

Signal client tests cover JSON-RPC parsing and event routing.

## Demo mode for manual testing

```sh
cargo run -- --demo
```

Demo mode populates the UI with dummy conversations and messages without
requiring signal-cli. This is the easiest way to manually test UI changes,
keybindings, and rendering.

## Linting

The project enforces zero clippy warnings:

```sh
cargo clippy --tests -- -D warnings
```

CI runs this on every push and pull request. Fix all warnings before pushing.
