---
title: Gitea CI/CD for Rust CLI tools
type: memory
tags: [ci, gitea, rust, deployment]
created_at: "2026-07-07T10:34:52.925Z"
updated_at: "2026-07-07T10:34:52.925Z"
---

schema_version: 1
state: |-
  Self-hosted Gitea CI for Rust projects: local-host runner, CARGO_TARGET_DIR cache at /home/gitea/actions-cache/cargo/target, smart skip pattern using `git diff --name-only HEAD~1` to skip tests when unrelated files change. Build + test on push to master/dev, release binary on tags via `cargo build --release`. No Docker, no DB needed for CLI/MCP tools. Pattern from gehenna-app.
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
