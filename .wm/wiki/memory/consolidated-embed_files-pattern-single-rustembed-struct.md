---
title: Consolidated embed_files pattern — single RustEmbed struct
type: memory
tags: [pattern, rust-embed, platform, config]
status: active
---

schema_version: 1
state: |-
  All embedded template files (shims, skills, configs) live under apps/wm-core/src/embed_files/{shims,skills,configs}/ with a single EmbeddedFiles RustEmbed struct. Config templates are static (no placeholder substitution — wm-cli is always on PATH). Merge logic (write_merged_json) lives in platform_service.rs. Reference: @wiki/patterns/embed-shim-templates
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
