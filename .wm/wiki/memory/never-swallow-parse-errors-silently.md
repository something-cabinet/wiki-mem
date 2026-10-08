---
title: Never swallow parse errors silently
type: memory
tags: [decision, parser, error-handling]
status: active
---

schema_version: 1
state: |-
  Parser `Err(_)` silently drops the error, making debugging require manual cross-referencing. Always log with `tracing::warn!` and include a content snippet. Fixed in `extract_frontmatter`. Decision: @wiki/decisions/silent-err-catch-in-parsers
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
