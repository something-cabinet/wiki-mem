---
title: Reasonix Connector — OpenCode Plugin Installed
type: memory
status: draft
---

schema_version: 1
state: |-
  ## Reasonix Connector

  Installed OpenCode plugin that intercepts DeepSeek provider calls and routes them through reasonix CLI for prefix cache stability.

  ### Files
  - ~/.config/opencode/plugins/reasonix-connector.ts — server plugin
  - ~/.config/opencode/plugins/reasonix-connector-tui.tsx — TUI sidebar plugin
  - ~/.config/opencode/tui.json — TUI plugin registration
  - ~/.config/opencode/opencode.json — deepseek provider + plugin entry

  ### How it works
  - Intercepts chat.message when providerID is "deepseek"
  - Spawns reasonix run as fire-and-forget concurrently with provider
  - Swaps output if reasonix finishes first (cache hit)
  - Sidebar panel shows interception count, cache hit %, status

  ### Reasonix
  - v1.17.14 installed
  - Config at %APPDATA%/reasonix/config.toml
  - Uses DEEPSEEK_API_KEY env var
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
