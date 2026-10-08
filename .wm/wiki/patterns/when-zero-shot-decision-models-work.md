---
title: 'Pattern: When zero-shot decision/classification models work (and when they don''t)'
type: pattern
id: "wiki:patterns:when-zero-shot-decision-models-work"
status: draft
tags: [pattern, zero-shot, decision-models, classification, eval, gliner-rs]
relates_to:
  - {type: references, target: wiki:concepts:gliner-decision-quality-findings}
---

schema_version: 1
state: |-
  ## Problem

  When is a **local, zero-shot** (no training) decision/classification model actually useful — and when does it fail, as ours did (0.491 vs a 0.623 majority baseline on subjective wiki decisions)?

  ## When local zero-shot works

  - **Well-posed, single-label, distinct-label prose** classification: `GLiNER2` (205M) ~**0.72** avg on public sets; `GLiClass` ~**0.70–0.74** macro-F1; `Laurer zeroshot-v2.0` ~0.676.
  - **Task kind dominates even for strong models**: sentiment ~0.88–0.9 · topic/intent ~0.4–0.55 · **subjective/emotion ~0.25–0.35**.
  - **Architecture > size**: models *trained for semantic matching* (NLI cross-encoders, rerankers, GLiClass uni-encoder) work; raw base encoders / embeddings do **not** (all-MiniLM ~0.37).
  - **Single-boolean-per-item framing** beats multi-option choice (the jevgrep shape).
  - **Code**: zero-shot **retrieval** is strong (jina-code-0.5b ~78.7 MTEB-Code); zero-shot **classification** is weak.

  ## When it fails

  - **Structured "typed decisions"** where the label is jointly determined by fields **and** options → base local models sit **at or below majority** (`Laya` 0.362 vs 0.461; `verdict` 0.32–0.39). Bi-encoders score each option against one vector of the state "by design".
  - **Subjective, cross-domain labels** with no labeled calibration set → near-majority.
  - **High label cardinality** (>~50) collapses accuracy.

  ## What fixes it (non-frontier)

  1. A **decision specialist** — `Julia-1` (144M, CPU, Apache-2.0) claims **73.15%** typed-decisions; specialists like `Kev`/`Lumma-Fev` similar.
  2. **Per-task fine-tuning/tuning** — `Laya` 0.36 → 0.77; `verdict` 0.42 → 0.86 (fitted).
  3. **Retrieval + per-candidate boolean** with a small local reranker (the jevgrep shape, without a frontier model).

  ## Zero-shot vs "running on our data"

  Zero-shot describes the **model**, not the input. Running a model over `.wm/` wiki content is still zero-shot **inference** — the page text is the *input*, not training. It stops being zero-shot only with **(a)** fine-tuning on labeled `(page, answers)` pairs, or **(b)** labeled examples supplied in-context (few-shot). So "it saw our wiki" does not make it adapted — which is why data alone can't fix subjective label semantics.

  ## Evidence

  Our run: `GLiNER2.5-small/Decide` best **0.491** vs majority **0.623**; serialization/`label_descriptions` were latency/limited levers. See `@doc/concepts/gliner-decision-quality-findings`.

  ## Related

  - @doc/concepts/decision-models-cannot-generalize-across-user-wikis
  - @doc/concepts/gliner-decision-quality-findings
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