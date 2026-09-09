---
work_package_id: WP01
title: Key-binding sets and action translation
dependencies: []
requirement_refs:
- C-001
- C-002
- C-003
- FR-021
- FR-025
- FR-027
- FR-033
subtasks:
- T001
- T002
- T003
- T004
- T005
phase: Phase 1 - Foundation
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: python-pedro
authoritative_surface: src/keymap.rs
create_intent:
- src/keymap.rs
execution_mode: code_change
model: ''
owned_files:
- src/keymap.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP01 – Key-binding sets and action translation

## ⚡ Do This First: Load Agent Profile

Use the `/ad-hoc-profile-load` skill to load the agent profile specified in the frontmatter (or any user-defined profile), and behave according to its guidance before parsing the rest of this prompt.

- **Profile**: `rust-implementer` (no dedicated Rust profile is registered in this project; use the closest general implementer profile available, e.g. `implementer-ivan`)
- **Role**: `implementer`
- **Agent/tool**: `claude`

If no profile is specified, run `spec-kitty agent profile list` and select the best match for this work package's `task_type` and `authoritative_surface`.

---

## ⚠️ IMPORTANT: Review Feedback

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_ref` field in the event log (via `spec-kitty agent tasks status` or the Activity Log below).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

*[If this WP was returned from review, the reviewer feedback reference appears in the Activity Log below or in the status event log.]*

---

## Markdown Formatting

Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ```` ```rust ````, ```` ```bash ````

---

## Objectives & Success Criteria

Build the single translation layer, `src/keymap.rs`, that turns a raw terminal key event plus the active key-binding set into a semantic action. Every later work package in this mission dispatches on the actions this file defines instead of matching raw keys directly, and the Edit menu / help window derive their key labels from it — so this is the one place C-002 ("the vim set contains no Ctrl+C/V/X/Z") can be enforced by a test rather than by discipline.

Done when:
- `KeyBindingSet { Normal, Vim }` exists with `Default = Normal`.
- `TreeAction` and `EditorAction` enums cover every action named in `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/keymap.md`.
- `translate(key: KeyEvent, set: KeyBindingSet, context: ...) -> Option<Action>` correctly maps every row of that contract's tables for the Structure pane (tree actions only in this WP — editor actions are consumed by WP08 but the enum and label plumbing for them must exist now).
- `label(action, set) -> Option<&'static str>` returns `None` exactly for actions with no key in that set (`PasteAsChild` in both sets, `Cut` in the vim set).
- `cargo test keymap::` passes, including a test that asserts no vim-set entry has `KeyModifiers::CONTROL` together with `KeyCode::Char('c'|'v'|'x'|'z')`.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` — Domain Language section (Key-binding set, Editor mode) and User Stories 2, 4, 5 for the behaviour these actions drive.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/keymap.md` — the authoritative key table for both sets; this WP implements exactly what it describes.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/research.md` R2 and R3 — why a translation layer exists, and how Shift+Up/Down and Ctrl+letters actually arrive from terminals.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/plan.md` IC-01 — the concern this WP implements.
- Existing code to study before writing: `src/tui.rs::handle_document_key` (lines ~310–348) and `handle_edit_key` (lines ~516–570) for the current raw-key matching style and the Ctrl-fold idiom already used there; `src/tui.rs::handle_content_scroll_key` for the Shift+bracket dual-encoding pattern this WP must mirror for Shift+Up/Down.

**Do not wire this file into `src/tui.rs`'s dispatch yet.** That happens in later WPs (WP03 onward), each adding its own `match` arms as its actions become meaningful. This WP only builds and unit-tests the translation layer in isolation.

**Constraint C-001**: no binding may rely on a combination terminals commonly cannot deliver (e.g. Ctrl+Shift+letter, already reserved as the terminal's own paste). Shift+Up/Down must be accepted in both the modifier form (`KeyCode::Up` + `SHIFT`) and any escape form crossterm already normalises to that same form — there is no second form to handle separately once crossterm has normalised it, but write the test using the modifier form explicitly so a future crossterm change that stops normalising is caught.

**Constraint C-002**: this is the one file whose vim table must be provably free of `Ctrl+C/V/X/Z`. Do not add a "just in case" fallback that maps those combinations to anything in the vim set, even to a no-op variant — leave them unmatched (`translate` returns `None`), so `handle_document_key`/`handle_edit_key` in later WPs simply fall through to their existing `_ => {}` arm.

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T001 [P] – Define KeyBindingSet, TreeAction, EditorAction enums

- **Purpose**: Establish the vocabulary every other WP and this file's own `translate`/`label` build on.
- **Steps**:
  1. Create `src/keymap.rs` with a module doc comment explaining its role (mirror the style of the doc comment at the top of `src/clipboard.rs` — a short "why this exists" paragraph, not just a one-liner).
  2. `#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)] pub enum KeyBindingSet { #[default] Normal, Vim }`. Add `impl std::str::FromStr` and `impl std::fmt::Display` using the strings `"normal"` and `"vim"` — WP02's settings file round-trips through these.
  3. `TreeAction`: `MarkUp, MarkDown, Delete, Cut, Copy, Yank, PasteAfter, PasteBefore, PasteAsChild`. Add a comment above the enum listing which existing keys (`i`, `I`, `e`, `E`, `J`, `K`, `z`, `/`) are deliberately **not** actions here — they stay hard-coded in `handle_document_key` because they are unchanged by this mission (FR-033) and adding them to this enum would be unnecessary churn.
  4. `EditorAction`: `EnterInsertAt, EnterInsertAfter, EnterInsertLineStart, EnterInsertLineEnd, MoveLeft, MoveRight, MoveUp, MoveDown, Home, End, DeleteUnderCursor, VisualStart, Yank, DeleteSelection, PutAfter, PutBefore, Undo, Increment, Decrement, Apply, Cancel, LeaveMode`. These are consumed by WP08; define them now so WP08 does not also need to touch this file's core enums (keeping this WP's ownership of `keymap.rs` complete).
  5. `pub enum Action { Tree(TreeAction), Editor(EditorAction) }` as the return type of `translate`.
- **Files**: `src/keymap.rs` (new).
- **Parallel?**: Yes — independent of T002's matching logic; can be written alongside T003.
- **Notes**: Every variant must be `Clone, Copy, PartialEq, Eq, Debug` so tests can compare and log them easily.

### Subtask T002 – Implement translate() for tree actions

- **Purpose**: The one function that decides what a key means, per `contracts/keymap.md`'s "Structure pane" table.
- **Steps**:
  1. Signature: `pub fn translate(key: crossterm::event::KeyEvent, set: KeyBindingSet) -> Option<Action>`. A `context` parameter (e.g. "is a mark currently active") turned out unnecessary for the tree table — every row's meaning does not depend on tree state, only on the key and the set — so omit it unless a later subtask proves otherwise; document that decision in a comment if you do omit it.
  2. Mark actions (both sets, per the contract row "Mark down / up"): `KeyCode::Down` with `key.modifiers.contains(KeyModifiers::SHIFT)` → `TreeAction::MarkDown`; `KeyCode::Up` + `SHIFT` → `TreeAction::MarkUp`.
  3. Delete (both sets): `KeyCode::Char('d')` with no modifiers → `TreeAction::Delete`. (The two-step confirmation is the caller's job, not this function's — `translate` only says "this key means Delete", every press.)
  4. Normal-set only: fold Ctrl+letter case exactly as `handle_edit_key` does today (`c.to_ascii_lowercase()`) before matching. `Ctrl+X` → `Cut`, `Ctrl+C` → `Copy`, `Ctrl+V` → `PasteAfter`.
  5. Vim-set only: `Char('y')` no modifiers → `Copy` (the contract's "Copy / yank" row: normal calls it Copy, vim calls it yank — both return `TreeAction::Copy`; the semantic difference, clipboard vs buffer-only, is the caller's job in WP04, not this function's). `Char('p')` → `PasteAfter`. `Char('P')` → `PasteBefore`.
  6. Everything else → `None`.
  7. **Explicitly do not match** `CONTROL` + `c`/`v`/`x`/`z` when `set == Vim` — leave the `match` arm space empty rather than adding a catch-all that happens to return `None` for those combinations too, so a future reviewer can see by inspection that no vim arm mentions them (supports the T005 test's intent, not just its assertion).
- **Files**: `src/keymap.rs`.
- **Parallel?**: No — depends on T001's enums.
- **Notes**: `PasteAsChild` is never returned by `translate` in either set (menu-only, per FR-014/contract). Do not invent a key for it.

### Subtask T003 [P] – Implement label() for menu/help key display

- **Purpose**: One source of truth for what key text the Edit menu (WP07) and the help window (WP09) show next to each action, so they cannot drift from what `translate` actually accepts.
- **Steps**:
  1. `pub fn label(action: TreeAction, set: KeyBindingSet) -> Option<&'static str>`.
  2. Return the human-readable key text matching `contracts/keymap.md`'s "Structure pane" table exactly: e.g. `label(TreeAction::MarkDown, KeyBindingSet::Normal)` → `Some("Shift+Down")` (same text for `Vim`, since marking is identical in both sets).
  3. `label(TreeAction::PasteAsChild, _)` → `None` in both sets — the menu renders "menu only" for a `None` result (that rendering logic belongs to WP07, not here).
  4. `label(TreeAction::Cut, KeyBindingSet::Vim)` → `None` (no vim key for cut, per the contract).
  5. Add a doc comment stating explicitly: "This function's output must stay in lock-step with `translate`'s acceptance — every `Some` here must correspond to a key `translate` actually accepts for that action and set, and vice versa." T005's coverage test enforces this.
- **Files**: `src/keymap.rs`.
- **Parallel?**: Yes — independent of T002, can be written alongside T001.
- **Notes**: Keep the returned strings exactly matching the contract's key column text (`"Shift+Down"`, `"d d"`, `"Ctrl+X"`, `"y"`, `"p"`, `"P"`) since WP07 and WP09 will render them verbatim.

### Subtask T004 – Declare pub mod keymap in lib.rs

- **Purpose**: Make the new module reachable from `main.rs`, `app.rs`, `tui.rs`, and the test binaries.
- **Steps**:
  1. Add `pub mod keymap;` to `src/lib.rs`, in the same alphabetically-adjacent position as the existing module list (near `dump`, `hashsig`, `input` — check the current ordering rather than assuming; if the existing list is not alphabetical, match its actual convention instead of imposing one).
- **Files**: `src/lib.rs`.
- **Parallel?**: No — trivial, do last so it does not need touching again if T001–T003 shuffle names.
- **Notes**: This is the only line this WP adds to a file it does not otherwise own.

### Subtask T005 – Unit tests: vim table, coverage, Shift+Up

- **Purpose**: Make C-002 and the contract's coverage claims machine-checked, not just asserted in prose.
- **Steps**:
  1. `#[cfg(test)] mod tests` at the bottom of `src/keymap.rs`, following the style of the test modules in `src/app.rs`/`src/ber.rs` (helper functions first, then `#[test] fn snake_case_description()`).
  2. `vim_set_has_no_ctrl_cvxz_bindings`: iterate every printable ASCII key `'a'..='z' ∪ 'A'..='Z' ∪ digits` combined with `KeyModifiers::CONTROL`, call `translate(key, KeyBindingSet::Vim)`, and assert that for `c`, `v`, `x`, `z` specifically (case-folded) the result is `None`. Do not assert this for *every* letter — only the four the contract reserves; other Ctrl+letters may be legitimately unmapped too but that is not what this test is protecting.
  3. `both_shift_up_encodings_translate_to_mark_up`: construct `KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::SHIFT, .. }` and assert `translate(_, Normal)` and `translate(_, Vim)` both return `Some(Action::Tree(TreeAction::MarkUp))`.
  4. `every_tree_action_has_a_key_or_is_documented_menu_only`: for each `TreeAction` variant and each `KeyBindingSet`, assert `label` is `Some` unless the variant/set pair is exactly `(PasteAsChild, _)` or `(Cut, Vim)` (the two menu-only/no-key cases named in `contracts/keymap.md`).
  5. `label_and_translate_agree`: for each `(action, set)` pair where `label` returns `Some(key_text)`, reconstruct a plausible `KeyEvent` from that text (a small local helper mapping `"Shift+Down"` → the event built in step 3's style, `"Ctrl+X"` → Ctrl+X, `"d d"` → the first `d`, `"y"`/`"p"`/`"P"` → their plain char events) and assert `translate` on that event returns `Some(Action::Tree(action))`. This is the drift-prevention test called out in T003.
- **Files**: `src/keymap.rs` (test module).
- **Parallel?**: No — exercises the finished T001–T003.
- **Notes**: Keep the helper that builds a `KeyEvent` from modifiers + code small and reusable — WP03/WP04/WP08's tests will want the same helper; consider whether it belongs in a `#[cfg(test)] pub(crate) fn` other test modules can import, but do not over-engineer this — a private helper duplicated once or twice across modules is fine too if a shared one adds friction.

## Test Strategy

- `cargo test keymap::` must pass with zero warnings.
- No integration test changes are needed for this WP — it adds a new, self-contained module with no external call sites yet.
- Confirm `cargo build` still succeeds (the crate must compile with the new module present but unused, which is normal for a foundation WP — do not add `#[allow(dead_code)]` broadly; if the compiler warns about genuinely unused items, that is a signal something in T001–T003 is unreachable and worth double-checking, not something to silence).

## Risks & Mitigations

- **Terminal Shift+Up delivery**: research.md R3 asserts crossterm normalises the common `CSI 1;2A` sequence to `KeyCode::Up` + `SHIFT` on all three target platforms. If a manual terminal check later shows otherwise for a specific terminal emulator, that is a plan-level finding to raise, not something to work around silently in this WP with a second matching path.
- **Scope creep into dispatch wiring**: it is tempting to also update `src/tui.rs` while this context is fresh. Resist it — that work belongs to WP03/WP04/WP06/WP08/WP07 respectively, each of which needs this WP's types to exist first but should each make their own narrow, reviewable edit at the point they actually consume a given action.

## Review Guidance

- Confirm `translate` and `label` for every row of `contracts/keymap.md`'s Structure-pane table, by reading the table alongside the `match` arms line by line — not just by running the tests (a passing test suite can still miss a contract row nobody wrote a test for).
- Confirm no `EditorAction` variant is exercised by any test or call site yet — that is expected and correct for this WP; WP08 owns exercising them.
- Confirm the vim-set Ctrl+C/V/X/Z omission is structural (no matching arm exists) rather than an arm that matches and returns `None` — read the `match`, do not just trust the test.

## Activity Log

> **CRITICAL**: Activity log entries MUST be in chronological order (oldest first, newest last).

### How to Add Activity Log Entries

**When adding an entry**:

1. Scroll to the bottom of this Activity Log section
2. **APPEND the new entry at the END** (do NOT prepend or insert in middle)
3. Use exact format: `- YYYY-MM-DDTHH:MM:SSZ – agent_id – <action>`
4. Timestamp MUST be current time in UTC (check with `date -u "+%Y-%m-%dT%H:%M:%SZ"`)
5. Agent ID should identify who made the change (claude-sonnet-4-5, codex, etc.)

**Format**:

```
- YYYY-MM-DDTHH:MM:SSZ – <agent_id> – <brief action description>
```

**Why this matters**: The acceptance system reads the LAST activity log entry as the current state. If entries are out of order, acceptance will fail even when the work is complete.

**Initial entry**:

- 2026-09-09T11:40:38Z – system – Prompt created.

---

### Updating Status

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP01 --to <status>` to change WP status.
