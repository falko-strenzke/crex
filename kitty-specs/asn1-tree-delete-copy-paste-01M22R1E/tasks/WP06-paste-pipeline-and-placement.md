---
work_package_id: WP06
title: Paste pipeline and placement
dependencies: ["WP04", "WP05"]
requirement_refs:
- C-004
- C-005
- C-008
- FR-011
- FR-012
- FR-013
- FR-016
- FR-017
- FR-018
- FR-019
- NFR-001
- NFR-002
- NFR-003
planning_base_branch: feat/asn1-tree-delete-copy-paste
merge_target_branch: feat/asn1-tree-delete-copy-paste
branch_strategy: Planning artifacts for this mission were generated on feat/asn1-tree-delete-copy-paste. During /spec-kitty.implement this WP may branch from a dependency-specific base, but completed changes must merge back into feat/asn1-tree-delete-copy-paste unless the human explicitly redirects the landing branch.
subtasks:
- T027
- T028
- T029
- T030
- T031
- T032
phase: Phase 3 - Copy and Paste
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: src/paste.rs
create_intent:
- src/paste.rs
execution_mode: code_change
model: ''
owned_files:
- src/paste.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP06 – Paste pipeline and placement

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

Close the loop opened by WP04 (bytes out via copy/cut/yank) and WP05 (clipboard text read into bytes): take bytes from either source, validate them as a complete BER/DER forest, and insert the resulting elements before, after, or as the first child of the selection — including the before/after placement dialog that appears only when the selection is the first sibling under normal bindings.

Done when:
- `PasteWhere { Before, After, AsChild }` and a `PasteSource` abstraction exist in new `src/paste.rs`.
- `paste_bytes(source, where)` validates full-byte consumption via `ber::parse_forest` and refuses (no document change) on any parse failure, reporting the reading used and the parser's own error.
- Ctrl+V (normal) prefers the system clipboard, falling back to the element buffer when the clipboard is empty/unavailable/holds no text; `p`/`P` (vim) always use the element buffer.
- The before/after dialog appears only when Ctrl+V is pressed with the selection as the first sibling of its parent, under normal bindings; vim's `p`/`P` never show it.
- Every region rule that `start_insert` already enforces (decrypted PKCS#8/PKCS#12 top-level SEQUENCE, placeholder/reveal rows) is honoured identically for paste.
- A successful paste selects the first pasted element, sets `dirty = true`, auto-expands a collapsed target parent, and reports the element count and the reading used.
- `cargo test paste::` passes for round-trip fidelity, dialog behaviour, invalid-input refusal (document unchanged), region-rule refusal, and clipboard-to-buffer fallback.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` User Story 2 (all eight acceptance scenarios) and User Story 5's `p`/`P` scenarios, plus the paste-related edge cases (empty document, breaking a region invariant, several PEM blocks, very large content).
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/clipboard-payload.md` — exact status wording for what paste reports (reading used, refusal messages).
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/research.md` R6 (placement/dialog decision) and R7 is WP08's concern, not this one's — skip it here.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/data-model.md` — `PasteWhere`, `PasteSource` field definitions.
- **`src/app.rs`'s existing `start_insert` in full** (search for `pub fn start_insert`) — this is the function whose region-rule checks (decrypted PKCS#8/PKCS#12 top-level SEQUENCE restriction, placeholder/reveal refusal, primitive-vs-constructed check for the `as_child` case) this WP must reuse, not reimplement. Read it closely before writing T028.
- `src/ber.rs`'s `parse_forest(data: &[u8], abs: usize) -> Result<Vec<Node>, ParseError>` — note it returns everything it could parse; **this WP must additionally check that parsing consumed every byte of `data`**, since `parse_forest` itself parses a sequence of top-level items and stops when `data` runs out, but does not itself guarantee the *caller's* full input was one clean forest with nothing trailing (re-read `parse_forest`'s loop in `src/ber.rs` around line 295 to confirm exactly what "consumed completely" means for this function before relying on it — the doc comment above it already says "filling `data` completely", so this may already be guaranteed; verify rather than assume, since NFR-002/FR-016 depend on this being true).
- `src/app.rs`'s `Mode::TypePicker`/`Mode::EditMenu` for the existing popup-menu (`MenuState`) pattern — T030 reuses this exact struct for the before/after dialog rather than inventing a new dialog type.

**Out-of-map edits this WP makes, with rationale**:
- `src/app.rs`: add a `Mode::PasteWhere(MenuState)` variant to the `Mode` enum — no other WP declares this variant.
- `src/tui.rs`: add a `handle_paste_where_key` function (or fold into the existing `handle_menu_key` if its shape already fits a two-item Up/Down/Enter/Esc/1/2 popup — check `handle_menu_key`'s current implementation before deciding whether to reuse or add a sibling function) and route `Mode::PasteWhere(_)` to it from the main `event_loop` match in `run`/`event_loop`; add `TreeAction::PasteAfter`/`PasteBefore` dispatch arms to `handle_document_key`.

This WP does **not** implement Paste as child's menu entry (`Edit ▸ Paste as child`, with no key in either binding set) — that menu wiring is WP07's job. This WP **does** implement the underlying `paste_bytes(source, PasteWhere::AsChild)` function itself, since WP07 needs it to exist to call it; just do not add a keyboard shortcut or menu item for it here.

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T027 – Define PasteWhere and PasteSource in new src/paste.rs

- **Purpose**: The vocabulary the rest of this WP is built on.
- **Steps**:
  1. Module doc comment in new `src/paste.rs` explaining the pipeline (bytes in from clipboard or buffer → validated forest → inserted at a position) — link to the flow diagram in `plan.md`'s "Key flows" section in your own words.
  2. `pub enum PasteWhere { Before, After, AsChild }`.
  3. `pub enum PasteSource<'a> { Clipboard, Buffer(&'a crate::buffer::ElementBuffer) }` (or, if a lifetime parameter proves awkward given how it will be called from `tui.rs`'s dispatch, an owned `enum PasteSource { Clipboard, Buffer(Vec<u8>) }` cloning the buffer's bytes at call time instead — pick whichever is more ergonomic once you attempt T029's call sites; record which you chose and why).
- **Files**: `src/paste.rs` (new).
- **Parallel?**: No — everything else in this WP builds on these two types.
- **Notes**: Keep `PasteSource` minimal — it exists only to tell `paste_bytes` where to read from, not to carry any placement or validation state.

### Subtask T028 – Implement paste_bytes(source, where)

- **Purpose**: FR-016, FR-018, C-004, C-008 — the validated, region-rule-respecting insertion itself.
- **Steps**:
  1. `impl App { pub fn paste_bytes(&mut self, source: PasteSource, at: PasteWhere) { ... } }` in `src/paste.rs`.
  2. Resolve `source` to `(Vec<u8>, description: String)`: for `Clipboard`, call `clipboard::read()` (existing function) then `clipboard::bytes_for_paste(&data)` (WP05); on any failure (no clipboard content, or `bytes_for_paste` erroring on odd hex) with `source == Clipboard`, **fall back to the element buffer** if one exists (per FR-017), rather than immediately failing — re-read FR-017's exact condition ("system clipboard is empty, holds no text, or no helper program is available") and decide whether an odd-hex-digit *error* from a clipboard that does hold text should also fall back, or should be reported as a paste failure in its own right (the contract's wording suggests a hex-looking-but-malformed clipboard is a genuine refusal, not a silent fallback — re-read `contracts/clipboard-payload.md`'s table once more and make the distinction explicit in a code comment, since this is a real judgment call the contract does not spell out to the letter).
  3. For `Buffer(buf)`, take `buf.bytes.clone()` directly — no `bytes_for_paste` reading needed, the element buffer's bytes are always raw DER already (WP04's `ElementBuffer::from_operand` guarantees this).
  4. Call `ber::parse_forest(&bytes, 0)`. Confirm (per this WP's Context note on `parse_forest`) that the returned nodes' total consumed length equals `bytes.len()` — if `parse_forest` does not already guarantee this itself, check the sum of parsed nodes' `header_len + content_len` against `bytes.len()` explicitly and treat a mismatch as a parse failure with a "trailing data" message.
  5. On parse failure: `self.status = format!("{reading}: {parser_message}", ...)` (matches `contracts/clipboard-payload.md`'s example format `"read as hex digits: truncated length field at offset 7"`), and **return without mutating anything** — no `rebuild()`, no `dirty = true`. This is the crux of NFR-003.
  6. On parse success, apply the same region-rule checks `start_insert` performs today (reuse its private helper functions or inline logic — read `start_insert` once more here specifically for this step): refuse if the target row is `DecryptedPlaceholder`/`CmsRevealed`/an elided filter row; refuse `Before`/`After` at the top level of a decrypted PKCS#8 value or PKCS#12 region (must remain one top-level SEQUENCE, per C-008); refuse `AsChild` on a primitive, non-encapsulating target.
  7. Insert the parsed nodes at the right index: `After` → `parent.children.insert(last + 1, ...)` for each parsed node in order (or splice all at once); `Before` → `parent.children.insert(last, ...)`; `AsChild` → `parent.children.splice(0..0, parsed_nodes)` on the target's own children (index 0, first children). Handle the empty-document case (per spec.md's edge case "Pasting into an empty document"): `Before`/`After`/`AsChild` on an empty document all insert the parsed nodes as the new top level.
  8. `self.rebuild()`, `self.dirty = true`, auto-expand the target parent if it was collapsed (reuse whatever `start_insert` does for this — it already has this behaviour for its own inserts), select the first pasted row, set the success status message naming the count and the reading used.
- **Files**: `src/paste.rs`.
- **Parallel?**: No — the central subtask of this WP.
- **Notes**: This function's region-rule reuse is the same kind of "call `start_insert`'s existing helpers, do not duplicate" discipline WP03's `operand()` needed for `delete_selected`'s helpers — if those helpers are private to `app.rs`, make them `pub(crate)` rather than copying their logic.

### Subtask T029 – Implement Ctrl+V / p / P dispatch

- **Purpose**: Route the keyboard to `paste_bytes` with the right source and placement, including the dialog trigger.
- **Steps**:
  1. In `src/tui.rs`'s `handle_document_key`, add dispatch for `TreeAction::PasteAfter` and `TreeAction::PasteBefore` (from WP01's `translate`).
  2. Normal bindings, `PasteAfter` (Ctrl+V): if the current selection is **not** the first sibling of its parent, call `app.paste_bytes(PasteSource::Clipboard, PasteWhere::After)` directly. If it **is** the first sibling, do not paste yet — instead open the before/after dialog (T030) so the user chooses; do not assume "After" as a silent default in this specific case, per FR-012.
  3. Vim bindings, `PasteAfter` (`p`) → `app.paste_bytes(PasteSource::Buffer(...), PasteWhere::After)` unconditionally, no dialog, using `app.element_buffer.as_ref()` (handle `None` gracefully — "nothing to paste" status, per `contracts/clipboard-payload.md`'s element-buffer-fallback section, applied here to the vim-only, buffer-only case).
  4. Vim bindings, `PasteBefore` (`P`) → same as step 3 but `PasteWhere::Before`, no dialog.
  5. Determining "is the first sibling of its parent": derive from the current selection's row `path` (its last path segment `== 0`), the same technique WP03 used for deriving sibling index — reuse that pattern rather than inventing a new one.
- **Files**: `src/tui.rs` (part of the declared out-of-map edit).
- **Parallel?**: Can be drafted once T027/T028 exist, in parallel with T030's dialog UI.
- **Notes**: `keymap::translate` returns the same `TreeAction::PasteAfter` for both normal-Ctrl+V and vim-`p` per the contract (re-check `contracts/keymap.md` here too) — the *source* (clipboard vs. buffer) and *dialog-or-not* behaviour is entirely this dispatch function's job to decide based on `app.bindings`, not something `translate` encodes.

### Subtask T030 [P] – Add PasteWhere dialog mode

- **Purpose**: FR-012 — let the user choose before/after when pasting at a first sibling under normal bindings.
- **Steps**:
  1. Add `PasteWhere(MenuState)` to the `Mode` enum in `src/app.rs` (the declared out-of-map edit), reusing the existing `MenuState { title, items: Vec<MenuItem>, selected }` struct from `src/app.rs`'s popup-menu machinery (used today by `Mode::EditMenu`).
  2. When the dialog needs to open (from T029 step 2), construct `MenuState { title: "Paste", items: vec![MenuItem { .. "Paste before" .. }, MenuItem { .. "Paste after" .. }], selected: 1 }` (default to "after", matching the common case, but let the user pick either) and set `app.mode = Mode::PasteWhere(state)`.
  3. Add key handling: reuse `handle_menu_key`'s existing Up/Down/`1`/`2`/Enter/Esc pattern (check whether it is generic enough over `Mode::EditMenu` to extend directly to `Mode::PasteWhere`, or whether a near-identical sibling function `handle_paste_where_key` is cleaner — prefer extending the existing one if it is already written generically against `MenuState`, to avoid duplicating the same five-line key-handling logic twice).
  4. On confirm, call `app.paste_bytes(PasteSource::Clipboard, PasteWhere::Before | After)` per the selected item, then return to `Mode::Browse`. On Esc, return to `Mode::Browse` with no paste and no status change (per FR-012: "Esc pastes nothing").
- **Files**: `src/app.rs` (the `Mode` variant), `src/tui.rs` (key handling and the dispatch arm in the main mode `match` in `event_loop`).
- **Parallel?**: Yes — can be drafted against a stub call to `paste_bytes` while T028 is still being finished, then wired to the real function once available.
- **Notes**: This dialog is deliberately small (two items, no description column needed beyond "Paste before"/"Paste after") — do not over-build it relative to the existing `EditMenu` popup's visual weight.

### Subtask T031 – Wire cursor/dirty/expand/status on paste

- **Purpose**: Confirm T028's side effects (cursor placement, dirty flag, auto-expand, status wording) are complete and correctly ordered — this subtask is a verification and polish pass over what T028 built, not new independent logic.
- **Steps**:
  1. Re-read T028 step 8 against spec.md User Story 2's acceptance scenarios 3 and 4 (Given the clipboard holds valid data … the cursor moves to the first pasted element … the status line says how many elements were pasted and how the clipboard text was read) and confirm every clause is implemented, not just "some status message is set".
  2. Confirm the auto-expand behaviour specifically: paste into a **collapsed** constructed element as its child, or after/before a sibling whose parent is collapsed — the parent must expand so the pasted rows are visible, matching `start_insert`'s existing behaviour for the same situation (re-verify by testing this exact scenario in T032, not just by reading the code).
  3. Confirm the status wording distinguishes `PasteKind::Hex`/`Base64`/`Pem(n)`/`Binary` per `contracts/clipboard-payload.md`'s reading column, and the element-buffer-fallback wording per that contract's "Element buffer fallback" section, and the "nothing to paste" wording when both are empty.
- **Files**: `src/paste.rs` (any gaps found are fixed here, not treated as a new file).
- **Parallel?**: No — depends on T028/T029/T030 all existing to verify against.
- **Notes**: If this subtask finds T028 already complete on every point, that is a valid, expected outcome — record in the Activity Log that verification passed with no changes needed, rather than inventing busywork.

### Subtask T032 – Unit tests: paste round trip / dialog / refusal

- **Purpose**: Prove NFR-002, NFR-003, and the dialog/fallback behaviours concretely.
- **Steps**:
  1. `copy_then_paste_round_trips_byte_identical`: using `test_app`, select an element, `copy_operand()`, select another sibling, `paste_bytes(PasteSource::Buffer(...), PasteWhere::After)`, `save()` (or directly compare `ber::encode_node` of the pasted node against the original's encoding), assert byte-for-byte identity — this is the concrete test behind NFR-002/SC-003.
  2. `dialog_triggers_only_on_first_sibling_under_normal_bindings`: set `app.bindings = Normal`, select a non-first sibling, simulate Ctrl+V, assert no dialog (`app.mode` stays `Browse`, paste happens immediately); select the first sibling, simulate Ctrl+V, assert `app.mode` is now `PasteWhere(_)` and no paste has happened yet.
  3. `vim_paste_never_shows_dialog`: set `app.bindings = Vim`, select the first sibling, simulate `p`, assert the paste happens immediately with no dialog, regardless of sibling position.
  4. `invalid_paste_leaves_document_unchanged`: feed `paste_bytes` a `PasteSource` whose bytes are deliberately truncated/invalid, assert `app.dirty` is still `false` and the document's row count is unchanged before and after the call.
  5. `region_rule_refuses_paste_into_decrypted_pkcs8_top_level`: reuse whatever existing test fixture/flow the codebase already has for a decrypted PKCS#8 document (check `src/pkcs8.rs`'s or `src/app.rs`'s existing tests for a pattern), attempt `Before`/`After` at that top level, assert refusal with no change.
  6. `clipboard_empty_falls_back_to_element_buffer`: with `app.bindings = Normal`, an empty/unavailable clipboard (realistic in CI, per WP04's T022 note), and a non-empty `app.element_buffer`, simulate Ctrl+V, assert the buffer's content was pasted and the status says so.
- **Files**: `src/paste.rs` (test module).
- **Parallel?**: No — exercises the finished T027–T031.
- **Notes**: For the round-trip test, prefer comparing full document `save()` output over comparing individual node encodings, since that is closer to what NFR-002/SC-003 actually promise the user ("the pasted element's saved DER encoding is byte-identical to the source").

## Test Strategy

- `cargo test paste::` must pass.
- Run the full `cargo test` once at the end, including `tests/dumpasn1_compat.rs`, to confirm the round-trip fidelity claim holds against real certificate/CRL/CMS test data, not just hand-built fixtures — consider adding one test in this WP that pastes a real extension copied from `testdata/chain/server.der` into another test document and confirms `dumpasn1`-style structural comparison still matches (reuse whatever comparison helper `dumpasn1_compat.rs` already exposes, if it is reusable from a unit test — check before assuming it is only usable as a standalone integration test).

## Risks & Mitigations

- **`parse_forest`'s "consumes completely" guarantee**: verify this by reading the function, not by assuming the doc comment is exhaustive — a subtle gap here would let truncated pastes silently succeed with dropped trailing bytes, violating FR-016/NFR-003.
- **Region-rule duplication drift**: if `start_insert`'s checks cannot be cleanly reused (e.g. they are tightly coupled to insert-specific state), resist the temptation to copy-paste the logic — refactor the shared check into a `pub(crate)` helper both functions call, even if that means a small, justified edit to `start_insert` itself beyond this WP's own new file (note any such edit explicitly in the Activity Log with rationale, since it is technically outside `src/paste.rs`).

## Review Guidance

- Confirm the round-trip test (T032) actually asserts byte equality, not just "some elements exist afterward" — a weak assertion here would hide a real fidelity bug.
- Confirm the before/after dialog truly never appears under vim bindings — trace T029's dispatch logic by hand for the vim branch.
- Confirm no `rebuild()`/`dirty = true` occurs on any refusal path — this is the concrete guarantee behind NFR-003 and is worth tracing by hand across every early-return in `paste_bytes`.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP06 --to <status>` to change WP status.
