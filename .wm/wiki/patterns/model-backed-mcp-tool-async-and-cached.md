---
title: 'Pattern: Model-backed MCP tools must be async and cache the backend'
type: pattern
id: "wiki:patterns:model-backed-mcp-tool-async-and-cached"
status: draft
tags: [pattern, mcp, async, caching, model-runtime]
relates_to:
  - {type: references, target: wiki:tasks:td-13-fix-wmdecisionanswer-blocking--model-reload}
---

schema_version: 1
state: |-
  ## Problem

  A model-backed MCP tool that (a) runs blocking fs/HTTP/model-load on the runtime thread and (b) reloads the model on every call will stall the server and waste hundreds of MB per request. A backend that fabricates a default answer when a result is missing produces silent wrong answers.

  ## Solution

  - Register the tool **async** (`register_typed_async`) and run download/load/inference inside `tokio::task::spawn_blocking`.
  - **Load the model once** behind a keyed cache (model dir + manifest revision); reuse it across calls.
  - **Stream-hash** downloads (buffered read + `sha2`) instead of buffering the whole payload.
  - **Error** when the backend omits a required task/label result — never synthesize `labels[0]` with probability 0.
  - Enforce the tool's input format on **create** (not just validate).

  ## When to Use

  Any MCP tool that loads models, does heavy/blocking work, or calls a backend that can return partial results.

  ## When Not to Use

  Pure in-memory tools with no external calls.

  ## Evidence

  P1 fixes in TD-13/14/15: `wm_decision.answer` made async + cached, download stream-hashed, missing task now errors, record format enforced on create/validate.
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