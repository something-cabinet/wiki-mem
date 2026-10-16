---
title: 'Decision: Scope is provenance, not page data'
type: decision
id: "wiki:decisions:scope-is-provenance-not-page-data"
status: approved
tags: [decision, architecture, global, identity]
relates_to:
  - {type: implements, target: wiki:specs:global-wiki-layer}
---

schema_version: 1
state: |-
  ## Context

  The memory layer encoded cross-project location as an enum variant on a single page type's data model — `MemoryLayer::Global` (`packages/wm-engine/src/models/memory/layer_model.rs:4-12`) wired onto `MemoryData` (`memory_data_model.rs:6-14`). It was never persisted (0 of 134 memory pages carry a `layer:` key), never read (`List` branched only on `is_session`, `memory.rs:220`), and its sole writer `promote` was a write-only dead end.

  ## Decision

  Model scope as **provenance** — which root supplied the bytes — rather than as page data. Identity stays `(type, slug)`; there is no frontmatter `scope:` field and no `wiki:global:*` id namespacing.

  ## Rationale

  Three desirable properties follow from the single choice:

  - **Zero migration** — existing page ids, `@wiki/...` refs and graph edges keep resolving unchanged.
  - **Promotion is pure relocation** — every `relates_to`/`extends`/`supersedes` edge survives, whereas namespacing ids would churn every edge on every promotion.
  - **Collisions become shadowing, not duplication** — a false-confidence duplicate pair becomes one visible shadow report.

  Encoding location in page data was the original category error: a global memory page is simply a `type: memory` page in the global root. The multi-root design is the proof of that.

  ## Consequences

  Any "which root did this come from" question is answered by the composer, never by reading a page field. `MemoryLayer::Global` is deleted as wrong-typed rather than repurposed. The same reasoning applies to the relocatable state root, which re-points a scope rather than annotating pages.

  ## Related

  - @wiki/specs/global-wiki-layer
  - @wiki/specs/remove-global-memory-layer
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