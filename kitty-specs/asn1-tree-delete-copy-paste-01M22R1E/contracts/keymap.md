# Contract: key bindings per set

Source of truth for `src/keymap.rs`, the Edit menu labels, the help window
and DESIGN.md §11. "—" means no key; the action is reachable only through
the Edit menu.

## Structure pane (Browse mode, document focus)

| Action | Normal set | Vim set | Notes |
|--------|-----------|---------|-------|
| Mark down / up | Shift+Down / Shift+Up | Shift+Down / Shift+Up | anchor = selection at first press |
| Delete operand | `d` `d` | `d` `d` | second press confirms; vim also fills the element buffer |
| Cut operand | Ctrl+X | — | copy then delete, no confirmation |
| Copy / yank operand | Ctrl+C | `y` | normal: clipboard as hex + buffer; vim: buffer only |
| Paste after | Ctrl+V | `p` | normal: before/after dialog when selection is first sibling |
| Paste before | (dialog) | `P` | |
| Paste as child | — | — | Edit ▸ Paste as child only |
| Clear mark | Esc or any selection change | same | |

All pre-existing keys (`j`/`k`, `i`/`I`, `e`/`E`, `J`/`K`, `z`, `/`, `s`,
Ctrl+S, `[`/`]`, `q`) keep their behaviour in both sets.

## Value editors

| Action | Normal set | Vim set (mode) |
|--------|-----------|----------------|
| Type characters | always | Insert |
| Enter insert at / after / start / end | n/a | `i` / `a` / `I` / `A` (Normal) |
| Leave insert or visual | n/a | Esc |
| Move | arrows, Home, End | arrows always; `h` `l` `j` `k` `0` `$` (Normal, Visual) |
| Extend selection | Shift+arrows | `v` then motions (Visual) |
| Select all | Ctrl+A | — (future `ggVG`) |
| Delete under cursor | Delete | `x` (Normal) |
| Yank selection | Ctrl+C | `y` (Visual) |
| Cut selection | Ctrl+X | `d` (Visual) |
| Put | Ctrl+V (clipboard) | `p` / `P` (register, Normal) |
| Undo | Ctrl+Z | `u` (Normal) |
| Increment / decrement number | — | Ctrl+A / Ctrl+X (Normal) |
| Apply | Enter | Enter (Normal or Insert) |
| Cancel | Esc | Esc (Normal) |
| Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+Z | as above | **unbound** (C-002) |

Bracketed paste from the terminal (Ctrl+Shift+V) continues to insert into
the editor in both sets.

## Menus and dialogs

- Menu bar toggle (Alt / F10 / Alt+M), arrows, Enter, Esc: unchanged.
- Before/after paste dialog: Up/Down or `j`/`k` choose, Enter confirms, Esc
  cancels, `1`/`2` select directly (as the edit-menu popup does).
- Settings dialog: Up/Down or `j`/`k` choose the set, Enter saves and
  applies, Esc discards.

## Tests derived from this contract

- Vim table has no entry whose modifiers contain CONTROL for `c`, `v`, `x`,
  `z` in any context.
- Every `TreeAction` except `PasteAsChild` has a key in the normal set;
  every one except `Cut` and `PasteAsChild` has a key in the vim set.
- `Shift+Up` given as `KeyCode::Up` + SHIFT translates to `MarkUp` in both sets.
