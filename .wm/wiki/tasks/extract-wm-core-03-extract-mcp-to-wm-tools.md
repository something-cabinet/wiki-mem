---
title: "EXTRACT-03 Extract mcp/** to packages/wm-tools"
id: "wiki:tasks:extract-wm-core-03-extract-mcp-to-wm-tools"
type: task
status: todo
priority: high
tags: [from-spec, spec:wm-core-layer-extraction, refactor, wm-tools, mcp]
acceptance_criteria:
  - text: "AC-2: apps/wm-core/src/mcp/ no longer exists; wm-core source is 12,000 lines or fewer, measured"
  - text: "packages/wm-tools exists and declares no rmcp and no clap; error/ stays in wm-core"
  - text: "AC-7: tools/code.rs delegates file walking and language inference to wm-code-intel; wm-tools adds no new walkdir/ignore traversal"
  - text: "cargo build, test and clippy --workspace are green"
---

schema_version: 1
state: |-
  D-6 step 2: extract `mcp/**` (8,789 lines) to `packages/wm-tools`. `ToolError`/`error/` stays behind with its 36 consumers (D-8). wm-tools declares no `rmcp` and no `clap` - it is transport-free (D-5).

  The import-rewrite blast radius is roughly 30 cross-boundary paths; split file-moves first, import-rewrite second, never half the tools moved. `tools/code.rs` must delegate traversal/language inference to wm-code-intel rather than arrive with its own walk. Part of @wiki/specs/wm-core-layer-extraction.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
