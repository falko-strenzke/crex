---
work_package_id: WP08
title: Vim modal value editors
dependencies: []
requirement_refs:
- FR-028
- FR-029
- FR-030
- FR-031
subtasks:
- T038
- T039
- T040
- T041
- T042
phase: Phase 3 - Copy and Paste
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: src/vim.rs
create_intent:
- src/vim.rs
execution_mode: code_change
model: ''
owned_files:
- src/vim.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP08 – Vim modal value editors

## ⚡ Do This First: Load Agent Profile

Use the `/ad-hoc-profile-load` skill to load the agent profile specified in the frontmatter (or any user-defined profile), and behave according to its guidance before parsing the rest of this prompt.

- **Profile**: `implementer-ivan`
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

Give every value editor (hex, text, integer, OID, base64, date/time) normal/insert/visual modes under vim bindings, mapping vim commands onto the editors' existing move/select/delete/undo operations rather than building a second editor implementation, plus Ctrl+A/Ctrl+X increment/decrement of the number at or after the cursor.

Done when:
- `EditorMode { Normal, Insert, Visual }` and `VimState { mode, register }` exist in new `src/vim.rs`, attached to `EditState` only when vim bindings are active at the moment an editor opens.
- `i`/`a`/`I`/`A` enter Insert at the right cursor position; Esc returns to Normal (from Insert or Visual); Enter applies the edit from either Normal or Insert; Esc in Normal cancels.
- `h`/`l`/`j`/`k`/`0`/`$` move; `x` deletes under the cursor into the register; `v` starts Visual, `y`/`d` in Visual copy/cut the selection into the register and return to Normal; `p`/`P` put the register after/before the cursor; `u` undoes.
- Ctrl+A/Ctrl+X increment/decrement the number at or after the cursor: hex octet (wraps), integer (arbitrary size), OID arc, or a digit run in text/date-time.
- Ctrl+C/V/X/Z are no-ops in every editor mode under vim bindings.
- `cargo test vim::` passes for mode transitions, the documented `v l l y $ p` scenario, and each increment/decrement case.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` User Story 6 (all twelve acceptance scenarios) — read every one; this is the densest scenario list in the mission and each maps to a specific piece of this WP.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/research.md` R7 and R8 — the mapping from vim commands to the existing `Editor` API, and the increment/decrement semantics per editor kind.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/data-model.md` — `VimState` fields and the mode transition diagram (also in `plan.md`'s "Key flows").
- **`src/app.rs`'s `Editor`/`EditHistory`/`HexEditor`/`TextEditor`/`DateTimeEditor` types in full** (roughly lines 320–870) — this WP does not reimplement selection, undo, or paste; it drives the *existing* `move_horizontal`, `move_vertical`, `home`, `end`, `select_all`, `delete_selection`, `undo`, `paste` methods via vim commands. Read every one of these methods' current signatures before writing T039/T040.
- `src/tui.rs`'s existing `handle_edit_key` (search `fn handle_edit_key`) — the function this WP's T042 modifies to add a vim branch; read its current Ctrl-combination handling and the `extend` (Shift-selection) logic it already has for the normal-bindings case, since vim's Visual mode reuses the same underlying `EditHistory` selection mechanism, just triggered by `v` instead of Shift+motion.

**Out-of-map edits this WP makes, with rationale**:
- `src/app.rs`: add a `vim: Option<VimState>` field to `EditState` — populated when the editor is constructed (wherever `EditState::hex`/`Editor::text`/etc. are called to open an editor — check every call site) based on whether `App.bindings == KeyBindingSet::Vim` at that moment; no other WP declares this field.
- `src/tui.rs`: `handle_edit_key` gains a branch that, when `edit.vim.is_some()`, dispatches through this WP's vim command interpreter instead of (or in addition to, for the parts that stay identical, like `Esc`/`Enter` at the top of the function) its existing normal-bindings logic.

This WP does **not** touch the tree-level `y`/`p`/`P`/`d` bindings (that is WP03's mark and WP04's buffer, dispatched via WP01's `keymap::translate` with `TreeAction`) — this WP is scoped entirely to the **value editors** (`EditorAction`, opened with `e`/`E` on a tree element), a semantically separate vim surface with its own register, separate from the tree's `ElementBuffer`.

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T038 – Define EditorMode, VimState in new src/vim.rs

- **Purpose**: The mode machine and its per-editor register.
- **Steps**:
  1. Module doc comment in new `src/vim.rs` explaining the design: vim commands drive the *existing* editor operations rather than a parallel implementation, and the register here is distinct from the tree's `ElementBuffer` (link both points to research.md R7 in your own words).
  2. `#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum EditorMode { Normal, Insert, Visual }`.
  3. `pub struct VimState { pub mode: EditorMode, pub register: Vec<char> }` — `register` holds content in the editor's own unit (hex digits for `HexEditor`, characters for `TextEditor`), matching `EditHistory`'s existing unit convention (`HexEditor::UNIT`/`TextEditor::UNIT`, see `src/app.rs` near line 567/653) — read those constants before assuming `char` is the right element type throughout (it is, for both, since even the hex editor's "digits" field is `Vec<char>`).
  4. `impl VimState { pub fn new() -> Self { VimState { mode: EditorMode::Normal, register: Vec::new() } } }` — editors always open in Normal mode under vim bindings, per FR-028.
- **Files**: `src/vim.rs` (new).
- **Parallel?**: No — the base every other subtask in this WP builds on.
- **Notes**: Do not give `VimState` any knowledge of *which* editor it is attached to (hex vs. text vs. date-time) — that distinction is handled by the calling code in `src/tui.rs`/T039–T041 dispatching to the right `Editor` variant's methods, not by `VimState` itself.

### Subtask T039 – Implement normal-mode motion and mode transitions

- **Purpose**: FR-028's mode transitions and FR-029's motion, for every editor kind.
- **Steps**:
  1. `impl VimState { pub fn handle_normal_key(&mut self, editor: &mut Editor, key: KeyEvent) -> VimOutcome { ... } }` where `VimOutcome` is a small enum (`Continue, Apply, Cancel`) telling the caller (`handle_edit_key` in `tui.rs`, T042) what to do next (apply the edit, cancel it, or nothing — stay in the editor). Only meaningful in `Normal` mode; other modes get their own handler (T040 for Visual, and Insert simply falls through to the editor's existing character-insertion path per T042).
  2. Mode transitions: `i` → `self.mode = Insert` (cursor unchanged); `a` → move cursor right one unit first (if not at the end), then Insert; `I` → move to line/content start (`editor`'s `home(false)`), then Insert; `A` → move to end (`end(false)`), then Insert. Esc in Normal → `VimOutcome::Cancel`. Enter in Normal → `VimOutcome::Apply`.
  3. Motion: `h`/`l` → `editor.move_horizontal(-1/1, false)`; `j`/`k` → `editor.move_vertical(-1/1, false)` (a no-op in single-line editors, per existing `move_vertical` behaviour — verify this is already handled gracefully by the existing method rather than assuming); `0` → `editor.home(false)`; `$` → `editor.end(false)`.
  4. For the `DateTimeEditor` specifically: it has no single char buffer, six fields instead (`fields: [String; 6]`, `active: usize`). `h`/`l` should move `active` between fields (clamped 0..6) rather than calling `move_horizontal` (which likely does not apply to this editor variant — check `Editor`'s `match` arms for `move_horizontal`/`move_vertical` to see whether `DateTime` is even handled there today, and if not, add the field-navigation logic here in `vim.rs` as a special case, gated on `matches!(editor, Editor::DateTime(_))`). `x` is a documented no-op for this editor (per spec.md acceptance scenario 4's note); do not attempt to delete a "character" from a field.
- **Files**: `src/vim.rs`.
- **Parallel?**: No — the central dispatch subtask.
- **Notes**: Keep `VimOutcome` minimal — resist adding variants beyond `Continue`/`Apply`/`Cancel` unless a later subtask genuinely needs one; over-designing this enum early makes T040–T042 harder to reason about, not easier.

### Subtask T040 – Implement x / visual mode / put / undo

- **Purpose**: FR-029's remaining normal-mode commands.
- **Steps**:
  1. `x`: select exactly one unit at the cursor (construct a one-unit selection via `EditHistory`'s existing selection-anchor mechanism, or add a small `Editor::select_unit_at_cursor()` helper if the existing API has no direct one-line way to select "just the next unit" — check `EditHistory`'s fields/methods near line 365 first), then call the editor's existing `delete_selection()`, copying the deleted content into `self.register` first (read `delete_selection`'s current signature — does it return what was deleted, or must the caller read the selection before calling it? Check before assuming).
  2. `v`: set `self.mode = Visual` and record a selection anchor at the current cursor position (reuse `EditHistory`'s existing anchor field — the same one Shift+motion already sets under normal bindings — rather than adding a second, parallel selection-tracking mechanism).
  3. In Visual mode, motion keys (`h l j k 0 $`) extend the selection exactly as Shift+motion does under normal bindings (call the same `move_*(delta, true)` — `extend = true` — methods, just triggered by a bare key instead of a Shift-modified one). `y` → copy the current selection into `self.register`, return to Normal. `d` → cut the current selection into `self.register` (delete + capture), return to Normal. Esc → return to Normal without changing the selection's content (just leaves Visual mode; per vim convention the selection itself is abandoned, not left highighted — confirm this matches spec.md's scenario 6 wording, which says "Esc leaves visual mode", not "clears the selection e.g. leaving text marked" — implement the simpler "just changes mode" reading unless the selection needs explicit clearing too, which you should check against how the existing Shift-selection is cleared when Shift is released, i.e., never explicitly — it is just not extended further; match that same "do nothing extra" behaviour here).
  4. `p`/`P`: call the editor's existing `paste(text: &str)` method (construct the string from `self.register`'s chars) at the cursor, after (`p`) or before (`P`) — check whether `paste`'s existing behaviour inserts at the cursor already matching "after" or "before" semantics, and adjust the cursor position by one before calling it for whichever of `p`/`P` does not match the existing default, rather than modifying `paste` itself.
  5. `u`: call the editor's existing `undo()` method directly — no new logic needed, this is a straight pass-through.
- **Files**: `src/vim.rs`.
- **Parallel?**: Can be drafted in parallel with T041 once T038/T039 exist (both consume the same `Editor`/`EditHistory` API but touch different commands).
- **Notes**: This subtask is the one most likely to reveal that `Editor`'s existing API needs one small new public method (e.g. `select_unit_at_cursor`) — if so, add it to the existing `impl Editor` block in `src/app.rs` as a narrow, additive method (not a behaviour change to anything existing), and note this out-of-map addition explicitly in the Activity Log, since `app.rs` is not this WP's declared owned file.

### Subtask T041 [P] – Implement adjust_number_at_cursor(delta) and wire Ctrl+A/Ctrl+X

- **Purpose**: FR-030 — the one genuinely new piece of editor logic this mission adds (everything else in this WP re-drives existing operations).
- **Steps**:
  1. `impl Editor { pub fn adjust_number_at_cursor(&mut self, delta: i8) -> Result<(), String> { ... } }` — added to the existing `impl Editor` block in `src/app.rs` (a declared, necessary out-of-map edit — this logic is inherently a method on `Editor`, which lives in `app.rs`; it cannot reasonably live in `vim.rs` as a free function without awkwardly exposing `Editor`'s private fields, so keep it as a method where the type already is, note this clearly in the Activity Log).
  2. `Editor::Hex`: find the octet (two hex digits) at or covering the cursor position; parse it as `u8`; add `delta` with wrapping (`wrapping_add`/`wrapping_sub`, so `0xFF + 1 = 0x00` and `0x00 - 1 = 0xFF`); write the two digits back in place (uppercase, matching the rest of the hex editor's convention).
  3. `Editor::Text` with a `TextFormat` indicating an integer field (check `TextFormat`'s variants — if there is a dedicated integer text format used by the OID/number field editors, use that; if integers are only ever edited via a dedicated non-`Editor::Text` path, check where — this may require reading `EditKind`/the call sites that construct integer editors, referenced in `plan.md`'s note about the integer editor needing "arbitrary size, sign-aware" arithmetic): parse the full buffer as a decimal integer using string-based arithmetic (do **not** use `i128` — research.md R8 explicitly warns real certificate INTEGERs exceed it; implement simple decimal string increment/decrement by hand, handling a leading `-` sign, or use a big-integer approach only if one is already available in the dependency tree — check `Cargo.toml` before adding anything, and if none exists, hand-rolled string arithmetic for `+1`/`-1` specifically is a small, bounded piece of code, not a full bignum library).
  4. OID editor: if OIDs are edited via a dedicated field type (check how `oid_arcs`/`encode_oid` in `src/ber.rs` are invoked from the editing side — likely a text buffer holding dot-notation, e.g. `"1.2.840.113549.1.1.11"`), find the arc (the digits between two dots, or before the first/after the last dot) at or after the cursor, parse and adjust it as a `u64` (or bigger — OID arcs can exceed `u64` in theory; matching the integer field's arbitrary-size handling is the more consistent choice if the effort is small, otherwise document the `u64` limitation clearly as a known bound in a code comment) and write it back.
  5. Any other text/date-time field: find the first run of ASCII digits at or after the cursor, parse and adjust as a plain decimal number (growing the digit count if needed, e.g. `99 → 100`), write it back, keeping the rest of the buffer unchanged.
  6. `DateTimeEditor`: adjust the active field's numeric value by `delta`, clamped to that field's valid range (e.g. month 1–12, day 1–31 — check whatever validation `DateTimeEditor` already performs on commit, and reuse the same bounds rather than inventing new ones).
  7. When no number is found anywhere at or after the cursor in the current buffer: `Err("no number at or after the cursor".to_string())`, and the caller (T039/T042) sets this as the status message rather than silently doing nothing.
  8. In `vim.rs`'s normal-mode key handler (T039), wire `Ctrl+A` → `adjust_number_at_cursor(1)`, `Ctrl+X` → `adjust_number_at_cursor(-1)`.
- **Files**: `src/app.rs` (the `adjust_number_at_cursor` method, the declared necessary out-of-map addition), `src/vim.rs` (the Ctrl+A/Ctrl+X wiring).
- **Parallel?**: Yes — this subtask's logic is largely independent of T040's visual-mode/put/undo work, sharing only T038's base types.
- **Notes**: This is the highest-effort subtask in the WP — budget accordingly, and write its tests (part of T043 below, in WP09's coverage, but also directly exercised by this WP's own T-numbered tests if `cargo test vim::` includes them — check `tasks.md`'s subtask list; this WP's own test subtask, not numbered separately here, is folded into the WP-level "Test Strategy" below since no dedicated `Txxx` was allocated for it beyond what's implicit in T038-T042 — see Test Strategy).

### Subtask T042 – Route handle_edit_key through vim dispatch; render mode indicator

- **Purpose**: Wire the finished `vim.rs` into the actual keyboard path, and FR-031's visible mode indicator.
- **Steps**:
  1. In `src/tui.rs`'s `handle_edit_key`: at the very top (before the existing `Esc`/`Enter` handling), check `if let Mode::Edit(ref mut edit) = app.mode { if let Some(vim) = edit.vim.as_mut() { ... } }` — when vim is active, dispatch based on `vim.mode`: `Normal` → `vim.handle_normal_key(&mut edit.editor, key)` (T039/T040/T041's combined entry point — you may need to consolidate T039/T040/T041 into one `handle_normal_key` method rather than three separate ones, matching whatever shape T039 actually settled on; adjust as needed), mapping its `VimOutcome` to `app.commit_edit()`/`app.cancel_edit()`/nothing, exactly as the existing top-of-function `Esc`/`Enter` handling already does for normal bindings — reuse those same two calls, do not duplicate their logic. `Insert` → fall through to the **existing** character-insertion logic already in `handle_edit_key` (the `KeyCode::Char(c) => edit.editor.insert_char(c)` arm and friends), except Esc specifically returns to `Normal` instead of cancelling (this one case needs a small branch before the fallthrough). `Visual` → dispatch motion/`y`/`d` through T040's visual-mode handling.
  2. Ensure Ctrl+C/V/X/Z are no-ops when `edit.vim.is_some()`: the existing `if key.modifiers.contains(KeyModifiers::CONTROL) { match folded { 'a' => select_all... 'c' => copy... } }` block at the top of `handle_edit_key` must be **skipped entirely** when vim is active, replaced by vim's own Ctrl+A/Ctrl+X handling (T041) inside the Normal-mode dispatch — do not let both blocks run (that would make Ctrl+C both copy via the old path and hit vim's dispatch); gate the existing Ctrl-block with `edit.vim.is_none()` explicitly.
  3. Mode indicator (FR-031): wherever the editor's first line currently renders "live feedback" text (search `src/tui.rs` for where the hex/text editor's status/preview line is drawn), prepend the mode name (`"NORMAL"`, `"INSERT"`, `"VISUAL"`) when `edit.vim.is_some()`, otherwise render unchanged.
- **Files**: `src/tui.rs` (`handle_edit_key` and the editor's first-line rendering — the declared out-of-map edit for this WP).
- **Parallel?**: No — the integration point, needs T038–T041 all finished.
- **Notes**: Populating `EditState.vim` itself (the `Some`/`None` decision) happens wherever an editor is *opened*, not inside `handle_edit_key` — find every call site that constructs `EditState`/`Mode::Edit(...)` (search `Mode::Edit(` across `src/app.rs`) and add `vim: if app.bindings == KeyBindingSet::Vim { Some(VimState::new()) } else { None }` to each — this may be several call sites (value edit via `e`, editor-chooser via `E`, structured extension editors are separate `Mode` variants so unaffected). List every call site found in the Activity Log.

## Test Strategy

- `cargo test vim::` must pass, covering: mode transitions (`i`/`a`/`I`/`A`/Esc/Enter in both modes); the documented scenario `v l l y $ p` on the hex editor duplicating three octets; Ctrl+A wrapping a hex octet `FF→00`; Ctrl+A on an integer `99→100`; Ctrl+A on an OID arc; `DateTimeEditor` field navigation via `h`/`l` and `x`'s no-op there; Ctrl+C/V/X/Z confirmed as no-ops when vim is active (assert editor state unchanged after simulating each).
- Run the full `cargo test` once at the end, watching specifically for any existing normal-bindings editor test (there are several already in `src/app.rs`'s test module, e.g. `edit_primitive_value_reencodes_lengths`) to confirm they still pass unchanged — this WP's `edit.vim.is_none()` gating (T042 step 2) must leave the normal-bindings path completely untouched.

## Risks & Mitigations

- **`DateTimeEditor` is not a char buffer**: the single most likely place for a naive implementation to panic or silently misbehave, since it doesn't share `HexEditor`/`TextEditor`'s shape. Handle it as an explicit special case throughout (T039, T041, T040's `x` no-op) rather than assuming the general-purpose logic degrades gracefully.
- **Double-handling Ctrl-combinations**: T042's gating of the existing Ctrl-block is easy to get subtly wrong (e.g. gating only part of it). Trace this by hand and add the explicit no-op test in T-testing above.
- **Integer arithmetic without i128**: hand-rolled decimal string increment/decrement is a small, well-known algorithm (find the rightmost non-9 digit for +1, handle all-9s by prepending a 1, mirror for -1 and borrowing) but is easy to get wrong at the boundaries (empty string, single "0", negative numbers). Write it carefully and test the boundary cases explicitly, not just the `99→100` example from the spec.

## Review Guidance

- Confirm Ctrl+C/V/X/Z are genuinely inert under vim bindings by tracing T042's gating logic, not just by running the no-op test (a test that accidentally does nothing would also "pass").
- Confirm `adjust_number_at_cursor`'s hex-octet wrapping is tested at both boundaries (`FF→00` incrementing, `00→FF` decrementing), not just one direction.
- Confirm every `Mode::Edit(...)` construction site found in T042 was updated to populate `vim` correctly — cross-check the Activity Log's list against a fresh `grep -n "Mode::Edit("` over `src/app.rs`.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP08 --to <status>` to change WP status.
