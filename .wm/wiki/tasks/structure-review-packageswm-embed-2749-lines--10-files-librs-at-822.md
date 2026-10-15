---
title: 'Structure review: packages/wm-embed (2,749 lines / 10 files; lib.rs at 822)'
type: task
id: "wiki:tasks:structure-review-packageswm-embed-2749-lines--10-files-librs-at-822"
status: todo
priority: medium
tags: [review, structure, wm-embed, refactor]
acceptance_criteria:
  - text: "lib.rs is reduced to re-exports with its logic relocated, or explicitly justified"
  - text: "vector_db.rs placement is decided (root vs services/) and overlap with vector_store_service.rs is resolved or ruled out"
  - text: "Feature-gated code is confirmed confined to appropriate modules"
  - text: "Findings are recorded with file:line and severity, and each actionable finding has a follow-up task"
  - text: "No code is changed by the review itself"
---

schema_version: 1
state: |-
  Scope: `packages/wm-embed` — 2,749 lines across only 10 files (~275 lines/file). Root dirs: `models/ services/`.

  **Measured: 3 files exceed 300 lines.** `services/onnx/mod.rs` 862, **`lib.rs` 822**, `vector_db.rs` 690 (at the crate root, outside any directory); `services/vector_store_service.rs` is 269.

  **Specific suspects:**
  - `lib.rs` at 822 lines is the strongest smell: a crate root that large usually means re-exports plus real logic that belongs in a module.
  - `vector_db.rs` sits at the crate root while its siblings live under `services/` — is it misplaced, and does it overlap `services/vector_store_service.rs`?
  - `services/onnx/mod.rs` at 862 lines: ONNX session management, tokenisation and batching.
  - This crate owns the `onnx` feature and a heavy dependency profile — check that feature gating is confined to the right modules rather than threaded through the crate.

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
