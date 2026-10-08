---
title: wm-vectors-bin extracted as zero-dependency crate
type: memory
tags: [vectors, crate, extraction]
created_at: "2026-07-14T04:41:47.531Z"
updated_at: "2026-07-14T04:41:47.531Z"
---

schema_version: 1
state: |-
  vectors.bin binary format (WMV\0 magic, SHA-256 content hashing) extracted to apps/wm-vectors-bin/. Zero dependencies, pure std. Used by wm-core via path dependency.
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
