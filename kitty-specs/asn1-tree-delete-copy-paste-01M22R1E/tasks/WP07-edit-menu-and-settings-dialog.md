---
work_package_id: WP07
title: Edit menu, File ▸ Settings entry and dialogs
dependencies: []
requirement_refs:
- FR-014
- FR-021
- FR-024
- NFR-005
planning_base_branch: feat/asn1-tree-delete-copy-paste
merge_target_branch: feat/asn1-tree-delete-copy-paste
branch_strategy: Planning artifacts for this mission were generated on feat/asn1-tree-delete-copy-paste. During /spec-kitty.implement this WP may branch from a dependency-specific base, but completed changes must merge back into feat/asn1-tree-delete-copy-paste unless the human explicitly redirects the landing branch.
subtasks:
- T033
- T034
- T035
- T036
- T037
phase: Phase 4 - Discoverability
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: src/app.rs
create_intent: []
execution_mode: code_change
model: ''
owned_files:
- src/app.rs
- src/tui.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP07 – Edit menu, File ▸ Settings entry and dialogs

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

Make every tree operation this mission adds discoverable through a new top-bar Edit menu — including the only route to Paste as child, which has no key binding in either set — and let users change their key-binding set through a new File ▸ Settings dialog that applies immediately and persists via WP02's `settings::save()`.

Done when:
- `TOP_MENUS` gains an `Edit` heading listing Delete, Cut, Copy, Paste before, Paste after, Paste as child, in that order, each showing the active binding set's key (via WP01's `keymap::label`) or "menu only" where there is none.
- `File` gains a `Settings` entry.
- `paste_as_child` is implemented, restricted to constructed/encapsulating targets, with a clear refusal on primitive ones.
- `Mode::Settings(SettingsState)` lets the user pick normal/vim, Enter saves (via `settings::Settings::save()`) and applies immediately (updates `app.bindings`), Esc discards.
- `cargo test app::tests::edit_menu` and `cargo test app::tests::settings_dialog` (or wherever these land — see T037) pass.

This WP is **explicitly this mission's primary owner of `app.rs`'s and `tui.rs`'s menu and dialog machinery**, placed last among the code work packages (after WP02, WP04, WP06, WP08) specifically so no other WP is concurrently editing those areas — see `tasks.md`'s file-ownership note.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` User Story 3 (all six acceptance scenarios) and User Story 4 (Settings dialog scenarios).
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/keymap.md`'s "Menus and dialogs" section, and `contracts/settings-file.md`.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/data-model.md` — `SettingsState`, `TopMenuAction` additions.
- `src/app.rs`'s existing `TOP_MENUS` const, `TopMenu`/`TopMenuItem`/`TopMenuAction` types, and `MenuBarState` (search near line 1080–1150) — read this in full; this WP extends it, does not replace it.
- `src/app.rs`'s existing `EditPubKey`/`PubKeyState` dialog (search `EditPubKey`) as the closest existing example of a radio-choice dialog — follow its rendering/interaction pattern for the new Settings dialog rather than inventing a new dialog style.
- WP01's `src/keymap.rs` (`translate`, `label`), WP02's `src/settings.rs` (`Settings`, `save()`, `App.bindings`), WP04's `src/buffer.rs` (`delete_operand`/`cut_operand`/`copy_operand`), WP06's `src/paste.rs` (`paste_bytes`, `PasteWhere`) — this WP is glue over all four; read each one's public API (not their internals) before starting.

**This WP's edits to `src/app.rs`/`src/tui.rs` are the primary, declared purpose of this WP** — unlike other WPs' "narrow out-of-map edit" framing, this one genuinely owns these two files for the scope of the Edit/Settings menu and dialog machinery. Stay within that scope: do not refactor unrelated parts of either file while you are in there.

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T033 [P] – Add Edit and Settings top-menu entries

- **Purpose**: FR-021 — make every operation discoverable.
- **Steps**:
  1. Extend `TopMenuAction` with `Settings, EditDelete, EditCut, EditCopy, EditPasteBefore, EditPasteAfter, EditPasteAsChild`.
  2. Add a `Settings` entry to the existing `File` `TopMenu` in `TOP_MENUS`, after `Save`: `TopMenuItem { label: "Settings", desc: "change the key-binding set", action: TopMenuAction::Settings }`.
  3. Add a new `Edit` `TopMenu` to `TOP_MENUS`, positioned between `File` and `About` (matching spec.md's stated order: Delete, Cut, Copy, Paste before, Paste after, Paste as child): six `TopMenuItem`s with static `desc` text (one line each explaining the action, matching the tone of existing entries like `"start an empty document"`) and the six new `TopMenuAction` variants.
  4. **Key labels cannot be static strings on these `TopMenuItem`s** (see T036) — leave the existing `TopMenuItem.label`/`desc` fields as the action name/description only; the per-set key text is computed and appended at render time, not stored here.
  5. Handle the six new `TopMenuAction` variants wherever `TopMenuAction` is currently matched to perform its effect (search for the existing `match` on `TopMenuAction`, likely in `src/tui.rs`'s menu-confirm handling) — `EditDelete` → `app.delete_selected()`, `EditCut` → `app.cut_operand()`, `EditCopy` → `app.copy_operand()`, `EditPasteBefore`/`EditPasteAfter` → `app.paste_bytes(source_per_bindings, Before/After)` (same source-selection logic as WP06's T029, factor it into a small shared helper if convenient, or duplicate the few lines if not — your call), `EditPasteAsChild` → T034's `paste_as_child()`, `Settings` → open `Mode::Settings(...)` (T035).
- **Files**: `src/app.rs` (menu tables and the action-dispatch match).
- **Parallel?**: Yes — the menu table and its dispatch wiring can be drafted alongside T035's dialog state, converging only where `Settings` needs to open the dialog.
- **Notes**: Re-read `contracts/keymap.md`'s note that Cut has **no vim key** — confirm the Edit menu still lists "Cut" with "menu only" under vim bindings rather than omitting it (the menu lists all six actions regardless of set, per spec.md acceptance scenario 1 for this story: "the drop-down lists exactly Delete, Cut, Copy, Paste before, Paste after, Paste as child, in that order").

### Subtask T034 – Implement paste_as_child

- **Purpose**: FR-014 — the only route to filling an empty constructed element.
- **Steps**:
  1. `impl App { pub fn paste_as_child(&mut self) { ... } }` (can live in `src/app.rs` directly, or call through to `self.paste_bytes(source, PasteWhere::AsChild)` from WP06 if that function already handles the constructed/primitive check per its own T028 step 6 — check WP06's implementation first; if the check already lives there, this function may be a thin wrapper that only needs to select the right `PasteSource` per `app.bindings`, exactly like T033's `EditPasteBefore`/`EditPasteAfter` dispatch).
  2. Confirm the refusal wording on a primitive target: `"cannot paste as a child of a primitive element — use Paste before/after instead"` (matches the "Notes" hint style already used by `start_insert`'s equivalent refusal for `'I'`: `"cannot insert a child into a primitive element (use 'i' for a sibling)"` — mirror that exact phrasing convention, substituting the right verb and key hint).
- **Files**: `src/app.rs`.
- **Parallel?**: No — depends on T033's menu wiring existing to call it from, and WP06's `paste_bytes`.
- **Notes**: This is reachable **only** from the Edit menu — do not add a keyboard shortcut for it (Alt+V or otherwise); the discovery decision explicitly settled on menu-only.

### Subtask T035 [P] – Implement Settings dialog

- **Purpose**: FR-024 — the only in-app way to change the key-binding set.
- **Steps**:
  1. `pub struct SettingsState { pub choice: KeyBindingSet, pub path: Option<PathBuf>, pub error: Option<String> }` (per `data-model.md`), added near `App`'s other dialog-state structs in `src/app.rs`.
  2. Add `Settings(SettingsState)` to the `Mode` enum.
  3. Opening the dialog (from `TopMenuAction::Settings`): `SettingsState { choice: app.bindings, path: settings::config_path(), error: None }`, `app.mode = Mode::Settings(state)`.
  4. Rendering (in `src/tui.rs`, near wherever `EditPubKey`/similar dialogs are drawn): two radio rows "Key bindings: ( ) normal / (•) vim" following the exact `(•)`/`( )` glyph convention already used by the existing `s.use_existing` radio rows in `src/tui.rs` (search for that literal pattern near line 1223) — reuse it, do not invent a new glyph pair. Show `path` beneath (or "no configuration directory found for this system" if `None`, matching WP02's `save()` error wording exactly since that is the situation this message describes).
  5. Key handling: Up/Down (or `j`/`k`) toggles `choice` between the two variants; Enter → build `Settings { key_bindings: state.choice, unknown: <preserve from a fresh load() call so any existing unknown keys survive> }`, call `.save()`; on `Ok`, set `app.bindings = state.choice`, status message confirming the change, return to `Mode::Browse`; on `Err(reason)`, still set `app.bindings = state.choice` (FR-024: "if writing fails, the set still applies for this run"), and set `app.status` to report the save failure, then return to `Mode::Browse`. Esc → discard, return to `Mode::Browse`, no change to `app.bindings`.
  6. **Preserving unknown keys on save** (step 5's "preserve from a fresh load()"): call `settings::Settings::load()` again at save time (not at dialog-open time, to pick up any external edits made while the dialog was open — an edge case, but the cheap-and-correct choice) to get its `unknown` entries before constructing the new `Settings` to save; if `load()` returns anything other than `Loaded`, use an empty `unknown` list (no prior file, or it was already broken, so there is nothing to preserve).
- **Files**: `src/app.rs` (`SettingsState`, `Mode::Settings`, save/apply logic), `src/tui.rs` (rendering, key handling, dispatch arm in the main mode `match`).
- **Parallel?**: Yes — can be drafted in parallel with T033/T034, converging at the `TopMenuAction::Settings` open-dialog call.
- **Notes**: `Settings`'s `unknown` field is not `pub` per WP02's design (T007) — if this WP genuinely needs to read/rebuild it, either WP02 already exposed a `pub(crate)` accessor (check first) or this WP needs a small, justified visibility change to `src/settings.rs` (note it explicitly in the Activity Log if so, since `settings.rs` is WP02's owned file, not this WP's).

### Subtask T036 – Compute menu key labels from keymap

- **Purpose**: Keep the Edit menu's key display in lock-step with what the keyboard actually does, per WP01's `label()` function — this is the concrete mechanism behind NFR-005 for the menu surface.
- **Steps**:
  1. Wherever `TOP_MENUS`' drop-down is rendered (in `src/tui.rs`, the menu-bar drawing code — search for where `TopMenuItem.label`/`desc` are currently drawn), for each `Edit`-heading item, compute the key text at draw time: `keymap::label(action_to_tree_action(item.action), app.bindings)` (write a small `TopMenuAction → Option<TreeAction>` mapping for just the six new variants — `None` for every pre-existing `TopMenuAction` variant, which then renders with no key column exactly as it does today).
  2. Render `Some(key_text)` as a right-aligned or bracketed key hint next to the item's `desc` (match whatever spacing convention the existing menu rendering already uses for consistency); render `None` as the literal text `"menu only"` in the same position.
  3. Confirm this is computed fresh on every draw (not cached anywhere) so a Settings change made via T035 is reflected in the very next frame the Edit menu is opened.
- **Files**: `src/tui.rs` (menu-bar rendering).
- **Parallel?**: No — depends on T033's menu table and WP01's `label()` both existing.
- **Notes**: This is the one place a naive implementation could "bake in" a label string at menu-construction time and get it subtly wrong after a binding-set change — be deliberate that the lookup happens in the render path, not in `TOP_MENUS`'s definition or at start-up.

### Subtask T037 – Unit tests: Edit menu / Settings dialog

- **Purpose**: Prove the menu contents, labels, and dialog behaviour concretely.
- **Steps**:
  1. `edit_menu_lists_exactly_six_actions_in_order`: assert `TOP_MENUS`'s `Edit` heading's items, by `TopMenuAction` variant, equal `[EditDelete, EditCut, EditCopy, EditPasteBefore, EditPasteAfter, EditPasteAsChild]` in that exact order.
  2. `edit_menu_key_labels_match_active_binding_set`: for `KeyBindingSet::Normal` and `Vim` separately, compute each Edit item's label via the T036 lookup function directly (not by rendering, which is harder to assert against in a unit test) and assert it matches `contracts/keymap.md`'s table, including `"menu only"` for `Cut` under vim and for `PasteAsChild` under both.
  3. `paste_as_child_refused_on_primitive_target`: using `test_app`, select a primitive element (an INTEGER, say), call `paste_as_child()` with a non-empty element buffer, assert nothing changed and the status names the refusal.
  4. `settings_dialog_save_applies_and_persists`: open the dialog (construct `SettingsState` directly), change `choice` to `Vim`, simulate Enter, assert `app.bindings == Vim` and (using a temp `config_path` override per WP02's injectable-path testing approach, if `save()`'s path resolution is reachable that way in tests — otherwise assert via `settings::Settings::load()` immediately after, from whatever path the test environment resolves to) that the persisted value round-trips.
  5. `settings_dialog_esc_discards`: open the dialog with `choice` different from `app.bindings`, simulate Esc, assert `app.bindings` is unchanged.
- **Files**: `src/app.rs` (test module, alongside its existing tests).
- **Parallel?**: No — exercises the finished T033–T036.
- **Notes**: For T037.4's persistence assertion, coordinate with however WP02's own T010 tests injected the config path — reuse the same seam rather than inventing a second one.

## Test Strategy

- `cargo test app::tests::edit_menu` and `cargo test app::tests::settings_dialog` (or the actual test names/module chosen) must pass.
- Run the full `cargo test` once at the end — this WP touches the shared `Mode` enum and `TOP_MENUS`, both used throughout `tui.rs`'s existing tests; watch for any compile or behaviour regression there specifically.
- Manually run `cargo run` and open the menu bar (F10) to visually confirm the Edit heading renders correctly in both binding sets — automated tests cover content and logic, not visual layout.

## Risks & Mitigations

- **Stale key labels**: the single biggest risk in this WP is a label computed once and cached, going stale after a Settings change. T036's explicit "computed fresh on every draw" requirement exists to prevent this — verify by hand, not just by a test that happens to check both states independently without also checking the *transition* between them (consider adding one test that opens the menu under Normal, changes to Vim via the dialog, and reopens the menu, asserting the label changed).
- **Unknown-key loss on Settings save**: if T035's "reload before save" step is skipped, saving the key-binding choice could silently drop any unrelated setting a future mission added to the file — this is exactly what C-007 exists to prevent; do not skip that reload.

## Review Guidance

- Confirm the Edit menu's item order matches spec.md's acceptance scenario 1 exactly (Delete, Cut, Copy, Paste before, Paste after, Paste as child) — order matters here, it is asserted in the spec, not just implied.
- Confirm key labels are recomputed per draw, per the T036 risk note above.
- Confirm `paste_as_child` is unreachable by any key in either binding set — grep `src/tui.rs`/`src/keymap.rs` for any accidental binding.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP07 --to <status>` to change WP status.
