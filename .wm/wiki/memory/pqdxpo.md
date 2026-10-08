---
title: Knowns/WM is a memory layer, not a spec system
type: memory
tags: [memory, knowns, architecture, openspec]
created_at: "2026-07-07T03:52:29.276Z"
updated_at: "2026-07-07T03:52:29.276Z"
---

schema_version: 1
state: |-
  OpenSpec (@fission-ai/openspec) is a dedicated spec system with change folders, lifecycle, and cross-repo Stores. Knowns/WM specs are a thin technique (Socratic exploration + doc template) on top of the memory engine. Knowns' real value is the memory substrate: typed graph edges, semantic retrieval, context assembly, cross-references, AC tracking. WM should double down on the memory layer, not try to compete with OpenSpec. Full reference: @doc/learnings/learning-knowns-memory-layer-not-a-spec-system
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
