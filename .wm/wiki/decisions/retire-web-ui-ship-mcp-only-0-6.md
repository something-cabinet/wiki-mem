---
title: 'Decision: Ship 0.6 as MCP + CLI only (retire web UI)'
type: decision
id: "wiki:decisions:retire-web-ui-ship-mcp-only-0-6"
status: draft
tags: [decision, '0.6', mcp-only, retirement, wm-web, wm-server]
relates_to:
  - {type: references, target: wiki:tasks:rw-01-retire-wm-web--wm-server--wm-web}
---

schema_version: 1
state: |-
  ## Context

  WM carried a browser surface: the `wm-web` Angular SPA, the `wm-server` Axum daemon, an HTTP MCP transport, and four browser-only WASM crates. It added build/CI/packaging complexity and a second deployment mode, while real usage ran through MCP + CLI.

  ## Decision

  Ship **0.6 as MCP + CLI only**. Delete `wm-web`, `wm-server`, the `wm web` command, the HTTP MCP transport, the browser-only WASM crates (`fjadra`, `graph-algo`, `bm25-rerank`, `md-parse`), `wm-mock-server`, and the conformance script. Keep MCP as **in-process stdio** (`wm-cli mcp`).

  ## Rationale

  MCP was already in-process — no daemon dependency (`wm-cli` depends only on `wm-core`); nothing compiled against `wm-server` or the WASM crates; the daemon existed only to serve the SPA. So removal was a clean deletion, not a refactor.

  ## Consequences

  Simpler workspace, CI, and packaging; docs (`README`, `ARCHITECTURE`, `CONVENTIONS`, `SECURITY`) updated and stale stdio→HTTP-proxy claims removed. Historical wiki pages still reference the web stack (allowed).
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