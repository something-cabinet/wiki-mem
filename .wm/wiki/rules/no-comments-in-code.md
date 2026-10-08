---
id: wiki:rules:no-comments-in-code
title: "No Comments in Project Code"
type: rule
status: active
category: naming
rationale: "Comments rot, drift from code, and create false confidence. Named functions, descriptive variables, and self-documenting patterns are refactor-safe and always up to date."
example: "Extract a named function `validate_transition()` instead of `// validate state transition`."
anti_pattern: "Inline comments explaining what code does (// increment counter, // check if valid)"
---

schema_version: 1
state: |-
  # No Comments in Project Code

  All comments are banned in production code, with no exemptions. This explicitly includes Rustdoc `///` and `//!` and the equivalent `#[doc = "..."]` attributes, in addition to inline `//` and block comments.

  Code must be self-documenting: extract named functions, use descriptive variables, and express intent through structure. Comments rot, drift from code, and create false confidence; names and structure are refactor-safe and always up to date.

  ## Banned

  - Inline and block comments: `// ...`, `/* ... */`
  - Rustdoc comments: `/// ...` (outer) and `//! ...` (inner/crate-level)
  - Doc attributes: `#[doc = "..."]`

  ## Allowed

  - Functional attributes: `#[...]` and `#![...]` (subject to the `no-allow-attributes` rule)
  - String literals and assertion messages (not comments)

  ## Instead Of

  Extract a named function `validate_transition()` instead of `// validate state transition`.
questions:
  - id: enforcement
    type: choice
    instructions: How is this rule enforced?
    options:
    - ci-enforced
    - tool-enforced
    - review-enforced
    - convention-only
  - id: severity
    type: score
    instructions: How severe is a violation of this rule?
    levels:
    - advisory
    - recommended
    - required
    - blocking
  - id: has_exception
    type: noul
    instructions: This rule has documented exceptions.
  - id: applies_to
    type: choice
    instructions: Which surfaces does this rule apply to?
    multi: true
    options:
    - code
    - tests
    - docs
    - configuration
    - workflow
    - security
answers: {}
