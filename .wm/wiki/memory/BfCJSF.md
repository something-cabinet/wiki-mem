---
title: ToolRegistry access levels are dead code
type: memory
tags: [mcp, toolregistry, dead-code]
created_at: "2026-07-14T06:39:04.982Z"
updated_at: "2026-07-14T06:39:04.982Z"
---

schema_version: 1
state: |-
  register_read/write/admin are identical — same impl, no permission stored, check_permission never set, dispatch() is first-match-wins. The entire typed.rs module (225 lines) is decorative. Can be deleted in the action-enum refactor.
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
