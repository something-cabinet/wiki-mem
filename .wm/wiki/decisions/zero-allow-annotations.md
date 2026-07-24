---
id: wiki:decisions:zero-allow-annotations
title: "Decision: Zero `#[allow(...)]` Annotations"
type: decision
status: approved
category: code-quality
rationale: "Every `#[allow(...)]` annotation is an accepted defect. Compiler warnings must be fixed at the root, not suppressed."
---
id: wiki:decisions:zero-allow-annotations

## Context

The codebase had 17 `#[allow(...)]` annotations across patterns: `#[allow(dead_code)]` on MCP schema fields, `#[allow(ambiguous_glob_reexports)]` in model modules, `#[allow(clippy::*)]` for should_implement_trait and type_complexity. Each was individually justifiable but collectively normalized suppression over fixing.

## Decision

Zero `#[allow(...)]` annotations in the entire workspace. All compiler warnings must be fixed at the root:
- `dead_code` → `_` prefix or `#[serde(rename)]` pattern
- `ambiguous_glob_reexports` → explicit individual `pub use`
- `clippy::should_implement_trait` → implement `FromStr` trait
- `clippy::type_complexity` → extract type alias

## Rationale

- Suppressions hide real issues and compound over time
- Each suppression type has a mechanical fix that is cleaner than the suppression
- Eliminating suppressions forces the code to be self-documenting about intent
- The `_` prefix convention is the idiomatic Rust way to express "intentionally unused"

## Consequences

- All 17 `#[allow(...)]` removed across the workspace
- `cargo check --workspace --all-targets` produces zero warnings
- New code should never introduce `#[allow(...)]` — the rule is enforced at compile time

relates_to:
  - {type: references, target: "wiki:specs:dead-code-clone-cleanup"}
  - {type: references, target: "wiki:specs:fix-clone-calls"}
  - {type: references, target: "wiki:specs:fix-rust-anti-patterns"}
  - {type: supersedes, target: "wiki:rules:no-warnings"}

## Related

- @wiki/rules/no-dead-code-clone-scanning
- @wiki/rules/no-warnings
- @wiki/patterns/mcp-schema-field-rename
