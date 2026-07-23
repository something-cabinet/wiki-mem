---
title: Zero Tolerance for Compiler Warnings
type: rule
status: active
category: quality
rationale: "Compiler warnings indicate code quality issues: unused code, dead paths, or potential bugs. Over time they accumulate and hide real problems."
---

## Rule

Every crate in the workspace MUST compile with zero warnings. A warning is a future error.

## Enforcement

- Run `cargo check --workspace` before every commit
- If `cargo check --workspace` produces any warnings, fix them before committing
- `cargo fix` may be used to auto-fix trivial warnings, but verify the result
- CI must pass `cargo check --workspace` with zero warnings

## Exceptions

- `#[allow(dead_code)]` is acceptable for struct fields that exist for API/serialization compatibility (e.g., input structs deserialized from frontend requests)
- `#[allow(unused_variables)]` is acceptable for handler parameters required by trait signatures but unused in the implementation
- All exceptions must include a comment explaining why the warning is suppressed

## Rationale

Compiler warnings indicate code quality issues: unused code, dead paths, or potential bugs. Over time they accumulate and hide real problems. Zero-tolerance keeps the codebase clean and CI feedback immediate.
