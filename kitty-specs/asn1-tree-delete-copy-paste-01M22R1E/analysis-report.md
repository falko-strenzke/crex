---
schema_version: 1
artifact_type: spec-kitty.analysis-report
command: /spec-kitty.analyze
mission_slug: asn1-tree-delete-copy-paste-01M22R1E
mission_id: 01M22R1ESWGJN19C8X7SKS1N1G
generated_at: '2026-09-09T12:06:53.417194+00:00'
analyzer_agent: unknown
input_artifacts:
  spec.md:
    path: kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/spec.md
    sha256: 45948dbbc999f614979f6fe7fd223b43b211fb43810bec0b9f9a53a56e163e99
  plan.md:
    path: kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/plan.md
    sha256: 6248bb3f4a839c5950bb2a39283bb7eed66ab98e9cb993fe6214b7c4f472551c
  tasks.md:
    path: kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/tasks.md
    sha256: a4a1bda65f9c975e22119ae34163f7974862e65da5cec0568739f631f47aaf3a
  charter:
    path:
    sha256:
verdict: blocked
issue_counts:
  medium: 0
  high: 1
  critical: 0
  low: 1
  info: 0
findings:
- id: I1
  severity: high
  category: inconsistency
  summary: TreeAction::Yank is defined in data-model.md/WP01 but translate() (per WP01 T002 and contracts/keymap.md) never produces it — vim's 'y' returns TreeAction::Copy — while WP04 T021 and tasks.md's Subtask Index title the dispatch wiring 'Copy/Yank' as if Yank were a separate, reachable action.
- id: I2
  severity: low
  category: inconsistency
  summary: "plan.md IC-04 names the new App field as element_buffer: Option<Vec<u8>>, while data-model.md and WP04 correctly specify Option<ElementBuffer> (a struct of bytes + count) — a stale shorthand in plan.md's Implementation Concern Map, not the authoritative design."
---

## Specification Analysis Report

**Mission**: asn1-tree-delete-copy-paste-01M22R1E — ASN.1 Tree Delete, Copy and Paste
**Artifacts analyzed**: spec.md, plan.md, data-model.md, research.md, contracts/keymap.md, tasks.md, all 9 WP prompt files
**Charter**: none exists at `.kittify/charter/charter.yaml` for this project — charter alignment checks skipped (not a blocker).

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| I1 | Inconsistency | HIGH | data-model.md:19; tasks/WP01-key-binding-sets-and-action-translation.md:122,137; tasks/WP04-element-buffer-and-operand-editing.md:174; tasks.md:145,419 | `TreeAction` is defined with a `Yank` variant (data-model.md, WP01 T001), but WP01 T002's own matching logic and `contracts/keymap.md`'s "Copy / yank" row both establish that vim's `y` key returns `TreeAction::Copy`, never `Yank` — making `Yank` a dead enum variant. WP04's T021 subtask title ("Wire TreeAction::Delete/Cut/Copy/Yank dispatch") and tasks.md's Subtask Index row for T021 both refer to "Yank" as if it were a separate, reachable action, contradicting WP01's design. WP01's own T005 test `every_tree_action_has_a_key_or_is_documented_menu_only`, as specified, iterates every `TreeAction` variant and would need an explicit exemption for `Yank` that the WP01 prompt never states — as written, this test is untestable/would fail once someone tries to implement it faithfully against the enum as defined. | Before WP01 is implemented: remove `Yank` from the `TreeAction` enum in data-model.md and WP01/T001 (since T002's design deliberately routes vim's `y` through `TreeAction::Copy`, with the clipboard-vs-buffer distinction handled by `App.bindings` inside `copy_operand`, per WP04/T019); update WP04's T021 title and tasks.md's Subtask Index/tasks.md body wording from "Copy/Yank" to "Copy" to match. Alternatively, if a distinct `Yank` action is actually wanted (e.g. to let `label()` show "y" specifically for the vim entry rather than reusing `Copy`'s), make `translate()` return `TreeAction::Yank` for vim's `y` and add the corresponding dispatch arm in WP04/T021 explicitly — either fix is acceptable, but the three artifacts must agree on one shape before WP01/WP04 are implemented. |
| I2 | Inconsistency | LOW | plan.md:164 | plan.md's IC-04 concern description names the planned `App` field `element_buffer: Option<Vec<u8>>`, but the authoritative, later-written data-model.md and WP04's own frontmatter/body consistently specify `element_buffer: Option<ElementBuffer>` where `ElementBuffer` is a struct holding `bytes: Vec<u8>` and `count: usize`. This is a stale shorthand left over from an earlier planning pass and does not contradict what WP04 actually instructs (WP04 is correct and self-consistent), but a reader of plan.md alone would form the wrong mental model of the field's type. | Update plan.md IC-04's affected-surfaces bullet to read `element_buffer: Option<ElementBuffer>` for consistency with data-model.md. Purely cosmetic; does not block implementation since WP04 (the executable source of truth for that WP) already has the correct shape. |

**Coverage Summary Table:**

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|
| FR-001..FR-005 (marking) | Yes | T011–T016 | WP03 |
| FR-006..FR-010, FR-020, FR-026 (delete/copy/cut/yank) | Yes | T017–T022 | WP04 |
| FR-011..FR-013, FR-016..FR-019 (paste pipeline) | Yes | T027–T032 | WP06 |
| FR-014 (paste as child) | Yes | T028, T034 | Split across WP06 (function) and WP07 (menu entry) — intentional, documented in both WPs |
| FR-015, FR-016 (clipboard reading) | Yes | T023–T026 | WP05 |
| FR-021, FR-025, FR-027, FR-033 (keymap) | Yes | T001–T005 | WP01 |
| FR-022..FR-024 (settings) | Yes | T006–T010, T033, T035 | Split across WP02 (persistence) and WP07 (dialog) — intentional |
| FR-028..FR-031 (vim editors) | Yes | T038–T042 | WP08 |
| FR-032 (documentation) | Yes | T043–T046 | WP09 |
| NFR-001..NFR-006 | Yes | Spread across WP04, WP06, WP02, WP07, WP09 | All 6 non-functional requirements have at least one owning WP; see tasks.md's Requirements Coverage Summary for the full per-NFR table |
| C-001..C-008 | Yes | Spread across WP01, WP02, WP06 | All 8 constraints covered; see tasks.md's Requirements Coverage Summary |

100% of functional, non-functional, and constraint requirements have at least one mapped task (confirmed via `spec-kitty agent tasks map-requirements`'s coverage report: `unmapped_functional: []`).

**Charter Alignment Issues:** None — no charter exists for this project.

**Unmapped Tasks:** None — every T001–T046 belongs to exactly one WP and every WP has requirement_refs.

**Terminology check:** "Mark", "Operand", "Element buffer", "Key-binding set", "Editor mode" are used consistently across spec.md's Domain Language table, data-model.md, and every WP prompt file. "Yank" is used consistently as the *user-facing* term (spec.md FR-025, User Story 5) independent of finding I1, which is purely about the internal `TreeAction` enum shape, not user-facing wording.

**Task ordering check:** tasks.md's dependency graph (WP01/WP05 → WP02/WP03 → WP04 → WP06/WP08 → WP07 → WP09) is a valid DAG with no cycles, confirmed by `spec-kitty agent mission finalize-tasks`'s successful lane computation (9 lanes, correct `depends_on_lanes` at each parallel_group). No task-ordering contradictions found.

**Ambiguity check:** No vague, unmeasurable adjectives found in spec.md's requirements — every NFR carries a concrete numeric threshold (e.g. NFR-001: 250 ms / 500 elements / 1 MiB). No `TODO`/`TKTK`/`???`/`<placeholder>`/`NEEDS CLARIFICATION` markers remain in spec.md, plan.md, or tasks.md (the discovery and planning interviews resolved every decision; the decision ledger reports `deferred_count: 0`).

**Metrics:**

- Total Requirements: 47 (33 FR, 6 NFR, 8 C)
- Total Tasks: 46 (T001–T046)
- Coverage %: 100% (47/47 requirements have ≥1 mapped task)
- Ambiguity Count: 0
- Duplication Count: 0
- Critical Issues Count: 0
- High Issues Count: 1 (I1)
- Low Issues Count: 1 (I2)

## Next Actions

Both findings are inconsistencies inside the *design* artifacts (plan.md/data-model.md/WP prompts), not the spec itself, and both are cheap to fix before implementation starts:

- **I1 (HIGH)** should be resolved before WP01 is implemented, since it defines the enum WP04/WP07's dispatch code is built against — implementing WP01 as currently written would leave a dead `Yank` variant and an untestable coverage test. Recommended fix: remove `Yank` from `TreeAction`, update WP04/T021's title and tasks.md's two references from "Copy/Yank" to "Copy".
- **I2 (LOW)** is cosmetic and does not block implementation; fix at convenience by updating plan.md IC-04's one line.

No CRITICAL issues exist and no charter conflicts exist. The mission is otherwise ready for implementation once I1 is resolved (I2 may be deferred).
