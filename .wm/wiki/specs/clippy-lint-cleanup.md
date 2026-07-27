---
title: Clippy Lint Cleanup — Enable New Lints, Fix Violations
type: spec
tags:
- spec
- clippy
- lint
- code-quality
status: reviewed
---

## Overview

Enable 11 new clippy lints across the workspace and fix all resulting violations (~400 total). The lints catch risky casts, missing docs, overflow-prone arithmetic, module naming issues, and code-smell patterns.

## Requirements

### Functional Requirements

- FR-1: All crate root files define `#![warn(clippy::*)]` for the 11 new lints
- FR-2: Zero `as_conversions` warnings across the workspace
- FR-3: Zero `missing_errors_doc` warnings across the workspace
- FR-4: Zero `arithmetic_side_effects` warnings across the workspace
- FR-5: Zero `module_name_repetitions` warnings across the workspace
- FR-6: Zero `cognitive_complexity` warnings across the workspace (threshold 25)
- FR-7: Zero `cast_lossless` / `cast_possible_truncation` / `cast_sign_loss` warnings
- FR-8: Zero `allow_attributes_without_reason` warnings
- FR-9: Zero `unnecessary_wraps` warnings
- FR-10: Zero `branches_sharing_code` warnings

### Non-Functional Requirements

- NFR-1: Each fix must preserve behavior — no test changes beyond lint fixes
- NFR-2: Trivial fixes (cast lossless, allow reasons) can be auto-applied with `cargo clippy --fix`

## Acceptance Criteria

- [ ] AC-1: Running `cargo clippy --workspace` produces zero warnings for all 11 new lints
- [ ] AC-2: All existing tests still pass
- [ ] AC-3: No behavior changes — only refactoring and doc additions

## Technical Notes

- Use `cargo clippy --fix` for auto-fixable lints (cast_lossless, branches_sharing_code, collapsed if)
- `missing_errors_doc` requires adding `# Errors` doc sections to every `Result`-returning public function
- `arithmetic_side_effects` may need `.wrapping_*()` or `.saturating_*()` alternatives for intentional overflow
- `module_name_repetitions` requires renaming items or allowing on a case-by-case basis
- `cognitive_complexity` requires breaking down complex functions — the 2 violations are in pre-existing code