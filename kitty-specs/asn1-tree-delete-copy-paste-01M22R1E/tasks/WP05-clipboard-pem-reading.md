---
work_package_id: WP05
title: Clipboard paste-source reading with PEM support
dependencies: []
requirement_refs:
- FR-015
- FR-016
planning_base_branch: feat/asn1-tree-delete-copy-paste
merge_target_branch: feat/asn1-tree-delete-copy-paste
branch_strategy: Planning artifacts for this mission were generated on feat/asn1-tree-delete-copy-paste. During /spec-kitty.implement this WP may branch from a dependency-specific base, but completed changes must merge back into feat/asn1-tree-delete-copy-paste unless the human explicitly redirects the landing branch.
subtasks:
- T023
- T024
- T025
- T026
phase: Phase 1 - Foundation
history:
- at: '2026-09-09T11:40:38Z'
  actor: system
  action: Prompt generated via /spec-kitty.tasks
agent_profile: implementer-ivan
authoritative_surface: src/clipboard.rs
create_intent: []
execution_mode: code_change
model: ''
owned_files:
- src/clipboard.rs
role: implementer
tags: []
task_type: implement
tracker_refs: []
---

# Work Package Prompt: WP05 – Clipboard paste-source reading with PEM support

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

Extend `src/clipboard.rs`'s existing three-step paste interpretation (hex → base64 → raw, used today by the value editors' Ctrl+V) with a fourth step, PEM-armoured text, and expose it as a function the tree-level paste pipeline (WP06) can call directly on whatever bytes/text it has in hand — from the system clipboard or, indirectly, validated as regular bytes when the source is the in-app element buffer.

Done when:
- `clipboard::bytes_for_paste(data: &[u8]) -> Result<(Vec<u8>, PasteKind), String>` exists and implements the reading order in `contracts/clipboard-payload.md` exactly: hex → base64 → PEM (one or more blocks) → raw.
- `PasteKind` gains a `Pem(usize)` variant (block count) with `describe()` returning wording matching the contract (`"decoded from PEM (n blocks)"`).
- Odd-length hex input is refused at step 1 with `"odd number of hex digits"` rather than silently falling through to base64.
- `cargo test clipboard::` passes for hex, base64, one PEM block, multiple PEM blocks, raw fallback, and the odd-hex-digit refusal.

This WP does **not** touch `src/app.rs` or `src/tui.rs` — it is a pure, independently testable extension of the existing `clipboard.rs` module and can be implemented and reviewed without any other work package having landed.

## Context & Constraints

Read before starting:
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/clipboard-payload.md` — the authoritative reading order and exact status wording this WP must produce.
- `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/research.md` R5 — why PEM is added and why the order is hex-first.
- `src/clipboard.rs` in full (only 285 lines) — read the existing `hex_digits(data) -> (String, PasteKind)` function and its module doc comment closely; this WP's `bytes_for_paste` follows the exact same pattern one level up (bytes out, not just hex-digit text), and the existing function's ordering rationale (hex before base64, because valid hex is *also* valid base64 and guessing wrong there silently pastes the wrong bytes) is the same reasoning driving PEM's position after base64 but before raw.
- `src/input.rs` — locate its PEM `BEGIN`/`END` marker search (used for opening PEM-wrapped documents) and **reuse it** rather than reimplementing marker scanning; if it is structured for a single block only, extract a small shared helper for finding one block's span and call it in a loop here for the multi-block case, rather than copying its logic inline.

**Important distinction from the existing `hex_digits`**: that function always succeeds (worst case, it falls through to raw bytes) and returns a `String` of hex digits for the caller to feed into a hex editor's digit buffer. `bytes_for_paste` is different in two ways: (1) it returns raw `Vec<u8>`, because the tree paste pipeline needs actual bytes to hand to `ber::parse_forest`, not hex-digit text; (2) it can fail (`Result`, not always-succeeds) at the hex step specifically, for the new odd-digit-count case — nothing else in this WP's function should ever return `Err`, since base64/PEM/raw all have a defined fallback.

Do not change `hex_digits`'s existing behaviour or signature — it is still used by the value editors' Ctrl+V (unrelated to this mission) and must keep working exactly as documented in the existing help text ("Editing values" topic in `src/app.rs`).

## Branch Strategy

- **Strategy**: {{branch_strategy}}
- **Planning base branch**: feat/asn1-tree-delete-copy-paste
- **Merge target branch**: feat/asn1-tree-delete-copy-paste

> These fields are populated automatically by `spec-kitty agent mission tasks`.
> Do NOT change them manually unless you are certain the branch topology has changed.
> Your execution worktree is allocated per the lane computed by `finalize-tasks` in `lanes.json`; do not assume a specific worktree path here.

## Subtasks & Detailed Guidance

### Subtask T023 – Implement clipboard::bytes_for_paste()

- **Purpose**: The core reading function the tree paste pipeline (WP06) will call.
- **Steps**:
  1. Signature: `pub fn bytes_for_paste(data: &[u8]) -> Result<(Vec<u8>, PasteKind), String>`.
  2. Step 1 (hex): if `data` is valid UTF-8 and, once whitespace is stripped, every character is an ASCII hex digit and there is at least one digit, decode it. If the stripped digit count is odd, return `Err("odd number of hex digits".to_string())` immediately — do not fall through. Otherwise decode pairs into bytes and return `Ok((bytes, PasteKind::Hex))`.
  3. Step 2 (base64): if not recognised as hex, try `crate::input::b64_decode` on the whitespace-stripped text (mirror `hex_digits`'s existing base64 branch); on success with non-empty output, return `Ok((bytes, PasteKind::Base64))`.
  4. Step 3 (PEM): scan `data` (as UTF-8 text) for one or more `-----BEGIN <label>-----` / `-----END <label>-----` pairs, in order of appearance, regardless of label (a mismatched label pair, e.g. BEGIN CERTIFICATE / END X509 CRL, is still accepted — the label is not validated here, only used to find the block boundaries; label consistency is not this function's concern). For each block, base64-decode the body between the markers and concatenate the decoded bytes across blocks in order. If at least one well-formed block was found and decoded, return `Ok((bytes, PasteKind::Pem(block_count)))`.
  5. Step 4 (raw): otherwise, return `Ok((data.to_vec(), PasteKind::Binary))`.
- **Files**: `src/clipboard.rs`.
- **Parallel?**: No — the other subtasks build on this function's shape.
- **Notes**: This function does not know or care whether the resulting bytes parse as a valid ASN.1 forest — that check belongs entirely to WP06's `paste_bytes`. Keep this function's job strictly to "what encoding was this text, and what bytes does it decode to".

### Subtask T024 – Add PasteKind::Pem variant

- **Purpose**: Let the status line say how many PEM blocks were read, matching the contract's wording.
- **Steps**:
  1. Add `Pem(usize)` to the existing `PasteKind` enum (alongside `Hex`, `Base64`, `Binary`).
  2. Extend `describe()`'s `match` with `PasteKind::Pem(n) => ` — return an **owned `String`** for this arm specifically (`format!("decoded from PEM ({n} block{})", if *n == 1 {""} else {"s"})`), since the existing arms return `&'static str`. This means `describe()`'s return type must change from `&'static str` to `std::borrow::Cow<'static, str>` (or `String`) — check every existing call site of `describe()` (the value editors' paste status line) and update them for the new return type; do not special-case `Pem` with a separate method, keep one `describe()` for all four variants.
- **Files**: `src/clipboard.rs`, plus any call site of `PasteKind::describe()` found by `grep -rn "\.describe()" src/`.
- **Parallel?**: No — depends on T023's decision to add the variant, but can be interleaved with it (same function edit session).
- **Notes**: Match the contract's exact singular/plural wording: `"decoded from PEM (1 block)"` for one block, `"decoded from PEM (3 blocks)"` for three.

### Subtask T025 [P] – Odd hex-digit refusal at step 1

- **Purpose**: Match the existing hex editor's stricter rule (an odd hex-digit count is a typo, not base64) rather than the original three-step `hex_digits` function's more permissive fallback — this is a deliberate, documented divergence called out in `contracts/clipboard-payload.md`.
- **Steps**:
  1. Confirm this is implemented as part of T023's step 1 (it should already be, if T023 was done correctly) — this subtask exists mainly to make the divergence from `hex_digits`'s behaviour explicit and tested, not to add new production code beyond what T023 already requires.
  2. Add a one-line comment at the odd-digit check in `bytes_for_paste` explaining *why* this differs from `hex_digits` (which would fall through to trying base64 on an odd-length hex-looking string): "`DEADBEE` reads as a typo, not base64 — refuse outright rather than guessing, per contracts/clipboard-payload.md."
- **Files**: `src/clipboard.rs`.
- **Parallel?**: Yes — a documentation/verification pass, not new logic; can be done alongside T024.
- **Notes**: Do not change `hex_digits`'s own behaviour to match this stricter rule — the two functions serve different callers with different established behaviour, and `hex_digits` is out of this mission's scope.

### Subtask T026 – Unit tests: hex, base64, one PEM block, multiple PEM blocks, raw fallback, odd-hex-digit refusal

- **Purpose**: Prove the reading order and every branch against concrete inputs.
- **Steps**:
  1. In `src/clipboard.rs`'s existing `#[cfg(test)] mod tests` (or a new one if none exists yet — check first), add:
     - `bytes_for_paste_reads_hex`: `b"DEADBEEF"` → `Ok((vec![0xDE,0xAD,0xBE,0xEF], PasteKind::Hex))`.
     - `bytes_for_paste_refuses_odd_hex_digit_count`: `b"DEADBEE"` → `Err("odd number of hex digits")` (exact message).
     - `bytes_for_paste_reads_base64`: a known base64 string (e.g. `base64::encode(b"hello world")` computed inline or hard-coded) → decodes to the right bytes, `PasteKind::Base64`.
     - `bytes_for_paste_reads_one_pem_block`: build a minimal PEM block (`-----BEGIN TEST-----\n<base64 of a few bytes>\n-----END TEST-----\n`) → `Ok((bytes, PasteKind::Pem(1)))`.
     - `bytes_for_paste_reads_multiple_pem_blocks_concatenated_in_order`: two blocks with different content → the decoded bytes are the first block's bytes followed by the second's, `PasteKind::Pem(2)`.
     - `bytes_for_paste_falls_back_to_raw`: arbitrary non-hex, non-base64, non-PEM bytes (e.g. `&[0x00, 0x01, 0xFF, 0x02]` — note this could theoretically also look like something else; pick bytes that are unambiguously none of the other three, such as including a byte >0x7F making it invalid UTF-8) → `Ok((data.to_vec(), PasteKind::Binary))`.
     - `pem_kind_describe_wording_matches_contract`: `PasteKind::Pem(1).describe()` == `"decoded from PEM (1 block)"`; `PasteKind::Pem(3).describe()` == `"decoded from PEM (3 blocks)"`.
  2. Use real certificate/key test fixtures from `testdata/` where convenient for the PEM tests (e.g. read `testdata/keylink/cert_ec.der`, wrap it in a `-----BEGIN CERTIFICATE-----` block yourself in the test rather than searching for a pre-existing `.pem` file — check `ls testdata/**/*.pem` first in case one already exists and is simpler to reuse).
- **Files**: `src/clipboard.rs` (test module).
- **Parallel?**: No — exercises the finished T023–T025.
- **Notes**: Keep test data small and inline (a handful of bytes) rather than depending on large fixture files, except where reusing an existing DER test file is genuinely simpler than constructing bytes by hand.

## Test Strategy

- `cargo test clipboard::` must pass.
- Run `cargo test` (full suite) once at the end of this WP to confirm the `describe()` signature change (if made `Cow`/`String`) did not break any existing call site — grep for `.describe()` across `src/` before declaring this WP done, not just within `clipboard.rs`.

## Risks & Mitigations

- **`describe()` signature change ripples**: changing `&'static str` to an owned type touches every call site. Grep first (`grep -rn "PasteKind\|\.describe()" src/`), list every call site in this WP's Activity Log before editing, and confirm each compiles after the change — do not guess there are only one or two.
- **PEM label mismatch bodies**: some real-world PEM files have inconsistent labels between BEGIN/END for the same logical block due to hand-editing. Per T023 step 4, this function is deliberately lenient (does not check label equality) — do not add a label-equality check that would make legitimately mismatched-but-still-valid files fail to paste; that stricter check, if ever wanted, belongs to a different requirement.

## Review Guidance

- Confirm the odd-hex-digit refusal happens strictly before any base64 attempt (read the code path, not just the test) — a subtle bug would be checking hex validity but not short-circuiting before falling through.
- Confirm `describe()`'s existing three call sites (Hex/Base64/Binary status-line wording used by the value editors today) are byte-for-byte unchanged in wording — this WP must not alter existing user-visible text, only add new text for the new `Pem` case.
- Confirm multi-block PEM concatenation preserves block order — check the test asserts on order, not just total byte count.

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

Status is managed via `status.events.jsonl`. Use `spec-kitty agent tasks move-task WP05 --to <status>` to change WP status.
