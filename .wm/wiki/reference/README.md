---
title: Knowns — Reference
type: reference
tags: []
---

schema_version: 1
state: |-
  id: wiki:reference:README
  # Knowns — Reference

  ## GitHub
  - Repo: https://github.com/knowns-dev/knowns
  - Docs: https://github.com/knowns-dev/knowns/tree/main/.knowns/docs
  - opencode.json: https://github.com/knowns-dev/knowns/blob/main/opencode.json

  ## MCP Configuration
  Knowns runs as an MCP server via: knowns mcp --stdio
  OpenCode config entry: {"knowns": {"command": ["knowns", "mcp", "--stdio"], "enabled": true, "type": "local"}}

  ## Architecture
  - Language: Go (rewritten from previous version)
  - CLI style: Bubble Tea TUI with --json/--plain/--no-pager flags
  - Storage: Markdown files as canonical store
  - Search: Keyword (BM25 — industry-standard text search, used by Elasticsearch. Scores docs by term frequency vs rarity) + Semantic (embeddings — matches by meaning, not exact words. "Fix login" matches "resolve auth error".) + Hybrid (RRF — merges keyword + semantic result lists by rank) + Rerank
  - MCP: JSON-RPC 2.0 over stdio
  - Skills: YAML frontmatter + markdown instructions

  ## Key Differences from WM Engine
  | Feature | Knowns | WM Engine |
  |---------|--------|-----------|
  | Language | Go | Rust |
  | Graph edges | Tag-based | Typed (extends, depends_on, etc.) |
  | CLI TUI | Bubble Tea | Ratatui (scaffolded) |
  | Search modes | Keyword + semantic + hybrid (RRF + rerank) | Keyword + semantic + hybrid (RRF — merges keyword + semantic result lists by rank) |
  | Time tracking | Full (start/stop/add/report) | Full (start/stop/add/report) |
  | Source state machine | Not built-in | Full (add/process/complete) |
  | Skill system | Trigger-based | Trigger-based + MCP registration |
questions:
  - id: kind
    type: choice
    instructions: What kind of reference material is this?
    options:
    - api
    - cli
    - configuration
    - error-catalog
    - schema
    - scoring
  - id: surface
    type: choice
    instructions: Which surfaces does this reference document?
    multi: true
    options:
    - mcp-tool
    - cli-command
    - rust-api
    - http-api
    - config-file
  - id: stability
    type: score
    instructions: How stable is the documented surface?
    levels:
    - unstable
    - evolving
    - stable
    - frozen
  - id: has_examples
    type: noul
    instructions: This reference includes usage examples.
answers: {}
