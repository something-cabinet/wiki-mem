---
id: wiki:patterns:domain-splitting-section-markers
title: 'Pattern: Domain Splitting — Section Markers Signal Modules'
type: pattern
tags: [pattern, architecture, module-structure, refactor]
status: active
relates_to:
  - {type: references, target: wiki:specs:domain-splits-page-codeintel-template-graph}
---

schema_version: 1
state: |-
  id: wiki:patterns:domain-splitting-section-markers

  ## Problem

  A file reaches 300-900 lines with `// ─── Section ───` markers separating distinct concerns. The code works but is hard to navigate, merge conflicts are painful, and related types are scattered.

  ## Solution

  The section marker IS the module boundary. Split each marked section into its own file:

  ```
  // ─── CRUD ───       → crud.rs
  // ─── YAML Helpers ─→ yaml.rs
  // ─── Path ───       → path.rs
  // ─── Migration ───  → migration.rs
  ```

  For MCP tools, the structure is always:
  - `action.rs` — action enum (data only)
  - `output.rs` — output structs (data only)
  - `mod.rs` — handler dispatch + register

  ## When to Use

  - File has 3+ section markers → split into sub-directory
  - MCP tool file → always split into action.rs + output.rs + mod.rs
  - "What" comments before code blocks → extract into named function

  ## When Not to Use

  - Files under 200 lines with clear single responsibility
  - Files where sections are tightly coupled (would require passing too many params)

  ## Related

  - @wiki/specs/domain-splits-page-codeintel-template-graph
  - Applied to: code_intel/, template_engine/, graph/, page/, mcp/tools/task/, mcp/tools/template/, mcp/tools/page/
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
