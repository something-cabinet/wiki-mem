---
title: 'Decision: Markov chains are parked for the memory write gate'
type: decision
id: "wiki:decisions:markov-chain-parked"
status: approved
tags: [decision, markov, typed-decisions, parked]
relates_to:
  - {type: references, target: wiki:specs:memory-write-decision-gate}
---

schema_version: 1
state: |-
  ## Context

  A Markov chain was proposed **for the memory write gate** — modelling transitions in order to strengthen the gate's four answers. The wiki is memoryful: `.wm/versions/*.json` record field-level changes and typed edges are stored, so structural transitions are estimable. Measured on 2026-10-16: 294 recorded status transitions (top five ≈ 68% of mass), and **0 of 151** memory pages carry a non-empty `answers` map (6/900 across all types).

  ## Decision

  Park the Markov chain **for the memory write gate only**. The gate stays memoryless: no transition model enters it or its spec (`@wiki/specs:memory-write-decision-gate`), and nothing Markov-shaped — including 'temporary' instrumentation of the gate's response to capture data 'for later' — is added.

  This park does **not** decide Markov chains for anything else. Explicitly **out of scope and undecided** here:

  - the decision layer at large (typed-decision answering beyond the gate);
  - search / ranking / retrieval;
  - retention / lifecycle modelling;
  - graph edge typing (stage 2 remains its existing GO with plain deterministic rules; whether a chain ever applies there is its own future question).

  ## Why it does not belong in the gate

  - **No data for the gate's own decisions.** Decision-answer history is empty (0/151), so a gate chain would model noise.
  - **The self-distillation trap.** Once merged, accumulated `answers` are gate-generated, not ground truth; learning transitions from them teaches a model to imitate the heuristic. A re-open therefore needs *override* signal, which has no capture design.
  - **Determinism.** A runtime sampler breaks reproducibility (NFR-1); a counts-plus-argmax rule collapses to a lookup table a plain heuristic already expresses — a compensating layer (`@wiki/rules:no-compensating-layers`).
  - **Scope.** The gate is memoryless by design: its four answers are functions of `(title, content, layer)` plus the corpus, with no sequence to condition on.

  ## Consequences

  The gate ships clean and Markov-free. If ever revisited **for the gate**, the sequence is: gate merge -> organic answer accumulation -> an optional override-telemetry spec -> a decision-transition spike that must beat a per-row unconditional majority (argmax per row vs the constant).

  ## Related
  - @wiki/specs/memory-write-decision-gate
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