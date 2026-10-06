---
title: 'Concept: Decision models cannot generalize across user wikis'
type: concept
id: "wiki:concepts:decision-models-cannot-generalize-across-user-wikis"
status: draft
tags: [decision-models, gliner-rs, generalization, product, typed-decisions]
relates_to:
  - {type: references, target: wiki:concepts:gliner-decision-quality-findings}
---

schema_version: 1
state: |-
  ## Problem

  A local decision model (`choice`/`score`/`noul` over a wiki page) is evaluated on **WM's own** docs — but WM is a tool others run on **their own** wikis, with their own vocabulary, conventions, and decisions. The model's usefulness therefore has to hold for arbitrary user domains, not just ours.

  ## Why it cannot serve the product

  - **Zero-shot is too weak.** Even on our own docs, no configuration beats a per-question majority constant (best 0.491 vs 0.623).
  - **Fine-tuning only helps our domain.** A preset trained on WM labels is specialized to WM's vocabulary; for any user's wiki it is still zero-shot and mismatched — the same failure, relocated.
  - **Per-user training is infeasible.** End users will not (and should not) train; no labelled corpus exists in a user's wiki; the model artifact cannot be version-controlled; the training stack is heavy.
  - **In-context few-shot needs labels the user doesn't have.** It also only helped marginally where tested (0.447 vs 0.430 raw).

  ## Consequence

  A local decision model cannot be a shipped default for arbitrary users. The **agent's own reasoning + deterministic heuristics** — which use the user's live context adaptively, with no training — remain the right approach. Keep the `decision` feature **off**.

  ## Why this generalizes

  Any approach that needs a model specialised to the user's domain fails the same way unless the user trains. Only two paths survive: (a) genuinely domain-general zero-shot (not met), or (b) the agent reading the user's context at query time (status quo).

  ## Related

  - @doc/decisions/gliner-decision-runtime-not-viable-keep-heuristics
  - @doc/concepts/gliner-decision-quality-findings
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}