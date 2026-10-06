---
title: 'Pattern: Evaluate a model against a per-primitive majority baseline'
type: pattern
id: "wiki:patterns:eval-model-against-per-primitive-majority-baseline"
status: draft
tags: [pattern, eval, baseline, uncertainty, model-selection]
relates_to:
  - {type: references, target: wiki:concepts:gliner-decision-quality-findings}
---

schema_version: 1
state: |-
  ## Problem

  Deciding whether a candidate model is actually useful for a task — without being fooled by tiny gains, untrained prompt hacks, or an expensive sweep that answers the wrong question.

  ## Solution

  - Fix a **labeled fixture** and compute a **per-primitive majority baseline** — aggregates hide where you lose (here: wins `choice`, collapses on `noul`/`score`).
  - Use **Wilson confidence intervals**; treat spreads within noise (≈2–5 labels at n=114) as **no effect** — don't over-claim.
  - Feed the model through its **trained channels** (e.g. GLiNER `label_descriptions` / `examples`), not ad-hoc prompt text; the trained channel is the real lever.
  - Separate **latency levers from accuracy levers** (input serialization was a latency lever, not accuracy).
  - Test the **purpose-built** model as an endpoint before large matrices; run **staged, cheap-first**.
  - Persist **per-config artifacts** (JSON per run), not just stdout.

  ## When to Use

  Evaluating any candidate model/encoder for a decision/classification task.

  ## When Not to Use

  Trivial yes/no checks with an obvious metric.

  ## Evidence

  gliner deepwork: serialization differences within noise; `label_descriptions` +0.044; `Decide` +0.017 at ~9.6× latency; **no config beat the 0.623 baseline**.
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}