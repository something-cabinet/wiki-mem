---
title: 'Structure review: packages/wm-engine (3,635 lines / 64 files)'
type: task
id: "wiki:tasks:structure-review-packageswm-engine-3635-lines--64-files"
status: todo
priority: medium
tags: [review, structure, wm-engine, refactor]
acceptance_criteria:
  - text: "The three oversized files in decision/ are split-proposed or explicitly justified"
  - text: "Very small files that exist only for convention compliance are identified"
  - text: "Findings are recorded with file:line and severity, and each actionable finding has a follow-up task"
  - text: "No code is changed by the review itself"
---

schema_version: 1
state: |-
  Scope: `packages/wm-engine` — 3,635 lines across 64 files (~57 lines/file). Root dirs: `decision/ helpers/ models/ status/`.

  **Measured: 3 files exceed 300 lines, all three inside `decision/`:** `record_validator.rs` 514, `canonical_questions.rs` 420, `question_wire_map.rs` 314.

  **Specific suspects:**
  - `decision/record_validator.rs` at 514 lines validates the whole typed-decision record surface; can the rules per concern (envelope, questions, answers, canonical-set matching) be separated?
  - `decision/canonical_questions.rs` at 420 holds the per-page-type question sets — is that data better as one file per page type, matching the one-type-per-file convention?
  - 64 files for 3,635 lines means many very small files: confirm none exists only to satisfy a convention without holding a real type.

  **Review checklist (the same lens just applied to `apps/wm-cli`):**
  1. **Layout**: are domains feature dirs at the root, with role dirs inside them? A **single-domain crate legitimately uses role dirs at the root** — flag an inversion only where a crate holds multiple domains.
  2. **One primary type per file**; role-suffixed filenames; a `mod.rs` barrel in every directory.
  3. **Oversized files**: `apps/wm-cli` now enforces 300 lines/file and 20 lines/function via a CI check. This checklist does NOT assume that budget applies here — measure, then recommend whether it should.
  4. **Shared-code rule**: an item with 2+ consumers is shared; a single-consumer item lives with its consumer; no item has two homes.
  5. **Dead code and ceremony**: unused modules, empty marker traits, declarations nothing enforces.
  6. **Boundary violations**: does a module reach into a sibling it should not?
  7. **Cross-crate duplication**: logic re-implemented here that already exists in another crate (one confirmed instance is filed: the `wm-cli` copy of `wm_core::search::indexed_doc_from_section`).

  **This is a review, not a refactor.** Do not change code while reviewing. Findings become follow-up tasks per rule `findings-first-task-spec`.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
