---
title: 'Eval: Measure gliner-rs Typed-Decision Runtime'
type: spec
id: "wiki:specs:eval-gliner-typed-decisions"
status: draft
tags: [eval, gliner-rs, typed-decisions, measurement, calibration]
---

schema_version: 1
state: |-
  ## Overview

  End-to-end **measurement** of the `gliner2.5-small-v1` typed-decision runtime on real WM pages: accuracy, ECE, and latency per primitive and per page type. Research/report — the only new code is a **dev eval harness**; no product behaviour change.

  ## Locked Decisions

  - **D1**: Local, offline, no training (per `@doc/specs/typed-decision-doc-format`).
  - **D2**: Evaluate the real `gliner2.5-small-v1` via the `decision` cargo feature.
  - **D3**: Ground truth = a **hand-labeled fixture** (~30 pages spanning decision/pattern/concept/howto/reference), authored by reading real pages.
  - **D4**: Report accuracy, macro-F1 (choice), **ECE** (noul/score probabilities), and latency — per primitive and overall; broken down by page type.
  - **D5**: Include a trivial baseline (majority/constant) so lift is visible.
  - **D6**: If the model cannot be downloaded/loaded, deliver the harness with a Mock run and document the blocker precisely.

  ## Requirements

  ### Functional
  - **FR-1**: A committed labeled fixture (~30 `{state, questions, expected answers}` rows) spanning all five types and all three primitives.
  - **FR-2**: A harness that runs `DecisionRuntime` over the fixture with the real `GlinerBackend` and computes per-primitive + overall accuracy, macro-F1 (choice), ECE (noul/score), and latency.
  - **FR-3**: A report doc with the numbers, the baseline comparison, and an explicit verdict on whether gliner is useful for WM decisions.
  - **FR-4**: Document the run recipe (`--features decision`, model download) for reproducibility.

  ### Non-Functional
  - **NFR-1**: Offline, deterministic given the model.
  - **NFR-2**: No training; no hosted API.
  - **NFR-3**: Zero warnings; rules honored (no comments/else/magic).

  ## Acceptance Criteria
  - [ ] AC-1: Fixture exists with ~30 labeled rows across all types/primitives.
  - [ ] AC-2: Harness computes accuracy / macro-F1 / ECE / latency per primitive + overall.
  - [ ] AC-3: The real model ran (or a precise blocker is documented) and results are recorded.
  - [ ] AC-4: Report doc includes numbers + a clear verdict, linked to this spec.

  ## Scenarios
  ### Scenario 1: Model available
  **Given** the model downloaded
  **When** the harness runs
  **Then** it prints per-primitive metrics and writes the report.

  ### Scenario 2: Offline / no model
  **Given** no network or model
  **When** the harness runs
  **Then** it still runs with a Mock backend to validate the harness, and the blocker is documented.

  ## Technical Notes
  - Build: `cargo build -p wm-cli --features decision`; model via `wm model download gliner2.5-small-v1` (checksum-pinned manifest).
  - `score`/`noul` are classification-mapped (no ordinal head) — ECE is the key calibration signal.
  - Keep the fixture honest: prefer objective questions (e.g. decision `outcome`) where the page states the answer; mark subjective ones as such.

  ## Open Questions
  - [ ] Is 30 enough for stable ECE, or do we need a larger n?
  - [ ] Which questions are objectively labelable vs subjective?
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
