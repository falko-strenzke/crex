---
description: "Work package task list for ASN.1 Tree Delete, Copy and Paste"
---

# Work Packages: ASN.1 Tree Delete, Copy and Paste

**Inputs**: Design documents from `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md — all present

**Tests**: Required. The spec requires automated coverage (NFR-006) and the existing `dumpasn1_compat` integration test must keep passing unchanged.

**Organization**: Fine-grained subtasks (`Txxx`) roll up into work packages (`WPxx`). Each work package is independently deliverable and testable against its own subset of acceptance scenarios from spec.md.

## Subtask Format: `[Txxx] [P?] Description`

- **[P]** indicates the subtask can proceed in parallel with sibling subtasks in the same WP (different files/concerns).
- Subtasks are **reference rows**, not checkboxes: record completion with `spec-kitty agent tasks mark-status <Txxx> --status done`.

## Path Conventions

Single Rust crate: `src/` for implementation, `tests/` for the existing integration suite. Unit tests for this mission live in `#[cfg(test)] mod tests` blocks beside the code they test, matching the codebase's existing convention (see `src/app.rs`, `src/ber.rs`).

## File-ownership note (monolith codebase)

`src/app.rs` (~11k lines) and `src/tui.rs` (~5k lines) are shared, pre-existing files that most work packages must touch narrowly (one new struct field, one new `match` arm, one dispatch call). To keep `owned_files` non-overlapping across work packages, each WP below **creates and primarily owns one new module** (`keymap.rs`, `settings.rs`, `mark.rs`, `buffer.rs`, `paste.rs`, `vim.rs`) holding its new types and an `impl App { … }` block (Rust permits `impl` blocks for a type in files other than the one that defines it, within the same crate). The one or two lines each WP must add to `app.rs` / `tui.rs` to wire the new module in (a struct field, a `match` arm, a `mod` declaration) are **narrow, well-justified out-of-map edits**, called out explicitly with a one-line rationale in each WP's Context section, per the ownership rules. WP07 (Edit menu, Settings dialog) and WP09 (documentation) are the two WPs whose primary job **is** to edit `app.rs`/`tui.rs` directly (menu tables, help topics); both are placed at the end of the dependency chain so no other WP is concurrently editing those files at the same time.

---

## Work Package WP01: Key-binding sets and action translation (Priority: P0)

**Goal**: One shared translation from raw key events to semantic tree/editor actions for both the normal and vim key-binding sets, so every later WP dispatches on actions instead of raw keys and C-002 (vim table free of Ctrl+C/V/X/Z) is enforceable by a test.
**Independent Test**: `cargo test keymap::` passes: the vim table contains no CONTROL+{c,v,x,z} entries, every `TreeAction` has a key or is documented as menu-only per set, and both Shift+Up encodings translate to `MarkUp`.
**Prompt**: `tasks/WP01-key-binding-sets-and-action-translation.md`
**Requirement Refs**: FR-021, FR-025, FR-027, FR-033, C-001, C-002, C-003

### Included Subtasks

T001 [P] Define `KeyBindingSet`, `TreeAction`, `EditorAction` enums in new `src/keymap.rs`
T002 Implement `translate(key, set, context) -> Option<Action>` for tree actions (both sets)
T003 [P] Implement `label(action, set) -> Option<&'static str>` for menu/help key display
T004 Declare `pub mod keymap;` in `src/lib.rs`
T005 Unit tests: vim table has no Ctrl+C/V/X/Z; action/key coverage per contract; Shift+Up encodings

### Implementation Notes

Build the enums and the pure `translate`/`label` functions first (T001, T003 in parallel), then the matching logic (T002) which is the heart of contract `contracts/keymap.md`.

### Parallel Opportunities

T001 and T003 touch disjoint parts of the new file and can be drafted together before T002 wires them up.

### Dependencies

- None (starting package, alongside WP05).

### Risks & Mitigations

- Shift+Up may arrive as `KeyCode::Up` + `SHIFT` or as a distinct escape sequence on some terminals → test both forms per research.md R3.
- Ctrl+letter case folding must match `handle_edit_key`'s existing pattern → reuse the same fold-before-match idiom.

---

## Work Package WP02: Settings persistence and start-up loading (Priority: P0)

**Goal**: A persisted, per-user key-binding preference that survives restarts, using no new dependencies, that never blocks start-up even when the file is missing or broken.
**Independent Test**: `cargo test settings::` passes: path resolution per OS (via env-var overrides in-test), round trip, unknown-key preservation, malformed-file fallback, missing-file silent default.
**Prompt**: `tasks/WP02-settings-persistence-and-startup-loading.md`
**Requirement Refs**: FR-022, FR-023, FR-024, NFR-004, C-005, C-006, C-007

### Included Subtasks

T006 [P] Implement `settings::config_path()` per OS from environment variables
T007 Implement `Settings`, `LoadOutcome`, and the TOML-subset `load()`/parse() reader preserving unknown keys
T008 Implement `save()` with temp-file-then-rename write and directory creation
T009 Wire `src/main.rs` to load settings before the TUI starts, pass the binding set and any `LoadOutcome::Invalid` notice into `App`
T010 Unit tests: path per OS, round trip, unknown keys preserved, malformed file → `Invalid` with reason, missing file → silent defaults

### Implementation Notes

Follow `contracts/settings-file.md` exactly for location and format. `App` gains a `bindings: KeyBindingSet` field (a narrow, one-line out-of-map edit to `src/app.rs`'s struct — rationale: no other WP declares that field) and a `Mode::Notice` call when `LoadOutcome::Invalid` is returned (reuses the existing notice mechanism).

### Parallel Opportunities

T006 (path resolution) is independent of T007/T008 (parsing/writing) and can be drafted first or in parallel.

### Dependencies

- Depends on WP01 (`KeyBindingSet` type).

### Risks & Mitigations

- No config-directory environment variable set (rare, minimal containers) → `config_path()` returns `None`; settings stay in-memory only, Settings dialog says so (FR-024 covers this).
- Concurrent app.rs struct edit with WP03 (Mark field) → both are single-line additions to `App`'s field list; low conflict risk, resolved trivially at merge if it occurs.

---

## Work Package WP03: Mark model and highlighting (Priority: P0)

**Goal**: Represent a contiguous-sibling mark anchored at the selection, extend/shrink it with Shift+Up/Down, and render it distinctly in the Structure pane.
**Independent Test**: `cargo test app::tests::mark` passes: extend/shrink, parent bounds, anchor-crossing, filter refusal, clearing on any non-mark selection change, refusal on placeholder/reveal rows.
**Prompt**: `tasks/WP03-mark-model-and-highlighting.md`
**Requirement Refs**: FR-001, FR-002, FR-003, FR-004, FR-005

### Included Subtasks

T011 Define `Mark` struct and `mark_extend(delta)` in new `src/mark.rs` (`impl App`)
T012 Implement bounds checking (stop at first/last sibling with status message)
T013 Implement `clear_mark()` routed through a single `set_selected()` funnel used by every navigation method
T014 Implement `operand()` deriving the marked range or the selection, with region/placeholder refusals
T015 [P] Add marked-row highlighting to `draw_tree` in `src/tui.rs`, distinct from the cursor style
T016 Unit tests: extend/shrink/anchor-crossing/bounds/filter-refusal/clear-on-navigation/region-refusal

### Implementation Notes

`Mark` is stored as `(RowSource, parent path, anchor index, active index)`, never row indices (those are invalidated by every `rebuild()`), per data-model.md. `App` gains a `mark: Option<Mark>` field (narrow out-of-map edit to `src/app.rs`) and `handle_document_key` in `src/tui.rs` gains two `match` arms for `TreeAction::MarkUp`/`MarkDown` (narrow out-of-map edit).

### Parallel Opportunities

T015 (rendering) can be drafted against a stub `Mark` type while T011–T014 (model) are finished.

### Dependencies

- Depends on WP01 (`TreeAction::MarkUp`/`MarkDown`).

### Risks & Mitigations

- Every existing navigation method (`move_by`, `select`, `collapse_or_parent`, `expand_or_child`, `toggle_expand`, browser focus switch, filter changes) must route through the clearing funnel — missing one leaves a stale mark. Mitigate with a test that exercises each navigation method with a mark active and asserts it clears.
- Marking must be refused while `filter` is non-empty (edge case in spec.md) — test this explicitly, it is easy to miss.

---

## Work Package WP04: Element buffer, delete/cut/copy/yank of the operand (Priority: P0)

**Goal**: Turn the operand (mark or selection) into DER bytes once, share those bytes between the system clipboard and an in-app element buffer, and extend the existing two-step delete confirmation to multi-element ranges.
**Independent Test**: `cargo test buffer::` and `cargo test app::tests::delete_operand` pass: range delete with correct cursor placement, copy fills clipboard + buffer, copy survives clipboard failure, cut aborts when copy reaches neither destination, yank fills the buffer only.
**Prompt**: `tasks/WP04-element-buffer-and-operand-editing.md`
**Requirement Refs**: FR-006, FR-007, FR-008, FR-009, FR-010, FR-020, FR-026, NFR-001, NFR-003

### Included Subtasks

T017 Define `ElementBuffer` struct in new `src/buffer.rs`
T018 Implement `delete_operand()` in `src/buffer.rs` (`impl App`), replacing `delete_selected`'s body: multi-row two-step confirm, high-to-low removal, cursor-placement rule (FR-008)
T019 Implement `copy_operand()`: `ber::encode_forest` of the operand into the buffer, `clipboard::write` for normal bindings, clipboard-failure status wording
T020 Implement `cut_operand()` and `yank_operand()`: copy-then-delete without re-confirmation (abort with no change if neither destination received the data); yank fills the buffer only
T021 Wire `TreeAction::Delete/Cut/Copy` dispatch in `src/tui.rs`'s `handle_document_key` (vim's `y` also translates to `Copy`, not a separate `Yank` action)
T022 Unit tests: range delete + cursor placement (FR-008); copy fills both destinations; copy survives clipboard failure; cut aborts on total copy failure; yank is buffer-only

### Implementation Notes

`delete_selected` in `src/app.rs` is superseded: its body moves to `delete_operand()` in the new file and the old function becomes a one-line wrapper (`self.delete_operand()`) so the existing `d`-key call site keeps working without touching `handle_document_key` twice. This is the one deliberate, documented out-of-map edit to `src/app.rs` in this WP.

### Parallel Opportunities

T017 (struct) blocks the rest; T019 and T020 can be drafted together once T018's removal logic exists, since both build on the same operand-consuming pattern.

### Dependencies

- Depends on WP03 (`Mark`/`operand()`).

### Risks & Mitigations

- Deleting a multi-element range must remove indices high-to-low or later removals shift earlier ones — cover with a test deleting three non-adjacent-looking (but sibling-contiguous) indices.
- NFR-001 (250 ms budget for 500 elements / 1 MiB): keep deletion and buffer-fill to O(n) over the operand, no extra full-tree scans.

---

## Work Package WP05: Clipboard paste-source reading with PEM support (Priority: P0)

**Goal**: Extend the existing three-step clipboard reading (hex → base64 → raw) with PEM-armoured text (one or more blocks), producing the bytes and `PasteKind` the paste pipeline needs.
**Independent Test**: `cargo test clipboard::` passes for hex, base64, single PEM block, multiple PEM blocks, raw fallback, and the odd-hex-digit refusal, each with the exact status wording from `contracts/clipboard-payload.md`.
**Prompt**: `tasks/WP05-clipboard-pem-reading.md`
**Requirement Refs**: FR-015, FR-016

### Included Subtasks

T023 Implement `clipboard::bytes_for_paste(data) -> Result<(Vec<u8>, PasteKind), String>`
T024 Add `PasteKind::Pem(usize)` variant and its `describe()` wording per contract
T025 [P] Ensure odd hex-digit count is refused at step 1 rather than falling through to base64
T026 Unit tests: hex, base64, one PEM block, multiple PEM blocks, raw fallback, odd-hex-digit refusal

### Implementation Notes

Reuse the PEM label/body splitting already implemented in `src/input.rs` (see its BEGIN/END marker handling) rather than reimplementing PEM parsing. Read `contracts/clipboard-payload.md` for the exact reading order and status wording before writing tests.

### Parallel Opportunities

Fully independent of WP01–WP04: touches only `src/clipboard.rs` and can run in the same lane as WP01 from the start.

### Dependencies

- None (starting package, alongside WP01).

### Risks & Mitigations

- Reusing `input.rs`'s PEM logic without duplicating it: import the existing helper rather than copy-pasting the marker search, to avoid drift between the two PEM readers.

---

## Work Package WP06: Paste pipeline and placement (Priority: P0)

**Goal**: Read bytes from the clipboard (falling back to the element buffer) or from the buffer directly, validate them as a complete BER/DER forest, and insert them before, after, or as the first child of the selection — including the before/after dialog for a first sibling under normal bindings.
**Independent Test**: `cargo test paste::` passes: copy → paste round trip byte-identical (NFR-002), before/after dialog only on a first sibling under normal bindings, vim `p`/`P` never dialog, invalid paste leaves the document unchanged (`dirty` stays false, NFR-003), region-rule refusal for decrypted PKCS#8/PKCS#12 top level, clipboard-empty falls back to the buffer.
**Prompt**: `tasks/WP06-paste-pipeline-and-placement.md`
**Requirement Refs**: FR-011, FR-012, FR-013, FR-016, FR-017, FR-018, FR-019, NFR-001, NFR-002, NFR-003, C-004, C-005, C-008

### Included Subtasks

T027 Define `PasteWhere { Before, After, AsChild }` and `PasteSource` in new `src/paste.rs`
T028 Implement `paste_bytes(source, where)` (`impl App`): `parse_forest` full-consumption validation, region-rule checks reused from `start_insert`, insertion at the right index
T029 Implement Ctrl+V / `p` / `P` dispatch: normal bindings prefer clipboard then fall back to the buffer; vim bindings always use the buffer; before/after dialog triggers only when the selection is the first sibling under normal bindings
T030 Add `Mode::PasteWhere(MenuState)` dialog state and its key handling (up/down, `1`/`2`, Enter, Esc) in `src/tui.rs`
T031 Wire cursor placement on the first pasted element, `dirty = true`, auto-expand of a collapsed parent, status wording naming the element count and the reading used
T032 Unit tests: round trip byte-identical; dialog only on first sibling under normal bindings; vim no-dialog; invalid paste leaves document unchanged; region-rule refusal; buffer fallback

### Implementation Notes

`start_insert`'s existing region-rule checks (decrypted PKCS#8/PKCS#12 top level, placeholder/reveal rows) are the pattern to reuse, not reimplement — read that function in `src/app.rs` before writing T028. `App` gains a `Mode::PasteWhere` match arm in `src/tui.rs`'s mode dispatch (narrow out-of-map edit) and a `TreeAction::PasteAfter/PasteBefore` dispatch in `handle_document_key`.

### Parallel Opportunities

T027 (types) blocks the rest; T030 (dialog rendering) can be drafted in parallel with T028/T029 once `PasteWhere` exists.

### Dependencies

- Depends on WP04 (element buffer) and WP05 (`bytes_for_paste`).

### Risks & Mitigations

- Pasted `Node`s from `parse_forest` carry offsets relative to the pasted bytes and a fresh `expanded: true` — `rebuild()` must recompute offsets for the whole document; verify no stale offset survives into the redraw.
- NFR-002 (byte-identical round trip): the encoder must be the single source of truth — never hand-construct bytes for the buffer or clipboard outside `ber::encode_forest`.

---

## Work Package WP07: Edit menu, File ▸ Settings entry and dialogs (Priority: P1)

**Goal**: Make every tree operation discoverable through a new top-bar Edit menu (including the only route to Paste as child) and let users change the key-binding set through a File ▸ Settings dialog.
**Independent Test**: `cargo test app::tests::edit_menu` and `cargo test app::tests::settings_dialog` pass: the Edit menu lists exactly the six actions with the correct key label per active binding set, Paste as child is refused on a primitive target, and the Settings dialog saves, applies immediately, and persists.
**Prompt**: `tasks/WP07-edit-menu-and-settings-dialog.md`
**Requirement Refs**: FR-014, FR-021, FR-024, NFR-005

### Included Subtasks

T033 Add `TopMenuAction` variants (`Settings`, `EditDelete`, `EditCut`, `EditCopy`, `EditPasteBefore`, `EditPasteAfter`, `EditPasteAsChild`) and extend `TOP_MENUS` with an `Edit` heading and a File ▸ Settings entry
T034 Implement `paste_as_child` restricted to constructed/encapsulating targets, with a refusal status message on primitive ones
T035 Implement `Mode::Settings(SettingsState)` dialog (radio rows, following the existing public-key dialog pattern) with Enter saving and applying immediately, Esc discarding
T036 Compute Edit-menu key labels at draw time from `keymap::label(action, active_set)` rather than static strings
T037 Unit tests: Edit menu contents and per-set labels; Paste-as-child refusal on primitive; Settings dialog save/apply/persist

### Implementation Notes

This WP is the primary owner of the `app.rs`/`tui.rs` menu and dialog machinery for this mission; it is placed after WP04/WP06/WP08 in the dependency chain specifically so no other WP is concurrently editing those areas.

### Parallel Opportunities

T033 (menu table) and T035 (dialog state) touch different parts of `app.rs` and can be drafted in parallel.

### Dependencies

- Depends on WP02 (Settings persistence), WP04 (buffer, for Cut/Copy/Delete menu entries), WP06 (paste, for Paste menu entries), WP08 (so vim-mode menu labels are correct once both binding sets exist).

### Risks & Mitigations

- `TOP_MENUS` is a `const` with `&'static str` labels today; key labels must move to a function computed at draw time, not baked into the const — a naive edit would freeze the label to whichever set was active at compile time (it isn't, but be careful not to introduce a stale cache).

---

## Work Package WP08: Vim modal value editors (Priority: P1)

**Goal**: Under vim bindings, give every value editor normal/insert/visual modes mapped onto the existing editor operations, plus Ctrl+A/Ctrl+X increment/decrement.
**Independent Test**: `cargo test vim::` passes: mode transitions, `v l l y $ p` duplicates three octets in the hex editor, Ctrl+A wraps a hex octet FF→00, Ctrl+A on an integer 99→100, Ctrl+A on an OID arc, DateTime field navigation, and Ctrl+C/V/X/Z are no-ops under vim bindings.
**Prompt**: `tasks/WP08-vim-modal-value-editors.md`
**Requirement Refs**: FR-028, FR-029, FR-030, FR-031

### Included Subtasks

T038 Define `EditorMode`, `VimState` in new `src/vim.rs`
T039 Implement normal-mode motion (`h l j k 0 $`) and mode transitions (`i a I A` → Insert, Esc → Normal) mapped onto `Editor::move_horizontal/move_vertical/home/end`
T040 Implement `x` (delete-under-cursor into register), `v` visual mode with `y`/`d` into register, `p`/`P` put from register, `u` undo
T041 Implement `Editor::adjust_number_at_cursor(delta)` (hex octet wrap, arbitrary-size integer, OID arc, digit-run) and wire Ctrl+A/Ctrl+X in vim normal mode
T042 Route `handle_edit_key` in `src/tui.rs` through vim dispatch when `VimState` is active (Ctrl+C/V/X/Z become no-ops); render the mode indicator on the editor's first line

### Implementation Notes

`EditState` gains a `vim: Option<VimState>` field (narrow out-of-map edit to `src/app.rs`), populated only when `App.bindings == KeyBindingSet::Vim` at the moment the editor opens. The `DateTimeEditor` is field-based, not a char buffer: `h`/`l` move between its six fields and `x` is a documented no-op there — cover this explicitly in tests, don't assume the hex/text behaviour transfers unchanged.

### Parallel Opportunities

T038 (types) blocks the rest; T041 (increment/decrement) is independent of T039/T040 (motion/visual) and can be drafted in parallel once `Editor` is in scope.

### Dependencies

- Depends on WP01 (`KeyBindingSet`), WP02 (`App.bindings`), and WP04 (serialized after the tree's `app.rs` changes land, to avoid concurrent edits to the same struct in a parallel lane, even though this WP's own logic does not need the element buffer).

### Risks & Mitigations

- Integer increment must use string/arbitrary-size arithmetic, not `i128` — certificate INTEGERs exceed it (research.md R8).
- Keep the vim editor's `register` (per-editor buffer content) separate from the tree's `ElementBuffer` (whole elements) — they are different types for different data, do not conflate them.

---

## Work Package WP09: Documentation and documentation tests (Priority: P2)

**Goal**: Keep the help window and `DESIGN.md` truthful for both binding sets and every new operation, and verify that mechanically.
**Independent Test**: `cargo test app::tests::help_topics_cover_actions` passes; a person reading only the in-app help can perform every acceptance scenario in spec.md User Stories 1–6.
**Prompt**: `tasks/WP09-documentation-and-doc-tests.md`
**Requirement Refs**: FR-032, NFR-005, NFR-006

### Included Subtasks

T043 Update `HELP_TOPICS` in `src/app.rs`: new "Marking and multi-element edits" topic, extend "Changing structure" for range delete, extend the clipboard/copy-paste topic for tree-level copy/paste and PEM reading, add a vim-bindings topic, add a settings topic
T044 Update `DESIGN.md` §3 (architecture list: `keymap.rs`, `settings.rs`, `mark.rs`, `buffer.rs`, `paste.rs`, `vim.rs`), §7 (mark/buffer/paste model), §11 (two key-binding tables), new §11a (settings)
T045 [P] Update the README feature summary to mention configurable key-binding sets
T046 Documentation test: every `TreeAction`/`EditorAction` label appears in some `HELP_TOPICS` body

### Implementation Notes

Write this WP last, after every action and its final wording exist, so the help text does not describe behaviour that changed mid-mission.

### Parallel Opportunities

T045 (README) is independent of T043/T044/T046 and can be done alongside them.

### Dependencies

- Depends on WP03, WP04, WP06, WP07, WP08 (documents the behaviour all of them add).

### Risks & Mitigations

- Documentation drift is exactly what T046 exists to catch mechanically — do not skip it as "just docs".

---

## Dependency & Execution Summary

- **Sequence**: `{WP01, WP05}` (parallel start) → `{WP02, WP03}` (both need only WP01) → WP04 (needs WP03) → `{WP06 (needs WP04, WP05), WP08 (needs WP01, WP02, WP04)}` (parallel) → WP07 (needs WP02, WP04, WP06, WP08) → WP09 (needs WP03, WP04, WP06, WP07, WP08).
- **Parallelization**: WP01 ∥ WP05 at the start; WP02 ∥ WP03 once WP01 lands; WP06 ∥ WP08 once WP04 lands. WP07 and WP09 are placed last specifically to avoid concurrent `app.rs`/`tui.rs` edits with any other WP.
- **MVP Scope**: WP01 + WP03 + WP04 (mark and delete a range) is the smallest independently valuable slice — User Story 1. Add WP05 + WP06 for copy/paste (User Story 2). WP02/WP07/WP08 (Settings, Edit menu, vim) and WP09 (docs) complete the mission.

---

## Requirements Coverage Summary

| Requirement ID | Covered By Work Package(s) |
|----------------|----------------------------|
| FR-001 | WP03 |
| FR-002 | WP03 |
| FR-003 | WP03 |
| FR-004 | WP03 |
| FR-005 | WP03 |
| FR-006 | WP04 |
| FR-007 | WP04 |
| FR-008 | WP04 |
| FR-009 | WP04 |
| FR-010 | WP04 |
| FR-011 | WP06 |
| FR-012 | WP06 |
| FR-013 | WP06 |
| FR-014 | WP06, WP07 |
| FR-015 | WP05 |
| FR-016 | WP05, WP06 |
| FR-017 | WP06 |
| FR-018 | WP06 |
| FR-019 | WP06 |
| FR-020 | WP04 |
| FR-021 | WP01, WP07 |
| FR-022 | WP02 |
| FR-023 | WP02 |
| FR-024 | WP02, WP07 |
| FR-025 | WP01 |
| FR-026 | WP04 |
| FR-027 | WP01 |
| FR-028 | WP08 |
| FR-029 | WP08 |
| FR-030 | WP08 |
| FR-031 | WP08 |
| FR-032 | WP09 |
| FR-033 | WP01 |
| NFR-001 | WP04, WP06 |
| NFR-002 | WP06 |
| NFR-003 | WP04, WP06 |
| NFR-004 | WP02 |
| NFR-005 | WP07, WP09 |
| NFR-006 | WP09 |
| C-001 | WP01 |
| C-002 | WP01 |
| C-003 | WP01 |
| C-004 | WP06 |
| C-005 | WP06 |
| C-006 | WP02 |
| C-007 | WP02 |
| C-008 | WP06 |

---

## Subtask Index (Reference)

| Subtask ID | Summary | Work Package | Priority | Parallel? |
|------------|---------|--------------|----------|-----------|
| T001 | Define KeyBindingSet/TreeAction/EditorAction enums | WP01 | P0 | Yes |
| T002 | Implement translate() for tree actions | WP01 | P0 | No |
| T003 | Implement label() for menu/help | WP01 | P0 | Yes |
| T004 | Declare pub mod keymap in lib.rs | WP01 | P0 | No |
| T005 | Unit tests: vim table, coverage, Shift+Up | WP01 | P0 | No |
| T006 | Implement settings::config_path() per OS | WP02 | P0 | Yes |
| T007 | Implement Settings/LoadOutcome/load() | WP02 | P0 | No |
| T008 | Implement save() with atomic write | WP02 | P0 | No |
| T009 | Wire main.rs to load settings at start-up | WP02 | P0 | No |
| T010 | Unit tests: settings path/round-trip/fallback | WP02 | P0 | No |
| T011 | Define Mark struct and mark_extend() | WP03 | P0 | No |
| T012 | Implement mark bounds checking | WP03 | P0 | No |
| T013 | Implement clear_mark() via selection funnel | WP03 | P0 | No |
| T014 | Implement operand() with region refusals | WP03 | P0 | No |
| T015 | Add marked-row highlighting in draw_tree | WP03 | P0 | Yes |
| T016 | Unit tests: mark extend/shrink/bounds/clear | WP03 | P0 | No |
| T017 | Define ElementBuffer struct | WP04 | P0 | No |
| T018 | Implement delete_operand() | WP04 | P0 | No |
| T019 | Implement copy_operand() | WP04 | P0 | No |
| T020 | Implement cut_operand()/yank_operand() | WP04 | P0 | No |
| T021 | Wire Delete/Cut/Copy dispatch | WP04 | P0 | No |
| T022 | Unit tests: delete/copy/cut/yank | WP04 | P0 | No |
| T023 | Implement clipboard::bytes_for_paste() | WP05 | P0 | No |
| T024 | Add PasteKind::Pem variant | WP05 | P0 | No |
| T025 | Odd hex-digit refusal at step 1 | WP05 | P0 | Yes |
| T026 | Unit tests: hex/base64/PEM/raw/odd-hex | WP05 | P0 | No |
| T027 | Define PasteWhere/PasteSource | WP06 | P0 | No |
| T028 | Implement paste_bytes() | WP06 | P0 | No |
| T029 | Implement Ctrl+V/p/P dispatch | WP06 | P0 | No |
| T030 | Add PasteWhere dialog mode | WP06 | P0 | Yes |
| T031 | Wire cursor/dirty/expand/status on paste | WP06 | P0 | No |
| T032 | Unit tests: paste round trip/dialog/refusal | WP06 | P0 | No |
| T033 | Add Edit/Settings top-menu entries | WP07 | P1 | Yes |
| T034 | Implement paste_as_child | WP07 | P1 | No |
| T035 | Implement Settings dialog | WP07 | P1 | Yes |
| T036 | Compute menu key labels from keymap | WP07 | P1 | No |
| T037 | Unit tests: Edit menu/Settings dialog | WP07 | P1 | No |
| T038 | Define EditorMode/VimState | WP08 | P1 | No |
| T039 | Implement normal-mode motion/mode transitions | WP08 | P1 | No |
| T040 | Implement x/visual/put/undo | WP08 | P1 | No |
| T041 | Implement adjust_number_at_cursor()/Ctrl+A/X | WP08 | P1 | Yes |
| T042 | Route handle_edit_key through vim dispatch | WP08 | P1 | No |
| T043 | Update HELP_TOPICS | WP09 | P2 | No |
| T044 | Update DESIGN.md | WP09 | P2 | No |
| T045 | Update README | WP09 | P2 | Yes |
| T046 | Documentation test: action label coverage | WP09 | P2 | No |

