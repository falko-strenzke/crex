# Data Model: ASN.1 Tree Delete, Copy and Paste

All types are in-memory application state unless noted. Names are
proposals for the implementer; behaviour is normative.

## KeyBindingSet (`src/keymap.rs`)

| Field / variant | Meaning |
|-----------------|---------|
| `Normal` | Ctrl+C/V/X in the tree, unchanged editors |
| `Vim` | `y`/`p`/`P`/`d` in the tree, modal editors, Ctrl+A/Ctrl+X only |

Invariants: exactly two variants (C-002); `Default` is `Normal`;
`Display`/`FromStr` use the strings `normal` and `vim` (the settings file
values).

## TreeAction and EditorAction (`src/keymap.rs`)

`TreeAction`: MarkUp, MarkDown, Delete, Cut, Copy, Yank, PasteAfter,
PasteBefore, PasteAsChild (menu-only; no key in either set), plus the
existing navigation/edit actions if the layer is extended to them.

`EditorAction` (vim normal/visual mode only): EnterInsert{At, After,
LineStart, LineEnd}, Left, Right, Up, Down, Home, End, DeleteUnderCursor,
VisualStart, Yank, DeleteSelection, PutAfter, PutBefore, Undo, Increment,
Decrement, Apply, Cancel, LeaveMode.

Invariants: `translate(key, Vim, _)` never returns an action for
CONTROL+{c,v,x,z}; `label(action, set)` returns `None` for actions with no
key in that set (rendered as "menu only").

## Mark (`src/app.rs`)

| Field | Type | Meaning |
|-------|------|---------|
| `source` | `RowSource` | Which forest the mark is in (Document, Decrypted, Pkcs12Revealed(i)) |
| `parent` | `Vec<usize>` | Path of the common parent (empty = top level) |
| `anchor` | `usize` | Sibling index where marking began |
| `active` | `usize` | Sibling index of the cursor end |

Derived: `range() = min(anchor,active)..=max(anchor,active)`;
`count() = range.len()`.

Invariants: `anchor` is always inside the range; `active` never leaves
`0..parent.children.len()`; a mark never exists while `filter` is non-empty
or while `source` is `DecryptedPlaceholder` / `CmsRevealed`; any selection
change other than `mark_extend` clears it; every operation that consumes it
clears it.

State transitions: see the mark lifecycle diagram in plan.md.

## Operand (derived, `src/app.rs`)

`Operand { source, parent, range }` computed as the mark when present,
otherwise `(row.source, row.parent, last..=last)` of the selection. Rejected
(status message, no change) when the selection is elided, a placeholder, a
read-only reveal, or the protected top-level SEQUENCE of a decrypted
PKCS#8/PKCS#12 region.

## ElementBuffer (`src/app.rs`)

| Field | Type | Meaning |
|-------|------|---------|
| `bytes` | `Vec<u8>` | DER encoding of the buffered elements, concatenated in tree order |
| `count` | `usize` | Number of top-level elements in `bytes` |

Lifecycle: `None` at start; replaced whole by copy, cut, yank and (vim)
delete; never partially updated; lives until process exit. Separate from
the editors' `VimState.register`.

## PasteWhere (`src/app.rs`)

`Before | After | AsChild`. `AsChild` requires a constructed or
encapsulating target and inserts at child index 0; `Before`/`After` insert
at `last` / `last + 1` in the parent's children. All three honour the
region rules (C-008) before mutating, then `rebuild()`, `dirty = true`,
select the first pasted row, expand a collapsed parent.

## PasteSource (transient)

`Clipboard(Vec<u8>)` read through `clipboard::bytes_for_paste` (order hex →
base64 → PEM blocks → raw; reports `PasteKind`) or `Buffer(&ElementBuffer)`.
Both must satisfy `ber::parse_forest(bytes, 0)` consuming every byte;
otherwise the paste is refused with the parser's message.

## Settings (`src/settings.rs`)

| Field | Type | Default | File key |
|-------|------|---------|----------|
| `key_bindings` | `KeyBindingSet` | `Normal` | `key_bindings = "normal" \| "vim"` |
| `unknown` | `Vec<(String, String)>` | empty | preserved verbatim on save (C-007) |

`LoadOutcome`: `Loaded(Settings)`, `Missing` (defaults, silent),
`Invalid { path, reason }` (defaults + start-up Notice), `NoLocation`
(no config directory resolvable; in-memory only, dialog says so).

`config_path()`: see contracts/settings-file.md.

## SettingsState (dialog, `src/app.rs`)

`{ choice: KeyBindingSet, path: Option<PathBuf>, error: Option<String> }`.
Enter → `settings.save()`, apply to `App.bindings`, status message; Esc →
discard. Open editors keep the set they opened with.

## VimState (`src/vim.rs`, inside `EditState`)

| Field | Type | Meaning |
|-------|------|---------|
| `mode` | `EditorMode { Normal, Insert, Visual }` | Current mode; editors open in `Normal` |
| `register` | `Vec<char>` | Editor-local yank/delete buffer in the editor's unit (digits or chars) |

Transitions: Normal →(i/a/I/A) Insert; Insert →(Esc) Normal; Normal →(v)
Visual; Visual →(Esc, y, d) Normal; Normal →(Esc) cancel edit; Normal or
Insert →(Enter) apply edit. `Some` only when `App.bindings == Vim` at the
moment the editor opens.

## Editor additions (`src/app.rs`)

`Editor::adjust_number_at_cursor(delta) -> Result<(), String>` per R8;
`Editor::select_unit_at_cursor()` for `x`; `Editor::paste_at(text, before:
bool)` for `p`/`P` (existing `paste` inserts at the cursor).

## Menu additions (`src/app.rs`)

`TopMenuAction`: `+ Settings, EditDelete, EditCut, EditCopy,
EditPasteBefore, EditPasteAfter, EditPasteAsChild`. `TOP_MENUS` gains an
`Edit` heading between File and About; File gains `Settings`. Key labels
are looked up at draw time from `keymap::label`.
