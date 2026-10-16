---
title: "EXTRACT-04 Rewire wm-cli to wm_tools::ToolRegistry"
id: "wiki:tasks:extract-wm-core-04-rewire-wm-cli"
type: task
status: done
priority: high
tags: [from-spec, spec:wm-core-layer-extraction, refactor, wm-cli, wm-tools]
acceptance_criteria:
  - text: "wm-cli uses wm_tools::ToolRegistry; wm_core::ToolRegistry no longer exists"
  - text: "No transitional re-export shim remains (shims are how vector_db/mod.rs happened)"
  - text: "Both transport paths still work: stdio (mcp_server.rs) and the second path (engine_proxy.rs)"
  - text: "cargo build, test and clippy --workspace are green"
implementation_notes: |-
  EXTRACT-04 status 2026-10-16: landed with EXTRACT-03 rather than as a separate move (D-6 step 3 allows "same change as step 2 or immediately after").
  - Verified: apps/wm-cli uses wm_tools::ToolRegistry in all three transports (src/mcp_server.rs:15, src/mcp/mcp_service.rs:9+43, src/shared/services/engine_proxy.rs:9+25-26); zero wm_core::ToolRegistry / wm_core::mcp references remain; no transitional re-export shim in wm-core.
  - Completed the one gap EXTRACT-03 left: apps/wm-cli/Cargo.toml now depends on wm-tools with default-features=false and forwards onnx/code-intel/lsp/decision to it, so a no-default-features wm-cli build no longer drags wm-tools' onnx default (ort) in.
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