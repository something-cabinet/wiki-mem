---
title: Doc Comment Convention
type: rule
id: wiki:rules:doc-comment-convention
status: superseded
tags: [convention, documentation, rust, code-quality]
---

# Doc Comment Convention — VOID (Retired)

**This rule is retired and void.** It has been superseded by @wiki/rules/no-comments-in-code and @wiki/specs/ban-all-comments, which ban all comments in production code — including Rustdoc `///` (outer) and `//!` (inner/crate-level) and `#[doc = "..."]` attributes.

The former requirement to write doc comments on public API no longer applies. Do not add doc comments. Make code self-documenting through naming and structure instead.
