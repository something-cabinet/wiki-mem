---
title: 'Failure: ONNX Model token_type_ids Input + Max Sequence Truncation'
type: concept
id: wiki:concepts:onnx-token-type-ids-truncation
relates_to:
  - {type: references, target: wiki:specs:onnx-embedding-integration}
---

schema_version: 1
state: |-
  id: wiki:concepts:onnx-token-type-ids-truncation

  ---
  id: wiki:concepts:onnx-token-type-ids-truncation
  title: Failure: ONNX Model token_type_ids Input + Max Sequence Truncation
  type: concept
  tags: [failure, onnx, embedding, model]
  ---
  id: wiki:concepts:onnx-token-type-ids-truncation

  ## What went wrong

  The ONNX embedding model (bge-small-en-v1.5) silently failed during `wm index rebuild` with two separate errors:

  1. **Missing Input: token_type_ids** — The model expected a third input tensor that the embedder never provided.
  2. **Broadcast error on Add_1** — Input sequences exceeded the model's max position embeddings (512 tokens).

  ## Root cause

  1. The ONNX-exported BERT model included `token_type_ids` as a required input, but the embedder only provided `input_ids` and `attention_mask`.
  2. The tokenizer had no truncation configured, so sequences >512 tokens produced position IDs beyond the model's learned embedding table.

  ## How it was fixed

  Two changes in `packages/wm-embed/src/services/onnx/mod.rs`:

  1. Added a zero `token_type_ids` tensor of the same shape as `attention_mask`
  2. Configured tokenizer truncation to 512 (BERT's max position embeddings)

  ## Prevention

  When adding an ONNX model for embedding:
  - Inspect the model's input signature for all required inputs
  - Always set tokenizer truncation to the model's max position embeddings (typically 512 for BERT)

  ## Time lost

  ~30 minutes debugging opaque ONNX Runtime errors.

  ## Related
  - @wiki/specs/onnx-embedding-integration
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
