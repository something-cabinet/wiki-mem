---
title: Code intelligence via regex for Rust projects
type: memory
tags: [code, rust, regex, code-intelligence]
created_at: "2026-07-09T07:54:43.995Z"
updated_at: "2026-07-09T07:54:43.995Z"
---

schema_version: 1
state: |-
  For Rust projects without full AST tooling, regex-based code search works well. wm_code.search uses walkdir+regex for pattern search, wm_code.symbols parses pub fn/struct/enum/trait declarations with regex, wm_code.deps parses use statements. No external dependencies needed. Adequate for most code navigation needs without bringing in rust-analyzer or tree-sitter.
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
