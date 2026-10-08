---
title: dead_code ban — dead input fields masked by allow
type: memory
tags: [clippy, lint, dead-code, contract]
status: active
---

schema_version: 1
state: |-
  allow(dead_code) on API input fields hides dead contract fields (issue #126 root cause: wm_doc.r#type declared but never wired). Banned repo-wide 2026-08-14, CI grep enforces; use expect(dead_code) which errors when the lint stops firing (self-cleaning). No clippy lint can ban attributes. Full reference: @wiki/decisions/clippy-lint-curated-list-not-all
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
