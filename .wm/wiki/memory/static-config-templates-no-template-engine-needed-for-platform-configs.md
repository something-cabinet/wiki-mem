---
title: Static config templates — no template engine needed for platform configs
type: memory
tags: [decision, platform, config, knowns]
status: active
---

schema_version: 1
state: |-
  Platform config templates (opencode.json, .mcp.json, etc.) are static files with "wm-cli" hardcoded — no placeholder substitution needed since wm-cli is on PATH. Don't reach for a template engine just because Knowns uses Handlebars: their Handlebars is for code generation (knowns template run), not platform config. Reference: @wiki/decisions/static-config-templates-no-substitution
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
