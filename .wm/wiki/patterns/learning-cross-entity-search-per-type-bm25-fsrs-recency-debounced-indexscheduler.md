---
id: wiki:patterns:learning-cross-entity-search-per-type-bm25-fsrs-recency-debounced-indexscheduler
title: 'Learning: Cross-Entity Search — Per-Type BM25, FSRS Recency, Debounced IndexScheduler'
type: pattern
tags: [learning, search, architecture]
relates_to:
  - {type: references, target: wiki:tasks:4hk4kz}
---
id: wiki:patterns:learning-cross-entity-search-per-type-bm25-fsrs-recency-debounced-indexscheduler

## Patterns

### Per-Type BM25 Indexes + RRF Fusion
- **What:** Maintain separate BM25 indexes for different entity types (pages, memory) instead of a unified index. Merge results via RRF (Reciprocal Rank Fusion) with per-type rank partitioning.
- **When to use:** Any search system with heterogeneous document types where document length varies significantly between types (short memory entries vs long wiki pages). A unified index lets short documents dominate rankings due to artificially high TF.
- **Source:** @wiki/tasks/4hk4kz, @spec/cross-entity-hybrid-search

### Debounced IndexScheduler
- **What:** Replace `AtomicBool stale_flag` with a debounced index scheduler using tokio channels + timers. Rapid mutations coalesce into a single rebuild within a configurable window (default 500ms). Matches Knowns' `sync.go` pattern.
- **When to use:** Any system where mutations trigger index rebuilds and multiple rapid mutations are common. Debouncing prevents redundant rebuilds without adding polling overhead.
- **Source:** @wiki/tasks/4hk4kz, `engine.rs:IndexScheduler`, `refs/knowns/internal/search/sync.go`

### rust-embed for Binary Assets
- **What:** Use `rust-embed` crate (not `include_str!()`) to embed directory trees of skill files (`wm-*/SKILL.md`) into the binary at compile time. This is the Rust equivalent of Go's `//go:embed`.
- **When to use:** Embedding multiple files in a directory hierarchy where you need runtime iteration (list, get by path). Use `include_str!()` for single files.
- **Source:** @spec/wm-sdd-skills, `skill.rs:SkillAssets`

### FSRS-6 Forgetting Curve as Generic Decay Model
- **What:** The FSRS-6 formula `R(t,S) = (1 + factor × t/S)^(-w_20)` can serve as a general-purpose decay function anywhere time-based relevance is needed. Only `stability_days` (the half-life parameter) needs to be configurable — the 21 FSRS weights are proven defaults.
- **When to use:** Any scoring system that needs non-linear time decay. More principled than ad-hoc linear/exponential formulas.
- **Source:** @spec/cross-entity-hybrid-search, `search.rs:recency_boost()`

## Decisions

### Per-Type BM25 Over Unified Index
- **Chose:** Separate `ArcSwap<Bm25Index>` for pages and memory, merged via RRF.
- **Over:** A single unified BM25 index with entity type tags.
- **Tag:** GOOD_CALL
- **Outcome:** Short memory entries (10-50 words) don't dominate long wiki pages (500+ words). Each type has correct IDF statistics. Per-type indexes rebuild independently without affecting each other.
- **Recommendation:** Keep this pattern. If more entity types are added (e.g., decisions), add another ArcSwap rather than merging indexes.

### FSRS-6 as Default Recency Model
- **Chose:** FSRS-6 forgetting curve with hardcoded 21 default parameters (from open-spaced-repetition/awesome-fsrs). Only `recency_stability_days` is configurable.
- **Over:** Simple linear decay (boost = max(1, 3 - days/3.5)), exponential decay, or no recency.
- **Tag:** GOOD_CALL
- **Outcome:** FSRS is stricter than linear — a 30-day-old task gets ~0.78 retrievability vs linear's clamped 1.0. This correctly deprioritizes stale tasks while not completely burying them.

### Debounced IndexScheduler Over Polling
- **Chose:** MCP tool handlers submit rebuild jobs with 500ms debounce. Tokio channels + timers coalesce rapid mutations.
- **Over:** Filesystem polling (checking .wm/memory/ mtime every N seconds) or Knowns MCP callback.
- **Tag:** GOOD_CALL
- **Outcome:** Zero overhead when idle. Rapid mutations (5 page saves) trigger 1 rebuild, not 5. Matches Knowns' proven pattern from `sync.go`.
- **Recommendation:** Wire remaining tool handlers (page.update, page.delete, page.unlink) to submit scheduler jobs too.

### No Separate Task Index
- **Chose:** Tasks stay in the unified page index with FSRS recency boost applied post-ranking.
- **Over:** A separate BM25 index for tasks with its own IDF statistics.
- **Tag:** GOOD_CALL
- **Outcome:** No double-counting problem (task appearing in both page and task index). No sync issues. Recency boost + status filter provide sufficient differentiation.
- **Recommendation:** If task-specific ranking requirements grow (e.g., priority boost, assignee filter), add a dedicated task search tool rather than a separate index.

### No Backward Compat for type Param
- **Chose:** Default `type` is `"all"` (searches both pages and memory). Existing callers without explicit `type` now get memory results too.
- **Over:** Backward compat (default = `"page"`, only pages).
- **Tag:** GOOD_CALL
- **Outcome:** Simplified API. All callers in this project use `wm_initial` first anyway. No compatibility incidents.
- **Recommendation:** This was the right call for an early-stage project. Document breaking change clearly.

### FSRS-6 R(t=S) = 0.9, Not 0.5 — SURPRISE
- **Chose:** The FSRS-6 forgetting curve defines stability S as the interval when retrievability R = 90%, not 50%.
- **Over:** My initial assumption that t=S means R≈0.5 (common in spaced repetition where S is the half-life).
- **Tag:** SURPRISE
- **Outcome:** Test assertions at day7 were initially wrong (expected ~0.5, got ~0.9). Fixed tests to check ~0.9 at t=S.
- **Recommendation:** When using FSRS for the first time, verify the stability definition. S is the 90% recall interval, not the 50% interval. A stability of 7 days means R=0.9 at day 7, ≈0.78 at day 30.

## Failures

### merge_by_rrf Keyed by Position, Not Document ID
- **What went wrong:** The initial RRF merge function used position in the concatenated input array as the RRF key instead of document ID. Since pages are always before memory, memory results always got worse RRF scores regardless of relevance.
- **Root cause:** Copied the RRF concept without understanding that RRF requires per-item identity across lists. The hashmap key was `usize` (position) instead of document `id`.
- **Time lost:** ~30min to fix during code review.
- **Prevention:** When implementing RRF, ensure the merge function keys by document identity (ID string), not by position. Each ranking pass contributes `1/(k + rank)` per document, summed across passes.

### Salience Clamp Used min() Instead of Proper Formula
- **What went wrong:** `final_score = (score * salience_boost).min(clamp)` with default clamp=0.1 destroyed all memory scores. A memory scoring 0.9 became `min(1.8, 0.1) = 0.1`.
- **Root cause:** Misunderstood the spec formula. The correct formula is `boost = min(salience_boost, clamp / score)` which caps the absolute score at `clamp` without destroying results.
- **Time lost:** ~15min during code review.
- **Prevention:** When implementing clamping, use the spec formula directly. The clamp should cap the multiplier, not the final score.

### Recency Boost Hardcoded 7.0 Days
- **What went wrong:** `recency_boost(7.0, &scoring.recency_model, ...)` passed a constant `7.0` as `days_since_update` instead of the actual page age. Every task got the same recency multiplier.
- **Root cause:** The `updated_at` field from `WikiPageMeta` was available in the graph node but never read. The FSRS formula was purely ornamental.
- **Time lost:** ~15min during code review.
- **Prevention:** When using a time-based scoring function, always source the time delta from the actual data. Hardcoded test values in production code are a red flag.

### Memory Path Used Hardcoded Relative Path
- **What went wrong:** `let memory_dir = std::path::Path::new(".wm").join("memory")` resolved relative to CWD, not project_root. If `wm serve` was launched from a different directory, memory reads silently returned empty results.
- **Root cause:** Other tool handlers also used CWD-relative paths. The memory retrieve handler followed the same pattern without considering project_root override.
- **Time lost:** ~20min during code review (including path traversal security fix).
- **Prevention:** Always use `e.project_root.read()` for filesystem operations in tool handlers. The engine has `resolve_path()` for exactly this purpose.

### FSRS-6 Test Assertions Wrong
- **What went wrong:** Test expected `recency_boost(7.0, "fsrs", 7.0) ≈ 0.5` but actual result was `≈ 0.9`.
- **Root cause:** FSRS-6 defines stability S as the 90% recall interval, not the 50% half-life.
- **Time lost:** ~5min to fix test assertions.
- **Prevention:** Verify the mathematical definition of stability before writing test assertions for spaced repetition formulas.

### Multiple Config Lock Acquisitions in Single Handler
- **What went wrong:** The `wm_search.query` handler acquired `e.config.read()` four separate times within one request. Each acquisition could theoretically see a different config snapshot.
- **Root cause:** The handler evolved incrementally — each new feature (hybrid search, recency boost, salience boost, RRF merge) added its own config read.
- **Time lost:** ~10min during code review to consolidate.
- **Prevention:** Acquire config once at the top of each handler, extract owned values, drop the guard.