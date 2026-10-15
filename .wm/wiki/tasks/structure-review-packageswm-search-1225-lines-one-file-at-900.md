---
title: 'Structure review: packages/wm-search (1,225 lines; one file at 900)'
type: task
id: "wiki:tasks:structure-review-packageswm-search-1225-lines-one-file-at-900"
status: todo
priority: medium
tags: [review, structure, wm-search, refactor]
acceptance_criteria:
  - text: "bm25_index_service.rs has a proposed split by responsibility with target file names and sizes"
  - text: "The relationship between the small sibling files and the monolith is stated (separate concerns vs fragments)"
  - text: "Findings are recorded with file:line and severity, and each actionable finding has a follow-up task"
  - text: "No code is changed by the review itself"
---

schema_version: 1
state: |-
  Scope: `packages/wm-search` — 1,225 lines across 8 files. Root dirs `helpers/ services/` (role-first, single domain — defensible here).

  **Measured: 1 file exceeds 300 lines** — `services/bm25_index_service.rs` at **900**, which is 73% of the crate. The rest are small: `field_model.rs` 226, `scoring_helper.rs` 52, `search_result_model.rs` 23.

  **Specific suspects:**
  - `bm25_index_service.rs` at 900 lines: index construction, incremental add/remove, field weighting, RRF fusion and rerank heuristics may all live here. Propose a split by responsibility.
  - With the crate at 1,225 lines and one 900-line file, assess whether the remaining files are genuinely separate concerns or fragments of the monolith.

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
