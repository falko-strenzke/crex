---
affected_files: []
cycle_number: 1
mission_slug: asn1-tree-delete-copy-paste-01M22R1E
reproduction_command:
reviewed_at: '2026-09-09T14:17:24Z'
reviewer_agent: user
wp_id: WP09
---

# WP09 review feedback — DESIGN.md §3 dependency claim for paste.rs is inaccurate

Overall this WP is very strong: HELP_TOPICS prose matches the existing
topics' voice closely, the Ctrl+X-under-vim-is-decrement correction is
carried through correctly (no "no-op" language anywhere), the Edit menu and
File ▸ Settings entries are documented, the new doc-test genuinely enforces
coverage (independently verified: shortening the "Ctrl+A increments" phrase
in HELP_TOPICS made `help_topics_cover_actions` fail with a clear message;
change was reverted and `git diff` confirms no trace left), `cargo build`
is clean, `cargo test` gives exactly 469 passed / 2 pre-existing unrelated
failures (hsslms_single_level_verifies_via_openssl_and_multi_level_via_botan,
rekeying_to_single_level_lms_verifies_via_openssl_with_the_rfc9802_oid), and
`cargo clippy --lib` shows only pre-existing warnings (verified via
`git blame` that all four warned lines predate this commit, from July 2026).
Three spec.md acceptance scenarios (US1 mark+delete a run, US3 Edit ▸ Paste
as child, US4/US5 switch to vim in Settings then yank) are all fully
performable by someone who has read only the new help text.

One factual inaccuracy found while spot-checking DESIGN.md §3's dependency
claims against the modules' actual `use crate::` statements (as the review
guidance directs):

**DESIGN.md §3, the paste.rs dependency sentence**, currently reads:

> `paste.rs` depends on `app.rs`, `ber.rs` and `clipboard.rs` directly — it
> does not depend on `buffer.rs`, taking either raw bytes or a
> `PasteSource::Buffer(Vec<u8>)` cloned from `app.element_buffer` at the
> call site instead, since a borrow of the buffer alongside a `&mut App`
> call did not type-check;

This is true of the `PasteSource::Buffer` path, but `src/paste.rs`'s
`resolve_clipboard` (used for the FR-017 clipboard-empty/unavailable
fallback) takes `element_buffer: Option<&crate::buffer::ElementBuffer>` and
reads its `.bytes` field directly — a real, if narrow, coupling to
`buffer.rs`'s type that the "it does not depend on buffer.rs" sentence
contradicts.

**Requested fix**: soften/correct that one sentence, e.g. note that
`paste.rs`'s main entry point (`App::paste_tree` / `PasteSource::Buffer`)
avoids a `buffer.rs` dependency for the reason given, but `resolve_clipboard`
still reads `ElementBuffer.bytes` directly for the FR-017 fallback. A
one-line addition/qualification is sufficient — no other content in this WP
needs to change.
