---
title: "Extract Standalone Packages from wm-core"
page_type: spec
status: draft
tags: [spec, refactor, monorepo, packages, workspace]
---

## Overview

Extract well-bounded, zero-dependency modules from `apps/wm-core/` into standalone crates under `packages/`. Each extracted crate becomes independently versionable, testable, and reusable outside this monorepo.

## Boundary Rule

A module qualifies for extraction ONLY if it has **zero or minimal** dependency on `wm-core`'s internal types (EngineState, WikiPageMeta, graph, search, MCP infrastructure). If extracting requires pulling in wm-core's data model, don't extract — finish the internal module decomposition instead.

## Extraction Candidates

### 1. Template Engine → `packages/wm-template-engine`

**Current location:** `apps/wm-core/src/template_engine/` (535 lines, already split into sub-modules)

**Dependencies:** None on wm-core internals. Only stdlib + `serde_json` + `crate::error::ToolError`.

**Problem:** Depends on `ToolError` from wm-core. Would need its own error type or a shared error crate.

**Options:**
- A: Extract with its own `TemplateEngineError` type (small, 2-3 variants). Clean break.
- B: Extract `wm-error` first, have both use it. More ceremony.
- C: Don't extract. Keep as module within wm-core.

**Recommendation:** Option A. ~200 lines of extraction + ~20 lines of error type. The template engine is a pure renderer — it belongs as a standalone crate.

### 2. VectorDb → `packages/wm-vector-db`

**Current location:** `apps/wm-core/src/vector_db.rs` (409 lines)

**Dependencies:** `turso` crate. Zero dependency on wm-core internals.

**Consumers:** Only `wm-core` today, but turso-backed vector storage is a generic primitive.

**Recommendation:** ✅ Extract. Clean boundary, zero internal deps, useful outside wm-core.

### 3. PageRepo → `packages/wm-page-repo`

**Current location:** `apps/wm-core/src/page_repo.rs` (91 lines)

**Dependencies:** Zero on wm-core internals. Only stdlib traits.

**Consumers:** Only `wm-core` today.

**Recommendation:** ❌ Skip. Too small to justify a separate crate's overhead (Cargo.toml, CI, versioning).

### 4. Error Types → `packages/wm-error`

**Current location:** `apps/wm-core/src/error.rs` (194 lines)

**Dependencies:** Zero. Only stdlib.

**Consumers:** Every other crate would use it. Currently `ToolError` is used by wm-core and referenced by template_engine.

**Recommendation:** ⏳ Defer. Only extract if it enables extracting another crate (like template engine). The value is enabling crate boundaries, not standing alone.

### 5. Reference System → `packages/wm-reference`

**Current location:** `apps/wm-core/src/reference.rs` (198 lines)

**Dependencies:** `EngineState`, `WikiPageMeta` — tightly coupled to wm-core's graph.

**Recommendation:** ❌ Don't extract. Internal module decomposition is sufficient.

### 6. Graph Builder → `packages/wm-graph`

**Current location:** `apps/wm-core/src/graph.rs` (370 lines, being split)

**Dependencies:** `WikiPageMeta`, `EdgeType`, `petgraph` — tightly coupled to wm-core's data model.

**Recommendation:** ❌ Don't extract. Internal module decomposition is sufficient.

## Acceptance Criteria

- [ ] AC-1: `packages/wm-vector-db` compiles standalone, all tests pass
- [ ] AC-2: `packages/wm-template-engine` compiles standalone, all tests pass
- [ ] AC-3: wm-core depends on both extracted packages via workspace dependency
- [ ] AC-4: `cargo build --workspace` succeeds
- [ ] AC-5: `cargo test --workspace` passes same count as before

## Non-Goals

- Do NOT extract EngineState or any graph/search/page types — they're too coupled
- Do NOT publish any packages to crates.io (local workspace only)
- Do NOT change any behavior or API surface during extraction

## Execution Order

1. Extract `wm-vector-db` (cleanest boundary, zero deps)
2. Extract `wm-template-engine` (needs own error type)
3. Update `wm-core` Cargo.toml to depend on both
4. Verify all tests pass
