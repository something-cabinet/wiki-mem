---
id: wiki:decisions:uniform-mcp-schema-structs
title: "Decision: Uniform Schema Structs for MCP Tool Actions"
type: decision
status: approved
tags: [decision, mcp, schema, uniformity]
decision:
  context: "Action enums mix two patterns: inline variant fields (for handler-used params) and #[allow(dead_code)] (for schema-only fields). Every new variant requires a judgment call."
  options:
    - "Current mixed pattern (inline + #[allow])"
    - "Uniform schema structs for every variant"
  rationale: "Uniformity eliminates judgment calls. Every variant with parameters gets a schema struct. New devs never wonder which pattern to use. Zero #[allow(dead_code)]."
  outcome: "Every MCP tool action variant with parameters gets a dedicated schema struct named Wm{Domain}{Variant}Schema."
---

schema_version: 1
state: |-
  id: wiki:decisions:uniform-mcp-schema-structs
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
