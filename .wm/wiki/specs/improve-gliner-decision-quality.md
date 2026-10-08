---
title: Improve gliner Typed-Decision Quality (State Format + Prompting + Model)
type: spec
id: "wiki:specs:improve-gliner-decision-quality"
status: draft
tags: [gliner-rs, typed-decisions, experiment, eval, serialization, calibration]
---

schema_version: 1
state: |-
  ## Overview

  `gliner2.5-small-v1` scored **0.404** overall vs a **0.623** trivial majority baseline (27 s/row CPU) on the WM typed-decision eval — **not useful as shipped**. This spec runs a scoped experiment to find a configuration that beats the baseline, or concludes that gliner is not viable and documents the fallback. Factors: **state serialization**, **question phrasing**, and **model variant**.

  ## Locked Decisions

  - **D1**: Local, offline, no hosted API, no training in the first pass.
  - **D2**: Baseline = per-question majority (current fixture = 0.623).
  - **D3**: Experimental factors:
    1. **State serialization** — `prose` (current) vs `compact-json` vs `key-facts` bullets.
    2. **Question phrasing** — raw `instructions` vs `label_descriptions` vs few-shot examples.
    3. **Model variant** — `small` (74M) vs `base` (194M) vs `multi` (205M) vs `GLiNER2.5-Decide` (486M).
  - **D4**: Success = beat baseline materially (target ≥ 0.70 overall accuracy) without unacceptable latency.
  - **D5**: Evaluate with the existing fixture + extended harness; keep the fixture fixed for comparability.
  - **D6**: Fine-tuning is a **last resort** (training was out of the 0.6 scope); revisit only if no configuration wins.
  - **D7**: CPU latency is the binding constraint — scope runs to a subset where full-matrix cost is prohibitive.

  ## Requirements

  ### Functional
  - **FR-1**: Extend the eval harness with serialization + phrasing modes and model-variant selection, emitting per-primitive accuracy/macro-F1/ECE/latency.
  - **FR-2**: Run a scoped matrix that answers, in priority order: (a) does serialization move the needle? (b) does phrasing/label_descriptions? (c) does model size/Decide?
  - **FR-3**: Report each configuration's metrics vs the baseline and a **verdict** (viable / not viable + best config).
  - **FR-4**: Document the reproducible recipe (feature flag, model download, runner).

  ### Non-Functional
  - **NFR-1**: Offline, deterministic; fixed fixture.
  - **NFR-2**: Zero warnings; rules honored (no comments/else/magic).

  ## Acceptance Criteria
  - [ ] AC-1: Harness supports `serialization ∈ {prose, compact-json, key-facts}` and phrasing modes.
  - [ ] AC-2: Serialization comparison run (small model, subset) with results.
  - [ ] AC-3: Model-variant comparison run (best serialization, subset) with results, or a documented compute blocker.
  - [ ] AC-4: Report doc with the metrics table, baseline comparison, best config, and an explicit viability verdict.

  ## Scenarios
  ### Scenario 1: A config beats the baseline
  **Given** the matrix runs
  **When** a config reaches ≥ 0.70 overall
  **Then** it is recommended as the runtime default (serialization + model + phrasing).

  ### Scenario 2: Nothing beats the baseline
  **Given** all tried configs stay ≤ baseline
  **When** the matrix completes
  **Then** the report states gliner is not viable for WM decisions and proposes the fallback (keep heuristics / consider fine-tuning).

  ## Technical Notes
  - `state` is already prose today (block scalar of the original body); "YAML" is only the record *wrapper*, not the model input — so serialization tests must feed the *content* in each format.
  - GLiNER2 supports `label_descriptions` and few-shot examples; test them for the choice/score/noul questions.
  - Full CPU runs are slow (~25 s/row for small); use small subsets and record n per config.
  - The `GLiNER2.5-Decide` checkpoint is not auto-downloaded by `--model-variant`; supply it explicitly.

  ## Open Questions
  - [ ] Is accuracy driven more by model capacity or by question design?
  - [ ] Does `compact-json` help field-style state but hurt narrative state?
  - [ ] At what n does ECE become meaningful?
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
