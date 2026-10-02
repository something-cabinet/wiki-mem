---
title: 'Decision: Adopt gliner-rs as the typed-decision runtime'
type: decision
id: "wiki:decisions:adopt-gliner-rs-as-typed-decision-runtime"
status: draft
tags: [decision, typed-decisions, gliner-rs, runtime, system-one]
relates_to:
  - {type: references, target: wiki:reference:typed-decision-record-schema}
---

schema_version: 1
state: |-
  ## Context

  WM needed a local, free, CPU-only decision model to answer typed questions over wiki docs on ~8 GB RAM, no-GPU machines. Research found **no sub-200M System One model ships a Rust crate**: the Rust-native `laya` crate serves ≥322M models whose CPU peak (~2.4 GB) is too heavy, and `gliner-rs` (GLiNER2.5) is Rust-native but choice-only.

  ## Decision

  Adopt **`gliner-rs` + `GLiNER2.5-small-v1`** as the runtime, and map all three primitives onto its label-probability API: `choice` = N-label softmax; `noul` = two-label `false`/`true`; `score` = single-label classification over ordered level labels (no ordinal head).

  ## Rationale

  Rust-native (candle), permissive (Apache-2.0), small (282 MiB), returns a full per-label distribution, and ships a Jev-format `choice`/`score`/`noul` script. No training is required, so end users never train.

  ## Consequences

  `score` is emulated (no expected-value head); `noul` probabilities are only checkpoint-temperature calibrated and must be measured on our data. Laya remains an optional high-spec tier. The runtime is feature-gated (`decision`, default off) and distributed as a checksum-pinned download.
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