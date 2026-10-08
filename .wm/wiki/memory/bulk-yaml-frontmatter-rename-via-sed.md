---
title: Bulk YAML frontmatter rename via sed
type: memory
tags: [pattern, sed, yaml, migration]
status: active
---

schema_version: 1
state: |-
  For one-time bulk YAML frontmatter field renames across many wiki files, use line-start-anchored sed. This is safe for frontmatter keys but not for content. Always verify with cargo test and wm_index.rebuild after. Full pattern: @wiki/patterns/bulk-yaml-frontmatter-rename
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
