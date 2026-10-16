---
title: Markov Chains in the Memory Write Gate — Parked Spec
type: spec
id: "wiki:specs:markov-chain-parked"
status: draft
tags: [spec, markov, typed-decisions, parked, deferred]
relates_to:
  - {type: references, target: wiki:specs:memory-write-decision-gate}
---

schema_version: 1
state: |-
  ## Overview

  Specification for applying a **Markov chain to the memory write gate**. **PARKED for the gate**: not scheduled, not implemented until the re-open conditions below are met. Recorded in SDD so the idea is tracked without leaking into the gate or its PR.

  **Scope note:** this park covers the **gate only**. It does **not** park Markov chains for the decision layer at large, search / ranking, retention, or graph edge typing — those are separate, undecided questions.

  Companion rationale and measurements: `@wiki/decisions:markov-chain-parked`.

  ## Locked Decisions

  - **D1**: No transition model in the memory write gate or its spec, and no instrumentation of the gate "for later" — that would be a compensating layer (`@wiki/rules:no-compensating-layers`).
  - **D2**: A decision-transition chain **for the gate** is **PARKED** — not evaluable before the gate is merged and answers accumulate; a re-open additionally requires an override-telemetry design.
  - **D3 (scope boundary)**: Markov chains for search / ranking, retention / lifecycle, the wider decision layer, and graph edge typing are **out of scope of this park and undecided**. (Edge typing is stage 2 and already GO with plain deterministic rules.)

  ## Requirements

  Deferred — none are active while parked.

  - **R1 (deferred)**: any gate transition model must be estimated **per-wiki at runtime** from observed transitions (derived data, like the BM25 index — no pre-trained artifact).
  - **R2 (deferred)**: evaluation must use observed **counts + argmax** (deterministic), never sampling (reproducibility).
  - **R3 (prerequisite, undesigned)**: an **override-telemetry** design capturing human overrides of gate decisions.
  - **R4**: no magic constants — any threshold named and config-overridable.

  ## Acceptance Criteria

  These gate **re-opening the gate question**, not implementation.

  - [ ] AC-1: the memory write gate is merged and decision-answer history has accumulated organically (baseline at park time: **0/151** memory pages carried answers).
  - [ ] AC-2: an override-telemetry spec exists and captures override signal.
  - [ ] AC-3: a read-only spike shows argmax-per-row beats the per-row unconditional majority (the falsification test); if it does not, re-park.
  - [ ] AC-4: no Markov content ever enters `@wiki/specs:memory-write-decision-gate` or its PR.

  ## Scenarios

  ### Scenario 1: Gate re-open
  **Given** the gate is merged and answers have accumulated
  **When** override telemetry exists
  **Then** run the spike; proceed only if AC-3 passes, otherwise re-park with the numbers recorded.

  ### Scenario 2: Premature proposal for the gate
  **Given** a proposal to add transition modelling or capture to the gate
  **When** AC-1/AC-2 are unmet
  **Then** reject it, citing D1/D2.

  ### Scenario 3: Proposal elsewhere
  **Given** a proposal to apply a Markov chain to search, retention, or edge typing
  **Then** this park does not apply — evaluate it on its own terms (see D3).

  ## Technical Notes

  Measured at park time (2026-10-16): **294** recorded status transitions, top five ≈ **68%** of the mass; **0/151** memory pages with a non-empty `answers` map (6/900 pages across all types). See `@wiki/decisions:markov-chain-parked`.

  ## Open Questions

  - [ ] Where would override telemetry live (`.wm/versions`, a dedicated telemetry file) and who owns that design?
  - [ ] Does the argmax rule collapse to a lookup table a plain heuristic could express — i.e. is any chain needed at all?
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