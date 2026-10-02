---
title: 'Failure: Metadata String Leftover After Removal'
type: concept
id: "wiki:concepts:failure-metadata-string-leftover-after-removal"
status: draft
tags: [failure, cleanup, spec]
relates_to:
  - {type: references, target: wiki:tasks:remove-tui-code-and-ratatui-dependency-from-wm-cli}
---

schema_version: 1
state: |-
  ## What went wrong

  The remove-tui spec's FR-6 said "remove the ratatui dependency" but did not mention the package description string. After implementation, `apps/wm-cli/Cargo.toml` still read `description = "CLI and TUI for the Wiki Memory Engine"` — a stale TUI reference in metadata. The fixer flagged it; the orchestrator fixed it post-hoc.

  ## Root cause

  The spec scoped dependency removal to Cargo.toml dependencies and source code, but not metadata strings (package description, clap `about` text, help strings) that mention the removed feature by name.

  ## Prevention

  When speccing a removal, add an explicit requirement: grep the feature name across the whole crate — including Cargo.toml metadata (description), clap `about` strings, help text, and README — not just source code. Add an AC: "no reference to <feature> remains in the crate's metadata or docs."

  ## Time lost

  ~2 minutes (one-line fix + re-verify)

  ## Related

  - @wiki/tasks/remove-tui-code-and-ratatui-dependency-from-wm-cli
  - @wiki/specs/remove-tui
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}
