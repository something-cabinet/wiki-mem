---
title: MCP Bridge for Web UIs
type: memory
tags: [mcp, web-ui, bridge]
created_at: "2026-07-06T17:43:11.514Z"
updated_at: "2026-07-06T17:43:11.514Z"
---

schema_version: 1
state: |-
  Web UI communicates with Rust engine via wm serve child process + JSON-RPC over stdin/stdout. wm-bridge.ts spawns the process, sends/receives JSON-RPC. SvelteKit API routes delegate to the bridge. No HTTP server crate needed in Rust. Full reference: @doc/learnings/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration
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
