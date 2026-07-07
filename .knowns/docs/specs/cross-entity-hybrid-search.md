---
title: Cross-Entity Hybrid Search
description: Unified search across wiki pages, tasks, and memory entries with per-type BM25 indexes, RRF fusion, and hybrid mode
createdAt: '2026-07-07T04:14:12.631Z'
updatedAt: '2026-07-07T04:45:42.292Z'
tags:
  - spec
  - approved
  - search
  - memory
---

## Overview

Extend WM's search to query across wiki pages AND memory entries in a single call. Per-type BM25 indexes with RRF fusion for fair cross-type ranking, type-specific scoring, and hybrid (keyword + semantic) mode.

**Tasks are excluded from cross-entity search** — they are already wiki pages (type: task) in the page index. No separate task index. No dedup problem.

**Memory is the only new entity type** — `.wm/memory/*.json` parsed into a `MemoryEntry` struct and indexed via BM25.

- **D1**: Per-type BM25 indexes (pages + memory) merged via RRF fusion. Pages get the existing index. Memory gets a new index. Tasks are already pages — no separate index.
- **D2**: Flat unified result list with `type` label per result.
- **D3**: `type` param defaults to `"all"`. `"all"` → pages + memory. `"page"` → pages only. `"memory"` → memory only. No backward compat — existing callers that don't pass `type` now get memory results too.
- **D4**: Fusion order for hybrid mode: per-type BM25+vector RRF first (2 per-type lists), then cross-type RRF of the 2 fused lists. NOT single 6-list RRF.
- **D5**: Task dedup: tasks stay in the page index only. No separate task index. `type: "task"` filter queries the page index filtered to tasks.
- **D6**: Memory change detection via debounced index scheduler (matching Knowns' pattern). No poll timer. MCP tool handlers (page.create, page.update, etc.) submit rebuild jobs with 500ms debounce. Agent skills (wm-extract, wm-commit) instruct agents to call `wm_index.rebuild` after mutations. Replace `AtomicBool stale_flag` with `IndexScheduler` that supports per-type debounced scheduling.
- **D7**: Vector storage: extend `vectors.bin` format with entity type tag. New format version (`WMV\1`). Backward compat: old `vectors.bin` without type tag defaults to `"page"`.
- **D8**: `wm_search.retrieve` gets a `type` param. For memory results, context assembly uses flat text (no graph BFS — memory has no edges).
### Functional Requirements

- **FR-1**: BM25 index per entity type: `IndexPages` (existing), `IndexMemory` (new)
- **FR-2**: `wm_search.query` accepts `type` filter: optional string (default `"all"`). `"all"` → pages + memory. `"page"` → pages only. `"memory"` → memory only.
- **FR-3**: `wm_search.query` with `type: "all"` queries both indexes and merges via RRF
- **FR-4**: Each result includes `type` field: `"page"` or `"memory"`
- **FR-5**: `wm_search.retrieve` extends to accept `type` param. For memory results, context is flat text (no graph BFS). For pages/tasks, context assembly uses existing graph BFS.
- **FR-6**: `SearchResult` struct includes `type` field. `page_type_rank` and `centrality` are always present — set to `0` for memory results.
- **FR-7**: Memory entries with `critical` tag get a salience boost (2× multiplier, post-RRF, clamped to max 0.1 absolute score)
- **FR-8**: Hybrid fusion order: per-type BM25+vector RRF first, then cross-type RRF of fused lists
- **FR-9**: Graceful degradation: if embed feature is disabled, semantic/hybrid silently falls back to keyword
- **FR-10**: Index rebuild triggers: page/memory MCP tool handlers submit jobs to debounced index scheduler (500ms debounce). Agent skills instruct agents to call `wm_index.rebuild` after mutations. Replace `AtomicBool stale_flag` with `IndexScheduler`.
- **FR-11**: No persistence for memory BM25 index — rebuilt on `wm serve` startup and on scheduler-triggered rebuilds. Vectors ARE persisted in extended `vectors.bin`.
- **FR-12**: `IndexScheduler` supports per-type scheduling — page mutations don't trigger memory rebuilds and vice versa.
## Acceptance Criteria

- [ ] AC-1: `wm_search.query({ q: "auth", type: "all" })` returns pages + memory entries
- [ ] AC-2: Each result has `type` field: `"page"` or `"memory"`
- [ ] AC-3: RRF fusion produces fair cross-type ranking (short memory doesn't dominate long pages)
- [ ] AC-4: Critical-tagged memory entries rank higher than non-critical on same query (2× boost, clamped)
- [ ] AC-5: `wm_search.query({ q: "auth" })` (no type) returns pages + memory (default is `"all"`) with `type` field on each result
- [ ] AC-6: `wm_search.query({ q: "auth", type: "memory" })` returns memory only
- [ ] AC-7: `wm_search.retrieve({ q: "auth", type: "all" })` returns pages via graph BFS, memory as flat text
- [ ] AC-8: Debounced index scheduler coalesces 5 rapid memory writes into 1 rebuild (within 500ms debounce window)
- [ ] AC-9: `vectors.bin` with new type tag coexists with old format (read both, write new)
- [ ] AC-10: `cargo build` + `cargo test` pass
- [ ] AC-11: Unit tests for per-type BM25, RRF merge of 2 lists, debounced index scheduler
- [ ] AC-12: `wm_index.status` reports per-type doc counts (`pages`, `memory`)

## Scenarios

### Scenario 1: Agent searches for "auth" across all memory

**Given** a wiki with pages about JWT and a memory entry "Use bcrypt not argon2 for passwords"
**When** agent calls `wm_search.query({ q: "auth", type: "all" })`
**Then** both results return in a single ranked list with `type: "page"` and `type: "memory"`
**And** the JWT page has `page_type_rank` and `centrality` fields, the memory doesn't

### Scenario 2: Default query returns all types

**Given** existing code calling `wm_search.query({ q: "auth" })`
**When** the search runs (no `type` param)
**Then** both page and memory results are returned (default is `"all"`)
**And** each result includes a `type` field for disambiguation

### Scenario 3: Hybrid mode across types

**Given** an embedding model loaded (`--features embed`)
**When** agent calls `wm_search.query({ q: "OAuth2 token refresh", type: "all", mode: "hybrid" })`
**Then** BM25 runs per-type, vector cosine similarity runs per-type, per-type RRF fuses each, then cross-type RRF merges the two fused lists

### Scenario 4: Memory updated while WM is running

**Given** `wm serve` running with debounced index scheduler
**When** agent calls `wm_memory.add` (or writes a memory file and calls `wm_index.rebuild`)
**Then** the scheduler submits a memory rebuild job with 500ms debounce
**And** subsequent queries reflect the new memory
**And** 5 rapid writes coalesce into 1 rebuild

### Graph traversal depth per feature

Every WM feature that touches the graph uses a different traversal strategy. This table defines the **stop-rule** for each:

| Feature | Max Depth | Edge Filter | Stop Condition | Why |
|---------|-----------|-------------|----------------|-----|
| **Search RRF** (`wm_search.query` with `mode=hybrid`) | **1** | All | Direct neighbors only | Score propagation beyond depth 1 is noise. If B `implements` A, and C `depends_on` B, C should rank on its own BM25 merit — not inherit A's score through a chain. |
| **Context retrieval** (`wm_search.retrieve`) | **2** | Depth 2: priority ≥ 5 only | Token budget exhausted | Agent needs broader context. Depth 1: all connections. Depth 2: only strong connections (depends_on, implements, extends, part_of, supersedes). Deeper than 2 is noise — the agent should call `graph.neighbors` explicitly. |
| **Graph neighbors** (`wm_graph.neighbors`) | **Configurable** (default 2, max 5) | All | Configurable limit | User explicitly asked for exploration. Safe to go deeper because the agent controls the depth parameter. Max 5 prevents unbounded traversal on cyclic graphs. |
| **Graph path** (`wm_graph.path`) | N/A | All | Shortest path found | Uses Dijkstra's, not BFS. No depth limit — finds the shortest path between two nodes regardless of length. |
| **Graph stats** (`wm_graph.stats`, `wm_initial`) | 0 | N/A | N/A | Summary statistics only. No traversal. |
| **Index rebuild** (`wm_index.rebuild`) | 0 | N/A | N/A | Walks `.wm/wiki/` directory, not the graph. No traversal. |

### Depth behavior details

```
Depth 1 — immediate neighbors only:
  Match(es the query) → [ Edge ] → Neighbor
  
  Used by: Search RRF (Graph_results input)

Depth 2 — high-priority chain:
  Match → [ any edge ] → Neighbor → [ priority ≥ 5 only ] → Neighbor-of-Neighbor
  
  Used by: Context retrieval

Configurable depth — user-controlled:
  Match → [ any edge ] → ... up to N hops
  
  Used by: Graph neighbors tool
```

### Edge priority reference

Priority values are defined in `engine.rs:EdgeType::priority()`:

| Priority | Edge Types | Included in depth 2 traversal? |
|----------|------------|--------------------------------|
| 10 | extends | ✅ Yes |
| 9 | implements | ✅ Yes |
| 8 | part_of, supersedes | ✅ Yes |
| 7 | supports | ✅ Yes |
| 6 | example_of | ✅ Yes |
| 5 | depends_on, required_by | ✅ Yes (cutoff) |
| 4 | mitigates, causes | ❌ No |
| 3 | contradicts, questions | ❌ No |
| 2 | answers | ❌ No |
| 1 | references, similar_to | ❌ No |
| 0 | relates_to, custom | ❌ No |

### MemoryEntry struct

```rust
#[derive(Debug, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

Indexed fields: `title` (weight 4.0), `content` (weight 1.0). Tags parsed for `critical` salience boost.

### Source data

- **Pages**: existing `.wm/wiki/**/*.md` walker (unchanged). Includes tasks, specs, concepts, patterns, decisions, howto, reference.
- **Tasks**: part of page index. Filter with `type: "task"` in query — no separate index needed.
- **Memory**: `.wm/memory/*.json` — JSON files with title, content, tags, timestamps. Path resolved from project root (same dir as `.wm/`). Configurable via `config.json` `memory_dir` field.

### Fusion order (hybrid mode)

```
For each type (page, memory):
  BM25_results = index.search(query)
  Vector_results = cosine_similarity(query_embedding, type_vectors)
  PerType_fused[type] = RRF(BM25_results, Vector_results, k=60)

CrossType_final = RRF(PerType_fused["page"], PerType_fused["memory"], k=60)
```

### Debounced index scheduler (matching Knowns' pattern)

```rust
pub struct IndexScheduler {
    jobs: mpsc::Sender<IndexJob>,
    debounce: Duration,              // 500ms
    pending: Mutex<HashMap<String, Instant>>,
}

impl IndexScheduler {
    pub fn submit(&self, job_type: &str, rebuild_fn: Box<dyn Fn() + Send>) {
        // If same job_type submitted within debounce window, reset timer
        // After debounce elapses without new submission, run rebuild_fn
        // Supported job types: "page", "memory"
    }
}

// In MCP tool handler (tools.rs):
scheduler.submit("memory", || rebuild_memory_index());

// On wm_index.rebuild MCP tool:
scheduler.submit("page", || rebuild_page_index());
scheduler.submit("memory", || rebuild_memory_index());
```

### SearchResult format

```rust
pub struct SearchResult {
    pub id: String,
    pub r#type: String,        // "page" | "memory"
    pub score: f64,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_type_rank: Option<u8>,   // Some for pages, None for memory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub centrality: Option<usize>,    // Some for pages, None for memory
}
```

### vectors.bin extension

Add a 1-byte type tag after the existing magic bytes:
- `WMV\0` → all vectors are page vectors (old format, default)
- `WMV\1` → each vector entry has a 1-byte type prefix: `0` = page, `1` = memory

On read: detect version, parse accordingly. On write: always write `WMV\1`.

### ScoringConfig (new config.json section)

All scoring parameters live under `search.scoring` in `.wm/config.json`. Defaults below:

```json
{
  "search": {
    "default_mode": "hybrid",
    "default_limit": 20,
    "rrf_k": 60,
    "scoring": {
      "field_weights": {
        "title": 4.0,
        "body": 1.0
      },
      "recency_model": "fsrs",
      "recency_stability_days": 7,
      "memory_salience_boost": 2.0,
      "memory_salience_clamp": 0.1,
      "graph_depth_rrf": 1,
      "graph_depth_retrieve": 2,
      "graph_depth_retrieve_min_priority": 5,
      "graph_depth_neighbors_default": 2,
      "graph_depth_neighbors_max": 5,
      "debounce_ms": 500,
      "retrieve_token_budget": 2048
    }
  }
}
```

### Recency model

Task recency boosts results based on how recently they were updated. Configured by `recency_model` (default `"fsrs"`):

| Mode | Formula | Behavior |
|------|---------|----------|
| `"fsrs"` | FSRS-6 forgetting curve | Sharp non-linear decay: `R(t) ≈ 0.9 at t=0 → 0.5 at t=S → 0.15 at t=30d`. Best for actionable tasks where staleness = irrelevance. **Default.** |
| `"linear"` | `max(0, 1 - t/stability)` | Linear decay to 0 at t=stability. Gentler than FSRS for medium-old items. |
| `"exponential"` | `exp(-t/stability)` | Smooth exponential decay. Between FSRS and linear. |
| `"none"` | 1.0 | No recency boost. Every result ranks purely on BM25/vector/RRF. Useful for archival queries where age doesn't matter. |

### Stability parameter

`recency_stability_days` (default 7) defines the half-life of the decay curve:

| Mode | Meaning of stability_days |
|------|--------------------------|
| fsrs | Days until retrievability drops to ~50% (t=S → R≈0.5) |
| linear | Days until boost reaches 0 |
| exponential | Decay constant τ — boost ≈ 0.37 at t=stability |

FSRS-6 implementation (hardcoded in code):

```rust
const FSRS_W: [f64; 21] = [0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334,
    3.0194, 0.001, 1.8722, 0.1666, 0.796, 1.4835, 0.0614, 0.2629,
    1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542];

fn recency_boost(days_since_update: f64, model: &str, stability_days: f64) -> f64 {
    match model {
        "fsrs" => {
            let w20 = FSRS_W[20];
            let factor = 0.9_f64.powf(-1.0 / w20) - 1.0;
            let r = (1.0 + factor * days_since_update / stability_days).powf(-w20);
            r.max(0.0).min(1.0)
        }
        "linear" => (1.0 - days_since_update / stability_days).max(0.0),
        "exponential" => (-days_since_update / stability_days).exp(),
        _ => 1.0, // "none"
    }
}
```

### Recency vs salience stacking

Multiple boosts are multiplicative, capped at 4× total:
```
final_score = rrf_score * min(4.0, recency_boost * salience_boost)
```

This prevents a very recent critical-tagged task (0.9 retrievability × 2× salience) from dominating. Cap at 4× ensures fair cross-type ranking while still rewarding strong signals.

ScoringConfig struct (wm-core/src/config.rs):

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScoringConfig {
    #[serde(default = "default_field_weights")]
    pub field_weights: HashMap<String, f64>,
    #[serde(default = "default_recency_model")]
    pub recency_model: String,              // "fsrs" | "linear" | "exponential" | "none"
    #[serde(default = "default_recency_stability_days")]
    pub recency_stability_days: u32,
    #[serde(default = "default_memory_salience_boost")]
    pub memory_salience_boost: f64,
    #[serde(default = "default_memory_salience_clamp")]
    pub memory_salience_clamp: f64,
    #[serde(default = "default_graph_depth_rrf")]
    pub graph_depth_rrf: u32,
    #[serde(default = "default_graph_depth_retrieve")]
    pub graph_depth_retrieve: u32,
    #[serde(default = "default_graph_depth_retrieve_min_priority")]
    pub graph_depth_retrieve_min_priority: u8,
    #[serde(default = "default_graph_depth_neighbors_default")]
    pub graph_depth_neighbors_default: u32,
    #[serde(default = "default_graph_depth_neighbors_max")]
    pub graph_depth_neighbors_max: u32,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_retrieve_token_budget")]
    pub retrieve_token_budget: usize,
}
```

Edge type priorities (engine.rs EdgeType::priority) remain hardcoded — they define graph semantics, not tunable scoring. A page `extends` another is structurally 10.
## Open Questions

- [x] Should memory entries get their own BM25 index or Vec linear scan? → BM25 index with per-startup rebuild, not Vec. Memory count can grow beyond 100.
- [x] What about RRF for 6 lists? → Defined fusion order: per-type first, then cross-type.
- [x] FR-2 vs NFR-3 contradiction? → Resolved: no backward compat needed. Default is `"all"` (pages + memory). Simple.
- [x] Task dedup? → No separate task index. Tasks stay in page index.
- [x] Memory change detection? → Debounced index scheduler (500ms), no polling. MCP handlers submit jobs, agents call `wm_index.rebuild`.
- [x] Vector storage? → Extended `vectors.bin` format with type tag (`WMV\1`).
- [x] `wm_search.retrieve` for memory? → Flat text context, no graph BFS.
- [x] Memory data model? → `MemoryEntry` struct defined above.
