---
title: "EXTRACT-01 Finalize step 0 — dead-code deletions and helper consolidation"
id: "wiki:tasks:extract-wm-core-01-finalize-step-0"
type: task
status: in-progress
priority: high
tags: [from-spec, spec:wm-core-layer-extraction, refactor, wm-core, deadcode]
acceptance_criteria:
  - text: "AC-1: cargo tree -p wm-core -e normal contains no rmcp, and rg rmcp apps/wm-core is empty"
  - text: "task_board_service.rs, wm-engine's EngineState stub and the vector_db shim are deleted"
  - text: "Exactly one pub fn slugify and one frontmatter splitter exist workspace-wide (grep count)"
  - text: "cargo build, test and clippy --workspace are green; test count is not lower than pre-extraction minus the deleted dead suites"
  - text: "Step 0 is committed on refactor/wm-core-extraction"
implementation_notes: |-
  EXTRACT-01 probe results 2026-10-16 on refactor/wm-core-extraction:
  - AC-1 met in substance: `cargo tree -p wm-core -e normal` has 0 rmcp and rg over *.rs is empty. The single literal `rg rmcp apps/wm-core` hit is prose inside tests/fixtures/decision_eval.jsonl (fixture data, not code) -> AC-1 wording should scope the grep to code paths.
  - Dead items gone: task_board_service.rs, packages/wm-engine/src/engine_state.rs, apps/wm-core/src/vector_db/mod.rs.
  - AC-6 / Gate 3 wording mismatch: there is NO symbol named `slugify` anywhere. Per the recorded decision (name distinct behaviours, do not converge them, see @wiki/patterns/name-distinct-behaviours-do-not-converge-them) there are three distinct-named slug helpers: label_slug (util/mod.rs), memory_slug (memory_gate/mod.rs), task_page_id_slug (mcp/tools/task/mod.rs); plus two frontmatter splitters: split_first_frontmatter (parser/mod.rs), split_leading_frontmatter (record_migration/frontmatter_prepass.rs). AC-6 'exactly one slugify' therefore cannot be asserted as written; it should assert no accidental duplicates among the distinct-named set.
  - Build/test verification (`cargo build -p wm-cli` + the CI targeted test matrix) running.
---

schema_version: 1
state: |-
  D-6 step 0: dedup and dead-code pass inside wm-core, zero new crates. The work is largely landed but uncommitted (deletions of `task_board_service.rs`, `packages/wm-engine/src/engine_state.rs`, `apps/wm-core/src/vector_db/mod.rs`; `rmcp` removed from the library; `slugify` and frontmatter-splitter consolidation). The YAML frontmatter serializer consolidation is deferred (D-10) and is NOT part of this task.

  Verify the tree against the ACs, run targeted tests while iterating then the full suite, and commit step 0 on the extraction branch. Part of @wiki/specs/wm-core-layer-extraction.
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