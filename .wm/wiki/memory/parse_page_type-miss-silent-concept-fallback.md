---
title: parse_page_type miss — silent concept fallback
type: memory
tags: [failure, parser, enum]
status: active
---

schema_version: 1
state: |-
  When adding PageType::Core, missed parse_page_type() in apps/wm-core/src/parser/mod.rs. Caused silent concept fallback caught via graph stats. Use the 8 touch points checklist (@wiki/patterns/page-type-registration-touch-points) to prevent this.
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
