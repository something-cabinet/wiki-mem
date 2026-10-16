---
title: "EXTRACT-06 Dual-transport integration test + full-workspace green"
id: "wiki:tasks:extract-wm-core-06-dual-transport-test"
type: task
status: todo
priority: high
tags: [from-spec, spec:wm-core-layer-extraction, test, integration, wm-tools]
acceptance_criteria:
  - text: "AC-5: one integration test dispatches wm_search.query through both transports (mcp_server.rs and engine_proxy.rs) and asserts identical results"
  - text: "AC-4: cargo build, test and clippy --workspace are green; test count is not lower than pre-extraction minus only the named deleted dead suites"
  - text: "AC-8: the spec records accepted debt with follow-up task links (ToolError in wm-core, EngineState coupling in wm-tools, constants collision, deferred YAML serializer)"
---

schema_version: 1
state: |-
  The proof the extraction served its stated purpose: one integration test dispatches `wm_search.query` through **both** transports - the stdio path (`wm-cli/src/mcp_server.rs`) and the second path (`wm-cli/src/shared/services/engine_proxy.rs`) - and asserts identical results (AC-5).

  Also the final full-workspace verification gate (AC-4) and the accepted-debt record (AC-8). Part of @wiki/specs/wm-core-layer-extraction.
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
