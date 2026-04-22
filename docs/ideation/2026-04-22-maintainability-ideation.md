---
date: 2026-04-22
topic: maintainability-and-app-decomposition
focus: outstanding tech debt, monolithic app.rs (12,126 LOC, 44% of crate), long-term maintainability
mode: repo-grounded
---

# Ideation: Maintainability & App Decomposition

## Grounding Context

### Codebase state
- Rust TUI Signal client (Ratatui + Tokio + SQLite).
- `app.rs` **12,126 LOC (44% of crate)**, ~83 fields, 193 public methods.
- `ui.rs` **5,270 LOC** - second-largest module, no open tracking issue yet.
- Previous wave extracted: `ConversationStore` (530 LOC), 15 overlay state structs in `src/domain/`, `OverlayKind` enum collapsed 23 show/visible booleans.
- Hook pattern (`on_message_added`, PR #343) unified local-send and incoming-msg paths.

### Established patterns
- Domain state extraction: `#[derive(Default)]` sub-struct under `src/domain/<name>.rs`, registered in `domain/mod.rs`, mechanical rename across app.rs/ui.rs/main.rs/settings_profile.rs.
- Modal overlay: single `App.current_overlay: Option<OverlayKind>` with `open_overlay`/`close_overlay`/`try_open_overlay` entry points.
- One PR per extraction (Dowsley's rule).

### Open tracked tech debt
- **#352**: ranked extraction plan - InputState (5 fields, 116 refs, 2 external), PendingState (4, 13, 0), MouseState (6, 11), ScrollState (6, 61, 26 external - last).
- **#350**: filesystem migration test gap.
- **#326**: sync viewport fix (#313) delivered minimal perceived improvement - render pipeline coupling.
- **#202**: `KeyAction` and `InputAction` parallel type systems with no shared surface.

### Issue tracker read
Unusually healthy. 9 of 13 open issues are pure features, not tech debt. The backlog composition says the codebase exited "drowning in debt" and is in "periodic audit + targeted extraction" mode. #209 / #347 / #348 / #349 / #351 all closed within the last 6 weeks.

### External prior art
Helix (Editor / Compositor / Application split with `Context { editor, compositor, jobs }`), gitui (Component trait + Vec<Box<dyn Component>>), zellij (actor threads + typed instruction enums), Ratatui official Component template, Rust Compose Structs pattern, Niko Matsakis' view-types post, bottom PR #1558 (cautionary tale of parallel greenfield rewrite).

## Ranked Ideas

### 1. Introduce a `Cx` / `AppCore` handler context
**Description:** Replace `&mut self` on app.rs's 193 methods with free functions taking a narrow view struct, starting at `handle_signal_event` (830-line match dispatcher). `AppCore { store: &mut ConversationStore, ui: &mut UiState, signal: &mut SignalState, ... }` lets handlers take individual field refs and sidesteps the borrow-checker conflicts that block further sub-struct extraction today. Mirrors Helix's `Context { editor, compositor, jobs }` pattern - deliberately not a `Component` trait.
**Rationale:** Structural root cause of why app.rs is hard to decompose. Each extraction in #352 will hit the same "can't split-borrow through self" wall. A `Cx` struct makes split borrows the default and unlocks handler unit tests (construct Cx from narrow fakes vs full App). Apply incrementally, `handle_signal_event` first.
**Downsides:** Verbosity up; closure capture in async remains a gotcha; risk of premature abstraction if adoption stalls; `impl App` methods coexist with free functions during long transition.
**Confidence:** 80%
**Complexity:** High
**Status:** Unexplored

### 2. CI ratchet on `App` field count
**Description:** CI check comparing `App`'s field count to a committed baseline. Fails PRs that grow the field count without shrinking. The only way to add a field is to simultaneously extract one. Proactive guardrail.
**Rationale:** App grew to ~83 fields because every feature added "just one field." Every future feature (including agent-authored PRs) is now structurally forced to think "which sub-struct does this belong to?" before the PR can pass. Cheapest high-leverage lock-in move on the list.
**Downsides:** Blunt instrument; sub-struct growth bypasses it; baseline bumps needed for legit additions; could frustrate contributors if the signal is too loud.
**Confidence:** 95%
**Complexity:** Low
**Status:** Explored

### 3. Decompose `ui.rs` by layout region
**Description:** `ui.rs` is 5,270 LOC with no tracking issue. Split along draw-tree boundaries into `ui/sidebar.rs`, `ui/chat_pane.rs`, `ui/composer.rs`, `ui/status_bar.rs`, `ui/overlays/*.rs`, `ui/hyperlink_pass.rs`, `ui/theme.rs`. Keep render functions pure `fn(&State, &mut Frame, Rect)`. Avoid a `Component` trait for coupled pairs (Helix's warning).
**Rationale:** Second god object behind app.rs. Region split keeps render functions testable in isolation with existing `insta` + `TestBackend` snapshots. First step: file the tracking issue + scope the split.
**Downsides:** Second concurrent refactor front alongside #352; some render logic crosses regions and will need wider `&State` than its file implies.
**Confidence:** 90%
**Complexity:** Medium
**Status:** Unexplored

### 4. Unified redraw scheduler (close #326 structurally)
**Description:** Single `RenderScheduler` owns throttle state, a `DirtyRegions` bitset (`SIDEBAR | CHAT | INPUT | STATUS | OVERLAY`), and an adaptive poll mode (`Active` 16ms / `Idle` 200ms / `Syncing` 2Hz). One `terminal.draw(` call site by construction. A new async redraw source has no way to bypass the gate.
**Rationale:** #326 is open because #313's SyncState throttle was bypassed by the spinner redraw loop - multiple sources decide to redraw with no unified owner. Closing #326 by adding another targeted check would repeat the mistake; a single scheduler is the structural answer. Also eliminates the "everything re-renders on every key" waste.
**Downsides:** Non-trivial event loop refactor touching main.rs; region-dirty-tracking adds a mutation burden at every state-changing site; scheduler itself becomes a new concept contributors must understand.
**Confidence:** 85%
**Complexity:** Medium
**Status:** Unexplored

### 5. Conversation lifecycle state machine
**Description:** `enum ConversationState { Pending, Active, Muted, Archived, Deleted { purged: bool } }` owned by `ConversationStore`. Transitions are methods; invariants (Deleted → not in sidebar, Pending → not in DB until accepted) enforced via `debug_assert` + property tests. Single `purge_deleted()` sweeper handles orphan cleanup across messages / read_markers / attachments.
**Rationale:** #311 delete is the canonical partial-cleanup risk (HashMap, `conversation_order`, SQLite tables, focused_conv pointer, sidebar filter - forget one, get a ghost). Encoding the lifecycle once means #311, archive, mute, block, and multi-account (#260) reuse the same invariants. Multi-account specifically needs "deleted here but retained elsewhere" which the state machine gives for free.
**Downsides:** Premature if #311/#260 don't land soon; adds state transitions as a modeling surface contributors must track; needs a migration strategy for existing conversations.
**Confidence:** 75%
**Complexity:** Medium-High
**Status:** Unexplored

### 6. `cargo xtask extract-field` recipe codified
**Description:** `cargo xtask extract-field --name InputState --fields "input,cursor_x,pending_input"` scaffolds `src/domain/<name>.rs`, registers it in `mod.rs`, performs longer-prefix-first substring renames across app.rs/ui.rs/main.rs/settings_profile.rs, and leaves a single reviewable diff. The gotchas (substring collisions, SETTINGS fn pointers) become executable checks, not tribal knowledge.
**Rationale:** #352 has 4 extractions remaining and each hits the same manual gotchas. A tool turns each from a day of careful renaming into an hour of review. Compounds against future extensions (ui.rs decomposition reuses the mechanics).
**Downsides:** Tool maintenance itself becomes tech debt; ROI capped at ~4 remaining extractions unless adopted more broadly; `cargo xtask` introduces a new dev-time dependency pattern not currently in the repo.
**Confidence:** 65%
**Complexity:** Medium
**Status:** Unexplored

### 7. `--demo --headless --script=FILE` integration smoke
**Description:** Headless mode reads a JSON script of key events, feeds them through the real event loop via in-memory channels, and exits cleanly. Single integration test scripts through: open every overlay, type to filter, navigate j/k, close Esc, switch conversations, send a message, toggle every setting. Runs in under 5 seconds in CI.
**Rationale:** 16 overlay modules under `src/domain/` have zero tests today. Blanket smoke coverage in one test. Forcing function for keeping event handlers working through refactors - breaks loudly when #1 (Cx), #3 (ui.rs split), or #4 (scheduler) regress behavior.
**Downsides:** Scripts can drift from real UX; integration-test flakiness risk; building the scripting mechanism itself is a medium-sized initial cost before any test value materializes.
**Confidence:** 80%
**Complexity:** Medium-High
**Status:** Unexplored

## Rejection Summary

| # | Idea | Reason Rejected |
|---|------|-----------------|
| R1 | Split App into 3 coarse domains (SessionState/UiState/SignalState) | Conflicts with #352's finer-grained plan; coarse owners don't solve the borrow-checker problem |
| R2 | Generalize on_message_added into AppEvent bus | Overlaps with #1 Cx approach |
| R3 | Unify KeyAction + InputAction → Command enum (#202) | Valid but not most urgent; tracked as #202 already |
| R4 | Persistence module with typed DbCommand queue | Too expensive for current pain; async boundary added without observed write-latency problem |
| R5 | TestApp builder (standalone) | Subsumed by #1 Cx + #7 headless smoke |
| R6 | Golden JSON-RPC fixtures + rstest | Subsumed by #7 for overlay layer; signal-cli protocol testing is a valid separate issue but lower priority |
| R7 | proptest invariants | Valuable; file as follow-up issue tied to #1's reducer seam |
| R8 | cargo-fuzz CI job | Cheap win; file as standalone issue (separate follow-up) |
| R9 | SQLite migration round-trip tests | Already in #350 scope |
| R10 | Cross-platform CI matrix | Valuable; file as standalone issue |
| R11 | Redux-style `reduce(state, event) → Effects` full rewrite | Bottom PR #1558 anti-pattern risk; #1 + #4 give same benefits incrementally |
| R12 | Fixed-timestep render loop (game-loop pattern) | Same anti-pattern risk as R11 |
| R13 | Split data/display mpsc channels | Duplicates #4 scheduler's goal via a different mechanism |
| R14 | Extract EventLoop with injectable Clock | Subset of #4 |
| R15 | ViewportState sub-struct | Already in #352 (ScrollState extraction) |
| R16 | Ratatui TestBackend overlay-flow harness | Subset of #7 |
| R17 | Schema migration framework (generalized) | Premature abstraction; do simple round-trip tests first |
| R18 | Feature flags + experimental:: module | Premature for solo maintainer / current release cadence |
| R19 | signal-cli-client as standalone crate | Speculative; no observed ecosystem demand |
| R20 | Crate error taxonomy (thiserror/anyhow) | Not grounded in current pain; revisit with #260 |
| R21 | "Where do I put X?" decision tree in CLAUDE.md | Useful but minor; fold into #6 xtask work |
| R22 | Persistence Hardening synthesis | Component ideas rejected individually |

## Recommended Execution Order

1. Lock in progress with **#2 (CI ratchet)** - cheapest high-leverage move.
2. Finish **#352 roadmap** (InputState → PendingState → MouseState → ScrollState) before starting new structural work.
3. File the lower-priority satellite issues from the rejection pile (ui.rs tracking per #3, cross-platform CI per R10, cargo-fuzz per R8).
4. Defer the big bets (#1 Cx, #4 scheduler, #5 lifecycle) until #352 finishes - each will be cheaper once more state is already extracted.
5. Skip **#6 (xtask)** unless extraction continues past #352.
