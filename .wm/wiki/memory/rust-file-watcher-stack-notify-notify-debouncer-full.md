---
id: wiki:memory:rust-file-watcher-stack-notify-notify-debouncer-full
title: 'Rust file watcher stack: notify + notify-debouncer-full'
type: memory
tags: [rust, file-watcher]
status: active
---

schema_version: 1
state: |-
  Cross-platform Rust file watching: notify (108M downloads) for events, notify-debouncer-full for dedup/rename tracking. 500ms debounce window. Full reference: @wiki/patterns/rust-file-watcher-stack
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
