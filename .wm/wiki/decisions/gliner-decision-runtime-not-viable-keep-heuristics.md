---
title: 'Decision: gliner typed-decision runtime is non-viable as-is — keep heuristics'
type: decision
id: "wiki:decisions:gliner-decision-runtime-not-viable-keep-heuristics"
status: draft
tags: [decision, gliner-rs, typed-decisions, eval, non-viable]
relates_to:
  - {type: references, target: wiki:concepts:gliner-decision-quality-findings}
---

schema_version: 1
state: |-
  ## Context

  We adopted `gliner-rs` for the typed-decision runtime and measured it end-to-end (EV-01 + a deepwork experiment) on a fixed 30-page / 114-question fixture with a per-question majority baseline of **0.623**.

  ## Decision

  Declare the local gliner typed-decision runtime **non-viable as-is**. Keep deterministic heuristics; the `decision` feature stays **off by default**. Revisit only via **fine-tuning on WM labels**.

  ## Rationale

  No configuration beat the baseline. Best = `GLiNER2.5-Decide` + `key-facts` + `label-descriptions` = **0.491** (15/114 short). It beats the constant on `choice` (+0.024) but loses on `noul` (−0.238) and `score` (−0.200) — the calibration-heavy primitives. Calibration is poor (ECE 0.11–0.32); latency is impractical (7–118 s/row CPU); scale didn't help (6.5× params bought +0.017).

  ## Consequences

  The typed-decision doc format can stay (governed by heuristics), but the model runtime is not shipped on. The eval harness + fixture persist for future models. Fine-tuning is the only evidence-supported route (losses concentrate where labels bite).
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