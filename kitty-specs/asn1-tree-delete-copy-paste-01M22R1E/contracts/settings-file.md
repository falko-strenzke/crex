# Contract: settings file

## Location (`settings::config_path()`)

| Platform | Directory | Rule |
|----------|-----------|------|
| Linux / other Unix | `$XDG_CONFIG_HOME/crex/` else `$HOME/.config/crex/` | first variable that is set and non-empty |
| macOS | `$HOME/Library/Application Support/crex/` | |
| Windows | `%APPDATA%\crex\` | |

File name: `config.toml`. When no variable resolves, `config_path()` is
`None`: settings are in-memory only and the Settings dialog shows "no
configuration directory found" instead of a path.

## Format

A strict subset of TOML, so that standard tools can read it:

```toml
# crex settings — edited by File ▸ Settings
key_bindings = "vim"
```

- One `key = "value"` per line; keys are `[A-Za-z0-9_]+`; values are
  double-quoted strings without escapes.
- Blank lines and lines starting with `#` are ignored.
- Unknown keys are kept and rewritten unchanged on save (C-007).
- Duplicate keys: last one wins on load; a single line is written on save.
- Any other line makes the file `Invalid`; the reason names the line number.

## Recognised keys

| Key | Values | Default | Effect |
|-----|--------|---------|--------|
| `key_bindings` | `normal`, `vim` | `normal` | active key-binding set |

## Behaviour

| Situation | Result |
|-----------|--------|
| File missing | defaults, no message |
| File unreadable / invalid | defaults, start-up notice "settings file <path>: <reason>; using defaults" |
| Save with missing directory | directory created, then written |
| Save failure | setting applies for the session, status line reports the error |
| Write | to `config.toml.tmp` then renamed over `config.toml` |
