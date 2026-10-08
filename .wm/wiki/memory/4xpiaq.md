---
title: MCP response enrichment pattern — match Knowns depth
type: memory
tags: [mcp, responses, knowns-parity, format]
created_at: "2026-07-09T07:54:42.220Z"
updated_at: "2026-07-09T07:54:42.220Z"
---

schema_version: 1
state: |-
  WM tool responses should match Knowns response depth for AI agent compatibility. Key enrichments: wm_doc.list returns tags/description/timestamps per doc, wm_task.board returns full task detail (ACs, timestamps, priority, timeSpent) per task, wm_memory.list returns content/camelCase dates. Response format uses camelCase JSON (createdAt, updatedAt) matching Knowns convention.
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
