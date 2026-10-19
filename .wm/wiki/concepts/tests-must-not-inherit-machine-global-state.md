---
title: Tests must not inherit machine-global state
type: concept
id: "wiki:concepts:tests-must-not-inherit-machine-global-state"
status: draft
relates_to:
  - {type: references, target: wiki:tasks:security-test-deletes-the-developers-installed-model}
---

schema_version: 1
state: |-
  ## What went wrong

  Three failures in one session had the same shape: the result depended on whether the developer happened to have the ONNX model installed in `$HOME/.wm/models`.

  - `cli_test` went from 1.1s to over 120s and started failing, because every spawned `wm-cli` eagerly loaded a 133 MB model.
  - `semantic_degradation_without_model` asserted that no model was available, and only passed because another test had deleted the model first.
  - `golden_query_recall_at_5` built its index with `skip_embed: true` but branched on `embedder.is_loaded()`, so a present model sent it into a semantic assertion against an empty vector store.

  Worse: `security_test` called `wm_model remove bge-small-en-v1.5`, and the tool resolved its cache from ambient `$HOME`, so running the suite deleted a real 134 MB asset from the developer's machine. Because test binaries run concurrently, the ONNX tests then passed or failed depending on which binary ran first.

  ## Root cause

  Tooling resolved machine-global state at the point of use instead of taking it as an input. Tests inherited that ambient state, so the suite was not a function of the repository alone.

  ## Prevention

  - A test must not read or write outside its own temporary directory. If a tool resolves a machine-global path, give it an explicit input (a parameter, a config field) so tests can point it at a fixture.
  - Asserting an absence - degradation, no model, empty index - requires constructing that absence, not hoping for it.
  - A destructive operation must prove its target. A test that deletes an asset deletes a fixture, never a real install.
  - A branch on "is the model loaded" must match the branch used to build the index, otherwise the precondition and the assertion disagree.

  ## Time lost

  About three gate runs, plus two false regressions attributed to the ES-04 migration before the cause was found.

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