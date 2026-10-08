---
title: Page Type Registration — 8 touch points
type: memory
tags: [pattern, type-system, enum]
status: active
---

schema_version: 1
state: |-
  Adding a new PageType requires updating 8 locations: enum, page variant, parser/mod.rs parse_page_type, page/mod.rs filter+create, lint.rs, reference_service.rs, styles.css, test setup dirs. Missing parser/mod.rs causes silent concept fallback. Full reference: @wiki/patterns/page-type-registration-touch-points
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
