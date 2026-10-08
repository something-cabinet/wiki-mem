---
title: Duplicate frontmatter blocks hide data from parser
type: memory
tags: [frontmatter, validation, parser, failure]
status: active
---

schema_version: 1
state: |-
  Wiki pages can carry TWO --- delimited YAML frontmatter blocks (malformed). The parser reads only the FIRST block; ACs or data in the second block are invisible to validation and graph. When a task "already has ACs" but validation still fails, check whether the ACs are in the first block. Full reference: @wiki/concepts/failure-duplicate-frontmatter-blocks-hide-data
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
