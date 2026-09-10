# WP03 Review Feedback — Review Cycle 1 — REJECTED

## Verdict: Changes requested

Everything else in this WP is solid — see "What was verified" below — but one
concrete, spec-mandated behavior is missing: **Esc does not clear an active
mark in the Structure pane.**

## Blocking issue: Esc does not clear the mark (FR-005 / Acceptance Scenario 5)

- `spec.md` FR-005: "I want any selection change without Shift, **or Esc**, to
  clear the mark."
- `spec.md` User Story 1, Acceptance Scenario 5: "**Given** a mark exists,
  **When** the user presses a plain cursor key, **Esc**, or any key that
  changes the selection, **Then** the mark is cleared."
- The WP prompt's own T013 guidance called this out by name: "clear the mark
  ... when `Esc` is pressed while a mark is active in the document pane (this
  second case may already be partly handled if `Esc` routes through a
  selection-changing path — verify, do not assume)."

Traced directly in `src/tui.rs`: `handle_document_key` (the key handler for
`Mode::Browse` + `Focus::Document`, i.e. the Structure/tree pane) has no
`KeyCode::Esc` arm. The function's Ctrl+S check and the new
`keymap::translate` MarkUp/MarkDown match both fall through for Esc (and
`keymap.rs` itself has no Esc mapping at all — confirmed via
`grep -n "Esc" src/keymap.rs`, zero hits), so the final
`match key.code { ... _ => {} }` swallows Esc with no effect. Pressing Esc
in the tree pane today does nothing at all — mark or no mark — so this is
not "partly handled" as the WP prompt allowed for; it is not handled.

Concretely: mark two siblings with Shift+Down, press Esc, `app.mark` is
still `Some(..)`. No test in the new `mark` module (`src/app.rs`,
`mod tests::mark`) exercises this path — the eight `mark_clears_on_*` tests
cover `move_by`, `select`, `collapse_or_parent`, `expand_or_child`,
`toggle_expand`, `start_filter`, `toggle_focus`, but not Esc.

## What was verified and is correct (no action needed)

- `Mark { source, parent, anchor, active }` stored by parent path + sibling
  index, never by row index — confirmed in `src/mark.rs`.
- Anchor-crossing traced by hand against `mark_extend`'s actual code with the
  reviewer-supplied numbers (anchor=2, active starts at 3, `mark_extend(-1)`
  twice): active becomes 1, `range()` becomes `1..=2`, matching
  `mark_crosses_back_past_the_anchor`. `active` is never clamped to
  `anchor`'s side; only the parent's sibling-count bound is checked.
  15/15 `cargo test mark::` pass, run independently.
- Bounds checking at first/last sibling sets a status message and does not
  wrap — confirmed in code and via
  `mark_stops_at_last_sibling_with_status`/`mark_stops_at_first_sibling_with_status`.
- `clear_mark()` funnel: independently grepped every `self.selected = ` and
  `self.rows = ` site in `src/app.rs`. All non-`mark_extend` paths that
  reassign `self.selected` or `self.rows` go through `select()` or
  `rebuild_rows()`, both of which call `self.clear_mark()` at entry;
  `toggle_focus()` and `start_filter()` also call it directly. This covers
  `move_by`, `collapse_or_parent`, `expand_or_child`, `toggle_expand`, filter
  editing (`filter_insert_char`/`filter_backspace`/`filter_delete`, all via
  `rebuild_rows`), and file open (only reachable from the Browser pane, which
  already cleared the mark via `toggle_focus`). This is the WP's highest-risk
  area and it holds, apart from the Esc gap above.
- `operand()`'s refusal refactor: `reject_elided_selection`/
  `reject_uneditable_reveal` now delegate to new pure helpers
  (`elided_reason`/`uneditable_reveal_reason`) with identical logic and
  status strings; `delete_selected`/`start_insert` are otherwise untouched
  and still carry their own operation-specific wording for the protected-root
  case. Behavior-preserving, confirmed by direct diff/read, not by trusting
  the WP's claim.
- New wording `"the decrypted root must remain one top-level SEQUENCE"` in
  `protected_root_reason` is a reasonable, justified departure from reusing
  existing strings verbatim: `operand()` is generic across future
  delete/cut/copy/paste-target callers (unlike `delete_selected`'s
  deletion-specific wording or `start_insert`'s insertion-specific wording),
  so a new, operation-neutral message is the right call here, not a fidelity
  violation.
- Marked-row highlighting (`src/tui.rs` `draw_tree`): `MARK_STYLE` (blue
  background) is a separate named constant from the cursor row's
  `List::highlight_style` (`Modifier::REVERSED`/`UNDERLINED`), so a marked
  row, the cursor row, and a marked cursor row (mark style + bold) are all
  visually distinct, satisfying FR-004.
- Marking refused under active filter and on placeholder/reveal rows:
  confirmed in `mark_extend`/`operand`/`row_refusal` and via
  `marking_refused_under_active_filter` and
  `operand_refuses_elided_and_placeholder_rows`.
- `cargo build`: clean. `cargo test mark::`: 15/15 pass. Full `cargo test`:
  409 passed, exactly the 2 pre-existing unrelated failures
  (`verify::tests::hsslms_single_level_verifies_via_openssl_and_multi_level_via_botan`,
  `app::tests::rekeying_to_single_level_lms_verifies_via_openssl_with_the_rfc9802_oid`),
  no new failures. `cargo clippy --lib`: 4 pre-existing warnings, none in the
  diff (`src/mark.rs`, the `mark` field/methods in `src/app.rs`, or the
  `draw_tree`/`handle_document_key` changes in `src/tui.rs`) — zero new
  warnings from this WP.

## Anti-pattern checklist

1. Dead code: PASS — `Mark`, `operand()`, `mark_extend`, `clear_mark`,
   `row_in_mark`/`MARK_STYLE` all have live production callers
   (`handle_document_key`, `draw_tree`).
2. Synthetic-fixture tests: PASS — all `mark::` tests drive real `App`
   methods (`mark_extend`, `select`, `move_by`, etc.) against a real parsed
   DER fixture, not literal-struct assertions.
3. Silent empty return: PASS — the early `return`s in `mark_extend`/
   `row_refusal` all set `self.status` first, or are the documented
   no-selection/no-forest edge cases.
4. FR coverage: FR-001/002/003/004 covered by tests; **FR-005 partially
   covered** (navigation-clears-mark is tested; Esc-clears-mark is not, and
   does not hold — see blocking issue).
5. Frozen surface: N/A — no frozen files declared for this WP beyond the
   declared out-of-map edits, which are within the WP's stated scope.
6. Locked decision: PASS — no diffed code contradicts a spec/plan MUST NOT.
7. Shared-file ownership: PASS — `src/app.rs`/`src/tui.rs` changes are the
   WP's own declared out-of-map edits; no other in-review WP touches them
   concurrently.
8. Production fragility: PASS — no new bare `raise`/panic-prone path
   introduced; `.expect("just ensured Some above")` in `mark_extend` is
   provably safe (immediately follows the `is_none()` branch that
   guarantees `Some`).

## Requested fix

Add an `Esc` arm to `handle_document_key` in `src/tui.rs` that calls
`app.clear_mark()` when `app.mark.is_some()` (a no-op otherwise, to avoid
changing today's "Esc does nothing else in the tree pane" behavior). Add a
test analogous to the other `mark_clears_on_*` tests
(`mark_clears_on_esc_in_document_pane` or similar) that marks, dispatches an
Esc key event through the real key-handling path (or the `App`-level
equivalent if a direct unit test of `handle_document_key` is impractical —
check how other `tui.rs`-level behaviors are tested in this codebase), and
asserts `app.mark` is `None` afterward.
