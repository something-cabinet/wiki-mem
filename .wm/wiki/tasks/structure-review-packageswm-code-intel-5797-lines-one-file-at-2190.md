---
title: 'Structure review: packages/wm-code-intel (5,797 lines; one file at 2,190)'
type: task
id: "wiki:tasks:structure-review-packageswm-code-intel-5797-lines-one-file-at-2190"
status: todo
priority: medium
tags: [review, structure, code-intel, refactor]
acceptance_criteria:
  - text: "code_index_db.rs has a proposed split by responsibility with target file names and sizes"
  - text: "Each file over 300 lines is split-proposed or explicitly justified"
  - text: "Public API items with zero consumers across the workspace are identified"
  - text: "Findings are recorded with file:line and severity, and each actionable finding has a follow-up task"
  - text: "No code is changed by the review itself"
---

schema_version: 1
state: |-
  Scope: `packages/wm-code-intel` — 5,797 lines across 17 files. Root dirs `helpers/ models/ services/` (role-first).

  **Measured: 6 files exceed 300 lines.** Largest: `services/code_index_db.rs` **2,190**, `services/graph_resolver.rs` 1,008, `services/engine_service.rs` 851, `services/ts_config_resolver.rs` 510.

  **Note on layout**: this crate holds essentially ONE domain (code intelligence), so role dirs at the root are defensible — do not flag the inversion mechanically. The question here is file size and single responsibility, not directory shape.

  **Specific suspects:**
  - `services/code_index_db.rs` at 2,190 lines: confirm whether it conflates database schema, query building, snapshot loading and edge materialisation, then propose a split by responsibility.
  - `services/graph_resolver.rs` at 1,008: import resolution, symbol-edge resolution, receiver inference, path-distance tie-break and re-export chasing.
  - This crate is consumed by `wm-core` and only partially by `wm-cli` (see `@wiki/specs/code-index-cache-consumption`) — check for public API items with zero consumers across the workspace.

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
