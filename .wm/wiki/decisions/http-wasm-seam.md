---
{}
relates_to:
  - {type: references, target: wiki:specs:http-wasm-architecture-cleanup}
  - {type: references, target: wiki:patterns:wasm-crate-integration}
  - {type: references, target: wiki:patterns:engine-port-backend-abstraction}
  - {type: references, target: wiki:patterns:canvas2d-wasm-graph}
---

id: wiki:decisions:http-wasm-seam

## Context

The WM project has a Rust backend (wm-server) serving 26+ REST API endpoints, and an Angular frontend (wm-web) that communicates via HTTP. We also have a successful fjadra-wasm crate that runs graph layout in the browser.

Proposal was raised: should we ditch HTTP entirely and compile wm-core to WASM, running the entire engine in the browser?

## Decision

**Keep HTTP as the primary transport. WASM is for pure-compute extensions only, following the fjadra profile.**

The fjadra profile: a WASM crate must be fs-free, tokio-free, rayon-optional, and pure computation (data in → compute → data out).

## Rationale

### Why HTTP wins

1. **wm-core doesn't compile to wasm** — `tokio::fs` (file I/O for wiki pages), `rayon` (BM25 index build), `ort` (ONNX native C++ runtime), `turso` (SQLite), `walkdir`, `std::process::Command`. Porting would be months of feature-gating and fs shimming.

2. **Filesystem coupling is architectural** — the data model is `.md` files on disk. WASM in browser can't read them without File System Access API (Chromium-only) or OPFS (invisible to wm-cli and user's editor).

3. **Dual-engine problem** — AI agents mutate EngineState via MCP stdio while the UI is open. A browser WASM EngineState would diverge immediately. Fixing that requires HTTP or equivalent sync layer.

4. **Cold-start penalty** — singleton daemon pays cold start once per machine boot. Browser WASM pays it every page load and must rebuild index from files it can't read.

### What WASM is good for

Pure, stateless, CPU-bound computations called frequently (per-frame or per-interaction):

| Candidate | Why WASM fits |
|-----------|--------------|
| Force-directed layout (fjadra) | Per-frame tick loop — grotesque over HTTP |
| Graph algorithms on fetched subgraphs | Multi-call per view session |
| BM25 re-scoring of results | Interactive re-ranking without round-trip |
| Markdown/frontmatter parsing | Instant client-side rendering |

Each WASM addition should **delete its HTTP equivalent** — don't let both paths coexist.

### EnginePort: the abstraction, not the transport

The EnginePort pattern (InjectionToken with typed interfaces) provides the testability and transport-swappability benefits that the WASM proposal was reaching for — without actually changing the transport. It's the right abstraction seam.

## Consequences

- HTTP stays as the frontend↔backend transport
- WASM additions are evaluated individually against the fjadra profile
- WASM additions must delete their HTTP predecessor (no dual paths)
- The EnginePort abstraction provides testability without transport changes
- The architecture is kept simple: no CRDTs, no OPFS sync layer, no wasm-pack of the full engine

## Related

- @wiki/specs/http-wasm-architecture-cleanup
- @wiki/patterns/wasm-crate-integration
- @wiki/patterns/engine-port-backend-abstraction
- @wiki/patterns/canvas2d-wasm-graph
- @wiki/patterns/critical-patterns