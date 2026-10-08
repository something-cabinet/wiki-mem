---
title: "Parallel Fixer Agents for Batch File Editing"
type: memory
tags: [workflow, delegation, batch]
created_at: "2026-07-24"
relates_to:
  - {type: references, target: wiki:patterns:parallel-fixer-agents}
---

schema_version: 1
state: |-
  Dispatch parallel fixer subagents each handling 2–8 files for batch operations. Categorize by module, give explicit per-type rules, run straggler pass after. Used for ~90-file comment removal in 7 agents. Full reference: @wiki/patterns/parallel-fixer-agents
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
