---
title: 'Failure: ONNX version-tracking mechanism built but never wired'
type: concept
id: wiki:concepts:onnx-metadata-never-threaded
status: draft
tags:
- failure
- onnx
- embedding
- wiring
relates_to:
  - {type: references, target: wiki:tasks:onnx-model-version-tracking--detect-model-changes-and-trigger-full-re-embed}
---

schema_version: 1
state: |-
  ## What went wrong

  The ONNX version-tracking triggers (model `mtime` change, chunking-version change) and the orphan-vector deletion mechanism existed in `packages/wm-embed` but were **dead code** — every caller passed `EmbeddingMetadata::default()` + `model_path=None`, so:
  - the persisted baseline metadata was never written or compared → triggers never fired
  - the production rebuild path used upsert-only writes → deleted pages' vectors stayed in turso and still returned in search

  ## Root cause

  Two halves of a feature were built without wiring them together: the *mechanism* (checks in `lib.rs`, orphan cleanup in `VectorDb::rebuild`) and the *callers* (which always passed empty metadata and never invoked the reconciliation path). No test exercised the production path end-to-end.

  ## Prevention

  - When a mechanism depends on metadata threading, verify the callers actually pass real values — grep for `Metadata::default()` / `None` at call sites before shipping.
  - Wire production paths to the same code the tests exercise (the orphan cleanup only ran in tests).
  - Version/chunking metadata should be computed from the actual model file + crate version at the call site, not left to the callee's default.

  ## Time lost

  Multiple hours: investigation lane found the schism, then a full wiring epic (#89 model-version, #74 chunking-version, #14 deletion-reconciliation, CRUD invalidation) plus a CLI-side caveat (wm-cli still passes default — flagged in the task notes).

  ## Related

  - @task-onnx-model-version-tracking--detect-model-changes-and-trigger-full-re-embed
  - @task-onnx-deletion-reconciliation--remove-orphan-embeddings-on-rebuild
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
