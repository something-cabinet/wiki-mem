---
title: One Struct Per File Refactor
type: spec
---

## Overview

Refactor `apps/wm-core/src/` to enforce the **one type per file** convention imported from gehenna-app. Every struct, enum, and trait gets its own file. Tightly coupled groups get a subdirectory module. No logic changes — pure file moves, visibility adjustments, and `mod.rs` re-exports.

## Locked Decisions

- **D1 (Granularity)**: One type per file. Tightly coupled types get their own directory module.
- **D2 (Folder shape)**: `parent/child/mod.rs | child_of_child.rs` — tightly coupled groups are a subdirectory where `mod.rs` re-exports and each child file holds one type.
- **D3 (Scope)**: Only `apps/wm-core/src/`. Leaves `wm-cli`, `wm-server`, and `wm-bridge` untouched.
- **D4 (Timing)**: Mechanical refactor only. No behavioral changes, no public API breakage. All tests must pass identically.

## Requirements

### Functional Requirements

- FR-1: Every `struct`, `enum`, and `trait` in `apps/wm-core/src/` (excluding test helpers and trivial newtype wrappers) resides in its own file.
- FR-2: Tightly coupled types are grouped under a subdirectory. Each still gets its own file. `mod.rs` re-exports them.
- FR-3: Each directory has a `mod.rs` that re-exports all child types via `pub use`.
- FR-4: Public API surface must remain unchanged. All existing `use crate::xxx::Yyy` imports across the workspace must continue to compile.
- FR-5: All existing tests pass. No new test files added (existing tests move with their types).

### Non-Functional Requirements

- NFR-1: Each file is ≤200 lines of non-comment code (enforced by the split — not a lint rule).
- NFR-2: Module names match the primary type name, snake_cased.
- NFR-3: No circular module dependencies introduced.

## Acceptance Criteria

- [ ] AC-1: `apps/wm-core/src/engine/mod.rs` reduced from 25 types to only re-exports and submodule declarations.
- [ ] AC-2: `apps/wm-core/src/config.rs` split into `config/mod.rs` + one file per config struct/enum.
- [ ] AC-3: `apps/wm-core/src/code_intel.rs` (behind `code-intel` feature) split into `code_intel/` subdirectory.
- [ ] AC-4: `apps/wm-core/src/parser.rs` split into `parser/` subdirectory.
- [ ] AC-5: `apps/wm-core/src/version.rs` split into `version/` subdirectory.
- [ ] AC-6: `apps/wm-core/src/skill.rs` split into `skill/` subdirectory.
- [ ] AC-7: `apps/wm-core/src/embed.rs` split into `embed/` subdirectory.
- [ ] AC-8: `apps/wm-core/src/status.rs` split into 4 files in `status/` subdirectory.
- [ ] AC-9: MCP tool files with output types (`task.rs`, `template.rs`, `page.rs`, `doc.rs`, `code.rs`, `memory.rs`) each become a subdirectory.
- [ ] AC-10: `cargo build` passes for all features combinations.
- [ ] AC-11: `cargo test` passes with same count as before.
- [ ] AC-12: `cargo clippy` passes with no new warnings.
- [ ] AC-13: All workspace crates that depend on `wm-core` compile without changes to their code.

## Scenarios

### Scenario 1: Simple split (status.rs → status/)
**Given** `status.rs` has 4 enums (PageStatus, MemoryStatus, Priority, Confidence)
**When** split into `status/mod.rs` + `page_status.rs` + `memory_status.rs` + `priority.rs` + `confidence.rs`
**Then** `use crate::status::PageStatus` continues to work via `pub use` in `mod.rs`

### Scenario 2: Tightly coupled group (engine types → engine/page_data/)
**Given** `engine/mod.rs` has 25 types including Page, TaskData, SpecData, DecisionData, PatternData
**When** the per-page-type data structs are tightly coupled (they're all variants in the Page enum)
**Then** they become `engine/page_data/mod.rs` + `task_data.rs` + `spec_data.rs` + `decision_data.rs` + `pattern_data.rs`

### Scenario 3: MCP tool with output types (mcp/tools/task.rs → mcp/tools/task/)
**Given** `task.rs` has WmTaskAction enum + 3 output structs
**When** the output structs are tightly coupled to the action (only exist for it)
**Then** they become `mcp/tools/task/mod.rs` + `action.rs` + `create_output.rs` + `update_output.rs` + `delete_output.rs`

### Scenario 4: Feature-gated module (code_intel)
**Given** `code_intel.rs` is behind `#[cfg(feature = "code-intel")]`
**When** split into subdirectory
**Then** the `#[cfg]` guard moves to `lib.rs` on the `mod code_intel` declaration; each file inside is unconditionally compiled

## Technical Notes

- **No tokio/async changes** — pure file organization.
- **Module visibility**: Types currently `pub` stay `pub`. Types currently `pub(crate)` stay `pub(crate)`. Re-exports flatten the API.
- **Feature gates**: `#[cfg(feature = "code-intel")]` stays on `lib.rs pub mod code_intel`. Internal files are unconditional.
- **Inline tests**: Tests move with their type into the new file. No test files created or deleted.
- **`Relation` ambiguity**: Both `parser.rs` and `engine/mod.rs` define `Relation`. They serve different purposes (frontmatter vs engine graph). Keep them in their respective modules.
- **MCP tool refactoring**: Each tool file with output types becomes a subdirectory. Handler logic stays in `mod.rs` (~100-200 lines). Only types split out.

## Open Questions

- Q1: `template_engine.rs` has a `Template` struct alongside its engine logic. Should the struct move to `engine/template/` or stay co-located with its sole user? (Probable: co-locate with user)
- Q2: `vector_db.rs` has VectorDbConfig + VectorDb. Tightly coupled — keep as-is or split? (Probable: keep)