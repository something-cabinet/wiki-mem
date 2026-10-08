---
title: Frontmatter corruption prevention — line-based YAML edits, quote ids
type: memory
tags: [wiki, yaml, frontmatter, corruption]
status: active
---

schema_version: 1
state: |-
  Never round-trip a whole YAML frontmatter block through serde_yaml for a field edit — unquoted ids like `652e07` become floats (6520000000.0) and unmodeled fields get dropped. Use line-based helpers (set_yaml_field/remove_yaml_block/ac_set_checked) and always double-quote id. Full: @wiki/patterns/line-based-frontmatter-editing
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
