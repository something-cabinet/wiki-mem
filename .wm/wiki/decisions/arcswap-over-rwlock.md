---
title: arcswap over rwlock
id: wiki:decisions:arcswap-over-rwlock
type: decision
relates_to:
  - {type: references, target: wiki:patterns:arc-swap-graph}
  - {type: implements, target: wiki:specs:graph-engine}
---

schema_version: 1
state: |-
  id: wiki:decisions:arcswap-over-rwlock

  ## Context

  The wiki graph needs to support concurrent reads (search queries) while periodic rebuilds (page edits) reconstruct the entire graph. The naive approach is `RwLock<DiGraph>` — readers acquire read lock, writer acquires write lock during rebuild.

  ## Chosen approach

  `ArcSwap<(StableGraph, HashMap<String, NodeIndex>)>` — the graph and id_index are atomically co-swapped on rebuild. Readers hold an `Arc` to the old snapshot and never block.

  ## Alternatives considered

  - **RwLock<DiGraph>**: Simple but blocks all readers during rebuild (up to 200ms). Violates NFR-2 (lock-free lookups).
  - **RwLock<StableGraph>**: StableGraph preserves NodeIndex on removal but still blocks during rebuild.
  - **eventual consistency**: Version counter + lazy reader refresh. Complex, error-prone.

  ## Outcome

  **GOOD_CALL.** Zero reader blocking. Rebuild is invisible to queries. The ArcSwap pattern is now used for three components: graph, BM25 index, and vector registry.

  ## Source
  @wiki/tasks/awotvr
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}
