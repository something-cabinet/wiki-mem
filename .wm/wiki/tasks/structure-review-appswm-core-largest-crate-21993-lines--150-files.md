---
title: 'Structure review: apps/wm-core (largest crate, 21,993 lines / 150 files)'
type: task
id: "wiki:tasks:structure-review-appswm-core-largest-crate-21993-lines--150-files"
status: todo
priority: medium
tags: [review, structure, wm-core, refactor]
acceptance_criteria:
  - text: "Every root directory in apps/wm-core is classified as feature, role, or misplaced, with a recommendation"
  - text: "Each of the 24 files over 300 lines is split-proposed with a target decomposition, or explicitly justified as exempt"
  - text: "The empty marker traits shared/traits/{factory,repository}.rs are removed or justified in writing"
  - text: "Findings are recorded with file:line and severity, and each actionable finding has a follow-up task"
  - text: "No code is changed by the review itself"
---

schema_version: 1
state: |-
  Scope: `apps/wm-core` — 21,993 lines across 150 files, the largest crate in the workspace.

  **Measured: 24 files exceed 300 lines.** Largest: `parser/mod.rs` 1,154, `graph/mod.rs` 1,010, `mcp/tools/task/mod.rs` 968, `mcp/tools/code.rs` 914.

  **Root-directory taxonomy to assess** (18 entries, mixing features with role-ish names): `config decision embed_files engine error graph mcp page page_repo parser search shared skill status util vector_db version`. Suspects: `util/` is a catch-all name; `page_repo/` is a role name at feature level; `embed_files/` holds assets rather than code. Decide whether each is a feature, a role dir, or misplaced.

  **Specific suspects worth confirming:**
  - Monolith `mod.rs` files that are really modules-in-disguise: `parser/mod.rs` (1,154) and `graph/mod.rs` (1,010).
  - `mcp/tools/` — is this role-first inside the `mcp` feature?
  - `apps/wm-core/src/shared/traits/{factory,repository}.rs` are **empty marker traits** (`pub trait Factory {}`, `pub trait Repository {}`) with one no-op impl each — dead ceremony under `no-dead-code-clone-scanning`, or justified? Decide explicitly.

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
