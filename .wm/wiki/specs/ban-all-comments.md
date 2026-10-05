---
title: Ban All Comments (Including Rustdoc)
type: spec
id: "wiki:specs:ban-all-comments"
status: draft
tags: [policy, comments, no-comments-in-code, conventions, cleanup]
---

## Overview

Ban **all** comments in project code — including **rustdoc `///` and `//!`** — and remove the existing doc comments across the workspace. This supersedes the current exemption in `@doc/core:CONVENTIONS` ("rustdoc `///`/`//!` … are exempt") and retires the draft rule `doc-comment-convention` (which required doc comments on public API).

## Locked Decisions

- **D1**: No comments of any kind in production code: `//`, `///`, `//!`, and block comments. Code must be self-documenting.
- **D2**: Update the active rule `no-comments-in-code` to state the ban explicitly covers rustdoc.
- **D3**: Retire/void the draft rule `doc-comment-convention`.
- **D4**: Update `@doc/core:CONVENTIONS` to remove the rustdoc exemption.
- **D5**: Strip existing doc comments from the workspace (mechanical, line-based; do not alter code semantics).
- **D6**: Functional attributes are not comments and stay (`#[...]`, `#![...]` — subject to the separate `no-allow-attributes` rule).

## Requirements

### Functional Requirements
- **FR-1**: Rule `no-comments-in-code` updated to ban rustdoc too.
- **FR-2**: `doc-comment-convention` retired.
- **FR-3**: `CONVENTIONS` "Comments" section rewritten (no rustdoc exemption).
- **FR-4**: All `///`, `//!`, and `//` lines removed from production `.rs` code across the workspace.
- **FR-5**: Keep non-Rust comment conventions consistent (Angular/TS templates, etc.) if any remain.

### Non-Functional Requirements
- **NFR-1**: Removal is semantics-preserving — code still compiles, tests pass, zero warnings.
- **NFR-2**: No accidental deletion of code lines (only comment lines/attributes that are comments).
- **NFR-3**: No `#![warn(missing_docs)]` added (it would now fire).

## Acceptance Criteria
- [ ] AC-1: `no-comments-in-code` rule text bans rustdoc; `doc-comment-convention` void.
- [ ] AC-2: CONVENTIONS updated.
- [ ] AC-3: `rg '^\s*///|^\s*//!|^\s*//[^/]'` over the workspace returns zero in production code.
- [ ] AC-4: `cargo check --workspace` + clippy clean; tests pass; no `#![warn(missing_docs)]`.

## Scenarios
### Scenario 1: Clean sweep
**Given** the workspace after removal
**When** grepping for comment syntax in `.rs`
**Then** no comment lines remain in production code.

## Technical Notes
- Use a line-based sweep (e.g. strip lines matching `^\s*///`, `^\s*//!`, `^\s*//` in `.rs`), run only after `@doc/specs/retire-web-ui-mcp-only` deletes web crates.
- Watch for `///` inside strings/regexes (rare); verify with a build + tests.
- `#[doc = "..."]` attributes are comments too — remove them.

## Open Questions
- [ ] Do any doc comments carry load-bearing info that must move into code (naming/structure) rather than be deleted outright?
