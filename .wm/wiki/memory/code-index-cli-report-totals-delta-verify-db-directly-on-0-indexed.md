---
title: Code index CLI — report totals + delta; verify DB directly on "0 indexed"
type: memory
tags: [code-intel, cli, debugging]
status: active
---

schema_version: 1
state: |-
  Incremental hash-skip runs (wm index code) print 0 NEW symbols on no-change runs — indistinguishable from a broken index. rebuild_code_index returns CodeIndexStats (deltas + post-run totals). When a user reports "0 symbols indexed": query code.db directly (SELECT COUNT(*) FROM code_symbols) before suspecting the pipeline; jest.config.ts-style files legitimately extract 0 symbols (module.exports not captured). Full ref: @wiki/patterns/cli-delta-vs-total-reporting + @wiki/concepts/incremental-rebuild-zero-delta-false-alarm
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
