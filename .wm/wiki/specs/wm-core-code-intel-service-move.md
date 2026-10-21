---
title: wm-core Gate 1 — move code-intel services into the package
type: spec
id: "wiki:specs:wm-core-code-intel-service-move"
status: approved
tags: [ci, wm-core, wm-code-intel, structure, spec]
acceptance_criteria:
  - checked: false
    text: Spec reviewed and its requirements are implemented
---

schema_version: 1
state: |-
  ## Goal

  Bring `apps/wm-core` back under its 13,000-line Gate 1 ceiling by moving code-intel-specific modules into
  `packages/wm-code-intel`, without changing behavior and without waiving the ceiling.

  ## Verified current state

  - CI run 39397172947 (commit 394017c) fails Gate 1:
    `apps/wm-core: 13271 lines > ceiling 13000 and no waiver in .github/workspace-crates.toml`.
  - The violation predates the code-index commits (`28377ed` was 13,053); the earlier wm-cli file-size failure had
    masked Gate 1.
  - `apps/wm-core/src/engine/code_index_refresh_service.rs` is 243 lines and only depends on wm-code-intel plus
    wm-constants. `apps/wm-core/src/graph/code_edges.rs` is 73 lines and is a thin code-intel wrapper.
  - The ceiling is deliberate (spec @wiki/specs/wm-core-layer-extraction D-11) and exists to stop wm-core regrowth.

  ## Design

  - Move the refresh service to `packages/wm-code-intel/src/services/refresh_service.rs`, rewriting
    `crate::code_intel::*` imports to `crate::*`.
  - Move the graph wrapper to `packages/wm-code-intel/src/services/code_graph_service.rs`, importing
    `crate::services::refresh_service::refresh_if_stale` and the resolver from the same crate.
  - Replace both wm-core files with one-line re-export stubs so every existing path
    (`wm_core::engine::code_index_refresh_service::*`, `wm_core::graph::code_edges::*`) keeps working.
  - Register both modules in `packages/wm-code-intel/src/services/mod.rs`.

  Result: wm-core drops from 13,271 to 12,957 lines (43 under the ceiling); wm-code-intel absorbs 316 lines.

  ## Non-functional requirements

  - No behavior change: marker lifecycle, retry policy, single-flight, fallback, and public paths stay identical.
  - No waiver added; the D-11 ceiling stays at 13,000.
  - No comments, no `else`, no `#[allow(...)]`; rustfmt and clippy clean.

  ## Verification

  - `bash scripts/check-crate-sizes.sh` prints `wm-core (ceiling) 12957 ok` and `Gate 1 passed`.
  - `wm-code-intel`: 82 unit + 14 integration pass (refresh unit tests moved with the service).
  - `wm-core` lib 226; watcher 13; `e2e_code_intel` 7; process-lock 1; `graph_code_edges` 5.
  - `cargo clippy -p wm-code-intel -p wm-core -p wm-cli -p wm-tools -- -D warnings` clean.
  - rustfmt clean on every touched file.

  ## Out of scope

  - Further wm-core module moves; the 43-line headroom is left for small changes.
  - Raising or waiving the ceiling.

  ## References

  - @wiki/tasks/wm-core-exceeds-its-13000-line-gate-1-ceiling-after-the-code-index-work
  - @wiki/specs/wm-core-layer-extraction
  - @wiki/specs/code-index-db-serialization
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: migration
  scope: component
  status_class: approved
  needs_tasks: true