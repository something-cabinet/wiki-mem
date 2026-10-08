---
id: wiki:memory:xfobs2
title: 'Decision: register_with_schema() over description-only tools'
type: memory
tags: [mcp, schemas, good-call]
created_at: "2026-07-09T08:01:43.982Z"
updated_at: "2026-07-09T08:01:43.982Z"
---

schema_version: 1
state: |-
  Added register_with_schema() to ToolRegistry so each tool declares typed params, defaults, and required fields. Over: keeping empty inputSchema. Outcome: AI agents can now self-discover tool parameters via tools/list. Full reference: @doc/learnings/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5
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
