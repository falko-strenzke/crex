---
work_package_id: WP09
title: Documentation and documentation tests
dependencies: ["WP03", "WP04", "WP06", "WP07", "WP08"]
requirement_refs:
- FR-032
- NFR-005
- NFR-006
planning_base_branch: feat/asn1-tree-delete-copy-paste
merge_target_branch: feat/asn1-tree-delete-copy-paste
branch_strategy: Planning artifacts for this mission were generated on feat/asn1-tree-delete-copy-paste. During /spec-kitty.implement this WP may branch from a dependency-specific base, but completed changes must merge back into feat/asn1-tree-delete-copy-paste unless the human explicitly redirects the landing branch.
subtasks:
- T043
- T044
- T045
- T046
phase: Phase 5 - Documentation
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: DESIGN.md
create_intent: []
execution_mode: code_change
model: ''
owned_files:
- DESIGN.md
- README.md
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP09 – Documentation and documentation tests

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

Bring the in-app help window and `DESIGN.md` up to date with every operation this mission added, for both key-binding sets, and verify mechanically that nothing was missed — this is the last work package in the mission specifically so it documents final, settled behaviour rather than an interim design.

Done when:
- `HELP_TOPICS` in `src/app.rs` covers marking, range delete, tree-level copy/paste (including PEM reading), the Edit menu, Settings, and vim-mode value editing — a user who has only read the help window can perform every acceptance scenario in spec.md User Stories 1–6.
- `DESIGN.md` §3's architecture list names the six new modules; §7 (editing model) documents the mark/buffer/paste model; §11 (key bindings) has two tables (normal, vim); a new §11a documents settings.
- README's feature summary mentions configurable key-binding sets.
- A documentation test confirms every `TreeAction`/`EditorAction` with a key or menu entry has some corresponding mention in `HELP_TOPICS`.
- `cargo test app::tests::help_topics_cover_actions` (or wherever this lands) passes.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` in full — this WP's job is to make the help window and `DESIGN.md` say, accurately, everything the finished mission does; re-read the whole spec now that every other WP has landed, not just the sections most relevant to any one WP.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/keymap.md`, `contracts/settings-file.md`, `contracts/clipboard-payload.md` — the exact wording/behaviour to document.
- `src/app.rs`'s existing `HELP_TOPICS` const and `HelpTopic` struct — read every existing topic (there are several: file browsing, the structure pane, editing values, changing structure, clipboard, undo, and more — check the full list) to match their tone, length, and level of detail; this WP's new/extended topics must read as if the original author wrote them, not as a bolted-on addendum.
- `DESIGN.md` §3, §7, §11 in full (already read once during planning; re-read now against the actual, shipped implementation rather than the plan's proposal, since implementation details may have settled differently than planned — check each of WP01/02/03/04/05/06/07/08's Activity Logs for any judgment calls that changed the shape of something plan.md described).
- `README.md`'s current feature bullet list (the top of the file) for the existing tone and bullet style to match.

**This WP depends on WP03, WP04, WP06, WP07, WP08 all being complete** — do not start writing documentation for behaviour that might still change; if any dependency WP is not yet merged/finalized when this WP starts, treat that as a blocker and confirm status first (`spec-kitty agent tasks status`) rather than documenting speculatively.

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T043 – Update HELP_TOPICS

- **Purpose**: NFR-005 — every new operation must be findable in the help window.
- **Steps**:
  1. New topic "Marking and multi-element edits": explain Shift+Up/Down, the sibling-only/subtree-inclusive rule, how it is shown, and that it clears on any other navigation or Esc — written at the same level of detail as the existing "Structure" or "Editing values" topics (a few short paragraphs, not a bare bullet list).
  2. Extend the existing "Changing structure" topic (search for its current text, which documents `i`/`I`/`d`) with a paragraph on range delete: "when several elements are marked, `d` `d` deletes all of them; the confirmation names how many" — placed naturally after the existing single-element `d`-key paragraph, not as a disconnected addendum.
  3. Extend the existing clipboard-related topic(s) (there may be one for the value editors' Ctrl+C/V/X/Z, per the existing "Editing values" topic's later paragraphs) with tree-level copy/paste: Ctrl+C/Ctrl+V (normal) vs. `y`/`p`/`P`/`d` (vim), what goes to the system clipboard vs. the element buffer, and PEM reading — cross-reference rather than duplicate the existing clipboard-helper-programs paragraph (`wl-copy/wl-paste, xclip, ...`) since that infrastructure is unchanged and already documented once.
  4. New topic "Key-binding sets" (or fold into a natural existing location if one fits better — use your judgment, but a dedicated topic is likely clearest given how much it covers): explain the File ▸ Settings choice between normal and vim, and for vim specifically: the Edit menu's role for Paste as child, and a summary of the modal value-editor behaviour (`i`/`a`/Esc, motion keys, `x`/`v`/`y`/`d`/`p`/`u`, Ctrl+A/Ctrl+X) — this can reference `contracts/keymap.md`'s content in spirit without literally reproducing every row; keep it readable prose, matching the existing topics' style, with the full detail available via the Edit menu's own on-screen labels (per WP07) for anyone who wants the exact key.
  5. Extend the "File" or equivalent topic (if one exists documenting `TOP_MENUS`) with the new Edit heading and File ▸ Settings entry, or fold this into the new topic from step 4.
- **Files**: `src/app.rs` (`HELP_TOPICS` const — the declared out-of-map edit for this WP, though by this point in the dependency chain it is safe since no other WP is concurrently editing this file).
- **Parallel?**: No — the central subtask, feeding T046's coverage test.
- **Notes**: Read the existing topics' actual prose style closely (they favour short, dense paragraphs with occasional em-dash asides and precise key references in single-quotes, e.g. `'i' inserts...`) — match it exactly rather than writing in a noticeably different voice for the new topics.

### Subtask T044 – Update DESIGN.md

- **Purpose**: Keep the architecture document — this project's primary technical reference, per its own stated purpose — accurate for future contributors.
- **Steps**:
  1. §3 (architecture list): add one line each for `keymap.rs`, `settings.rs`, `mark.rs`, `buffer.rs`, `paste.rs`, `vim.rs`, matching the existing list's format (`  modulename.rs   one-line description`) and update the "Dependency rule" paragraph immediately below the tree to state each new module's dependencies (e.g. `keymap.rs` depends only on crossterm's key types; `settings.rs` on std only; `mark.rs`/`buffer.rs`/`paste.rs` on `app.rs`'s `App`/`Node`/`RowSource` and, respectively, on each other per WP06's dependency on WP04/WP05's outputs; `vim.rs` on `app.rs`'s `Editor` types) — re-derive this from what actually landed (check each WP's Activity Log for the real dependency shape, which may have settled slightly differently than `plan.md`'s proposal), do not just copy `plan.md`'s Structure Decision verbatim without verifying it against the merged code.
  2. §7 (editing model): add a subsection describing the mark model, the element buffer, and the paste pipeline at the same level of technical detail as the existing "Structural edits: insert, retag, delete, reorder" subsection (§7's existing content, read it again now) — this new material should read as a natural continuation of that existing subsection, since range-delete and paste are structural edits in the same sense insert/retag/delete/reorder already are.
  3. §11 (Key bindings): the existing single key-binding table becomes two — "Normal bindings" and "Vim bindings" — pulling from `contracts/keymap.md` but written as prose-adjacent tables matching the existing table's column style (`| Key | Action |`), not a raw dump of the contract file.
  4. New §11a "Settings": location, format, and behaviour, summarising `contracts/settings-file.md` at the level of detail `DESIGN.md`'s other sections use (a few paragraphs, cross-referencing the contract file by path for the exact format rather than reproducing the full TOML-subset grammar).
- **Files**: `DESIGN.md`.
- **Parallel?**: Can be drafted alongside T043 (different file), converging only in that both should describe the same final behaviour consistently — do not let the help-window wording and the DESIGN.md wording contradict each other on any point (e.g. exact key names).
- **Notes**: `DESIGN.md` is clearly a carefully maintained, detailed document (over 1800 lines, with a consistent section-numbering and cross-referencing style) — read enough of its surrounding sections to match its register; do not write terser or more casual prose than the rest of the document uses.

### Subtask T045 [P] – Update the README feature summary

- **Purpose**: A newcomer skimming `README.md`'s top-level feature list should learn this mission's headline capability exists.
- **Steps**:
  1. In `README.md`'s existing "Generic ASN.1/DER (PEM or binary) viewing and editing" bullet group (near the top, under "Content search function"), add one bullet: something like "Marking, deleting, copying and pasting whole elements or ranges in the tree, with configurable normal/vim key bindings" — match the terse, single-line style of the surrounding bullets exactly (they are short noun phrases, not full sentences).
  2. Check whether `README.md`'s "Usage" section (further down, documenting `-o`, `--dump`, etc.) needs any addition — likely not, since this mission adds no new CLI flags, only in-app behaviour; confirm this is indeed the case rather than assuming, by re-reading the full "Usage" section once.
- **Files**: `README.md`.
- **Parallel?**: Yes — independent of T043/T044/T046, can be done at any point once the feature set is settled.
- **Notes**: The repository is mid-rewrite of its README per the git history (`start rewrite readme.md; switch to Botan 3.13`) — check the current state of the file before assuming its exact current structure matches what is described above; adapt the insertion point to wherever the equivalent bullet group has landed.

### Subtask T046 – Documentation test: action label coverage

- **Purpose**: SC-007 and NFR-005/NFR-006 — make "every operation is documented" a compile-time-checked fact, not a hope.
- **Steps**:
  1. In `src/app.rs`'s existing test module, add a test that iterates every `TreeAction` variant and every `EditorAction` variant (from `crate::keymap`) and asserts that at least one `HelpTopic` body string (across all of `HELP_TOPICS`) contains a recognisable mention of it — since action variants are Rust identifiers, not English words, build a small mapping from each variant to a short English phrase or key fragment expected to appear somewhere in the help text (e.g. `TreeAction::MarkDown → "Shift+Down"` or `"mark"`, `TreeAction::PasteAsChild → "Paste as child"`), and assert `HELP_TOPICS.iter().any(|t| t.body.iter().any(|line| line.contains(phrase)))` for each.
  2. This mapping is necessarily a little manual (English phrases are not derivable from enum variant names automatically) — keep it small and directly adjacent to the test, as a `const` array of `(variant, expected_phrase)` pairs, so a future contributor adding a new action is nudged to add both the help text and the mapping entry together.
  3. Run this test and fix any gap it finds in T043's work — this is expected to happen at least once; do not treat a first-run failure as a test bug, treat it as the test doing its job.
- **Files**: `src/app.rs` (test module).
- **Parallel?**: No — depends on T043's finished help text.
- **Notes**: Do not make this test exhaustively check *wording quality* (that is a human review concern) — its job is narrowly "is this action mentioned anywhere at all", the cheapest possible check that still catches a wholesale omission.

## Test Strategy

- `cargo test app::tests::help_topics_cover_actions` (or the actual name chosen) must pass.
- Run the full `cargo test` one final time across the whole crate — this is the last work package in the mission, and a full green run here is the mission's own definition of done for automated coverage (NFR-006).
- Manually work through `quickstart.md`'s walkthrough end to end (`cargo run -- testdata/chain/server.der`, mark/copy/paste, open File ▸ Settings, switch to vim, restart, confirm it stuck) as a final human sanity check that the finished feature matches what was promised — this is not automatable and is worth doing deliberately rather than skipping because the automated tests are green.

## Risks & Mitigations

- **Documentation drift already baked in by the time this WP starts**: if an earlier WP's actual implementation diverged from what `plan.md`/`data-model.md` describe (a reasonable, expected outcome of "make this judgment call yourself" instructions scattered through WP03–WP08), this WP must document what was actually built, not what was planned — cross-check against each WP's Activity Log, not just the planning artifacts, before writing final prose.
- **Help window bloat**: six new/extended topics is a meaningful addition to an already-substantial help window — keep each addition proportionate to its importance (marking and paste deserve full topics; the Settings dialog needs only a short paragraph) rather than padding every topic to a uniform length.

## Review Guidance

- Confirm every acceptance scenario across spec.md's six user stories can be performed by someone who has read only the help window — spot-check at least three scenarios by literally following the help text's instructions.
- Confirm `DESIGN.md`'s new content matches the actually-shipped code, not `plan.md`'s proposal, on any point where the two might have diverged (dependency rules, module names, exact function names mentioned).
- Confirm T046's coverage test would actually fail if a topic were deleted — sanity-check by temporarily removing one topic's text locally (not committed) and confirming the test catches it, before trusting it as a real safety net.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP09 --to <status>` to change WP status.
