# Vim Normal Mode Keybinding Fixes

Fixes 4 related bugs in normal-mode key handling: #288, #289, #290, #291.

## Problem

Normal mode keybindings diverge from vim conventions in several ways:

1. **#288** - `j`/`k` scroll the viewport by one screen line and clear `focused_msg_index`, producing a misleading visual highlight derived from viewport position. Actions (Enter) still target the last message, not the visually highlighted one. Users expect `j`/`k` to navigate by message (the natural unit in a chat TUI).
2. **#289** - `g` immediately scrolls to top. Vim requires `gg`. Single `g` is a prefix key.
3. **#290** - `o` clears the input buffer and enters Insert mode. Vim's `o` opens a new line below without destroying existing text.
4. **#291** - `d` immediately opens delete confirmation. Vim's `d` is an operator prefix requiring a motion (`dd` to delete current line). Users typing `dd` trigger delete on first `d`, then the second `d` may confirm the dialog.

## Design

### New state: pending_normal_key

Add `pending_normal_key: Option<char>` to `App`. This holds a prefix key (`g` or `d`) waiting for its second keypress.

### Key handling flow in handle_normal_key()

1. If `pending_normal_key` is `Some(prev)`, check the incoming key:
   - `g` + `g` -> execute `ScrollToTop` action
   - `d` + `d` -> execute `DeleteMessage` action
   - Any other key -> clear pending, process the new key normally through the keybinding resolver
   - `Esc` -> clear pending, do nothing
2. If no pending key, resolve via keybindings as today. But `g` and `d` are removed from the keybinding map and instead set `pending_normal_key` in the match's fallback arm.

### Keybinding map changes (keybindings.rs)

Removed bindings:
- `J` (Shift+j) -> `FocusNextMessage`
- `K` (Shift+k) -> `FocusPrevMessage`
- `g` -> `ScrollToTop` (now handled by pending-key logic)
- `d` -> `DeleteMessage` (now handled by pending-key logic)

Changed bindings:
- `j` -> `FocusNextMessage` (was `ScrollDown`)
- `k` -> `FocusPrevMessage` (was `ScrollUp`)

Added bindings:
- `Ctrl-E` -> `ScrollDown` (single-line viewport scroll, replaces old `j`)
- `Ctrl-Y` -> `ScrollUp` (single-line viewport scroll, replaces old `k`)

Unchanged bindings:
- `G` -> `ScrollToBottom`
- `Ctrl-D` -> `HalfPageDown`
- `Ctrl-U` -> `HalfPageUp`

### Pending key cleared on mode transitions

`pending_normal_key` must be cleared on any mode transition - not just on second-keypress or Esc. This includes: entering Insert mode (via `i`, `a`, `A`, `I`, `o`, or any other Insert-entering action), overlay open, and returning to Normal mode via `ExitInsert`. This prevents stale prefix state from persisting across mode boundaries.

### Fix o (OpenLineBelow)

Change the `OpenLineBelow` handler to move cursor to end of current line, insert `\n`, and enter Insert mode - matching vim's `o` semantics. Do not clear the input buffer.

Also update the `action_label` for `OpenLineBelow` in `keybindings.rs` from `"Clear & insert"` to `"Open line below"`.

### Keybindings overlay display

After removing `g` and `d` from the binding map, `ScrollToTop` and `DeleteMessage` will appear unbound in the `/keybindings` overlay. Add display entries showing `gg` and `dd` as multi-key sequences so users can discover these bindings.

### Viewport-only scroll and highlight

`Ctrl-E`/`Ctrl-Y` and `Ctrl-D`/`Ctrl-U` clear `focused_msg_index` (existing behavior). With the derived-highlight removed, these keys will scroll the viewport with no highlight shown. This is intentional - the highlight only appears when the user explicitly focuses a message with `j`/`k`.

### Known limitation: pending keys bypass keybinding resolver

The `g` and `d` prefix handling is hardcoded in `handle_normal_key()` rather than routed through the keybinding resolver. Users who customize bindings via `keybindings.toml` cannot remap these prefix sequences. This is acceptable for the scope of these fixes. A future enhancement could introduce a `KeyAction::Prefix(char)` variant to make prefix keys configurable.

### UI changes (ui.rs)

1. Remove the derived-highlight logic: the `else` branch in the normal-mode rendering that fakes a highlight from viewport position when `focused_msg_index` is `None`. Only show a highlight when `focused_msg_index` is actually set.
2. Show pending key in the status bar - display the pending prefix character (e.g., `g` or `d`) so the user knows the app is waiting for a second keypress.

### Testing

New unit tests (there are no existing normal-mode key-handling tests to update):
- `j`/`k` call `jump_to_adjacent_message` and set `focused_msg_index`
- `gg` scrolls to top, `dd` opens delete confirm
- `g` + other key discards pending and processes the other key normally
- `d` + Esc clears pending without action
- `o` preserves input buffer content and inserts newline
- Pending key is cleared on mode transition to Insert
- `Ctrl-E`/`Ctrl-Y` scroll viewport without setting focus

## Files changed

- `src/app.rs` - Add `pending_normal_key` field, update `handle_normal_key()`, fix `OpenLineBelow`
- `src/keybindings.rs` - Update default binding map, update `OpenLineBelow` action label, add `gg`/`dd` display entries
- `src/ui.rs` - Remove derived-highlight, add pending-key status bar indicator

## Issues closed

- closes #288
- closes #289
- closes #290
- closes #291
