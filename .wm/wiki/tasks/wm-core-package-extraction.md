---
title: Extract cohesive packages out of wm-core (revive, mass-gated)
type: task
id: "wiki:tasks:wm-core-package-extraction"
status: todo
tags: [architecture, refactor, wm-core, from-finding]
acceptance_criteria:
  - text: A revived extraction spec exists that explicitly supersedes wiki:tasks:e688f0 and wiki:specs:extract-packages-from-wm-core, and states why this attempt will not repeat the sub-200-line ceremony failure
  - text: Every extracted package clears a stated mass and seam threshold; candidates below it (wm-util at 62 lines, wm-status) stay put
  - text: The MCP tool layer no longer lives in the library crate, and rmcp is no longer a dependency of it
  - text: No new circular dependency; the resulting dependency direction is recorded and checked
  - text: Duplication findings that fall inside the moved seams are resolved by the move, not left behind (dead board twin, page-type mapping, YAML serializers)
  - text: cargo build and cargo clippy stay warning-free and the workspace test suite stays green
priority: high
relates_to:
  - {type: implements, target: wiki:specs:wm-core-layer-extraction}
---

schema_version: 1
state: |-
  **Direction chosen by the owner, 2026-10-16:** revive the draft extraction and decompose `apps/wm-core` (22,230 lines / 151 files) into cohesive packages. Evidence base: @wiki/reference:wm-core-architecture-inventory.

  **This is the direction that was already reversed once, so the plan has to say why it will not fail the same way.** `wiki:tasks:e688f0` (done) collapsed the previous attempt on an @oracle review that found sub-200-line crates adding "multi-crate ceremony without delivering decoupling" - `wm-shared` (28 lines), `wm-util` (67), `wm-page-repo` (91). `wiki:specs:extract-packages-from-wm-core` (draft) was never executed: none of its 12 packages exist, and its own "~13K lines" is stale against today's 22,230.

  **Therefore this attempt is mass-gated:** a candidate is only extracted when it clears a stated size and seam threshold. The draft spec's small packages (`wm-util` at 62 lines, `wm-status`) stay where they are - extracting them is precisely the failure mode being corrected.

  **The structural prize is the same as the inventory's finding 1:** `mcp/` is 8,789 lines, 39.5% of the crate, and is the application layer rather than library code. Moving it out also removes `rmcp` from the library's dependency tree, where it is currently pulled for one error conversion (`error/mod.rs:165-177`).

  **Open questions the spec must answer:** which seams clear the threshold; whether the tool layer becomes its own crate or folds into `wm-cli`; the extraction order given the dependency direction; and which duplication findings inside the moved seams are resolved by the move itself rather than left behind.
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