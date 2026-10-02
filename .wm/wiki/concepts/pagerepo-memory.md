---
id: wiki:concepts:pagerepo-memory
title: PageRepo — Repository Trait for Filesystem I/O
type: concept
tags: [pattern, testing, filesystem]
relates_to:
  - {type: references, target: wiki:patterns:pagerepo-trait}
---

schema_version: 1
state: |-
  id: wiki:concepts:pagerepo-memory

  Extract filesystem I/O behind a `PageRepo` trait with two impls: `FsPageRepo` (prod) and `InMemoryPageRepo` (tests). Public API stays backward-compatible via internal delegation. Pattern applied to `page.rs` — 7 functions refactored.

  Full reference: @wiki/patterns/pagerepo-trait
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
