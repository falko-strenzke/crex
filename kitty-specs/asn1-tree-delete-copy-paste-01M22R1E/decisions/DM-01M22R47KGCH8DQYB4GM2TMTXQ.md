# Decision Moment `01M22R47KGCH8DQYB4GM2TMTXQ`

- **Mission:** `asn1-tree-delete-copy-paste-01M22R1E`
- **Origin flow:** `specify`
- **Slot key:** `specify.clipboard.keys-and-storage`
- **Input key:** `clipboard_keys_and_storage`
- **Status:** `resolved`
- **Created:** `2026-09-09T09:31:32.336758+00:00`
- **Resolved:** `2026-09-09T09:39:20.889719+00:00`
- **Opened by:** `cli`
- **Other answer:** `true`

## Question

Which keys trigger copy/paste in the tree and where does copied data live (in-app buffer vs system clipboard)?

## Options

- Ctrl+C/Ctrl+V via system clipboard as DER hex, plus in-app buffer
- y/p vim-style with in-app buffer only
- Other

## Final answer

Both paths (y/p in-app and Ctrl+C/Ctrl+V system clipboard), selected by a configuration option toggling between 'normal' (Ctrl+C/V) and 'vim' key bindings. Configuration persisted in the OS-typical user config location. The setting also governs the content editor: with vim bindings it gets a vim-style insert mode entered with 'i' etc. A new File top-bar menu entry opens a config editor (initially only the key-binding choice).

## Rationale

_(none)_

## Change log

- `2026-09-09T09:31:32.336758+00:00` — opened
- `2026-09-09T09:39:20.889719+00:00` — resolved (final_answer="Both paths (y/p in-app and Ctrl+C/Ctrl+V system clipboard), selected by a configuration option toggling between 'normal' (Ctrl+C/V) and 'vim' key bindings. Configuration persisted in the OS-typical user config location. The setting also governs the content editor: with vim bindings it gets a vim-style insert mode entered with 'i' etc. A new File top-bar menu entry opens a config editor (initially only the key-binding choice).")
