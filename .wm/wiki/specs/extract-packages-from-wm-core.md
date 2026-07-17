---
title: "Extract wm-core into Standalone Packages"
page_type: spec
status: draft
tags: [spec, refactor, monorepo, packages, workspace]
---

## Overview

Decompose `apps/wm-core` from a single ~13K line library crate into 12 focused packages under `packages/`. `apps/wm-core` becomes a thin wiring crate holding only EngineState, MCP tools, graph builder, and page CRUD — the application layer. All reusable library code lives in `packages/`.

Goal: loosely coupled, independently testable, no circular dependencies.

## Package Architecture

```
packages/
  wm-error           ← error types (ToolError, ToolResult)
  wm-status          ← PageStatus, MemoryStatus, Priority, Confidence
  wm-util            ← utility functions (slugify, truncate, format_duration)
  wm-page-repo       ← PageRepo trait + FsPageRepo + InMemoryPageRepo
  wm-vector-db       ← turso-based vector storage
  wm-template-engine ← template rendering (RenderResult, render_template)
  wm-embed           ← Embedder trait, SearchMode, EmbedVector, VectorStore, embed fns
  wm-engine          ← core types: EdgeType, PageType, WikiPageMeta, memory, source,
                         template, page_data, relation, EngineState
  wm-config          ← ProjectConfig, EmbeddingConfig, ScoringConfig, etc.
  wm-search          ← BM25 index, scoring, retrieval, query pipeline
  wm-parser          ← frontmatter extraction, wiki page parsing
  wm-code-intel      ← tree-sitter code intelligence

apps/
  wm-core            ← EngineState wiring + MCP tools + graph builder + page CRUD
  wm-cli             ← CLI binary (unchanged)
  wm-server          ← HTTP server (unchanged)
```

## Dependency Chain (strict, no cycles)

```
wm-error  wm-status  wm-util  wm-page-repo  wm-vector-db     ← layer 0 (zero deps)
    ↑                                                              
wm-template-engine  wm-embed (→wm-vector-db)                    ← layer 1
    ↑              ↑
wm-config (→wm-embed for SearchMode)                            ← layer 2
    ↑              ↑
wm-search (→wm-config, →wm-embed, →wm-engine for SectionDoc)   ← layer 3
    ↑
wm-engine (→wm-status, →wm-error)                               ← layer 4
    ↑              ↑
wm-parser (→wm-engine)  wm-code-intel (→wm-config via trait)    ← layer 5
    ↑              ↑
apps/wm-core (→all packages)                                     ← layer 6
```

## Trait Boundaries

Extraction requires trait interfaces at 3 points:

### 1. SectionDoc → wm-embed

`wm-embed::rebuild_embeddings_skip_unchanged` needs `SectionDoc` (defined in wm-engine). Solution:
```rust
// in wm-embed or a shared package
pub trait HasSectionId {
    fn section_id(&self) -> &str;
    fn body(&self) -> &str;
}
```
Implement for `SectionDoc` in wm-core. **~5 lines.**

### 2. SearchMode → wm-config

`wm-config::SearchConfig` has `default_mode: SearchMode` (defined in wm-embed). Solution: move `SearchMode` enum into wm-embed, keep wm-config depending on wm-embed. **No trait needed** — just a dependency edge.

### 3. LspLanguageSettings → wm-code-intel

`wm-code-intel::load_lsp_config` needs `config.lsp` field. Solution:
```rust
// in wm-code-intel  
pub trait ConfigProvider {
    fn lsp_settings(&self) -> Option<&HashMap<String, LspLanguageSettings>>;
}
```
Implement for `ProjectConfig` in wm-core. **~10 lines.**

## Execution Plan

### Phase 1: Layer 0 (zero deps) — 4 packages

| Package | From | Files | Effort |
|---------|------|-------|--------|
| `wm-error` | `error.rs` | Move + Cargo.toml | 20 min |
| `wm-status` | `status/` | Move + Cargo.toml | 15 min |
| `wm-util` | `util.rs` | Move + Cargo.toml | 10 min |
| `wm-page-repo` | `page_repo.rs` | Move + Cargo.toml | 10 min |

### Phase 2: Layer 1 — 2 packages

| Package | From | Files | Effort |
|---------|------|-------|--------|
| `wm-vector-db` | `vector_db.rs` | Move + Cargo.toml | 15 min |
| `wm-template-engine` | `template_engine/` | Move + own error type | 30 min |

### Phase 3: Layer 2-3 — 3 packages

| Package | From | Files | Effort |
|---------|------|-------|--------|
| `wm-embed` | `embed/` | Move + SectionDoc trait | 30 min |
| `wm-config` | `config/` | Move, depends on wm-embed | 20 min |
| `wm-search` | `search/` | Move + trait wiring | 30 min |

### Phase 4: Layer 4-5 — 3 packages

| Package | From | Files | Effort |
|---------|------|-------|--------|
| `wm-engine` | `engine/` (types only) | Move page_type, edge_type, memory, page_data, time_entry, audit_event, relation, source, template | 45 min |
| `wm-parser` | `parser/` | Move, dep on wm-engine | 15 min |
| `wm-code-intel` | `code_intel/` | Move + config trait | 30 min |

### Phase 5: Rewire apps/wm-core

Remove all source files that moved to packages. Keep only:
- `engine/mod.rs` (EngineState + submodules: state, write_channel, scheduler, main)
- `engine/page/` (graph-internal types stayed)
- `page/` (CRUD operations)
- `graph/` (graph builder)
- `mcp/` (MCP tools + transport)
- `reference.rs`, `source.rs`, `task.rs`
- `lib.rs` (now re-exports from packages)

### Phase 6: Build, test, verify

- Update `Cargo.toml` workspace members
- Fix all imports: `crate::foo` → `wm_foo`
- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace`

## Acceptance Criteria

- [ ] AC-1: All 12 packages compile independently
- [ ] AC-2: `cargo build --workspace` succeeds
- [ ] AC-3: `cargo test --workspace` passes (same count)
- [ ] AC-4: `cargo clippy --workspace` no new warnings
- [ ] AC-5: `apps/wm-core` is <2000 lines (from ~13K)
- [ ] AC-6: No circular dependencies between packages
- [ ] AC-7: Each package's Cargo.toml lists only its direct dependencies
