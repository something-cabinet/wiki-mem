---
id: wiki:tasks:mcp
title: 'Fix 9 MCP tool schemas missing root type: object'
type: task
status: done
acceptance_criteria:
- text: 'All MCP tools (wm_page, wm_source, wm_index, wm_task, wm_model, wm_time, wm_decision, wm_memory, wm_template) have valid input schemas with root "type": "object"'
- text: No "Failed to generate schema" errors on MCP server startup
---

schema_version: 1
state: |-
  id: wiki:tasks:mcp

  **Severity:** Medium

  **Observed:** 7 MCP tools have schemas missing root `"type": "object"`, falling back to empty schemas: wm_page, wm_source, wm_index, wm_task, wm_model, wm_time, wm_decision, wm_memory, wm_template.

  **Root Cause:** The schemars derive or manual schema definitions for these tools don't include the required root `"type": "object"` field. MCP spec requires inputSchema to have root type 'object'.

  **Acceptance Criteria:**
  - [ ] All MCP tools have valid input schemas with root `"type": "object"`
  - [ ] No "Failed to generate schema" errors on startup
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
