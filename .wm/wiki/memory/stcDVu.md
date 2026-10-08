---
id: wiki:memory:stcDVu
title: 'Failure: reqwest::blocking panics inside tokio runtime'
type: memory
tags: [rust, tokio, reqwest, ureq]
created_at: "2026-07-14T04:41:47.527Z"
updated_at: "2026-07-14T04:41:47.527Z"
---

schema_version: 1
state: |-
  reqwest::blocking::Client::new() panics when called inside #[tokio::main] because it creates/drops its own tokio runtime. Use ureq (pure blocking, no tokio dep) instead, or create via std::thread::spawn. Full reference: @doc/learnings/proxy-architecture-single-entrypoint
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
