---
title: Reinforce the Indexing / Retrieval Flow
type: spec
id: "wiki:specs:reinforce-indexing-flow"
status: draft
tags: [indexing, retrieval, freshness, search-quality, embedding]
---

## Overview

Reinforce the **indexing/retrieval flow** (not decision models). A recon ranked 12 weaknesses; this spec addresses the highest-impact ones: **freshness correctness**, **structured-frontmatter indexing**, **embedding integrity**, **incremental perf**, and **eval coverage**.

## Locked Decisions

- **D1**: Scope to the recon's ranked weaknesses; **exclude model training** and decision models.
- **D2**: Do **not** bundle the embedder; keep the checksum-pinned download and improve **onboarding + degraded surfacing**.
- **D3**: Index **structured frontmatter fields** (not just `title`/`tags`).
- **D4**: Preserve BM25 + RRF + post-rerank as the ranking core.

## Requirements

### Functional
- **FR-1 Freshness**: one `refresh_all_if_stale(engine)` that rebuilds **graph + sections + BM25 (+ optional embeddings)**; call it from search, validate, and graph tools. `stale_flag` must reflect **all** derived state (not cleared after a BM25-only heal). Index tools must use `engine.project_root`, not `std::env::current_dir()`.
- **FR-2 Structured-frontmatter indexing**: sections/BM25 docs must include structured fields — spec `functional_requirements`/`non_functional_requirements`/goals, task `acceptance_criteria`, decision context/options/rationale, and `aliases` — plus a robust `record_state_text` (tolerate leading blanks/extra keys; never index raw YAML keys).
- **FR-3 Embedding integrity**: validate the persisted vector-store `model_name`/chunking version against the configured model on load; **force re-embed or clear** on mismatch; surface `degraded` + an actionable reason in `wm_search.query`, `wm_search.retrieve`, and `wm_index_status`; keep the semantic path a clear error, hybrid a keyword fallback with `degraded`.
- **FR-4 Incremental perf**: a single-page graph update (add/update/remove one node + its edges) instead of an O(N) full rebuild per debounced event.
- **FR-5 Eval**: promote the golden harness out of `#[ignore]` with **hybrid + semantic baselines** and **structured-frontmatter cases**.

### Non-Functional
- **NFR-1**: No regression in search/graph/validate.
- **NFR-2**: Zero warnings; no comments (comment ban in force); no `else`/magic values.

## Acceptance Criteria
- [ ] AC-1: `refresh_all_if_stale` used by search/validate/graph; no stale reads; index tools use `project_root`.
- [ ] AC-2: A spec's FR/AC and a decision's context/options are **findable via keyword search** (tests).
- [ ] AC-3: A model/version mismatch on load forces re-embed or clear (test); `degraded`+reason surfaced in query/retrieve/status.
- [ ] AC-4: Single-page graph update replaces full rebuild on a page write (test/perf check).
- [ ] AC-5: Golden eval runs (not `#[ignore]`) with hybrid/semantic + frontmatter cases; zero warnings.

## Scenarios
### Scenario 1: Fresh after a write
**Given** a page created outside the watcher
**When** search/validate runs
**Then** the graph/sections/BM25 reflect it (no stale snapshot).

### Scenario 2: Find a requirement
**Given** a spec with FR-3
**When** searching a phrase from FR-3
**Then** the spec ranks (frontmatter indexed).

### Scenario 3: Model swap
**Given** persisted vectors from a different model
**When** the engine loads
**Then** it forces re-embed or clears rather than serving mismatched vectors.

## Technical Notes
- Recon refs: `apps/wm-core/src/graph/sections.rs:27-60`, `apps/wm-core/src/search/query.rs:94-111`, `apps/wm-core/src/mcp/tools/index.rs:144-146,238`, `apps/wm-core/src/mcp/tools/validate.rs:29-41`, `packages/wm-embed/src/vector_store_service.rs:49-80`, `packages/wm-embed/src/lib.rs:334-351`, `apps/wm-core/src/engine/main_engine_factory.rs:64-73`, `apps/wm-core/tests/golden_eval.rs:560+`, `packages/wm-engine/src/decision/record_state.rs:8-19`.
- `stale_flag` is currently cleared after a BM25-only heal (`query.rs:109`) — fix semantics.

## Open Questions
- [ ] Auto-download the embedder on first hybrid/semantic request, or only prompt?
- [ ] Include `code.db` freshness in `refresh_all_if_stale`?
