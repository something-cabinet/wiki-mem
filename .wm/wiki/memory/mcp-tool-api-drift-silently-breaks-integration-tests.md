---
title: MCP tool API drift silently breaks integration tests
type: memory
tags: [failure, testing, mcp]
status: active
---

schema_version: 1
state: |-
  Integration tests that launch wm-cli as a subprocess silently rot when the MCP tool surface evolves (action enums, tool renames). No compiler catches this. Fix: run full test suite in CI, update test fixtures in the same PR as the tool refactor. Full entry: @wiki/concepts/test-rot-mcp-api-drift
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
