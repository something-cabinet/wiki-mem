---
id: wiki:rules:no-warnings
title: Zero Tolerance for Compiler Warnings
type: rule
status: active
category: quality
rationale: "Compiler warnings indicate code quality issues: unused code, dead paths, or potential bugs. Over time they accumulate and hide real problems."
references: "@wiki/rules/no-dead-code-clone-scanning"
---
id: wiki:rules:no-warnings

## Rule

Every crate in the workspace MUST compile with zero warnings. A warning is a future error.

## Enforcement

- Run `cargo check --workspace` and `cargo clippy --workspace` before every commit
- If either produces any warnings or errors, fix them before committing
- `cargo fix` may be used to auto-fix trivial warnings, but verify the result
- CI must pass `cargo check --workspace` with zero warnings

## Exceptions

All exceptions MUST include a comment explaining why the suppression is necessary.

### Allowed (with comment)
- `#[allow(unused_variables)]` on trait/interface method parameters that the signature requires but the implementation legitimately does not use
- `#[allow(dead_code)]` for fields in the **MCP JSON Schema flatten pattern** — struct fields that exist solely for JSON Schema generation, using `#[serde(flatten)]` + `_schema` prefix naming + `..` in match arms (see WIKI-MEM.md §Enterprise Correctness)
- `#[allow(clippy::*)]` for a specific named lint, with a comment explaining why the lint is wrong for this particular case

### Forbidden
- `#[allow(dead_code)]` on items, fields, or modules outside the MCP schema flatten pattern — dead code must be removed or restructured, never suppressed
- `#![allow(dead_code)]` crate-level blanket suppression — never acceptable
- `#[allow(unused_imports)]` — remove the unused import instead
- `#[allow(unused_variables)]` for variables that aren't trait-mandated — prefix with `_` instead
- `#[allow(unused_mut)]` — drop `mut` instead
- `#[allow(clippy::*)]` wildcard — must name the specific lint

### Feature-Gated Code
Code behind `#[cfg(feature = "...")]` that is dead in the current build must gate the usage, not suppress the warning.

## Related

See `@wiki/rules/no-dead-code-clone-scanning` for Clone derives, `.clone()` audit requirements, and additional dead-code enforcement patterns.

## Rationale

Compiler warnings indicate code quality issues: unused code, dead paths, or potential bugs. Over time they accumulate and hide real problems. Zero-tolerance keeps the codebase clean and CI feedback immediate. "Suppress nothing, fix everything" — see WIKI-MEM.md §Enterprise Correctness.
