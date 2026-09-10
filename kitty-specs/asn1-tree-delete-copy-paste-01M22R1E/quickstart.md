# Quickstart: ASN.1 Tree Delete, Copy and Paste

## Build and test

```sh
cargo build
cargo test                       # unit + integration tests
cargo test --release -- --ignored perf   # optional 250 ms budget check
```

## Try it

```sh
cargo run -- testdata/chain/server.der
```

1. In the Structure pane move onto an extension inside the Extensions
   SEQUENCE, press Shift+Down twice: three rows are highlighted.
2. Press Ctrl+C. Status: "3 elements copied to the clipboard as hex".
3. Move to another sibling, press Ctrl+V: the three extensions appear after
   it, cursor on the first. Press `d` `d` on a marked range to delete it.
4. Open the menu bar (F10), Edit shows all operations with their keys.
5. File ▸ Settings → choose "vim" → Enter. Now `y`, `p`, `P` work in the
   tree and `e` opens editors in normal mode (`i` to type, Esc, Enter).
6. Quit and restart: vim bindings are still active; the file is at the path
   shown in the Settings dialog (e.g. `~/.config/crex/config.toml`).

## Paste from outside

Copy a certificate's PEM text from any file, select an element, Ctrl+V:
the certificate's SEQUENCE is inserted as a sibling; the status line says
"decoded from PEM (1 block)".

## Where things live

- `src/keymap.rs` — key tables and action translation
- `src/settings.rs` — config path, load, save
- `src/vim.rs` — editor modes
- `src/app.rs` — mark, element buffer, delete/cut/copy/paste operations
- `src/tui.rs` — dispatch, highlighting, dialogs
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/` — binding, file and payload contracts
