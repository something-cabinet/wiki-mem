---
id: wiki:specs:graph-connectivity-fix
title: Graph Connectivity Fix — Wire Body References Into Graph
type: spec
tags:
- graph
- edges
- connectivity
- spec
status: approved
---
id: wiki:specs:graph-connectivity-fix

## Overview

After adding 148 typed edges in the edge-verification deepwork session (50→198 edges), 334 of 444 pages (75%) still have zero inbound edges. Every component (graph, BM25, sections, embeddings, index.md) is rebuilt from scratch on every mutation — no incremental update path exists despite turso/SQLite being chosen for the embedding store with the expectation of incremental operations. FSRS6 recency scoring is runtime-only and not affected.

This spec addresses all root causes and establishes incremental rebuild for every indexed component.

## Locked Decisions

- D1: **Scope** — Full fix covering P0–P5 (graph connectivity, surfacing, knowns_id migration, symmetry, incremental rebuild across ALL components)
- D2: **P0 approach** — Wire body `@wiki/` reference extraction at graph build time in `build_graph_from_wiki()`, not at parse time
- D3: **P1 surfacing** — Upgrade debug→warn logging for unresolved targets, collect unresolved list in build return, surface as `unresolved_target` in `wm_lint.check`, and fix the dead `broken_ref` lint
- D4: **knowns_id** — Do not index; it's a legacy Knowns import artifact. A separate migration task should strip it from frontmatter.
- D5: **Symmetry** — Keep directed graph (per existing inverse-edge policy). When P0 extracts a body `@wiki/` reference from page A → target X, also add a reciprocal `references` edge from X → A. Fix the `↔` symbol in the edge table docs.
- D6: **index.md** — Exclude from graph; it's a generated artifact, not a real page
- D7: **Incremental rebuild — file watcher approach** — Instead of wiring rebuild into each tool independently, use a filesystem watcher (`notify` crate) on `.wm/wiki/` that detects file create/update/delete events and triggers incremental rebuild. Single implementation covers all tools + external editors + git operations. The full `wm_index.rebuild` stays available for batch operations.
- D8: **Incremental architecture** — Use copy-on-write on the existing `ArcSwap` pattern for in-memory components. BM25 gets `add_document()` / `remove_document()` methods. Sections get single-file parsing.

## Requirements

### Functional Requirements

- FR-1: Body `@wiki/` references (e.g., `@wiki/specs/graph-engine`) must produce graph edges during graph rebuild
- FR-2: Extracted body references must default to `references` edge type (priority 1)
- FR-3: For each extracted body reference A→X, a reciprocal `references` edge X→A must also be created
- FR-4: Duplicate edges (same source, same target, same type) must be deduplicated — one edge per (source, target, type) triple
- FR-5: Edge targets that fail to resolve against the existing graph must be surfaced in `wm_lint.check` as a new `unresolved_target` issue type, with the exact target string and source file
- FR-6: The `broken_ref` lint check must compare declared frontmatter edges against actual graph resolution, not the graph against itself
- FR-7: Log level for unresolvable targets must be `warn!` (not `debug!`)
- FR-8: Edge type docs (`concepts/edge-types.md`) must be updated to show all edge types as `→` (directed), not `↔`
- FR-9: No performance regression — graph rebuild with body reference extraction must not exceed 2× current rebuild time
- FR-10: A filesystem watcher (using the `notify` crate) must monitor `.wm/wiki/` for file create/update/delete events
- FR-11: On file create/update: parse the single changed file, update the graph incrementally (add/update node + edges without full rebuild), update sections incrementally, update BM25 incrementally
- FR-12: On file delete: remove the node and all its edges from the graph incrementally, remove from sections, remove from BM25
- FR-13: Debounce window (configurable, default ~500ms) must coalesce rapid consecutive events into a single rebuild cycle
- FR-14: `index.md` must be regenerated after any graph-changing mutation
- FR-15: Embeddings must run incrementally after page create/update/delete — the existing SHA-256 skip mechanism should be triggered by the watcher, not just full rebuild
- FR-16: Rebuild ordering must be: graph first → sections → BM25 → index.md → embeddings (graph must be up-to-date before downstream consumers run)
- FR-17: The existing stale-flag must still be set as a safety net — the file watcher is the primary path, stale-flag is the fallback

### Non-Functional Requirements

- NFR-1: Zero breaking changes to existing graphs — extracted body refs are additive only
- NFR-2: Must handle malformed `@wiki/` references gracefully (log and skip, don't crash)
- NFR-3: Incremental graph mutation must never block readers — existing `Arc` snapshots held by concurrent readers must continue working

## Acceptance Criteria

- [ ] AC-1: After rebuild, graph gains edges from body `@wiki/` refs — total edges > 350 (currently 198)
- [ ] AC-2: A page that previously had `@wiki/` refs in body now shows those as graph edges in `wm_graph.neighbors`
- [ ] AC-3: `wm_lint.check` lists `unresolved_target` issues for dangling targets with exact source file + target string
- [ ] AC-4: `knowns_id` using tasks like `awotvr` with short-ID body refs appear as `unresolved_target` until renamed to full slug
- [ ] AC-5: `broken_ref` lint passes — no false negatives (declared frontmatter target missing from graph)
- [ ] AC-6: `concepts/edge-types.md` shows all edge types as `→` (not `↔`)
- [ ] AC-7: Creating a page via any tool (`wm_page.create`, `wm_doc.create`, `wm_task.create`, `wm_memory.add`, `wm_source.add`) auto-triggers incremental rebuild — no manual `wm_index.rebuild` needed
- [ ] AC-8: Editing a page via an external editor (vim, VSCode) also triggers incremental rebuild within the debounce window
- [ ] AC-9: `wm_index.rebuild` still works identically as a full-reset operation
- [ ] AC-10: BM25 search results reflect a newly created page within the debounce window
- [ ] AC-11: BM25 search results stop returning a deleted page within the debounce window
- [ ] AC-12: `index.md` is up-to-date after any create/update/delete operation (within debounce window)
- [ ] AC-13: Newly created pages have embeddings computed without running `wm_index.embed`

## Scenarios

### Scenario 1: Body reference becomes graph edge
**Given** a wiki page `concepts/bm25-search.md` with body text `See @wiki/patterns/field-weighted-bm25`
**When** the graph is rebuilt from wiki pages
**Then** the graph has a `references` edge from `concepts:bm25-search` → `patterns:field-weighted-bm25`
**And** a reciprocal `references` edge from `patterns:field-weighted-bm25` → `concepts:bm25-search`

### Scenario 2: Unresolved target surfaced
**Given** a wiki page body has `@wiki/tasks/awotvr` (where the actual task slug is `task-awotvr-wiki-graph-engine`)
**When** `wm_lint.check` is run
**Then** an `unresolved_target` issue is reported with the file path, target string `wiki:tasks:awotvr`, and the candidate matches (task-awotvr-wiki-graph-engine)

### Scenario 3: Duplicate edges deduplicated
**Given** a page has both a `relates_to: [{type: references, target: wiki:specs:graph-engine}]` frontmatter entry AND a body `@wiki/specs/graph-engine` reference
**When** the graph is rebuilt
**Then** only one `references` edge exists from that page to `specs:graph-engine`

### Scenario 4: Full page lifecycle without manual rebuild
**Given** a user creates a new spec, then updates its tags, then deletes it
**When** each operation completes
**Then** after create: the page appears in graph neighbors and BM25 search
**And** after update: tag changes reflect in graph edges
**And** after delete: the page disappears from graph and BM25 search
**Without** calling `wm_index.rebuild` at any point

## Technical Notes

### Architecture constraint: ArcSwap snapshot pattern

All in-memory indices use `ArcSwap` for lock-free reads:
- Graph: `ArcSwap<GraphSnapshot>` where `GraphSnapshot = (StableGraph<WikiPageMeta, EdgeType>, HashMap<String, NodeIndex>)`
- Section corpus: `ArcSwap<Vec<SectionDoc>>`
- BM25 index: `ArcSwap<Bm25Index>`
- Embeddings: `VectorStore` (turso SQLite + in-memory `ArcSwap`)

Mutation requires: load Arc → clone inner → mutate clone → store new Arc. For small changes (single page), cloning the entire graph/BM25 is wasteful but acceptable at current scale (<1000 nodes, <500 documents).

### File watcher approach (P5)

Replace the existing stale-flag + mtime check with a real filesystem watcher. Use the modern standard stack:

```toml
[dependencies]
notify = "8.2.0"                    # 108M+ downloads, 3400+ stars — cross-platform file events
notify-debouncer-full = "0.7.0"     # 11.5M downloads — built-in debounce + rename tracking
```

**Why notify:** Uncontested standard. Used by Alacritty, rust-analyzer, Zed, mdBook. Cross-platform (macOS FSEvents, Linux inotify, Windows ReadDirectoryChangesW) with zero platform-specific code.

**Why notify-debouncer-full:** Text editors generate 3–5 raw events per save (temp file → rename → modify). Without debouncing you'd reprocess the same page 3–5 times. debouncer-full deduplicates, tracks renames, and merges events within a configurable window.

```rust
// In engine initialization or main_engine_factory.rs
let (tx, rx) = std::sync::mpsc::channel();
let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())?;
watcher.watch(wiki_dir, notify::RecursiveMode::NonRecursive)?;

// Debounce thread
loop {
    let events = debounce_events(&rx, Duration::from_millis(500));
    for event in events {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => handle_file_change(path),
            EventKind::Remove(_) => handle_file_delete(path),
            _ => {}
        }
    }
}
```

### Incremental rebuild cascade (on file watcher event)

```
1. Parse single .md file for meta + sections     [NEW — single file instead of walkdir]
2. Clone graph, add/remove node + edges, swap    [NEW — increment graph]
3. Regenerate index.md                             [NEW — call after graph swap]
4. Add/remove section from section corpus, swap  [NEW — increment sections]
5. Bm25Index::add_document / remove_document     [NEW — increment BM25]
6. Trigger embedding for changed sections         [EXISTS — SHA-256 skip, wire into mutation path]
7. Clear stale_flag                                [Safety net cleanup]
```

Steps 1-3 are synchronous (fast, <5ms). Steps 4-6 can be async/debounced.

### BM25 incremental API sketch

Add to `Bm25Index`:
```rust
fn add_document(&mut self, doc: IndexedDoc) {
    // tokenize doc, update term_freq, field_lengths, docs, total_docs
}
fn remove_document(&mut self, doc_id: &str) {
    // need to know the doc's tokens to decrement term_freq
    // Option 1: store token list per doc (memory cost)
    // Option 2: re-tokenize from section content (CPU cost)
    // Recommend Option 2 at current scale — <500 docs
}
fn update_document(&mut self, doc_id: &str, new_doc: IndexedDoc) {
    self.remove_document(doc_id);
    self.add_document(new_doc);
}
```

### IndexScheduler job type expansion

Current: only `"page"` job type with a single closure.
Proposed: ordered pipeline with `Vec<(job_type, Box<dyn FnOnce()>)>`:

```
Job pipeline on page mutation:
  1. graph     — graph incremental update (synchronous, before response)
  2. sections  — single-file section rebuild
  3. bm25      — add/remove document (triggered via debounce)
  4. embeddings — trigger single-section embedding
  5. index_md  — regenerate index.md (triggered after graph swap)
```

Jobs 2-5 are debounced and ordered. If another mutation arrives during debounce, pending jobs are coalesced (last-writer-wins per file).

### FSRS6 recency scoring

Not a rebuildable component — it is a runtime computation during search (forgetting curve applied to `updated_at` timestamps at query time). No incremental rebuild needed.

### Affected files

| File | Change |
|---|---|
| `apps/wm-core/src/graph/mod.rs` | Wire reference extraction (P0) + reciprocal edges + warn! logging + `add_page`/`update_page`/`remove_page` |
| `apps/wm-core/src/mcp/tools/lint.rs` | New `unresolved_target` issue type, fix `broken_ref` |
| `apps/wm-core/src/engine/main_engine_factory.rs` | Init file watcher on `.wm/wiki/` during engine startup |
| `apps/wm-core/src/graph/mod.rs` | New `handle_file_change()` / `handle_file_delete()` entry points |
| `apps/wm-core/src/mcp/tools/page/mod.rs` | Remove inline `rebuild_graph_snapshot()` — watcher handles it |
| `apps/wm-core/src/mcp/tools/doc.rs` | Remove stale-flag calls — watcher handles it |
| `apps/wm-core/src/engine/index_scheduler_service.rs` | Multiple job types, ordered pipeline |
| `apps/wm-core/src/graph/sections.rs` | Single-file section parsing function |
| `packages/wm-search/src/services/bm25_index_service.rs` | `add_document()` / `remove_document()` / `update_document()` |
| `packages/wm-embed/src/lib.rs` | Wire single-section embedding trigger |
| `apps/wm-core/src/graph/index_gen.rs` | Refresh-trigger function (rebuild only if graph changed) |
| `.wm/wiki/concepts/edge-types.md` | Fix `↔` → `→` in edge table |

## Open Questions

- [ ] Should reciprocal edges also be extracted for frontmatter `relates_to` entries, or only body `@wiki/` refs? (Currently scoped to body refs only per D5)
- [ ] BM25 `remove_document`: store pre-tokenized tokens per doc (memory cost) or re-tokenize from content (CPU cost)? Recommend re-tokenize at current scale.
- [ ] Should the debounce window be configurable per job type? (e.g., graph inline sync, BM25 500ms debounce)