---
title: 'Pattern: Safe rustdoc-strip sweep'
type: pattern
id: "wiki:patterns:safe-rustdoc-strip-sweep"
status: draft
tags: [pattern, comments, sweep, rust, refactor]
relates_to:
  - {type: implements, target: wiki:specs:ban-all-comments}
---

schema_version: 1
state: |-
  ## Problem

  The project bans all comments including rustdoc `///`/`//!`, but a naive "strip `//` to end-of-line" sweep corrupts code: `//` appears inside string literals (`"Read(//**)"`), URLs (`https://...`), and format strings.

  ## Solution

  Strip only lines whose **first non-whitespace token is exactly `///` or `//!`** (`^\s*///`, `^\s*//!`). Leave bare `//` untouched (there were effectively zero real `//` comments). Preserve:
  - `#[schemars(description = "...")]` attributes — MCP tool schemas come from these attributes, **not** from `///`;
  - string literals, assertion messages, raw strings.

  Verify with `cargo check`/`clippy` and the schema-contract test (`mcp_test`).

  ## When to Use

  Sweeping doc comments out of a Rust workspace.

  ## When Not to Use

  Languages/codegen where `///` may legitimately appear at line start in string content, or where `///` drives code generation.

  ## Evidence

  1,406 doc-comment lines removed across 113 `.rs` files with zero warnings; `#[schemars]` descriptions and string literals byte-identical.
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