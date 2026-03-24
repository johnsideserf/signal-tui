# Vim Normal Mode Keybinding Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 4 normal-mode keybinding bugs (#288, #289, #290, #291) to match vim conventions.

**Architecture:** Add a `pending_normal_key: Option<char>` field to `App` for `gg`/`dd` prefix sequences. Remap `j`/`k` from line-scroll to message-focus navigation. Fix `o` to insert a newline instead of clearing the buffer. Remove the derived-highlight logic from the renderer.

**Tech Stack:** Rust, Crossterm (KeyEvent), Ratatui

**Spec:** `docs/superpowers/specs/2026-03-24-vim-keybinding-bugs-design.md`

---

## File Map

- **Modify:** `src/keybindings.rs` - Change default binding map, update action label
- **Modify:** `src/app.rs` - Add `pending_normal_key` field, rewrite `handle_normal_key()` prefix logic, fix `OpenLineBelow`, add tests
- **Modify:** `src/ui.rs` - Remove derived-highlight, show pending key in status bar

---

### Task 1: Update keybinding map

**Files:**
- Modify: `src/keybindings.rs:509` (action label)
- Modify: `src/keybindings.rs:564-572` (default_profile normal scroll bindings)
- Modify: `src/keybindings.rs:598` (delete binding)

- [ ] **Step 1: Change j/k bindings and remove J/K/g/d**

In `default_profile()` (line 564-572), replace the scroll section:

```rust
// --- Normal: scroll ---
bind(&mut normal, KeyModifiers::NONE, KeyCode::Char('j'), KeyAction::FocusNextMessage);
bind(&mut normal, KeyModifiers::NONE, KeyCode::Char('k'), KeyAction::FocusPrevMessage);
bind(&mut normal, KeyModifiers::CONTROL, KeyCode::Char('d'), KeyAction::HalfPageDown);
bind(&mut normal, KeyModifiers::CONTROL, KeyCode::Char('u'), KeyAction::HalfPageUp);
bind(&mut normal, KeyModifiers::CONTROL, KeyCode::Char('e'), KeyAction::ScrollDown);
bind(&mut normal, KeyModifiers::CONTROL, KeyCode::Char('y'), KeyAction::ScrollUp);
bind(&mut normal, KeyModifiers::NONE, KeyCode::Char('G'), KeyAction::ScrollToBottom);
```

Removed: `J` -> `FocusNextMessage`, `K` -> `FocusPrevMessage`, `g` -> `ScrollToTop`.

In the actions section (line 598), remove the `d` -> `DeleteMessage` binding (delete that line entirely).

- [ ] **Step 2: Update OpenLineBelow action label**

At line 509, change:
```rust
KeyAction::OpenLineBelow => "Clear & insert",
```
to:
```rust
KeyAction::OpenLineBelow => "Open line below",
```

- [ ] **Step 3: Fix keybinding tests that reference old bindings**

Several tests in `src/keybindings.rs` reference the old `j`/`k` -> `ScrollDown`/`ScrollUp` mapping. Update them:

**`default_profile_resolves_j_in_normal` (line 991):** Change assertion from `ScrollDown` to `FocusNextMessage`:
```rust
Some(KeyAction::FocusNextMessage)
```

**`display_key_for_action` (line 1096):** Change `ScrollDown` display assertion. `ScrollDown` is now on `Ctrl+e`:
```rust
assert_eq!(kb.display_key(KeyAction::ScrollDown), "Ctrl+e");
```

**`rebind_works` (line 1102):** This rebinds `ScrollDown` to `ctrl+j` then checks old `j` is gone. After our change, `j` is `FocusNextMessage` (not `ScrollDown`), so after rebinding `ScrollDown` away, `j` still resolves to `FocusNextMessage`. Change the assertion at line 1113-1116:
```rust
// Old ScrollDown binding (ctrl+e) should be gone, but j is now FocusNextMessage
assert_eq!(
    kb.resolve(KeyModifiers::NONE, KeyCode::Char('j'), BindingMode::Normal),
    Some(KeyAction::FocusNextMessage)
);
```

**`rebind_detects_conflict` (line 1120):** This rebinds `ScrollDown` to `k`. After our change, `k` is `FocusPrevMessage`. Fix comment and assertion:
```rust
// Rebind ScrollDown to 'k' which is already FocusPrevMessage
let new_combo = parse_key_combo("k").unwrap();
let displaced = kb.rebind(BindingMode::Normal, KeyAction::ScrollDown, new_combo);
assert_eq!(displaced, Some(KeyAction::FocusPrevMessage));
```

**`reset_action_restores_default` (line 1128):** This rebinds `ScrollDown` then resets it. After our change, `ScrollDown` default is `Ctrl+e`, not `j`. The test checks `j` resolves to `None` after rebind, but `j` is now `FocusNextMessage`. Update:
```rust
let mut kb = default_profile();
// Change ctrl+e binding (ScrollDown)
let new_combo = parse_key_combo("ctrl+j").unwrap();
kb.rebind(BindingMode::Normal, KeyAction::ScrollDown, new_combo);
// Now ctrl+e shouldn't resolve to ScrollDown
assert_eq!(kb.resolve(KeyModifiers::CONTROL, KeyCode::Char('e'), BindingMode::Normal), None);
// Reset
kb.reset_action(BindingMode::Normal, KeyAction::ScrollDown);
assert_eq!(
    kb.resolve(KeyModifiers::CONTROL, KeyCode::Char('e'), BindingMode::Normal),
    Some(KeyAction::ScrollDown)
);
```

- [ ] **Step 4: Run tests to verify keybinding tests pass**

Run: `cargo test keybindings -- --nocapture`

Expected: All keybinding tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/keybindings.rs
git commit -m "refactor: remap j/k to message focus, remove J/K/g/d bindings (#288, #289, #291)"
```

---

### Task 2: Add pending_normal_key and prefix logic

**Files:**
- Modify: `src/app.rs:422` (add field near `focused_msg_index`)
- Modify: `src/app.rs:2627` (initialize in `App::new`)
- Modify: `src/app.rs:3167-3354` (`handle_normal_key`)

- [ ] **Step 1: Add the field to App**

After `focused_msg_index` (line 422), add:
```rust
/// Pending normal-mode prefix key (e.g. first `g` of `gg`, first `d` of `dd`)
pub pending_normal_key: Option<char>,
```

In `App::new` (after `focused_msg_index: None` at line 2627), add:
```rust
pending_normal_key: None,
```

- [ ] **Step 2: Add pending-key prefix handling to handle_normal_key**

At the top of `handle_normal_key()` (line 3167), before the existing `match`, insert pending-key handling:

```rust
pub fn handle_normal_key(&mut self, modifiers: KeyModifiers, code: KeyCode) -> Option<SendRequest> {
    // Handle pending prefix key (gg, dd sequences)
    if let Some(prev) = self.pending_normal_key.take() {
        match (prev, code) {
            ('g', KeyCode::Char('g')) => {
                // gg = scroll to top
                if let Some(ref id) = self.active_conversation {
                    if let Some(conv) = self.conversations.get(id) {
                        self.scroll_offset = conv.messages.len();
                    }
                }
                self.focused_msg_index = None;
                return None;
            }
            ('d', KeyCode::Char('d')) => {
                // dd = delete message
                if let Some(msg) = self.selected_message() {
                    if !msg.is_system && !msg.is_deleted {
                        self.show_delete_confirm = true;
                    }
                }
                return None;
            }
            (_, KeyCode::Esc) => {
                // Esc cancels pending prefix
                return None;
            }
            _ => {
                // Not a valid sequence — fall through to process this key normally
            }
        }
    }

    match self.keybindings.resolve(modifiers, code, BindingMode::Normal) {
        // ... existing match arms (minus ScrollToTop and DeleteMessage which are removed)
```

- [ ] **Step 3: Remove the ScrollToTop and DeleteMessage arms from the match**

In the existing match block, remove:
- The `Some(KeyAction::ScrollToTop)` arm (lines 3176-3184) - now handled by `gg` prefix
- The `Some(KeyAction::DeleteMessage)` arm (lines 3327-3334) - now handled by `dd` prefix

- [ ] **Step 4: Add fallback for g and d keys to set pending state**

In the `_ => None` catch-all at the bottom of the match (line 3353), replace with:

```rust
_ => {
    // Handle prefix keys that aren't in the binding map
    if let KeyCode::Char(c @ ('g' | 'd')) = code {
        if modifiers.is_empty() {
            self.pending_normal_key = Some(c);
        }
    }
    None
}
```

- [ ] **Step 5: Clear pending_normal_key on mode transitions**

The `pending_normal_key.take()` at the top of `handle_normal_key` already clears it when any key is pressed in normal mode. For mode transitions triggered by other paths (overlay open, etc.), add a defensive reset in `handle_insert_key`'s `ExitInsert` arm at `src/app.rs:3361`:

```rust
Some(KeyAction::ExitInsert) => {
    self.mode = InputMode::Normal;
    self.pending_normal_key = None; // defensive reset
    self.autocomplete_visible = false;
    // ... rest unchanged
```

- [ ] **Step 6: Verify build compiles**

Run: `cargo build 2>&1 | head -20`

Expected: Successful compilation (or warnings only).

- [ ] **Step 7: Commit**

```bash
git add src/app.rs
git commit -m "feat: add pending-key prefix for gg/dd vim sequences (#289, #291)"
```

---

### Task 3: Fix OpenLineBelow (o key)

**Files:**
- Modify: `src/app.rs:3195` (OpenLineBelow handler)

- [ ] **Step 1: Fix the OpenLineBelow handler**

At line 3195, change:
```rust
Some(KeyAction::OpenLineBelow) => { self.input_buffer.clear(); self.input_cursor = 0; self.mode = InputMode::Insert; None }
```
to:
```rust
Some(KeyAction::OpenLineBelow) => {
    let line_end = self.current_line_end();
    self.input_cursor = line_end;
    self.input_buffer.insert(self.input_cursor, '\n');
    self.input_cursor += 1;
    self.mode = InputMode::Insert;
    None
}
```

This moves to end of current line, inserts a newline, and enters Insert mode - matching vim's `o` behavior.

- [ ] **Step 2: Verify build compiles**

Run: `cargo build 2>&1 | head -20`

Expected: Successful compilation.

- [ ] **Step 3: Commit**

```bash
git add src/app.rs
git commit -m "fix: o key inserts newline instead of clearing input buffer (#290)"
```

---

### Task 4: Remove derived-highlight and show pending key in status bar

**Files:**
- Modify: `src/ui.rs:1265-1271` (derived-highlight block)
- Modify: `src/ui.rs:2130-2143` (status bar mode indicator)

- [ ] **Step 1: Remove the derived-highlight else branch**

At lines 1265-1272 in `ui.rs`, replace the else branch:

```rust
        } else {
            // j/k line-scroll without J/K — derive focus from viewport for display only.
            // Do NOT store into focused_msg_index; that would cause the "ensure visible"
            // logic on the next frame to snap the viewport back to the bottom.
            let idx = find_focused_msg_index(&lines, &line_msg_idx, inner_width, scroll_y, available_height);
            app.focused_message_time = idx.and_then(|i| messages.get(i)).map(|m| m.timestamp);
            render_focus = idx;
        }
```

with:

```rust
        } else {
            // Viewport-only scroll (Ctrl-E/Y, Ctrl-D/U) — no highlight without explicit focus.
            render_focus = None;
        }
```

- [ ] **Step 2: Show pending key in status bar**

In `draw_status_bar` (line 2130-2143), after the mode indicator match, add pending-key display. Change the Normal mode arm:

```rust
        InputMode::Normal => {
            let label = if let Some(pk) = app.pending_normal_key {
                format!(" [NORMAL] {pk}")
            } else {
                " [NORMAL] ".to_string()
            };
            segments.push(Span::styled(
                label,
                Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD),
            ));
        }
```

- [ ] **Step 3: Verify build compiles**

Run: `cargo build 2>&1 | head -20`

Expected: Successful compilation.

- [ ] **Step 4: Commit**

```bash
git add src/ui.rs
git commit -m "fix: remove misleading scroll highlight, show pending key in status bar (#288)"
```

---

### Task 5: Add keybindings overlay display for gg/dd

**Files:**
- Modify: `src/ui.rs:3360-3367` (keybindings overlay key display)

- [ ] **Step 1: Add special-case display for multi-key sequences**

In `draw_keybindings` (line 3360-3367), the key display section currently reads:

```rust
let key_display = if is_selected && app.keybindings_overlay.capturing {
    "[Press key...]".to_string()
} else {
    app.keybindings.display_key(action)
};
```

Change to:

```rust
let key_display = if is_selected && app.keybindings_overlay.capturing {
    "[Press key...]".to_string()
} else {
    // Multi-key sequences not in the binding map
    match action {
        KeyAction::ScrollToTop => "gg".to_string(),
        KeyAction::DeleteMessage => "dd".to_string(),
        _ => app.keybindings.display_key(action),
    }
};
```

Note: `ScrollToTop` and `DeleteMessage` are no longer in the binding map, so `display_key()` would return `"?"`. This override shows the correct multi-key sequence instead.

- [ ] **Step 2: Verify build compiles**

Run: `cargo build 2>&1 | head -20`

Expected: Successful compilation.

- [ ] **Step 3: Commit**

```bash
git add src/ui.rs
git commit -m "fix: show gg/dd in keybindings overlay for multi-key sequences"
```

---

### Task 6: Add tests

**Files:**
- Modify: `src/app.rs` (tests module starting at line 7065)

Uses the existing `app()` rstest fixture (line 7072) and `make_msg()` helper (line 9680). Add messages via `handle_signal_event(SignalEvent::MessageReceived(...))` to populate conversations, matching the existing test pattern.

- [ ] **Step 1: Write tests for pending-key sequences**

Add to the `#[cfg(test)] mod tests` block in `src/app.rs`. Uses `use crossterm::event::{KeyCode, KeyModifiers};` (add to the test imports if not already present):

```rust
#[rstest]
fn gg_scrolls_to_top(mut app: App) {
    // Populate a conversation with messages
    for i in 0..20 {
        let msg = make_msg("+1", Some(&format!("msg {i}")), None, false);
        app.handle_signal_event(SignalEvent::MessageReceived(msg));
    }
    app.active_conversation = Some("+1".to_string());
    app.scroll_offset = 0;
    app.mode = InputMode::Normal;

    // First g sets pending
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('g'));
    assert_eq!(app.pending_normal_key, Some('g'));

    // Second g scrolls to top
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('g'));
    assert_eq!(app.pending_normal_key, None);
    assert_eq!(app.scroll_offset, 20); // messages.len()
}

#[rstest]
fn dd_shows_delete_confirm(mut app: App) {
    let msg = make_msg("+1", Some("hello"), None, false);
    app.handle_signal_event(SignalEvent::MessageReceived(msg));
    app.active_conversation = Some("+1".to_string());
    app.mode = InputMode::Normal;

    // First d sets pending
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('d'));
    assert_eq!(app.pending_normal_key, Some('d'));
    assert!(!app.show_delete_confirm);

    // Second d triggers delete confirm
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('d'));
    assert_eq!(app.pending_normal_key, None);
    assert!(app.show_delete_confirm);
}

#[rstest]
fn pending_key_cancelled_by_esc(mut app: App) {
    app.mode = InputMode::Normal;
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('g'));
    assert_eq!(app.pending_normal_key, Some('g'));

    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Esc);
    assert_eq!(app.pending_normal_key, None);
}

#[rstest]
fn pending_key_discarded_on_other_key(mut app: App) {
    let msg = make_msg("+1", Some("hello"), None, false);
    app.handle_signal_event(SignalEvent::MessageReceived(msg));
    app.active_conversation = Some("+1".to_string());
    app.mode = InputMode::Normal;

    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('g'));
    assert_eq!(app.pending_normal_key, Some('g'));

    // Pressing 'j' clears pending and processes j normally
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('j'));
    assert_eq!(app.pending_normal_key, None);
}

#[rstest]
fn o_preserves_input_buffer(mut app: App) {
    app.mode = InputMode::Normal;
    app.input_buffer = "hello world".to_string();
    app.input_cursor = 5;

    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('o'));

    assert_eq!(app.mode, InputMode::Insert);
    // current_line_end() returns 11 (end of "hello world"), newline inserted there
    assert_eq!(app.input_buffer, "hello world\n");
    assert_eq!(app.input_cursor, 12);
}

#[rstest]
fn jk_focus_messages(mut app: App) {
    for i in 0..5 {
        let msg = make_msg("+1", Some(&format!("msg {i}")), None, false);
        app.handle_signal_event(SignalEvent::MessageReceived(msg));
    }
    app.active_conversation = Some("+1".to_string());
    app.mode = InputMode::Normal;

    // k (FocusPrevMessage) should invoke jump_to_adjacent_message
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('k'));
    assert!(app.focused_msg_index.is_some());
}

#[rstest]
fn pending_key_cleared_on_mode_transition(mut app: App) {
    app.mode = InputMode::Normal;

    // Press g to set pending
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('g'));
    assert_eq!(app.pending_normal_key, Some('g'));

    // Press i to enter Insert mode — pending should be cleared
    app.handle_normal_key(KeyModifiers::NONE, KeyCode::Char('i'));
    assert_eq!(app.pending_normal_key, None);
    assert_eq!(app.mode, InputMode::Insert);
}

#[rstest]
fn ctrl_e_scrolls_without_focus(mut app: App) {
    for i in 0..20 {
        let msg = make_msg("+1", Some(&format!("msg {i}")), None, false);
        app.handle_signal_event(SignalEvent::MessageReceived(msg));
    }
    app.active_conversation = Some("+1".to_string());
    app.mode = InputMode::Normal;
    app.scroll_offset = 5;
    app.focused_msg_index = Some(10);

    // Ctrl-E (ScrollDown) should scroll viewport and clear focus
    app.handle_normal_key(KeyModifiers::CONTROL, KeyCode::Char('e'));
    assert_eq!(app.scroll_offset, 4);
    assert_eq!(app.focused_msg_index, None);
}
```

- [ ] **Step 2: Run the new tests**

Run: `cargo test gg_scrolls_to_top dd_shows_delete_confirm pending_key_cancelled pending_key_discarded o_preserves jk_focus -- --nocapture`

Expected: All pass.

- [ ] **Step 3: Run full test suite**

Run: `cargo test`

Expected: All tests pass. If snapshot tests fail due to changed status bar rendering (pending key display), update with `cargo insta review`.

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "test: add tests for vim normal mode keybinding fixes (#288, #289, #290, #291)"
```

---

### Task 7: Final verification

- [ ] **Step 1: Run clippy**

Run: `cargo clippy --tests -- -D warnings`

Expected: No errors or warnings.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`

Expected: All tests pass.

- [ ] **Step 3: Manual smoke test (optional)**

Build and run: `cargo run`
- Enter Normal mode (Esc)
- Press `j`/`k` - should navigate by message with highlight
- Press `gg` - should scroll to top
- Press `G` - should scroll to bottom
- Press `dd` on a message - should show delete confirm
- Press `d` then `Esc` - should cancel
- Type some text, press Esc, press `o` - should insert newline without clearing
- Check `[NORMAL] g` shows in status bar when `g` is pending
