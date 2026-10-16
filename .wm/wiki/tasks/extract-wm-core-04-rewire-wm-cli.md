---
title: "EXTRACT-04 Rewire wm-cli to wm_tools::ToolRegistry"
id: "wiki:tasks:extract-wm-core-04-rewire-wm-cli"
type: task
status: todo
priority: high
tags: [from-spec, spec:wm-core-layer-extraction, refactor, wm-cli, wm-tools]
acceptance_criteria:
  - text: "wm-cli uses wm_tools::ToolRegistry; wm_core::ToolRegistry no longer exists"
  - text: "No transitional re-export shim remains (shims are how vector_db/mod.rs happened)"
  - text: "Both transport paths still work: stdio (mcp_server.rs) and the second path (engine_proxy.rs)"
  - text: "cargo build, test and clippy --workspace are green"
---

schema_version: 1
state: |-
  D-6 step 3: rewire `wm-cli` from `wm_core::ToolRegistry` to `wm_tools::ToolRegistry`. Same change as step 2 or immediately after. Any transitional re-export shim is dropped rather than left behind.

  This completes the layered shape: `wm-cli` (binary + transports) to `wm-tools` (tool handlers + ToolRegistry) to `wm-core` (domain library). Part of @wiki/specs/wm-core-layer-extraction.
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
