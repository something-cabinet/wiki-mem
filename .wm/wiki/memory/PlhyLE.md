---
title: Action-enum MCP tools — merge CRUD, single register()
type: memory
tags: [mcp, tools, action-enum, refactor]
created_at: "2026-07-14T06:39:04.957Z"
updated_at: "2026-07-14T06:39:04.957Z"
---

schema_version: 1
state: |-
  Refactoring 78 dot-notation tools to ~33 action-enum tools. CRUD domains merge (page, doc, memory, task...), distinct tools stay separate (search, graph, code...). Drop fake register_read/write/admin — single register(). Action names are snake_case. Serde handles unknown actions, fallback lists available ones. Full reference: @doc/specs/mcp-tool-surface-action-enums
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
