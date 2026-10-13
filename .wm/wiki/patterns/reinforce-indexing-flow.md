---
title: 'Pattern: Reinforce a derived-state indexing flow'
type: pattern
id: "wiki:patterns:reinforce-indexing-flow"
status: draft
tags: [pattern, indexing, retrieval, freshness, search-quality]
relates_to:
  - {type: implements, target: wiki:specs:reinforce-indexing-flow}
---

schema_version: 1
state: |-
  ## Problem

  A retrieval pipeline backed by derived state (index) had quality/robustness gaps: stale graph until an explicit rebuild, structured frontmatter invisible to search, silent semantic degradation, and cross-model vectors served.

  ## Solution

  - **Freshness**: one `refresh_all_if_stale(engine)` rebuilds graph + sections + BM25 (and optionally embeddings); never clear the stale flag after a *partial* heal. Call it from search, validate, and graph tools.
  - **Index structured frontmatter values** (not keys): spec FR/NFR/goals, task acceptance criteria, decision context/options/rationale, `aliases`.
  - **Embedding integrity**: validate the persisted model/version against the configured model on load; **clear on mismatch**; surface an actionable `degraded` reason in query/retrieve/status.
  - **Incremental** single-page graph update instead of an O(N) rebuild per write.
  - **Golden eval** de-ignored with hybrid/semantic + structured-frontmatter cases.

  ## When to Use

  Any search/indexing pipeline over derived state with incrementality and a swappable model.

  ## Evidence

  `wm_validate` all-scope 0 errors; golden eval recall@5 = 1.0 over 52 queries (hybrid `degraded=true` without a model).

  ## Related

  - @doc/specs/reinforce-indexing-flow
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