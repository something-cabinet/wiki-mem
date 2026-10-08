---
title: Snowball stemming in tokenizer — rust-stemmers for BM25
type: memory
tags: [search, bm25, tokenizer, stemming]
status: active
---

schema_version: 1
state: |-
  Tokenizer uses rust-stemmers (Snowball English, Porter2) to normalize morphological variants. "patterns"→"pattern", "designer"→"design", "styling"→"style". Stemmed form is additional (original kept). Uses LazyLock, shared across rayon threads. Full reference: @wiki/reference:search-scoring-formula
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
