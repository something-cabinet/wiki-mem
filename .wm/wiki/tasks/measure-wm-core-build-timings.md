---
title: Measure wm-core build timings before and after the mcp extraction
id: "wiki:tasks:measure-wm-core-build-timings"
type: task
status: todo
priority: medium
tags: [from-spec, build, timings, wm-core, performance]
acceptance_criteria:
  - text: "cargo build --timings (clean) captured on the pre-extraction tree and saved as an artifact"
  - text: "cargo build --timings captured after the mcp/ to packages/wm-tools extraction lands"
  - text: "The dominant crate in clean-build time is named and the heavy-dep (ort, tree-sitter) critical path is noted"
  - text: "The result is linked from the extraction spec (D-11) and states whether a further module-level split is warranted"
---

schema_version: 1
state: |-
  Measure `cargo build --timings` for the workspace before and after the `mcp/**` to `packages/wm-tools` extraction, to test the compile-time-parallelism hypothesis.

  Hardening from @wiki/specs/wm-core-layer-extraction D-11. The concern was that one 22K-line wm-core compiles as a single codegen unit; the review's answer was that data, not a structural bet, decides any future module-level split.

  Steps: capture a clean `cargo build --timings` on the pre-extraction tree; capture again after the extraction lands (extraction AC-3); record the dominant crate and whether the heavy deps (ort, tree-sitter) sit on the critical path.
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
