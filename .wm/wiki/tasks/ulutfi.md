---
title: Add input JSON schemas to all MCP tools
type: task
status: done
tags: [feature, mcp, schemas, knowns-parity]
priority: high
id: ulutfi
acceptance_criteria:
  - text: "tools/list returns inputSchema with typed properties for each tool"
  - text: "Required fields are marked as required in the schema, and descriptions explain each parameter's purpose"
---

schema_version: 1
state: |-
  # Add input JSON schemas to all MCP tools

  > *Imported from Knowns task `ulutfi`*

  # Add input JSON schemas to all MCP tools

  ## Description


  WM's tools/list returns empty inputSchema for all tools. AI agents can't discover what parameters each tool accepts. Add proper JSON schemas (property names, types, descriptions, required fields) to all tool registrations so agents can self-discover parameters.


  ## Acceptance Criteria

  - [x] #1 tools/list returns inputSchema with typed properties for each tool
  - [x] #2 Required fields marked as required in schema
  - [x] #3 Descriptions explain each parameter's purpose
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
