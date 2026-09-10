# Decision Moment `01M22RK502WGQNATNRXR2WWWC8`

- **Mission:** `asn1-tree-delete-copy-paste-01M22R1E`
- **Origin flow:** `specify`
- **Slot key:** `specify.vim-bindings.editor-depth`
- **Input key:** `vim_editor_depth`
- **Status:** `resolved`
- **Created:** `2026-09-09T09:39:41.186739+00:00`
- **Resolved:** `2026-09-09T09:45:08.349596+00:00`
- **Opened by:** `cli`
- **Other answer:** `true`

## Question

With vim key bindings configured, how much of vim's modal editing do the value editors (hex, text, integer, OID ...) adopt?

## Options

- Modal core: normal/insert modes, hjkl/0/$ motion, x, v visual selection, y/d/p, u undo
- Insert-mode toggle only: i enters typing, Esc leaves it, everything else unchanged
- Other

## Final answer

Modal core (normal/insert modes, i/a/I/A, Esc, h/j/k/l/0/$, x, v visual, y/d/p, u). In vim bindings Ctrl+C/X/V/Z do NOT act as in normal bindings; Ctrl+A increments and Ctrl+X decrements the number under the cursor as in vim. The vim key-binding space must stay free of normal-mode artifacts so more vim functions can be added later.

## Rationale

_(none)_

## Change log

- `2026-09-09T09:39:41.186739+00:00` — opened
- `2026-09-09T09:45:08.349596+00:00` — resolved (final_answer="Modal core (normal/insert modes, i/a/I/A, Esc, h/j/k/l/0/$, x, v visual, y/d/p, u). In vim bindings Ctrl+C/X/V/Z do NOT act as in normal bindings; Ctrl+A increments and Ctrl+X decrements the number under the cursor as in vim. The vim key-binding space must stay free of normal-mode artifacts so more vim functions can be added later.")
