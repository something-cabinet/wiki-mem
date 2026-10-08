---
title: cargo-npm + GitHub Actions multi-platform Rust distribution
type: memory
tags: [ci, rust, npm, github-actions]
status: active
---

schema_version: 1
state: |-
  Use cargo-npm with GitHub Actions matrix build to distribute Rust CLI binaries as npm packages. Cross-compile ARM64 Linux with gcc-aarch64-linux-gnu and CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER. Full reference: @wiki/patterns/cargo-npm-github-actions
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
