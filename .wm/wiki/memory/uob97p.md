---
title: MCP tool input schema pattern — register_with_schema()
type: memory
tags: [mcp, schemas, tool-discovery, ai]
created_at: "2026-07-09T07:54:40.460Z"
updated_at: "2026-07-09T07:54:40.460Z"
---

schema_version: 1
state: |-
  ToolRegistry should expose input JSON schemas per tool via tools/list. Added register_with_schema(name, desc, schema_json, handler) to ToolRegistry. Each tool declares its parameters with types, descriptions, defaults, and required fields. AI agents use these schemas to self-discover what arguments a tool accepts — no trial-and-error needed. Maps to MCP protocol's inputSchema field.
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
