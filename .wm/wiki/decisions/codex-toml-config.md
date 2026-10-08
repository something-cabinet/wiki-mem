---
title: Codex TOML Config
type: decision
id: "wiki:decisions:codex-toml-config"
status: draft
---

schema_version: 1
state: |-
  id: wiki:decisions:codex-toml-config

  ## Context

  The `"claude" | "codex"` match arm treated both platforms identically, generating `.mcp.json` with JSON `mcpServers`. Codex expects `.codex/config.toml` with TOML `[mcp_servers]` format.

  ## Chosen approach

  Split the arm. Codex generates TOML, Claude generates JSON.

  ## Outcome

  Both platforms get correct config. TOML writer is a simple format!() call — no dependency needed.

  ## Source

  @wiki/tasks/wkm5xh
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