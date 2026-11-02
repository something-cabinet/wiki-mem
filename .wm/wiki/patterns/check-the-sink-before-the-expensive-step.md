---
title: 'Pattern: check the sink before the expensive step'
type: pattern
id: "wiki:patterns:check-the-sink-before-the-expensive-step"
status: reviewed
tags: [pattern, tool-reliability, persistence]
relates_to:
  - {type: extends, target: wiki:patterns:verify-writes-by-read-back}
  - {type: references, target: wiki:tasks:index-rebuild-reports-success-when-vectors-cannot-persist}
---

schema_version: 1
state: |-
  ## Problem

  An expensive step runs to completion and only then discovers that its result cannot be saved. In this repo, wm-cli index rebuild embedded 659 sections (38 s wall, 294 s CPU), failed to write vectors.db because another process held the lock, logged a warning and reported Rebuild complete with exit 0.

  ## Solution

  - Keep the reason a resource is unavailable. VectorStore::new used .ok() on the open error, so the store became db: None with no cause.
  - Expose a cheap check, such as ensure_persistable, and call it before the expensive step. Fail fast with the cause.
  - Still treat a failed write after the step as an error. Never log it and return success.
  - Test both guards with a counting stand-in for the expensive component: assert it was called zero times when the sink is unavailable.

  ## When to Use

  Any step that is slow or costly and ends in a write: embedding, index builds, exports, migrations.

  ## When Not to Use

  Cheap steps where retrying costs nothing. The check itself must be cheap and must not take a lock that the real write then cannot get.

  ## Related

  - wiki:patterns:verify-writes-by-read-back covers what to do after the write.
  - wiki:tasks:index-rebuild-reports-success-when-vectors-cannot-persist
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
answers:
  problem_kind: error-handling
  preconditions_required: false
  complexity: simple
  language_specific: false