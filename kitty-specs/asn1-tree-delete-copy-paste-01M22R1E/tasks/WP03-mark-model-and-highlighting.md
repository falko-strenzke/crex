---
work_package_id: WP03
title: Mark model and highlighting
dependencies: []
requirement_refs:
- FR-001
- FR-002
- FR-003
- FR-004
- FR-005
subtasks:
- T011
- T012
- T013
- T014
- T015
- T016
phase: Phase 2 - Marking
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: src/mark.rs
create_intent:
- src/mark.rs
execution_mode: code_change
model: ''
owned_files:
- src/mark.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP03 – Mark model and highlighting

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

Let a user extend a mark over contiguous siblings with Shift+Up/Down, anchored at the element selected when marking began, and see it rendered distinctly in the Structure pane. This is the foundation every later tree operation (delete, cut, copy, paste-as-child target checks) reads via a single `operand()` accessor — get its edge cases right here and every downstream WP inherits correctness for free.

Done when:
- `Mark { source, parent, anchor, active }` exists per `data-model.md`, stored by `RowSource` + parent path + sibling indices, never by row index.
- Shift+Down/Shift+Up extend, shrink, and — critically — can cross back past the anchor to continue the mark on the other side, per spec.md's edge case "Mark that reaches the anchor from the other side".
- The mark stops at the parent's first/last sibling with a status message, rather than wrapping or silently doing nothing.
- Any selection change other than `mark_extend` clears the mark, enforced through one shared funnel every navigation method calls.
- Marking is refused (with a status message) while the tree filter is active, or on a placeholder/read-only-reveal row.
- Marked rows render with a highlight distinct from the cursor row in `draw_tree`.
- `cargo test app::tests::mark` (or wherever the test module ends up — see T016) passes every scenario above.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` User Story 1 (all seven acceptance scenarios) and its "Edge Cases" section, specifically: marking on the last/first sibling, marking with a filter active, marking a collapsed constructed element, and the anchor-crossing case.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/data-model.md` — the `Mark` field table and its invariants (anchor always inside the range; active never leaves `0..parent.children.len()`; never exists under a filter or on `DecryptedPlaceholder`/`CmsRevealed` rows; cleared by any non-`mark_extend` selection change).
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/research.md` R9 and R10 — why paths+indices, not row indices; and the highlighting approach.
- `src/app.rs`: read `Row`, `RowSource`, and every navigation method that currently changes `self.selected` — `move_by`, `select`, `collapse_or_parent`, `expand_or_child`, `toggle_expand`, the focus-toggle path, and how `start_filter`/filter changes interact with selection. List every one you find before writing T013; missing one is exactly the bug class this WP exists to prevent.
- `src/tui.rs`: read `draw_tree` (~line 2331 onward) in full to understand the existing row-styling pipeline (fold markers, field prefixes, existing selection highlight) before adding a second highlight layer.

**Out-of-map edits this WP makes, with rationale**:
- `src/app.rs`: add one field, `mark: Option<Mark>`, to the `App` struct — no other WP declares this field.
- `src/tui.rs`: add two `match` arms to `handle_document_key` for `TreeAction::MarkUp`/`MarkDown` (from WP01's `keymap::translate`) — the first WP to consume these two specific actions.

This WP does **not** implement delete/cut/copy/paste — those consume `operand()` in WP04/WP06. Build `operand()` to be genuinely useful to those callers (returning enough information to act on: source, parent path, and the sibling index range) without guessing at their exact call signature; read `data-model.md`'s `Operand` section for the agreed shape.

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T011 – Define Mark struct and mark_extend(delta) in new src/mark.rs

- **Purpose**: The data structure and its core mutation.
- **Steps**:
  1. Create `src/mark.rs` with a module doc comment explaining the mark model (why paths+indices, not row indices — summarise research.md R9 in your own words, do not just paste it).
  2. `pub struct Mark { pub source: crate::app::RowSource, pub parent: Vec<usize>, pub anchor: usize, pub active: usize }` (check `RowSource`'s actual visibility in `src/app.rs` — it may already be `pub`; if any field needs to become `pub` or `pub(crate)` to be usable from `mark.rs`, that is another narrow out-of-map edit to note in this WP's Context section update before merging, or better, check first — it likely already is `pub` given `Row` itself is `pub`).
  3. `impl Mark { pub fn range(&self) -> std::ops::RangeInclusive<usize> { self.anchor.min(self.active)..=self.anchor.max(self.active) } pub fn count(&self) -> usize { self.range().count() } }`.
  4. `impl App { pub fn mark_extend(&mut self, delta: isize) { ... } }` in this file: `delta` is `+1` for Shift+Down, `-1` for Shift+Up (the caller in `src/tui.rs` translates the `TreeAction` into this call). If `self.mark` is `None`, start one: anchor = active = the sibling index of the current selection within its parent (derive this from `self.rows[self.selected].path`); if the current selection's row cannot be marked (see T014's refusal list), set a status message and return without creating a mark. If `self.mark` is `Some`, move `active` by `delta`, clamped — see T012.
- **Files**: `src/mark.rs` (new).
- **Parallel?**: No — the base every other subtask in this WP builds on.
- **Notes**: Deriving "sibling index within its parent" from a `Row`'s `path: Vec<usize>` is just `path.last().copied()` for the index and `path[..path.len()-1]` for the parent — but double-check this against how `path` is actually constructed in `rebuild_rows`/`rebuild` (read those functions in `src/app.rs` first) rather than assuming.

### Subtask T012 – Implement bounds checking (stop at first/last sibling with status message)

- **Purpose**: FR-002 and the "marking on the last/first sibling" edge case.
- **Steps**:
  1. Before moving `active` by `delta` in `mark_extend`, compute the parent's sibling count (look up the parent node's `children.len()`, or `self.roots.len()` if `parent` is empty — for the correct `roots`/forest per `self.mark.source`, matching the pattern used elsewhere for `RowSource::Document`/`Decrypted`/`Pkcs12Revealed(idx)`).
  2. If `active + delta` would go outside `0..sibling_count`, do not change `active`; instead set `self.status` to a message matching spec.md's wording intent: `"the mark cannot extend past the parent's last element"` (Shift+Down at the end) or the symmetric wording for Shift+Up at the start.
  3. Otherwise, `active = (active as isize + delta) as usize` (i.e. it moves freely, including back past `anchor` — do not clamp `active` to stay on one side of `anchor`; that is exactly the anchor-crossing behaviour T011's caller relies on and this subtask must not accidentally prevent).
- **Files**: `src/mark.rs`.
- **Parallel?**: No — refines T011's `mark_extend`.
- **Notes**: The bound is the *parent's* sibling count, which can differ from what a filtered view would show — but marking is refused entirely under a filter (T014), so this bound only ever applies to the true, unfiltered sibling list.

### Subtask T013 – Implement clear_mark() via a shared selection funnel

- **Purpose**: FR-005 — the mark must never survive a selection change that was not itself extending the mark.
- **Steps**:
  1. `pub fn clear_mark(&mut self) { self.mark = None; }` on `App`, in `src/mark.rs`.
  2. Identify (by reading `src/app.rs` and `src/tui.rs`) every existing method that changes `self.selected`: at minimum `move_by`, `select`, `collapse_or_parent`, `expand_or_child`, `toggle_expand`, and the browser/document focus toggle. For each one found, add a `self.clear_mark();` call at its entry (or wrap the whole set behind one small private helper, e.g. `fn set_selected(&mut self, new: usize) { self.clear_mark(); self.selected = new; }`, and change each of those methods to route their final assignment through it — whichever is less invasive to the existing code without changing its other behaviour). Prefer the funnel-helper approach if the existing methods already converge on a small number of assignment points; prefer scattered explicit calls if they do not, rather than forcing an awkward refactor.
  3. Also clear the mark when the tree filter changes (`start_filter`/`filter_insert_char`/wherever `self.filter` is mutated) and when `Esc` is pressed while a mark is active in the document pane (this second case may already be partly handled if `Esc` routes through a selection-changing path — verify, do not assume).
  4. Do **not** call `clear_mark()` from within `mark_extend` itself (that would defeat the whole feature).
- **Files**: `src/app.rs` (the field addition, plus wiring `clear_mark()` calls into the navigation methods it finds — this is the WP's declared out-of-map edit, and it is inherently spread across several existing methods; keep each individual call a one-line addition, do not restructure the methods themselves), `src/mark.rs` (the `clear_mark` function itself).
- **Parallel?**: No — this is the subtask most likely to reveal a missed navigation method; do it carefully and exhaustively before moving on.
- **Notes**: T016's test suite exists specifically to catch a missed call site here — write that test before considering this subtask done, not after, so gaps are caught while the list of navigation methods is still fresh in mind.

### Subtask T014 – Implement operand() with region refusals

- **Purpose**: The one accessor WP04 (delete/cut/copy/yank) and WP06 (paste-as-child's target check) will call to get "what should this operation act on".
- **Steps**:
  1. `pub struct Operand { pub source: RowSource, pub parent: Vec<usize>, pub range: std::ops::RangeInclusive<usize> }` (or reuse `Mark`'s shape directly if it turns out identical enough — check `data-model.md`'s `Operand` section for the agreed field list before deciding; the two may differ only in that `Operand` is never `None` when a selection exists, whereas `Mark` legitimately is).
  2. `pub fn operand(&self) -> Result<Operand, String>` on `App`: if `self.mark` is `Some`, return it converted to `Operand`. Otherwise, derive `Operand` from the current selection's row (single-element range).
  3. Refuse (return `Err(message)`) when the relevant row is `RowSource::DecryptedPlaceholder` or `RowSource::CmsRevealed` ("decrypt the content before editing it", matching the existing wording used by `delete_selected`/`start_insert` for the same cases — reuse the exact string, do not invent new wording for an existing situation), when the selection is an elided filter placeholder (reuse whatever check `reject_elided_selection` already performs — call that existing helper rather than reimplementing its logic), or when the relevant row is the protected top-level SEQUENCE of a decrypted PKCS#8/PKCS#12 region (reuse `reject_uneditable_reveal` if that is the existing helper's name — check `src/app.rs` for the actual function name near `delete_selected`/`start_insert` first).
  4. This subtask's own refusals do **not** need to reject "no rows at all" (empty document) specially — an empty document has no selection to operate on, and callers (WP04) already handle "nothing to delete" by their own early return; keep `operand()` focused on the region/placeholder rules that are this mark model's responsibility.
- **Files**: `src/mark.rs`.
- **Parallel?**: No — depends on `Mark` (T011) and needs the existing refusal helpers identified.
- **Notes**: Do not duplicate the region-rule logic that already exists in `delete_selected`/`start_insert` — call the same private helper functions those use, so a future change to the rule only needs updating in one place. If those helpers are private to `app.rs` and not directly callable from `mark.rs`, either make them `pub(crate)` (a small, justified visibility change, not a logic change) or move the shared check into `mark.rs` itself and have `app.rs` call it instead — pick whichever keeps the logic in exactly one place.

### Subtask T015 [P] – Add marked-row highlighting to draw_tree

- **Purpose**: FR-004 — the user must be able to see what a mark spans before acting on it.
- **Steps**:
  1. In `src/tui.rs`'s `draw_tree`, for each row being styled, check whether it falls within `app.mark`'s range (same source, same parent, sibling index within `mark.range()`) and apply a distinct style — reuse the existing blue-background convention documented in the help window's "Editing values" topic for the value editors' selection, for visual consistency, but do not reuse the exact same style constant as the cursor-row highlight (they must be visibly distinct per FR-004, and the cursor row within a mark should show both, e.g. bold-on-mark-background, per research.md R10).
  2. Read how the existing cursor-row style is computed in `draw_tree` before adding this — the two must compose cleanly (a marked, non-cursor row; a marked, cursor row; an unmarked cursor row; a plain row — four visually distinct-enough combinations, though the unmarked/marked distinction is the one this subtask must get right per FR-004).
- **Files**: `src/tui.rs` (the out-of-map edit to `draw_tree` this WP declares).
- **Parallel?**: Yes — can be drafted against a stub/no-op `Mark` check while T011–T014 are still being finished, then wired to the real `app.mark` field once available.
- **Notes**: Keep the new style as a named constant near the top of `src/tui.rs`, alongside the existing style constants (e.g. `DIRTY_MARKER`), rather than an inline literal, matching the file's existing convention.

### Subtask T016 – Unit tests: extend/shrink/anchor-crossing/bounds/filter-refusal/clear-on-navigation/region-refusal

- **Purpose**: Prove every scenario in spec.md User Story 1 mechanically.
- **Steps**:
  1. Use the existing `test_app(data: &[u8]) -> App` helper (`src/app.rs`, near line 7231) as the fixture builder — do not write a new one. Pick or construct test DER data with at least four siblings under one constructed parent (check `testdata/` for a suitable existing file, e.g. a certificate with several extensions, before hand-constructing bytes).
  2. `mark_extends_and_shrinks_over_siblings`: select the first of several siblings, `mark_extend(1)` twice, assert `mark.range()` covers three elements; `mark_extend(-1)` once, assert it now covers two.
  3. `mark_stops_at_last_sibling_with_status`: extend past the last sibling, assert `active` unchanged and `status` is non-empty and mentions the boundary.
  4. `mark_crosses_back_past_the_anchor`: from a mark of `[anchor, anchor+1]` with the cursor at the extended end, shrink back past the anchor (`mark_extend(-1)` enough times) and assert the range now extends *above* into `[anchor-1, anchor]`, with the anchor still included — this is the scenario explained in the earlier conversation's clarification of that edge case; get it right.
  5. `marking_refused_under_active_filter`: set a non-empty `app.filter`, attempt `mark_extend(1)`, assert `app.mark` is still `None` and a status message explains why.
  6. `mark_clears_on_plain_navigation`: create a mark, call `app.move_by(1)` (or another navigation method), assert `app.mark` is `None` afterward. Repeat this same assertion pattern for at least `select`, `collapse_or_parent`/`expand_or_child`, and the filter-start path — one test per method or one parametrised test iterating over a list of closures, whichever reads more clearly.
  7. `operand_refuses_placeholder_and_reveal_rows`: construct or select a row with `RowSource::DecryptedPlaceholder` (or the closest reachable equivalent via the existing decrypt-flow test fixtures) and assert `operand()` returns `Err(..)`.
- **Files**: `src/mark.rs` (test module) — or `src/app.rs`'s existing test module if `Mark`/`operand` end up more naturally tested alongside `App`'s other tests; prefer `mark.rs` for cohesion since this WP owns that file, but do not fight the existing test organization if `test_app` and its neighbours are more conveniently reached from within `app.rs`'s own `#[cfg(test)] mod tests`.
- **Parallel?**: No — exercises the finished T011–T014.
- **Notes**: The anchor-crossing test (step 4) is the single most important test in this WP — it is the one behaviour most likely to be implemented subtly wrong (e.g. by clamping `active` to never pass `anchor`). Write it first if that helps drive the implementation, even though it is listed last here.

## Test Strategy

- `cargo test mark::` (or the actual module path chosen in T016) must pass.
- Run the full `cargo test` once at the end to confirm no existing navigation-method test broke from the `clear_mark()` wiring in T013.

## Risks & Mitigations

- **Missed navigation method in T013**: the single biggest correctness risk in this WP. Mitigate by listing every `self.selected = ` assignment site found via `grep -n "self.selected = " src/app.rs` before starting T013, and checking off each one explicitly in the Activity Log as it is routed through the clearing funnel.
- **Row-index vs. path/index confusion**: `self.rows` indices shift on every `rebuild()`; `Mark` must never store one. Review every place `self.selected` (a row index) is read inside `mark.rs` to confirm it is only ever used to *derive* the initial `parent`/`anchor` at mark-creation time, never stored directly.

## Review Guidance

- Manually trace the anchor-crossing test by hand against the implementation, not just by reading that the test passes — this is the scenario most likely to have a subtly wrong but test-passing implementation if the test itself has a bug.
- Confirm every method identified in T013 actually calls `clear_mark()` by reading the diff, not by trusting the WP's own claim.
- Confirm `operand()`'s refusal wording is copied verbatim from the existing `delete_selected`/`start_insert` strings where the situation is the same (placeholder, reveal-region), per DIRECTIVE_010 (fidelity) applied to user-facing text consistency.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP03 --to <status>` to change WP status.
