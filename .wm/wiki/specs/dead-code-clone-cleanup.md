---
id: wiki:specs:dead-code-clone-cleanup
title: Dead Code & Clone Cleanup
type: spec
status: approved
tags: [spec, dead-code, clone, rust, quality, linting, cleanup]
references: "@wiki/rules/no-dead-code-clone-scanning, @wiki/rules/no-warnings"
---
id: wiki:specs:dead-code-clone-cleanup

## Overview

Fix all identified violations of the new dead-code and clone rules across the Rust workspace. This covers removing/suppressing `#[allow(dead_code)]`, `#[allow(unused_*)]`, and `#[allow(clippy::*)]` annotations correctly, plus auditing Clone derives and `.clone()` calls for necessity.

## Locked Decisions

- **D1**: All 4 violation buckets in one spec with parallel fixer agents
- **D2**: Dead-code suppressions → use `_schema` flatten pattern for MCP schema fields, remove dead code for everything else
- **D3**: Unused variables → prefix with `_` or remove; never blanket-suppress
- **D4**: Clippy suppressions → named lints with comments or fix the root cause
- **D5**: Clone audit = review + justification pass, not bulk removal

## Requirements

### Functional Requirements

#### FR-1: Fix `#[allow(dead_code)]` (11 sites)
- **FR-1.1**: MCP tool input structs with dead fields must use the `_schema` flatten pattern: `#[serde(flatten)] _schema: InputSchema` + `..` in match arms
- **FR-1.2**: True dead code (`memory_dir`, `resolve_root`, `reader_task`) must be removed or restructured
- **FR-1.3**: Server route input struct fields (`skip_embed`, `status`, `depth`) must be either used or removed

#### FR-2: Fix `#[allow(unused_*)]` (5 sites)
- **FR-2.1**: Variables that are truly unused → prefix with `_` or remove the binding
- **FR-2.2**: `#[allow(unused_mut)]` → drop `mut`
- **FR-2.3**: Trait-signature-mandated parameters → keep `#[allow(unused_variables)]` with comment

#### FR-3: Fix `#[allow(clippy::*)]` (4 sites)
- **FR-3.1**: `should_implement_trait` on `from_str` → rename to `from_str` (it already is? needs investigation) or add named lint with comment explaining why `FromStr` trait cannot be implemented
- **FR-3.2**: `type_complexity` → extract type alias or add named lint with comment
- **FR-3.3**: `ambiguous_glob_reexports` (8 occurrences) → these are systematic and acceptable with a comment

#### FR-4: Clone Audit
- **FR-4.1**: Review every `#[derive(Clone)]` type — mark each as justified (async boundary, Arc wrapper, hot path) or remove Clone
- **FR-4.2**: Types used purely as DTOs (serialization only) should not derive Clone unless there is a code path that clones them
- **FR-4.3**: High-frequency `.clone()` calls should be reviewed for potential borrows or Arcs
- **FR-4.4**: Add justification comments where Clone is intentionally kept

### Non-Functional Requirements
- **NFR-1**: `cargo check --workspace` must pass with zero warnings after each change
- **NFR-2**: `cargo clippy --workspace` must not introduce new warnings or errors
- **NFR-3**: Tests must pass (`cargo test --workspace`)
- **NFR-4**: Each fix must be in its own commit with a descriptive message

## Acceptance Criteria

- [x] AC-1: No `#[allow(dead_code)]` remains without the `_schema` flatten pattern
- [x] AC-2: No `#[allow(unused_imports)]`, `#[allow(unused_variables)]` (except trait params), `#[allow(unused_mut)]` remain
- [x] AC-3: All `#[allow(clippy::*)]` use named lints with justification comments
- [x] AC-4: Every `#[derive(Clone)]` type has a documented justification or Clone is removed
- [x] AC-5: `cargo check --workspace` and `cargo clippy --workspace` pass clean
- [x] AC-6: `cargo test --workspace` passes

## Scenarios

### Scenario 1: Fix dead_code in MCP tool input structs
**Given** `apps/wm-core/src/mcp/tools/time.rs` has `#[allow(dead_code)]` on `note` and `group_by` fields in an enum
**When** the fix is applied
**Then** those fields either use the `_schema` flatten pattern, are removed, or the suppression is replaced with a named lint + comment

### Scenario 2: Fix dead_code in server route params
**Given** `apps/wm-server/src/routes/graph.rs` has `#[allow(dead_code)]` on a `depth` field
**When** the fix is applied
**Then** the field is either used in the handler or removed from the struct

### Scenario 3: Remove truly dead functions
**Given** `apps/wm-core/src/mcp/tools/memory.rs` has `memory_dir()` and `resolve_root()` with `#[allow(dead_code)]`
**When** the fix is applied
**Then** both functions are removed (they are unused)

### Scenario 4: Fix unused_variables
**Given** `apps/wm-core/src/mcp/tools/code.rs` has `#[allow(unused_variables)]` on `filter_lang`
**When** the fix is applied
**Then** the variable is prefixed with `_` and the `#[allow]` is removed

### Scenario 5: Clone derive review
**Given** a type with `#[derive(Clone)]` used purely for serialization (e.g., `config` models, parser models)
**When** audit is performed
**Then** Clone is removed if no clone path exists, or a comment explains why it's needed

## Technical Notes

### Priority order
1. FR-2 (unused variables) — easiest, mechanical
2. FR-1 (dead_code) — some need architectural decisions
3. FR-3 (clippy lints) — mostly mechanical
4. FR-4 (clone audit) — largest scope, requires judgment

### Fix pattern for dead MCP schema fields
The existing `_schema` pattern from WIKI-MEM.md:
```rust
#[derive(Deserialize, JsonSchema)]
struct TimeActionSchema {
    #[schemars(description = "Note about this time entry")]
    pub note: Option<String>,
}
enum WmTimeAction {
    Stop { id: String, #[serde(flatten)] _schema: TimeActionSchema },
}
```

### Tools
- `rg '#\[allow\(dead_code'` — find remaining dead_code suppressions
- `rg '#\[allow\(unused'` — find remaining unused suppressions
- `rg '#\[allow\(clippy'` — find clippy suppressions
- `rg '\.clone\(\)' | wc -l` — clone call count baseline
- `cargo clippy --workspace` — detection and auto-fix

## Open Questions

- [ ] OQ-1: The `reader_task` field in `LspTransport` (`packages/wm-lsp/src/transport.rs:14`) — is this intentionally dead (stored for RAII drop semantics) or truly removable?
- [ ] OQ-2: `wm-engine/src/models/mod.rs` glob re-exports — should these be migrated to explicit `pub use` in a follow-up, or keep the systematic `#[allow(ambiguous_glob_reexports)]`?

## Appendix: Full Violation Inventory

### FR-1: `#[allow(dead_code)]` sites

| # | File | Line | Item | Suggested Fix |
|---|------|------|------|---------------|
| 1 | `apps/wm-server/src/routes/index.rs` | 8 | `skip_embed` field in `RebuildInput` | Use `_` prefix or remove |
| 2 | `apps/wm-server/src/routes/tasks.rs` | 10 | `status` field in `BoardParams` | Use `_` prefix or remove |
| 3 | `apps/wm-server/src/routes/graph.rs` | 87 | `depth` field in `NeighborsInput` | Use `_` prefix or remove |
| 4 | `apps/wm-core/src/mcp/tools/memory.rs` | 34 | `category` field | `_schema` flatten pattern |
| 5 | `apps/wm-core/src/mcp/tools/memory.rs` | 62 | `memory_dir()` function | Remove (unused) |
| 6 | `apps/wm-core/src/mcp/tools/memory.rs` | 375 | `resolve_root()` function | Remove (unused) |
| 7 | `apps/wm-core/src/mcp/tools/time.rs` | 13 | `note` in `Stop` variant | `_schema` flatten pattern |
| 8 | `apps/wm-core/src/mcp/tools/time.rs` | 15 | `note` in `Add` variant | `_schema` flatten pattern |
| 9 | `apps/wm-core/src/mcp/tools/time.rs` | 17 | `group_by` in `Report` variant | `_schema` flatten pattern |
| 10 | `apps/wm-core/src/mcp/tools/log.rs` | 26 | `limit` in `WmLogSinceInput` | `_schema` flatten pattern |
| 11 | `apps/wm-core/src/mcp/tools/log.rs` | 35 | `limit` in `WmLogFilterInput` | `_schema` flatten pattern |
| 12 | `apps/wm-core/src/mcp/tools/graph.rs` | 10 | `depth` in `WmGraphNeighborsInput` | `_schema` flatten pattern |
| 13 | `apps/wm-core/src/mcp/tools/graph.rs` | 13 | `edge_type` in `WmGraphNeighborsInput` | `_schema` flatten pattern |
| 14 | `packages/wm-embed/src/vector_db.rs` | 132 | `dim` in `InnerDb` | Use `_` prefix or keep for clarity with comment |
| 15 | `packages/wm-lsp/src/transport.rs` | 14 | `reader_task` in `LspTransport` | Check: kept for RAII? If so, add comment, not `#[allow]` |

### FR-2: `#[allow(unused_*)]` sites

| # | File | Line | Item | Suggested Fix |
|---|------|------|------|---------------|
| 16 | `apps/wm-core/src/engine/engine_state_mediator.rs` | 185 | `path` param in `notify_file_changed` | Trait signature? If so, keep with comment; else prefix `_` |
| 17 | `apps/wm-core/src/version/version_store_repository.rs` | 167 | `old_count` binding | Remove the `let` binding or prefix `_` |
| 18 | `apps/wm-core/src/mcp/tools/lsp.rs` | 224 | `fallback_symbols` (unused mut) | Drop `mut` or remove |
| 19 | `apps/wm-core/src/mcp/tools/code.rs` | 209 | `filter_lang` binding | Prefix `_` |
| 20 | `apps/wm-core/src/mcp/tools/code.rs` | 416 | `filter_lang` binding | Prefix `_` |

### FR-3: `#[allow(clippy::*)]` sites

| # | File | Line | Lint | Suggested Fix |
|---|------|------|------|---------------|
| 21 | `apps/wm-core/src/skill/models/trigger_event_model.rs` | 19 | `should_implement_trait` | Check if `FromStr` can be implemented; if not, use named lint + comment |
| 22 | `packages/wm-embed/src/lib.rs` | 206 | `type_complexity` | Extract type alias or named lint + comment |
| 23 | `packages/wm-embed/src/models/search_mode_model.rs` | 21 | `should_implement_trait` | Same as #21 |
| 24 | `packages/wm-engine/src/models/mod.rs` | 15-31 | `ambiguous_glob_reexports` (8×) | Keep with comment explaining edition migration context |

### FR-4: Clone audit — key derive sites

| Category | File count | Assessment |
|----------|-----------|------------|
| Config models (apps/wm-core/src/config/models/) | 8 | Needed (Arc<Config>, shared state) |
| Parser models (apps/wm-core/src/parser/models/) | 7 | Questionable — DTOs only? Check clone paths |
| Version models (apps/wm-core/src/version/models/) | 5 | Questionable — DTOs only? Check clone paths |
| Skill models (apps/wm-core/src/skill/models/) | 3 | Needed (async dispatch) |
| Engine models (packages/wm-engine/src/models/) | ~30 | Most are DTOs — review each |
| Search models (packages/wm-search/src/services/) | 3 | DTOs — check |
| Code intel (packages/wm-code-intel/src/) | 5 | Needed (async boundaries) |
| Embed models (packages/wm-embed/src/) | 4 | Mix — review |
| Server state (apps/wm-server/src/routes/mod.rs) | 1 | Needed (Arc<AppState>) |
| TUI state (apps/wm-cli/src/tui.rs) | 2 | Needed (state management) |
