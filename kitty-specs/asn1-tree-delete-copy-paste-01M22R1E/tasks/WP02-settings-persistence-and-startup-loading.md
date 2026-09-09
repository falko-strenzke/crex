---
work_package_id: WP02
title: Settings persistence and start-up loading
dependencies: ["WP01"]
requirement_refs:
- C-006
- C-007
- FR-022
- FR-023
- FR-024
- NFR-004
planning_base_branch: feat/asn1-tree-delete-copy-paste
merge_target_branch: feat/asn1-tree-delete-copy-paste
branch_strategy: Planning artifacts for this mission were generated on feat/asn1-tree-delete-copy-paste. During /spec-kitty.implement this WP may branch from a dependency-specific base, but completed changes must merge back into feat/asn1-tree-delete-copy-paste unless the human explicitly redirects the landing branch.
subtasks:
- T006
- T007
- T008
- T009
- T010
phase: Phase 1 - Foundation
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: src/settings.rs
create_intent:
- src/settings.rs
execution_mode: code_change
model: ''
owned_files:
- src/settings.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP02 – Settings persistence and start-up loading

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

Persist the user's key-binding choice (normal or vim) across restarts, using no new dependencies (per the discovery decision recorded in `research.md` R1 / `decisions/DM-01M22YBGQX9QXP30582Z3JWY6V.md`), in the OS-typical per-user configuration location, with a file format that tolerates unknown keys so future settings can be added without breaking older builds.

Done when:
- `settings::config_path()` returns the right directory per OS from environment variables, and `None` when no variable resolves.
- `Settings::load()` returns a `LoadOutcome` distinguishing `Loaded`, `Missing`, `Invalid { path, reason }`, and `NoLocation` per `data-model.md`.
- `Settings::save()` writes atomically (temp file + rename) and preserves any unrecognised keys already in the file.
- `src/main.rs` loads settings before the TUI starts and a `LoadOutcome::Invalid` produces a start-up notice naming the file and the reason (reusing the existing `Mode::Notice` mechanism).
- `cargo test settings::` passes for path resolution, round trip, unknown-key preservation, malformed-file fallback, and missing-file silent defaults.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/settings-file.md` — the authoritative location table, file format, and behaviour table this WP implements exactly.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/data-model.md` — `Settings`, `LoadOutcome` field definitions.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/research.md` R1 and R11 — why std-only, and why atomic writes.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md` FR-022–024, NFR-004, C-006–007 — the acceptance scenarios in User Story 4 this WP must satisfy end to end once wired into `main.rs`.
- `src/main.rs` in full (it is short) — understand today's start-up sequence (argument parsing, then TUI init) before inserting the settings load.
- `src/app.rs`'s existing `Mode::Notice(NoticeState)` and how it is populated at start-up today for specification-load warnings (search for `Notice` in `src/app.rs`) — reuse this exact mechanism for the `Invalid` case, do not invent a second notice type.

**This WP does not build the Settings dialog** (File ▸ Settings, `Mode::Settings`) — that is WP07's job, which depends on this WP's `Settings::save()`/`load()` existing. This WP's own scope ends at: the type, its persistence, and loading it once at start-up.

**Out-of-map edits this WP makes, with rationale**:
- `src/app.rs`: add one field, `bindings: KeyBindingSet`, to the `App` struct (populated from the loaded `Settings` at construction) — no other WP declares this field, and every later WP that reads the active binding set needs it to exist on `App`.
- `src/main.rs`: insert the settings-load call before TUI init, and pass the resulting binding set (and any start-up notice) into `App::new(...)` (check that constructor's current signature and extend it, or add a setter called right after construction — whichever matches the existing construction style more closely).

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T006 [P] – Implement settings::config_path() per OS

- **Purpose**: Find the one file this mission's persistent state lives in, without a `dirs` crate.
- **Steps**:
  1. `pub fn config_path() -> Option<std::path::PathBuf>`, using `#[cfg(target_os = ...)]` or a runtime match on `std::env::consts::OS` — prefer compile-time `cfg` for correctness (matches `contracts/settings-file.md`'s per-platform table exactly), since it removes any chance of misdetecting the platform at runtime.
  2. Linux/other Unix (non-macOS, non-Windows): try `XDG_CONFIG_HOME` first (non-empty), else `HOME` + `/.config`; join `crex/config.toml`. Return `None` if neither variable is set and non-empty.
  3. macOS: `HOME` + `/Library/Application Support/crex/config.toml`; `None` if `HOME` is unset.
  4. Windows: `APPDATA` + `\crex\config.toml`; `None` if `APPDATA` is unset.
  5. Make the environment-variable reads swappable for tests: either accept an optional `env: &impl Fn(&str) -> Option<String>` parameter with `config_path()` as a thin wrapper calling `config_path_with(|k| std::env::var(k).ok())`, or (simpler, and consistent with how small this file is) put the core logic in a private `fn resolve(vars: impl Fn(&str) -> Option<String>) -> Option<PathBuf>` that `config_path()` calls with `std::env::var`, and that tests call directly with a stub closure. Do not literally mutate process environment variables in tests (flaky under parallel test execution) — use the injectable-closure approach.
- **Files**: `src/settings.rs` (new).
- **Parallel?**: Yes — independent of T007/T008's parsing logic.
- **Notes**: `contracts/settings-file.md`'s table is the exact spec; re-read it once more before writing the `cfg` blocks, the three platforms have three different join rules and it is easy to swap two by memory.

### Subtask T007 – Implement Settings, LoadOutcome, and the TOML-subset load() reader preserving unknown keys

- **Purpose**: Parse the tiny, strict TOML subset the settings file uses, without pulling in `toml`/`serde`.
- **Steps**:
  1. `pub struct Settings { pub key_bindings: crate::keymap::KeyBindingSet, unknown: Vec<(String, String)> }` (the `unknown` field is private — `save()` is the only thing that needs it, per `data-model.md`; do not make it `pub` unless a test genuinely needs direct access, in which case prefer `pub(crate)`).
  2. `impl Default for Settings` → `key_bindings: KeyBindingSet::Normal, unknown: vec![]`.
  3. `pub enum LoadOutcome { Loaded(Settings), Missing, Invalid { path: PathBuf, reason: String }, NoLocation }`.
  4. `pub fn load() -> LoadOutcome`: call `config_path()`; on `None` return `NoLocation`; on `Some(path)`, if the file does not exist return `Missing`; if it exists but cannot be read (`std::fs::read_to_string` error) or fails to parse, return `Invalid { path, reason }` with a short, specific reason (e.g. `"cannot read file: <io error>"` or `"line 3: not a key = \"value\" pair"`) — never panic.
  5. Parser rules, matching `contracts/settings-file.md` exactly: for each line, trim; skip if empty or starts with `#`; otherwise require the shape `key = "value"` (key matches `[A-Za-z0-9_]+`, value is a double-quoted string with no escape sequences — reject a value containing an unescaped `"` before the closing quote as invalid, do not attempt to support backslash escapes since the format spec doesn't call for them); any other shape makes the file `Invalid`, with the reason naming the 1-based line number. Duplicate keys: last one wins (do not error).
  6. Recognised key: `key_bindings` with value `"normal"` or `"vim"` (case-sensitive, matching `KeyBindingSet::FromStr` from WP01) sets `key_bindings`; any other value for that key makes the file `Invalid`. Every other key found is appended to `unknown` verbatim (key and value strings, quotes stripped) — this is what makes C-007 hold.
  7. On successful parse, wrap in `LoadOutcome::Loaded(Settings { .. })`.
- **Files**: `src/settings.rs`.
- **Parallel?**: No — the central logic other subtasks depend on.
- **Notes**: This function needs `KeyBindingSet` from WP01's `src/keymap.rs` (`pub mod keymap;` already declared in `lib.rs` by WP01/T004) — `use crate::keymap::KeyBindingSet;` at the top of this file.

### Subtask T008 – Implement save() with atomic write and directory creation

- **Purpose**: Never leave a truncated or half-written settings file behind, even on a crash mid-write.
- **Steps**:
  1. `pub fn save(&self) -> Result<(), String>` on `Settings`. `config_path()` → `None` returns `Err("no configuration directory found for this system".to_string())` (this is the wording the Settings dialog in WP07 will show — keep it exactly this, since that WP quotes it verbatim per `data-model.md`'s `SettingsState`).
  2. Create the parent directory if missing (`std::fs::create_dir_all`), mapping any error to a `String` reason.
  3. Render the file: a header comment line (`"# crex settings — edited by File ▸ Settings\n"`), then `key_bindings = "normal"` or `"vim"` per the current value, then one line per entry in `unknown` rendered as `key = "value"` (re-emit whatever was preserved from load, unchanged) — this is what "unknown keys are kept and rewritten unchanged on save" means concretely.
  4. Write to `<path>.tmp` in the same directory, then `std::fs::rename(tmp, path)` (atomic on all three target platforms for same-directory renames, per research.md R11). Map any I/O error to a `String` reason; on the write step failing, do not leave a stray `.tmp` file if avoidable (attempt `std::fs::remove_file` on the tmp path in the error path, best-effort, ignore its own error).
- **Files**: `src/settings.rs`.
- **Parallel?**: No — depends on T007's `Settings` struct shape.
- **Notes**: Do not use any crate for atomic file writes — `std::fs` alone is sufficient here and keeps this WP dependency-free as decided.

### Subtask T009 – Wire main.rs to load settings before the TUI starts

- **Purpose**: Make the persisted choice actually take effect, and surface a broken file as a notice instead of a crash or silent misbehaviour.
- **Steps**:
  1. In `src/main.rs`, before the TUI is initialized (locate the current call into `tui::run` or equivalent — read the file first to find the right insertion point), call `settings::load()` and match on the `LoadOutcome`:
     - `Loaded(settings)` → use `settings.key_bindings`.
     - `Missing` or `NoLocation` → use `KeyBindingSet::default()` (Normal), no notice.
     - `Invalid { path, reason }` → use `KeyBindingSet::default()`, and arrange for `App` to show a start-up notice: `"settings file {path}: {reason}; using defaults"` (matches `contracts/settings-file.md`'s exact wording template).
  2. Pass the resolved `KeyBindingSet` into wherever `App` is constructed (`App::new(...)` or equivalent) — check the constructor's current parameter list and add a parameter, or add a `bindings` field set immediately after construction if that matches the existing pattern better (e.g. if other post-construction setup already happens inline in `main.rs`, follow that convention rather than changing the constructor signature).
  3. For the `Invalid` case's notice: find how the existing specification-load warning notice is constructed at start-up (search `src/main.rs` and `src/app.rs` for the current `Mode::Notice`/`NoticeState` start-up path) and reuse the identical mechanism — do not add a second, parallel notice-queueing path.
- **Files**: `src/main.rs`, plus the one-line `bindings: KeyBindingSet` field addition to the `App` struct in `src/app.rs` (the documented out-of-map edit for this WP).
- **Parallel?**: No — depends on T007's `LoadOutcome` and T006's `config_path` (indirectly, via `load()`).
- **Notes**: If `App`'s constructor already takes several parameters, prefer adding `bindings` as one more constructor parameter over a separate setter call, for consistency — but match whatever style the surrounding code already uses rather than introducing a new one.

### Subtask T010 – Unit tests: path per OS, round trip, unknown keys preserved, malformed file fallback, missing file fallback

- **Purpose**: Prove every branch of `contracts/settings-file.md`'s behaviour table.
- **Steps**:
  1. `#[cfg(test)] mod tests` in `src/settings.rs`:
     - `config_path_resolves_xdg_config_home_first_on_linux` / `..._falls_back_to_home_dot_config`: using the injectable-closure form of `resolve()` from T006, assert the Linux branch picks `XDG_CONFIG_HOME` when set, else `HOME/.config`, else `None`. (Gate Linux-specific assertions behind `#[cfg(target_os = "linux")]` if `resolve()` itself is `cfg`-gated per-OS; if instead you kept one function with an OS parameter for testability, test all three branches unconditionally — pick whichever T006 actually produced and test that shape.)
     - `save_then_load_round_trips`: create a `Settings` with `key_bindings: Vim`, save to a temp directory (use `tempfile`-style manual temp dir via `std::env::temp_dir()` + a unique subdirectory name, since the crate has no `tempfile` dependency — clean up after the test), load it back, assert equality.
     - `unknown_keys_survive_a_save_after_load`: hand-write a settings file containing `key_bindings = "normal"` plus an unrelated `future_setting = "x"` line, load it, change `key_bindings` to `"vim"`, save, re-read the raw file text, and assert both the new `key_bindings` value and the untouched `future_setting = "x"` line are present.
     - `malformed_file_yields_invalid_with_reason`: a file containing a garbage line (e.g. `"not a valid line"`) → `LoadOutcome::Invalid { reason, .. }` where `reason` is non-empty and mentions the line.
     - `missing_file_yields_defaults_silently`: `load()` against a path that does not exist → `LoadOutcome::Missing`, and separately confirm the `main.rs`-level behaviour (if testable at this layer) results in `KeyBindingSet::default()`.
  2. Every test that touches the filesystem must clean up its temp files/directories on both success and failure paths (use a `Drop` guard or an explicit cleanup at the end of the test body — do not leave test artifacts in `std::env::temp_dir()`).
- **Files**: `src/settings.rs` (test module).
- **Parallel?**: No — exercises the finished T006–T009.
- **Notes**: Keep filesystem-touching tests independent of each other (unique temp subdirectory per test, e.g. named after the test function) so `cargo test` can run them in parallel without collisions.

## Test Strategy

- `cargo test settings::` must pass.
- Manually verify (not as an automated test, but as a sanity check before marking this WP done) that `cargo run -- testdata/chain/server.der` still starts normally with no settings file present, and that hand-creating `~/.config/crex/config.toml` with `key_bindings = "vim"` (Linux) is picked up on the next run — this is the acceptance scenario in spec.md User Story 4, scenario 1, and is worth a manual run even though later WPs (WP08) are what makes vim bindings visibly do anything.

## Risks & Mitigations

- **Windows/macOS path logic untestable on this Linux development machine**: write the per-OS branches from the contract's table carefully and unit-test them via the injectable-closure approach (T006) so the logic is exercised even though the `#[cfg(target_os = "windows")]` branch itself cannot be compiled and run here. Do not skip testing those branches just because CI on this machine cannot execute them — test the pure logic function directly.
- **Notice mechanism mismatch**: if `Mode::Notice`/`NoticeState` turns out to require more setup than a one-line reuse (e.g. it is queued differently at start-up vs. mid-session), do not build a parallel ad hoc mechanism — read the existing code fully first and match its actual shape, even if that takes an extra pass.

## Review Guidance

- Confirm `save()` truly never partially overwrites the real file — trace the write-then-rename path, do not just trust the docstring.
- Confirm unknown-key preservation actually round-trips arbitrary future keys, not just the one `key_bindings` key — this is the concrete test of C-007.
- Confirm the one-line `App` struct field addition and the `main.rs` wiring are the only edits made outside `src/settings.rs`, matching the "narrow out-of-map edit" scope declared in this WP's frontmatter and Context section.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP02 --to <status>` to change WP status.
