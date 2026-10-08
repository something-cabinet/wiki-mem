---
title: Sync Writes > Async Channels for Single-User Local Tools
type: memory
tags: [write-channel, async, tokio, race]
created_at: "2026-07-07T08:07:15.678Z"
updated_at: "2026-07-07T08:07:15.678Z"
---

schema_version: 1
state: |-
  For single-user tools, prefer `std::fs::write()` over async write channels through tokio. The async channel's fire-and-forget semantic creates a race between write-returned and file-on-disk. If you must use async writes, ensure a flush barrier that doesn't deadlock the tokio runtime (use `spawn_blocking`). Full reference: @doc/learnings/learning-e2e-test-infrastructure-sync-write-fix
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
