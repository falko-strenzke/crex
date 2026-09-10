# Decision Moment `01M22YBGQX9QXP30582Z3JWY6V`

- **Mission:** `asn1-tree-delete-copy-paste-01M22R1E`
- **Origin flow:** `plan`
- **Slot key:** `plan.settings.persistence-deps`
- **Input key:** `settings_persistence_deps`
- **Status:** `resolved`
- **Created:** `2026-09-09T11:20:22.525143+00:00`
- **Resolved:** `2026-09-09T11:26:50.305278+00:00`
- **Opened by:** `cli`
- **Other answer:** `false`

## Question

How should the settings file be located and parsed: with new crates (dirs + toml/serde) or with a zero-dependency std-only implementation?

## Options

- Std-only: env-var based OS paths and a hand-written key = value parser (TOML-compatible subset)
- Add dirs + toml + serde crates
- Other

## Final answer

Std-only: env-var based OS paths and a hand-written key = "value" parser (TOML-compatible subset), no new crates

## Rationale

_(none)_

## Change log

- `2026-09-09T11:20:22.525143+00:00` — opened
- `2026-09-09T11:26:50.305278+00:00` — resolved (final_answer="Std-only: env-var based OS paths and a hand-written key = "value" parser (TOML-compatible subset), no new crates")
