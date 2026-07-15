---
title: Model Rework — enum Page, per-type status validation, CDD fixes
type: concept
status: draft
tags: [learning, model, cdd, status, enum-page]
---

# Learning: Model Rework — enum Page, per-type status, CDD

## Patterns

### Per-type status validation over separate enums

Instead of splitting `PageStatus` into per-type enums (`TaskStatus`, `SpecStatus`, `DecisionStatus`), keep a single `PageStatus` enum and validate per-type at the tool layer via `PageType::allowed_statuses()`. This avoids the petgraph constraint — `StableGraph<WikiPageMeta, EdgeType>` can only hold one node type, so heterogeneous enums aren't possible at the graph level.

**When to use:** Any situation where a unified data model is stored in a homogenous collection (graph, Vec, DB table) but needs per-type semantics. Validate at the boundary, not in the type.

### enum Page dispatch over Option wrappers

Use `enum Page { Task { meta, data }, Spec { meta, data }, Concept { meta }, ... }` instead of `WikiPageMeta { task_data: Option<TaskData>, spec_data: Option<SpecData>, ... }`. The compiler catches bugs where a Concept page has TaskData — no runtime check needed.

**When to use:** CDD — make invalid states unrepresentable. When petgraph stores the loose format internally, the public API stays strict.

### vectors.bin as dedicated crate

The vectors.bin binary format was extracted into its own `wm-vectors-bin` crate — zero dependencies, pure std, ~200 LOC. The read/write API is: `VectorsBin::write(model_name, &entries, &hashes)` and `VectorsBin::read(&bytes)` returning `(header, entries)`.

**When to use:** Isolate a stable binary format into its own crate even if currently only one consumer exists. Zero-dependency crates compile instantly and document the format as the public API.

## Decisions

### wm-cli as the single entry point (GOOD_CALL)

`wm-cli` is the only standalone binary. `wm-server` and the deleted `wm-mcp` are library crates with no `src/main.rs`. The MCP server embeds the HTTP server in-process — no external processes needed. This eliminates the startup coordination problem and simplifies deployment.

**Over:** Having separate `wm-mcp` and `wm-server` binaries that require manual coordination.

### ureq over reqwest::blocking inside tokio runtimes (GOOD_CALL)

`reqwest::blocking::Client::new()` panics when called inside a `#[tokio::main]` context because it tries to create its own tokio runtime. `ureq` is a pure blocking HTTP client with no tokio dependency — safe to use in any context. No C compiler needed.

### rusqlite rejected, vectors.bin kept (CONFIRMED)

Knowns uses `modernc.org/sqlite` — a pure Go SQLite reimplementation with no C compilation. Rust has no equivalent; `rusqlite` always compiles the C amalgamation. The "Knowns uses SQLite, therefore WM should use rusqlite" argument is a false equivalence. vectors.bin at 200 LOC is optimal at the current scale (<10k vectors).

### Per-type XxxData with unified naming (GOOD_CALL)

Replaced `DecisionEntry` → `DecisionData`, `PatternInfo` → `PatternData`. Applied consistent `XxxData` naming for all per-type data structs (`TaskData`, `SpecData`, `DecisionData`, `PatternData`). Consistency reduces cognitive load.

## Failures

### format!("{:?}", meta.status).to_lowercase() produces wrong output

`PageStatus::InProgress` with Debug formatting produces `"inprogress"` (no hyphen), while serde produces `"in-progress"`. Same for `PageType::Note` → Debug produces `"Note"` (capital N), serde produces `"note"`. The `as_str()` method was already correct on `PageStatus` but unused — 9 call sites used Debug formatting instead.

**Prevention:** Never use `format!("{:?}", enum)` for serialization. Always add a proper `as_str()` method and lint against Debug formatting in production code.

### reqwest::blocking cannot be created inside tokio

Creating `reqwest::blocking::Client::new()` inside a `#[tokio::main]` async context panics because it tries to create a new tokio runtime. This affected both the MCP proxy handlers and any other blocking HTTP usage within tokio.

**Prevention:** Use `ureq` for blocking HTTP in tokio contexts. Or create `reqwest::blocking::Client` via `std::thread::spawn(|| Client::new()).join()` before entering the async runtime.
