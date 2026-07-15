---
title: Model Rework Session Learnings
type: concept
status: reviewed
tags: [learning, session, model-rework]
---

## Patterns

### Enum Dispatch over Option Wrappers (CDD)
- **What:** Use typed `enum Page` variants over `Option<XxxData>` on a flat struct. The compiler guarantees a Reference can never have TaskData — no runtime check needed. Petgraph still stores `WikiPageMeta` with Options internally via conversion.
- **When to use:** When data has distinct per-type shapes stored in a homogeneous collection (petgraph).
- **Source:** This session (D12)

### Background Thread for Tokio Runtime Isolation
- **What:** When an async-native crate (turso, reqwest) must be used from sync code inside `#[tokio::main]`, create the runtime in a separate thread and communicate via `std::sync::mpsc`. Better alternative: use `tokio::task::block_in_place` + `Handle::current().block_on()` for multi-thread runtimes.
- **When to use:** When mixing async SDK with sync CLI handlers.
- **Source:** This session (VectorDb refactor)

### Block_in_place Pattern for Sync↔Async Bridge
- **What:** `tokio::task::block_in_place(|| Handle::current().block_on(async { ... }))` safely bridges sync→async from inside a multi-thread tokio runtime. Falls back to creating a standalone Runtime when no runtime exists (e.g., `#[test]`).
- **When to use:** Any time an async crate (turso, reqwest) must be called from a sync MCP handler.
- **Source:** This session (VectorDb final fix)

## Decisions

### Memory as a Page Variant
- **Chose:** Memory entries became `Page::Memory { meta, data: MemoryData }` — a typed wiki page variant with graph edges, frontmatter, and version tracking.
- **Over:** Keeping memory as separate JSON blobs with `MemoryEntry` struct.
- **Tag:** GOOD_CALL
- **Outcome:** ~700 LOC deleted (`memory.rs`, `search/memory.rs`, `state.rs` fields). Memory now gets all wiki features for free.
- **Source:** This session (D16)

### Turso over wm-vectors-bin
- **Chose:** Replaced custom binary vector format with turso SQLite database.
- **Over:** Keeping wm-vectors-bin (zero-dep crate).
- **Tag:** TRADEOFF
- **Outcome:** Incremental vector updates, content hash tracking, metadata queries. Adds ~30-60s compile time and a tokio-runtime dependency.
- **Source:** This session (D22-D24)

### MCP Tool Surface 78→46 Action Enum Merge
- **Chose:** Merged CRUD domains into single tools with `#[serde(tag = "action")]` discriminated unions.
- **Over:** Keeping 78 individual dot-notation tools.
- **Tag:** GOOD_CALL
- **Outcome:** Cleaner `tools/list` output, better AI agent discovery. Dropped 225 LOC (typed.rs).
- **Source:** This session (mcp-tool-surface spec)

### YAGNI on Config Consumers
- **Chose:** Added config structs (StatusColors, LspLanguageSettings, GitTracking) with `#[serde(default)]` but deferred consumer wiring until features are needed.
- **Over:** Building all consumers upfront.
- **Tag:** GOOD_CALL
- **Outcome:** Config schema is stable and backward-compatible. Consumers added later without migration.

## Failures

### Vectors.bin Runtime Conflict Naive Fix
- **What went wrong:** Tried to create a `tokio::runtime::Runtime` inside `#[tokio::main]` which panics with "Cannot start a runtime from within a runtime."
- **Attempted fix:** Background thread with mpsc channels — overly complex, ~150 LOC of channel plumbing.
- **Final fix:** `block_in_place` + `Handle::current().block_on()` — the official tokio-recommended pattern. Simpler by ~100 LOC.
- **Time lost:** ~30 minutes
- **Prevention:** Research the crate's tokio integration guide before building workarounds.

### Not Researching Turso Tokio Guide
- **What went wrong:** Built a background thread + mpsc channel workaround before checking turso's official tokio integration pattern.
- **Root cause:** Assumed turso had no guidance — didn't check docs.rs or GitHub.
- **Time lost:** ~20 minutes
- **Prevention:** Always research the crate's official docs before building custom workarounds.
