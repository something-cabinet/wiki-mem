---
title: npm same-major override defeats bogus downgrade fix
type: memory
tags: [npm, audit, security, overrides]
status: active
---

schema_version: 1
state: |-
  When npm audit's only fix is a bogus downgrade of a direct dependency (exact-pinned vulnerable transitive, isSemVerMajor pointing at older version), use a same-major overrides entry to force the patched version. Can't override a direct dep unless specs match (EOVERRIDE) — bump the direct range instead. Major-jump overrides need real-suite verification, not just a build. Full reference: @wiki/patterns/npm-override-vs-bogus-downgrade
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
