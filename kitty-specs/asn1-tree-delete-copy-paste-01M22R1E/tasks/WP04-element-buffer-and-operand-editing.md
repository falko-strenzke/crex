---
work_package_id: WP04
title: Element buffer, delete/cut/copy/yank of the operand
dependencies: []
requirement_refs:
- FR-006
- FR-007
- FR-008
- FR-009
- FR-010
- FR-020
- FR-026
- NFR-001
- NFR-003
planning_base_branch: feat/asn1-tree-delete-copy-paste
merge_target_branch: feat/asn1-tree-delete-copy-paste
branch_strategy: Planning artifacts for this mission were generated on feat/asn1-tree-delete-copy-paste. During /spec-kitty.implement this WP may branch from a dependency-specific base, but completed changes must merge back into feat/asn1-tree-delete-copy-paste unless the human explicitly redirects the landing branch.
subtasks:
- T017
- T018
- T019
- T020
- T021
- T022
phase: Phase 2 - Marking
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: src/buffer.rs
create_intent:
- src/buffer.rs
execution_mode: code_change
model: ''
owned_files:
- src/buffer.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP04 – Element buffer, delete/cut/copy/yank of the operand

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

Turn the operand (from WP03's `App::operand()`) into DER bytes once, and share those bytes between the system clipboard and a new in-app element buffer, so copy/cut/paste have exactly one encode path and one decode path (the decode path arrives in WP06). Extend the existing single-element, two-step delete confirmation (`d` `d`) to work over a marked range without changing its behaviour for a lone selection.

Done when:
- `ElementBuffer { bytes: Vec<u8>, count: usize }` exists in new `src/buffer.rs`.
- `delete_operand()` removes every element in the operand, in one two-step confirmation, with the confirmation text naming the element count, and places the cursor per FR-008's rule (the row that followed the operand; failing that, the one before it; failing that, the parent).
- `copy_operand()` fills both the system clipboard (hex text, normal bindings only) and the element buffer; a clipboard write failure still fills the buffer and reports the clipboard problem without treating it as an overall failure.
- `cut_operand()` copies then deletes with no second confirmation, and makes **no change at all** if the copy step reached neither the clipboard nor the buffer.
- `yank_operand()` (vim bindings) fills the buffer only, never touching the system clipboard.
- The existing single-element `d`-key behaviour (confirmation wording, cursor placement) is unchanged when no mark is active — this WP must not regress it.
- `cargo test buffer::` passes for range delete + cursor placement, copy-fills-both, copy-survives-clipboard-failure, cut-aborts-on-total-failure, yank-buffer-only.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` User Story 1 (delete) and the FR-006–010, FR-020, FR-026 rows, plus NFR-001 (250 ms / 500 elements / 1 MiB) and NFR-003 (no partial mutation on failure).
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/data-model.md` — `ElementBuffer` field table and lifecycle (replaced whole, never partially updated; separate from the vim editors' own per-editor register, which is WP08's concern, not this one's).
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/research.md` R4 and R12 — why the buffer is bytes, not cloned `Node`s; and the exact cut semantics.
- **`src/app.rs`'s existing `delete_selected()` function in full** (search for `pub fn delete_selected`) — this is the function this WP supersedes. Read its two-step confirmation state machine, its region-rule checks, and its cursor-placement logic closely; T018 generalises this exact function to a range, and must preserve every behaviour it has today for the single-element case.
- `src/ber.rs`'s `encode_forest(nodes: &[Node]) -> Vec<u8>` — the encoder this WP's copy path must use exclusively (never hand-build bytes).
- `src/clipboard.rs`'s existing `write(text: &str) -> Result<(), String>` and `hex_pairs(bytes: &[u8]) -> String` — reuse both directly for the clipboard-write half of `copy_operand`.

**Out-of-map edits this WP makes, with rationale**:
- `src/app.rs`: `delete_selected`'s body is replaced with a one-line call to the new `delete_operand()` (so the existing `d`-key call site in `handle_document_key` needs no change at all — confirm this by re-reading that call site, do not assume); a new `element_buffer: Option<ElementBuffer>` field is added to the `App` struct.
- `src/tui.rs`: `handle_document_key` gains `match` arms for `TreeAction::Delete` (already calling `delete_selected`, now indirectly `delete_operand`, so this may need no change at all — verify), `Cut`, `Copy`, `Yank` from WP01's `keymap::translate`.

This WP does **not** implement paste — `ElementBuffer`'s `bytes` field is exactly what WP06 will feed into `ber::parse_forest` to read it back. Do not add any paste-reading logic here even if it feels natural; keep this WP's scope to producing bytes, not consuming them.

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T017 – Define ElementBuffer struct

- **Purpose**: The shared payload type for copy/cut/yank and (later, in WP06) paste.
- **Steps**:
  1. In new `src/buffer.rs`, module doc comment explaining the buffer's role and its separation from the system clipboard and from the vim editors' register (link the reasoning, in your own words, to research.md R4/R12).
  2. `pub struct ElementBuffer { pub bytes: Vec<u8>, pub count: usize }`.
  3. `impl ElementBuffer { pub fn from_operand(app: &crate::app::App, operand: &crate::mark::Operand) -> Self { ... } }` — collect the `Node`s named by `operand`'s source/parent/range (look them up in `app.roots` / `app.decrypted` / `app.pkcs12` per `RowSource`, mirroring the same `match` on `RowSource` used in `delete_selected` today for locating the right forest), call `ber::encode_forest(&nodes)`, and set `count = nodes.len()`.
- **Files**: `src/buffer.rs` (new).
- **Parallel?**: No — the type every other subtask in this WP fills or reads.
- **Notes**: Do not clone `Node`s into `ElementBuffer` itself — encode to bytes immediately in `from_operand` and discard the `Node` references; this keeps the buffer's lifetime fully independent of the document that produced it (important once the user switches files).

### Subtask T018 – Implement delete_operand() replacing delete_selected's body

- **Purpose**: FR-006, FR-007, FR-008 — generalise the existing single-element delete to a range without changing single-element behaviour.
- **Steps**:
  1. In `src/buffer.rs`, `impl App { pub fn delete_operand(&mut self) { ... } }`.
  2. Call `self.operand()` (from WP03); on `Err(msg)`, set `self.status = msg` and return (mirrors `delete_selected`'s existing placeholder/region refusals, now centralised in `operand()`).
  3. Confirmation state machine: reuse the existing `self.delete_confirm: bool` field. First call (not yet armed): set it, and build the status message as `format!("delete {n} element{} at offset {off}? press d again to confirm", if n == 1 {""} else {"s"}, ...)` — for `n == 1`, this must produce **exactly** the existing single-element wording (`"delete {type_name} at offset {offset}? press d again to confirm"`) so the change is invisible for that case; check the existing exact format string in `delete_selected` and match it precisely for `n == 1`, only diverging in wording for `n > 1`.
  4. Second call (already armed): un-arm it, then remove every element in the operand's range from the correct forest (found via `RowSource`, same lookup pattern as `ElementBuffer::from_operand`), **from the highest sibling index to the lowest** so earlier removals do not shift the indices of ones not yet removed. Call `self.rebuild()`, set `self.dirty = true`.
  5. Cursor placement (FR-008): after `rebuild()`, select the row that now occupies the position the operand's first removed sibling used to have, if one exists; else the row before the operand's original start, if one exists; else the parent row. Reuse `delete_selected`'s existing cursor-placement logic as the starting point — it already implements exactly this rule for the single-element case, generalise its target index calculation from "the removed index" to "the range's start index" rather than rewriting the logic from scratch.
  6. Status after deletion: `"element deleted — 'Ctrl+S' writes the file"` for one element, matching today's wording exactly; a new equivalent for `n > 1`, e.g. `"n elements deleted — 'Ctrl+S' writes the file"`; and the existing special case for the document becoming empty (`"element deleted — document is now empty ('i' inserts, 'Ctrl+S' writes)"`) generalised the same way.
  7. Change `delete_selected` in `src/app.rs` to `pub fn delete_selected(&mut self) { self.delete_operand(); }` — a one-line body, keeping the existing public name and its call site in `tui.rs` untouched. Do not delete `delete_selected` outright, in case anything else calls it by name (grep to check before deciding).
- **Files**: `src/buffer.rs` (new logic), `src/app.rs` (the one-line body replacement, the declared out-of-map edit).
- **Parallel?**: No — the central subtask of this WP.
- **Notes**: `delete_confirm` must still reset on any other key press, exactly as documented today (`handle_document_key`'s existing `if key.code != KeyCode::Char('d') { app.delete_confirm = false; }`) — this WP does not need to touch that line, just confirm it still applies correctly to the generalised flow (it does, since it is keyed on the raw key press, not on what `delete_operand` does).

### Subtask T019 – Implement copy_operand()

- **Purpose**: FR-009, FR-010.
- **Steps**:
  1. `impl App { pub fn copy_operand(&mut self) { ... } }` in `src/buffer.rs`.
  2. `self.operand()` → on `Err`, set status and return.
  3. Build `ElementBuffer::from_operand(self, &operand)`.
  4. If `self.bindings == KeyBindingSet::Normal` (from WP01/WP02's `App.bindings` field — confirm it exists by this point given WP02's dependency), attempt `clipboard::write(&ber::hex_pairs(&buffer.bytes).replace(' ', ""))` (uppercase hex, no separators, no trailing newline — match `contracts/clipboard-payload.md`'s example exactly, e.g. `hex_pairs` may include spacing that needs stripping, mirror however the existing hex-editor Ctrl+C already does this — check its implementation, likely `copy_selection` in `src/app.rs`, for the precise formatting and reuse the same approach).
  5. On clipboard write success: `self.status = format!("{n} element{} copied to the clipboard as hex", ...)`. On failure: still store the buffer (step 3's result is stored into `self.element_buffer` regardless), and set status to something like `"{n} elements copied — clipboard unavailable, paste within crex still works"` (matches FR-010's requirement; exact wording is this WP's call, but must convey both facts: it worked internally, and why the clipboard part didn't).
  6. Under `KeyBindingSet::Vim`, this same function is called for `y` (yank) too — vim's "yank" and normal's "copy" are the same underlying operation per `contracts/keymap.md`'s note that both map to `TreeAction::Copy`; **but** the vim case must never touch the system clipboard (FR-025, C-002's spirit) — so `copy_operand` must skip the clipboard-write branch entirely when `self.bindings == Vim`, filling only `self.element_buffer`, with status wording `"{n} element{} yanked"`.
  7. Always: `self.element_buffer = Some(buffer);` (this line runs regardless of binding set or clipboard outcome — the buffer is always filled by copy/yank).
- **Files**: `src/buffer.rs`.
- **Parallel?**: Can be drafted alongside T020 once T017/T018 exist, since both build on the same "build an ElementBuffer, then decide what to do with it" pattern.
- **Notes**: Re-read FR-025 (yank/paste under vim) and FR-009/FR-010 (copy under normal) side by side before writing this function — it is one function serving two rather different sets of user-facing behaviour, and it is easy to accidentally let clipboard-writing leak into the vim path.

### Subtask T020 – Implement cut_operand() and yank_operand()

- **Purpose**: FR-020 (cut), FR-026 (delete fills the buffer under vim, handled already by T018 since it always builds no buffer today — this subtask adds the *cut* command specifically, and clarifies `yank_operand` is a thin wrapper).
- **Steps**:
  1. `pub fn cut_operand(&mut self) { ... }` in `src/buffer.rs`: call `self.copy_operand()` first. If, after that call, `self.element_buffer` is still `None` **and** the clipboard write also failed (i.e., truly neither destination received the data — this can only happen if `ElementBuffer::from_operand` itself failed, which per T017 it should not, since encoding cannot fail for a valid operand; the realistic failure mode per NFR-003 is "operand() itself returned Err", already handled by returning early) — in practice, `copy_operand` as designed in T019 always fills the buffer if `operand()` succeeded, so `cut_operand`'s "abort if neither destination got the data" condition reduces to "abort if `operand()` failed", which `copy_operand`'s own early return already handles. Confirm this reasoning by re-reading T019 before implementing — if it holds, `cut_operand`'s body is: call `copy_operand()`; if `self.element_buffer` is `Some` (i.e., the copy actually happened), immediately call the deletion half of `delete_operand()` **without** the confirmation step (cut never asks for confirmation, per spec.md User Story 3 acceptance scenario 4) — this means `cut_operand` cannot simply call `self.delete_operand()` (which re-arms confirmation); factor `delete_operand`'s post-confirmation removal logic (index-descending removal + rebuild + cursor placement + dirty flag) into a small private helper both `delete_operand` (after confirmation) and `cut_operand` (immediately) call.
  2. `cut_operand`'s status message after success: `"{n} element{} cut to the clipboard as hex"` (normal) or `"{n} element{} cut"` (vim) — reuse `copy_operand`'s message-building logic where possible rather than duplicating the pluralisation.
  3. `pub fn yank_operand(&mut self) { self.copy_operand() }` — per T019 step 6, `copy_operand` already behaves correctly for vim's yank (buffer-only, no clipboard) as long as `self.bindings == Vim` at call time; `yank_operand` exists as a distinctly-named entry point purely for readability at the `tui.rs` call site (`TreeAction::Copy` dispatches to `copy_operand` regardless of binding set — see T021 — so `yank_operand` as a *separate* function may in fact be unnecessary; if `copy_operand` alone suffices for both call sites, do not create a redundant wrapper. Make this judgment call yourself and record which you chose and why in the Activity Log).
- **Files**: `src/buffer.rs`.
- **Parallel?**: Can be drafted alongside T019.
- **Notes**: The private helper factored out in step 1 (removal logic shared by `delete_operand` and `cut_operand`) is the key structural decision in this WP — get its signature right (it needs the already-computed `Operand`, not to recompute it) since T018 must also be updated to call it after confirmation, rather than duplicating the removal code inline.

### Subtask T021 – Wire TreeAction::Delete/Cut/Copy/Yank dispatch in handle_document_key

- **Purpose**: Make the new functions reachable from the keyboard, per WP01's translation layer.
- **Steps**:
  1. In `src/tui.rs`'s `handle_document_key`, before or alongside the existing raw `KeyCode::Char('d') => app.delete_selected()` arm, add a call to `keymap::translate(key, app.bindings)` and dispatch on the result: `Some(Action::Tree(TreeAction::Delete))` → `app.delete_selected()` (unchanged, now indirectly `delete_operand`); `Cut` → `app.cut_operand()`; `Copy` → `app.copy_operand()`; `Yank` → if you decided in T020 to keep `yank_operand` distinct, call it here for the vim `y` key specifically — otherwise route both `Copy`'s normal-Ctrl+C and vim-y through the same `copy_operand()` call, since `keymap::translate` already returns the same `TreeAction::Copy` for both per the contract (re-read `contracts/keymap.md`'s note on this before deciding how many `match` arms you actually need — there may be only one, not two).
  2. Decide the relationship between this new `translate`-based dispatch and the pre-existing raw `KeyCode::Char('d')` arm: since `translate` returns `TreeAction::Delete` for exactly the same key `d` produces raw, either replace the raw arm with the translated one (cleaner, converges dispatch onto one mechanism) or leave the raw arm as-is and only add the *new* arms (`Cut`/`Copy`/`Yank`) via `translate` (less invasive, but leaves two dispatch styles side by side in the same function). Prefer the first (fully convert `handle_document_key` to dispatch via `translate` for every action this mission adds, including `Delete`) since WP03/WP06/WP07/WP08 will each add more `translate`-based arms to this same function and a consistent single dispatch style will make those additions and this function's ongoing readability better — but this is your call to make and record.
- **Files**: `src/tui.rs` (the declared out-of-map edit for this WP).
- **Parallel?**: No — needs T018–T020 complete.
- **Notes**: `use crate::keymap::{translate, Action, TreeAction};` at the top of `src/tui.rs` if not already imported by WP03's edit to the same function — check what WP03 already added here (if this WP lands after WP03 in your lane, the import may already exist).

### Subtask T022 – Unit tests: delete/copy/cut/yank

- **Purpose**: Prove NFR-001, NFR-003, and FR-006–010/020/026 with concrete scenarios.
- **Steps**:
  1. Using `test_app`, build a document with several sibling elements under one parent (reuse WP03's test fixture choice if convenient, for consistency).
  2. `range_delete_removes_all_marked_elements_and_places_cursor_correctly`: mark three siblings (via `mark_extend`, reusing WP03's helpers — this WP's tests may depend on `src/mark.rs`'s public API, which is fine since WP04 depends on WP03), confirm the two-step delete (`delete_operand()` called twice), assert all three are gone and the cursor is on the row that followed them.
  3. `single_element_delete_wording_is_unchanged`: no mark, one selection, confirm the confirmation and success status strings match today's exact wording (paste the current strings from `delete_selected` as the expected value, to guarantee no accidental wording drift).
  4. `copy_fills_clipboard_and_buffer_under_normal_bindings`: set `app.bindings = Normal`, `copy_operand()`, assert `app.element_buffer` is `Some` with the right byte count, and (if the test environment has no clipboard helper available, which is likely in CI) assert the status message correctly reports the clipboard-unavailable case per T019 step 5 — i.e., this test doubles as the "copy survives clipboard failure" test in a headless environment; do not skip or ignore it because of that, treat the expected clipboard failure as the realistic CI condition and assert the buffer is filled regardless.
  5. `cut_aborts_when_operand_is_invalid`: attempt `cut_operand()` on a placeholder/reveal row (via `operand()`'s existing refusal), assert nothing was removed and `app.dirty` is still `false`.
  6. `yank_never_touches_clipboard_under_vim_bindings`: set `app.bindings = Vim`, call the copy/yank path, assert the buffer is filled and (since there is no direct way to assert "clipboard was not touched" without mocking `clipboard::write`) assert the status wording matches the vim-specific "yanked" phrasing from T019 step 6, which is the observable proxy for "the clipboard branch was skipped".
- **Files**: `src/buffer.rs` (test module).
- **Parallel?**: No — exercises the finished T018–T020.
- **Notes**: If `clipboard::write` is not mockable/injectable in the current `clipboard.rs`, do not add a mocking seam purely for this test — the CI-environment-has-no-clipboard-helper reality already exercises the failure path realistically, as noted in step 4.

## Test Strategy

- `cargo test buffer::` must pass.
- Run the full `cargo test` once at the end, specifically watching `dumpasn1_compat.rs` for regressions — this WP changes `delete_selected`'s internals and any existing test exercising delete (check `src/app.rs`'s existing test module for one) must still pass unchanged.

## Risks & Mitigations

- **Cursor-placement regression for the single-element case**: T018's generalisation of `delete_selected`'s cursor logic is the highest-risk edit in this WP for silently changing existing, working behaviour. Mitigate with T022's explicit "wording is unchanged" test, and additionally diff the generalised logic against the original by hand once written.
- **NFR-001 (250 ms / 500 elements)**: the index-descending removal in `delete_operand` and the byte-concatenation in `ElementBuffer::from_operand` are both O(operand size), not O(document size) — keep them that way; do not accidentally re-scan `self.rows` (which is O(document size)) more than once per operation (the one `rebuild()` call is expected and budgeted for).

## Review Guidance

- Confirm the single-element delete path is provably unchanged (read T022's wording-match test and the diff against the original `delete_selected`, not just that tests pass).
- Confirm `cut_operand` truly makes no change when the copy step fails — trace the early-return path by hand.
- Confirm the factored-out removal helper (T020 step 1) is called by both `delete_operand` (after confirmation) and `cut_operand` (immediately), so the removal logic exists in exactly one place.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP04 --to <status>` to change WP status.
