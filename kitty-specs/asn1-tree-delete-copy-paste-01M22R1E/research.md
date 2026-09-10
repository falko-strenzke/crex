# Research: ASN.1 Tree Delete, Copy and Paste

Phase 0 output. Each entry records a decision, its rationale and the
alternatives considered. Decision Moments are in `decisions/`.

## R1. Settings persistence without new dependencies

- **Decision**: Std-only. Directory from environment variables (Linux:
  `XDG_CONFIG_HOME`, else `HOME/.config`; macOS: `HOME/Library/Application
  Support`; Windows: `APPDATA`), file `crex/config.toml`, parsed by a
  hand-written reader for a strict TOML subset (`key = "string"`, `#`
  comments, blank lines). Unknown keys are kept verbatim on save.
  (DM-01M22YBGQX9QXP30582Z3JWY6V)
- **Rationale**: One scalar setting today; DESIGN.md's existing stance
  against pulling in crates for small jobs; no supply-chain review needed;
  the file remains valid TOML so a later switch to the `toml` crate is
  non-breaking.
- **Alternatives considered**: `dirs` + `toml` + `serde` (rejected: three
  dependency trees, serde proc-macros in the build, review overhead);
  storing settings beside the binary or in the working directory (rejected
  by C-006).
- **Supply-chain check (DIRECTIVE_051)**: no dependency added, upgraded or
  removed; nothing to verify. Adversarial-squad challenge pass:
  `deferred_with_rationale` — no security-impacting dependency decision was
  made, so the pass has nothing to contest.

## R2. Where the two binding sets are expressed

- **Decision**: A translation layer `src/keymap.rs`: `translate(key,
  set, context) -> Option<Action>` with `Action` split into `TreeAction` and
  `EditorAction` enums; `tui.rs` handlers match on actions. Labels for menus
  and help come from `keymap::label(action, set)`.
- **Rationale**: One behaviour implementation for both sets; C-002 becomes a
  unit test over the vim table; menus and help cannot drift from the real
  bindings.
- **Alternatives considered**: `if vim {..} else {..}` inside each handler
  (rejected: duplicated arms, untestable invariant); runtime-editable key
  maps loaded from the settings file (rejected: out of scope, C-007).

## R3. Delivery of Shift+Up/Down and Ctrl+letters in terminals

- **Decision**: Accept `KeyCode::Up/Down` with `KeyModifiers::SHIFT`. Fold
  Ctrl+letter case before matching, as `handle_edit_key` already does.
  Under vim bindings, Ctrl+A/Ctrl+X are the only Ctrl combinations bound.
- **Rationale**: crossterm normalises the common `CSI 1;2A` form to
  Up+SHIFT on all platforms; the hex editor already relies on it (see
  `handle_edit_key`, `extend`). No terminal reserves Shift+arrows.
- **Alternatives considered**: kitty keyboard protocol only (rejected:
  not universally available; `run` already treats it as optional).

## R4. Element buffer representation

- **Decision**: `Vec<u8>` holding the DER encoding of the operand
  (`ber::encode_forest`), plus the element count for status messages. Paste
  always re-parses with `ber::parse_forest`.
- **Rationale**: Single paste path for buffer and clipboard; fidelity
  guaranteed by the encoder round trip already tested in
  `tests/dumpasn1_compat.rs`; no lifetime or UI-state (expanded flags)
  leakage from cloned `Node`s.
- **Alternatives considered**: `Vec<Node>` clones (rejected: two paste
  paths, offsets and expansion state must be scrubbed anyway).

## R5. Clipboard text interpretation for paste

- **Decision**: Extend `clipboard::hex_digits` semantics into a new
  `clipboard::bytes_for_paste(data) -> Result<(Vec<u8>, PasteKind), String>`
  with order: hex digits → base64 → PEM armour (one or more blocks, any
  label, concatenated in order; reuse the PEM splitting in `input.rs`) →
  raw bytes. Then `parse_forest` must consume everything.
- **Rationale**: Mirrors the documented three-step reading users already
  know, adds PEM because certificates are usually shared that way; the
  parser already rejects trailing or truncated data.
- **Alternatives considered**: Parsing text ASN.1 notation (out of scope);
  guessing DER by leading `0x30` (rejected: order must stay predictable).

## R6. Paste placement and the before/after dialog

- **Decision**: `PasteWhere { Before, After, AsChild }`. Normal-bindings
  Ctrl+V: `After`, except when the selection is index 0 among its siblings,
  which opens a two-entry popup reusing `MenuState` ("Paste before", "Paste
  after"). Vim `p`/`P`: After/Before, no dialog. Menu entries carry the
  position explicitly. AsChild only via Edit menu, only on constructed or
  encapsulating elements, inserting at index 0.
- **Rationale**: As agreed in discovery; reusing the popup menu keeps a new
  dialog to a few lines.
- **Alternatives considered**: Alt+V for paste-as-child (rejected by the
  user); pasting into an empty constructed element implicitly (rejected:
  hidden behaviour).

## R7. Vim modes over the existing editors

- **Decision**: `EditState` gains `vim: Option<VimState>` (`Some` only under
  vim bindings). `VimState { mode, pending: Option<char>, register:
  Vec<char> }`. Normal-mode keys map onto the existing `Editor` API:
  `h/l` → `move_horizontal(±1, in_visual)`, `j/k` → `move_vertical`, `0/$`
  → `home/end`, `x` → select one unit then `delete_selection` into the
  register, `v` → set the selection anchor (`EditHistory.anchor`), `y`/`d`
  → copy/cut selection to the register, `p`/`P` → `paste(register)` after/
  before cursor, `u` → `undo`. `i/a/I/A` set Insert with cursor placement.
  Enter applies in both modes; Esc leaves Insert/Visual or cancels from
  Normal.
- **Rationale**: No second editor implementation; the selection/undo
  machinery in `EditHistory` already provides the primitives.
- **Alternatives considered**: A separate vim editor type (rejected:
  duplicates hex/text logic); Enter only applying in Normal (rejected by
  the user's confirmed summary: single-line editors).

## R8. Increment / decrement semantics (Ctrl+A / Ctrl+X)

- **Decision**: `Editor::adjust_number_at_cursor(delta: i8) -> Result<(),
  String>`: Hex → octet under cursor ±1 wrapping mod 256; Text with
  `TextFormat` integer → whole decimal value via string arithmetic
  (arbitrary size, sign-aware); OID → the arc containing or following the
  cursor; other text → the first run of ASCII digits at or after the
  cursor, grown as needed; DateTime → the active field, clamped to its
  range. Not found → `Err("no number at or after the cursor")`.
- **Rationale**: Matches vim's forward search for a number; keeps the
  integer editor free of i128 overflow.
- **Alternatives considered**: i128 arithmetic (rejected: INTEGERs in
  certificates exceed it); acting only in the integer editor (rejected:
  hex and OID adjustments are the common cases).

## R9. Mark storage and invalidation

- **Decision**: `Mark { source: RowSource, parent: Vec<usize>, anchor:
  usize, active: usize }` stored in `App`; marked rows derived at draw time.
  `clear_mark()` is called from a single `set_selected()` funnel used by
  every navigation method; `mark_extend` bypasses it. Marking is refused
  while `filter` is non-empty.
- **Rationale**: Indices into `rows` are invalidated by every rebuild;
  paths are stable until the operation that consumes the mark. A funnel
  makes the clearing rule enforceable by one test.
- **Alternatives considered**: storing row indices (rejected: stale after
  rebuild/expand); allowing marks under a filter (rejected: hidden
  elements would be deleted silently).

## R10. Highlighting the mark

- **Decision**: Marked rows get a reversed/blue-background style distinct
  from the cursor row (which keeps its current highlight); the cursor row
  inside a mark shows both (bold on the mark background).
- **Rationale**: Consistent with the blue selection background in the value
  editors documented in help.
- **Alternatives considered**: a gutter glyph per marked row (rejected: the
  gutter already carries fold and field markers).

## R11. Settings file write safety

- **Decision**: Create the directory if missing; write to
  `config.toml.tmp` then rename over `config.toml`; report any failure in
  the status line while keeping the in-memory setting.
- **Rationale**: NFR-004 and FR-024; rename is atomic on all three
  platforms for same-directory targets.
- **Alternatives considered**: direct overwrite (rejected: a crash mid-write
  leaves an empty file, which would then trigger a start-up notice).

## R12. Cut semantics

- **Decision**: Cut = copy (clipboard under normal, buffer always) then
  delete without confirmation; abort with no change if the copy reached no
  destination. Under vim bindings there is no cut key; `d` (with
  confirmation) fills the buffer, and Edit ▸ Cut is available.
- **Rationale**: Mirrors the value editors' documented "a cut removes
  nothing if the clipboard could not be written"; keeps the vim key space
  clean.
- **Alternatives considered**: `x` in the tree for cut under vim (rejected:
  reserve for future vim semantics).
