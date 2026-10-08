---
title: Doc Comment Convention
type: rule
id: wiki:rules:doc-comment-convention
status: superseded
tags: [convention, documentation, rust, code-quality]
---

schema_version: 1
state: |-
  # Doc Comment Convention — VOID (Retired)

  **This rule is retired and void.** It has been superseded by @wiki/rules/no-comments-in-code and @wiki/specs/ban-all-comments, which ban all comments in production code — including Rustdoc `///` (outer) and `//!` (inner/crate-level) and `#[doc = "..."]` attributes.

  The former requirement to write doc comments on public API no longer applies. Do not add doc comments. Make code self-documenting through naming and structure instead.
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
