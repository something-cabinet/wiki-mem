---
title: Frontmatter scalar quoting
type: memory
tags: [frontmatter, yaml, task-store]
status: active
---

schema_version: 1
state: |-
  Frontmatter written via raw format! without quoting breaks YAML when values start with [ or contain : or backslashes — tasks become invisible ("task not found"). Quote user-supplied scalars (title, ACs) or write through yaml_helper / a YAML-aware serializer. Full reference: @wiki/patterns/line-based-frontmatter-editing
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
