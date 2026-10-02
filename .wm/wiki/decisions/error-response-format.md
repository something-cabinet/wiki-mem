---
title: error response format
id: wiki:decisions:error-response-format
type: decision
relates_to:
  - {type: implements, target: wiki:patterns:mcp-response-format}
---

schema_version: 1
state: |-
  id: wiki:decisions:error-response-format

  ## Context

  `ToolError::to_json()` was wrapping error details in `{"error": {"code": ..., "message": ...}}`. The MCP transport layer (`JsonRpcResponse::error`) then wrapped this AGAIN in the JSON-RPC error envelope, producing `{"error": {"error": {"code": ..., "message": ...}}}`.

  ## Chosen approach

  `ToolError::to_json()` now returns the error object directly: `{"code": "REQUIRED_FIELD", "message": "id is required"}`. The transport layer adds the outer `"error"` wrapper.

  ## Why not the other way

  Alternative was to keep the double-wrap and change the transport to strip the outer wrapper. But the simpler fix is to make `to_json()` return what the transport expects — a plain code+message object.

  ## Outcome

  MCP clients now receive correctly formatted JSON-RPC error responses. MCP E2E tests validate this format.

  ## Source

  @wiki/tasks/s2ff4x
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}
