---
title: No Knowns references in commits, docs, or code
type: memory
tags: [rule, convention, branding]
status: active
---

schema_version: 1
state: |-
  Do not mention "Knowns" in commit messages, documentation, code comments, or code. Never reference the upstream project by name in any deliverable output. The project is independent and should be described on its own terms.
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
