---
title: 'Per-Wiki Markov Transition Foundation — First Consumer: Edge Typing'
type: spec
id: "wiki:specs:per-wiki-markov-transition-foundation"
status: draft
relates_to:
  - {type: references, target: wiki:concepts:typed-decision-application-map}
tags: [spec, markov, transitions, edge-typing, deterministic]
acceptance_criteria:
  - checked: false
    text: 'AC-1: for an observed `(source_type, previous_edge_type)` cell, the estimator returns the empirical argmax edge type (unit test).'
  - checked: false
    text: 'AC-2: for an unobserved cell (and for an empty graph), it reports no signal and the consumer keeps the `relates_to` default (test).'
  - checked: false
    text: 'AC-3: identical graph input yields identical predictions across repeated runs (determinism test).'
  - checked: false
    text: 'AC-4: the consumer response carries the evidence (cell + counts + predicted type) (test).'
  - checked: false
    text: 'AC-5: no MCP tool and no CLI command is registered for the estimator (test/assertion).'
  - checked: false
    text: 'AC-6: the memory write gate does not reference the estimator (test/assertion or code check).'
  - checked: false
    text: 'AC-7: an exact tie resolves to the canonical tie-break order, deterministically (test).'
---

schema_version: 1
state: |-
  ## Overview

  A deterministic, per-wiki **transition estimator**: given the wiki's own observed structure, it returns the empirical conditional distribution over next states, and a consumer acts on the **argmax**. No model, no training, no network, no persistence. This is the foundation the parked gate work left open (`@wiki/decisions:markov-chain-parked` scoped the gate park; this spec covers a *different* consumer).

  **First (and only) consumer:** graph **edge typing** — predicting an edge's type from the wiki's own edge history, replacing the current unconditional "body links default to `relates_to`" behaviour (`@wiki/concepts:typed-decision-application-map`, stage 2 = GO).

  ## Locked Decisions

  - **D1**: Deliver a reusable estimator **plus one named consumer** (edge typing). No second consumer; no framework ahead of need.
  - **D2**: The model conditions on **`P(edge_type | source_type, previous_edge_type)`** — sequence-aware, which is where a chain beats a page-type-pair lookup table.
  - **D3**: Counts are computed **on demand from the in-memory graph**. No derived artifact, no cache, no staleness layer (the corpus is tiny: ~833 edges).
  - **D4**: The estimator is an **internal module**; the consumer returns the **counts/evidence it used** so the decision is auditable. No new MCP tool or CLI command.

  ## Requirements

  ### Functional Requirements

  - **FR-1**: A module computes, from the in-memory graph, the observed counts for `(source_type, previous_edge_type) -> edge_type` over all existing edges.
  - **FR-2**: Given a context, it returns the **argmax** edge type over those counts (never a sample, never a probability threshold).
  - **FR-3**: Ties are broken deterministically by a named canonical edge-type order.
  - **FR-4**: When the context cell is **unobserved** (or the graph is empty), the estimator reports "no signal" and the consumer keeps the existing default behaviour (`relates_to`).
  - **FR-5**: The edge-typing consumer applies the prediction when a link lacks an authored type (the current `relates_to` default path).
  - **FR-6**: The consumer's response includes the **evidence**: the chosen cell, its counts, and the predicted type.
  - **FR-7**: The estimator is a private module — no MCP tool and no CLI command is registered for it.
  - **FR-8**: No magic values — the fallback edge type, tie-break order, and any threshold are named constants or config.

  ### Non-Functional Requirements

  - **NFR-1**: **Deterministic** — identical graph → identical prediction; reproducible (no sampling, no randomness).
  - **NFR-2**: Offline — no model, no training, no network.
  - **NFR-3**: **Isolation** — the estimator is never invoked from the memory write gate, and does not touch decision answers (`@wiki/specs:memory-write-decision-gate` stays memoryless).
  - **NFR-4**: Zero new compiler warnings; no `#[allow]`; no comments; no dead code; TDD (test first).
  - **NFR-5**: No new persistence or index plumbing.

  ## Acceptance Criteria

  - [ ] AC-1: for an observed `(source_type, previous_edge_type)` cell, the estimator returns the empirical argmax edge type (unit test).
  - [ ] AC-2: for an unobserved cell (and for an empty graph), it reports no signal and the consumer keeps the `relates_to` default (test).
  - [ ] AC-3: identical graph input yields identical predictions across repeated runs (determinism test).
  - [ ] AC-4: the consumer response carries the evidence (cell + counts + predicted type) (test).
  - [ ] AC-5: no MCP tool and no CLI command is registered for the estimator (test/assertion).
  - [ ] AC-6: the memory write gate does not reference the estimator (test/assertion or code check).
  - [ ] AC-7: an exact tie resolves to the canonical tie-break order, deterministically (test).

  ## Scenarios

  ### Scenario 1: Observed context
  **Given** a graph where `(concept, relates_to)` has most often been followed by `implements`
  **When** a body link is added to a concept whose last edge was `relates_to`
  **Then** the predicted type is `implements` and the response reports the counts.

  ### Scenario 2: Unobserved context
  **Given** no observed edges for the `(task, supersedes)` cell
  **When** a link is added in that context
  **Then** the estimator reports no signal and the link keeps the `relates_to` default.

  ### Scenario 3: Tie
  **Given** two edge types with equal counts for the cell
  **When** the estimator picks one
  **Then** the canonical tie-break order decides, identically every run.

  ### Scenario 4: Empty graph
  **Given** a fresh project with no edges
  **When** edge typing is consulted
  **Then** there is no signal and behaviour is unchanged from today.

  ## Technical Notes

  State space: 11 page types × the canonical edge-type set; transitions read from the in-memory petgraph (and, if needed, `@wiki/concepts/edge-types` for the canonical order). Consumption is counts + argmax — no probabilities are surfaced to callers.

  ## Open Questions

  - [ ] Does a page's edge list preserve a meaningful "previous edge" order across reload/rebuild? If not, define the ordering key (e.g. authored order, then id).
  - [ ] Is `(source_type, previous_edge_type)` sufficient, or does the target's type change the argmax materially? (Measure before widening.)
  - [ ] Should the prediction only ever *replace the untyped default*, or may it override an authored type? (Current scope: default path only.)
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}