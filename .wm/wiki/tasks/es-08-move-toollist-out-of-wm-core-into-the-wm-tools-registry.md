---
title: ES-08 Move tool_list out of wm-core into the wm-tools registry
type: task
id: "wiki:tasks:es-08-move-toollist-out-of-wm-core-into-the-wm-tools-registry"
status: done
priority: medium
acceptance_criteria:
  - text: "tool_list is no longer a field on EngineState and set_tool_list no longer exists (FR-4)"
  - text: "The MCP tool cache lives in packages/wm-tools and the tool listing output is unchanged (FR-3)"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-tools, wm-core]
implementation_notes: 'Implemented in 015cf58. Cache moved to ToolRegistry as Arc<OnceLock<Vec<Value>>>, frozen once at the same registration point; no shim left. Flat fields 5 -> 4. NOTE: rg ''tool_list'' apps/wm-core now matches exactly one line - the AC-6 test, which names it in order to assert its absence. Verified: cargo test --workspace exit=0, 42 suites.'
---

schema_version: 1
state: |-
  Implements D-6 of @wiki/specs/engine-state-decomposition. `tool_list` leaves wm-core and becomes a
  field on the tool registry in `packages/wm-tools`. Its only writer and reader are already in the tools layer,
  and an MCP-protocol cache is application state rather than domain state, which is what
  @wiki/specs/wm-core-layer-extraction D-5 assigns to wm-tools.

  Remove the `tool_list` field and `set_tool_list` so wm-core stops carrying an MCP concern. The tool listing
  output must be unchanged (FR-3).
  Part of @wiki/specs/engine-state-decomposition
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