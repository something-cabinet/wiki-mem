---
id: wiki:memory:zfdv25
title: 'Failure: Stale binary after revert breaks tests'
type: memory
tags: [test, build, cargo, stale-binary]
created_at: "2026-07-09T08:01:47.257Z"
updated_at: "2026-07-09T08:01:47.257Z"
---

schema_version: 1
state: |-
  After git reverting tool name changes, cargo test still used stale cached binary (wm-cli.exe). MCP tests spawn the binary directly, not through cargo. Full clean rebuild (cargo clean) needed. Lesson: always rebuild wm-cli after reverting wm-core changes since MCP tests spawn the binary.
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
