---
id: wiki:patterns:mcp-schema-field-rename
title: "MCP Schema Field: `_` Prefix Over `#[allow(dead_code)]`"
type: pattern
status: active
category: code-quality
rationale: "`#[allow(dead_code)]` on schema-only fields violates the zero-tolerance rule for suppressions. The `_` prefix + `#[serde(rename)]` pattern preserves the wire format while satisfying the compiler."
see: "@wiki/rules/no-dead-code-clone-scanning"
relates_to:
  - {type: references, target: "wiki:specs:dead-code-clone-cleanup"}
  - {type: references, target: "wiki:specs:fix-clone-calls"}
---

schema_version: 1
state: |-
  id: wiki:patterns:mcp-schema-field-rename

  ## Problem

  MCP tool input structs often have fields that exist only for JSON Schema generation (`#[derive(JsonSchema)]`). These are never read at runtime, so the compiler emits `dead_code` warnings. Using `#[allow(dead_code)]` is forbidden.

  ## Solution

  Prefix the field with `_` and add `#[serde(rename = "original_name")]` to preserve the wire format:

  ```rust
  // Before: #[allow(dead_code)] required
  struct WmLogLimitSchema {
      #[allow(dead_code)]
      #[schemars(description = "Max entries")]
      limit: Option<i32>,
  }

  // After: no annotation needed
  struct WmLogLimitSchema {
      #[serde(rename = "limit")]
      #[schemars(description = "Max entries")]
      _limit: Option<i32>,
  }
  ```

  The `_` prefix tells Rust the field is intentionally not read in application code. `#[serde(rename)]` ensures JSON deserialization still maps from the original key name. `JsonSchema` derive respects the rename, so generated schemas also use the original name.

  ## When to Use

  - Struct fields behind `#[serde(flatten)]` in MCP schema structs
  - Fields that exist solely for `JsonSchema` derive
  - Any field where `_` prefix is preferred over `#[allow(dead_code)]`

  ## When Not to Use

  - Fields that are actually read at runtime — use a normal name
  - Public API fields consumed by callers — keep the original name
  - Fields behind `#[cfg(feature)]` gates — use `cfg_attr` instead

  ## Related

  - @wiki/rules/no-dead-code-clone-scanning
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
