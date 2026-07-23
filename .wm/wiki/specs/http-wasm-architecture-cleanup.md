---
title: HTTP/WASM Architecture Cleanup
type: spec
tags: [architecture, angular, wasm, http, engine-port]
---

---
title: HTTP/WASM Architecture Cleanup
type: spec
status: approved
tags: [architecture, angular, wasm, http, engine-port]
---

# HTTP/WASM Architecture Cleanup

## Overview

Refactor the Angular frontend's communication layer and clean up stale HTTP endpoints following the Oracle architecture review. Three related changes: introduce an `EnginePort` abstraction for the backend transport, extend WASM to additional pure-compute candidates following the fjadra pattern, and remove dead layout-related HTTP code.

## Locked Decisions

- D1: **Broad EnginePort** — abstract class/interface with typed response interfaces (no `any`), plus `MockEngineService` for component tests
- D2: **All three WASM candidates in scope** — graph algorithms on fetched subgraphs, BM25 re-scoring, markdown/frontmatter parsing
- D3: **Thorough layout cleanup** — delete server route + handler + SSE stub, Angular `computeLayout()`, all mock server mappings and JSON stub files

## Requirements

### Functional Requirements

#### FR-1: EnginePort Abstraction
- FR-1.1: Define `EnginePort` as an Angular `InjectionToken` with typed method signatures replacing `any` returns
- FR-1.2: Extract typed response interfaces from the current `ApiService` (e.g., `GraphFullResponse`, `SearchResponse`, `PageResponse`)
- FR-1.3: Rename `ApiService` → `HttpEngineService` and implement `EnginePort` using fetch
- FR-1.4: Provide `HttpEngineService` as the production implementation via `@Injectable({ providedIn: 'root' })`
- FR-1.5: Keep `api.service.ts` file as a thin re-export shim or migrate all consumers directly to `EnginePort`

#### FR-2: MockEngineService
- FR-2.1: Implement `MockEngineService` that returns canned responses for all `EnginePort` methods
- FR-2.2: Canned responses must match the typed interfaces (no `as any` casts)
- FR-2.3: `MockEngineService` must be usable in standalone component tests without a running mock server

#### FR-3: WASM Extended — Graph Algorithms
- FR-3.1: Create a wasm-bindgen crate `packages/graph-algo-wasm` wrapping `wm-engine` (petgraph) for client-side graph operations
- FR-3.2: Expose BFS shortest-path, neighbor extraction, and subgraph extraction on an already-fetched subgraph
- FR-3.3: Wire into the graph view component: after fetching full graph via HTTP, run path/neighbor/subgraph queries locally in WASM
- FR-3.4: Build follows the fjadra pattern: `wasm-pack build --target web`, output in `apps/wm-web/src/assets/wasm/`

#### FR-4: WASM Extended — BM25 Re-scoring
- FR-4.1: Create a wasm-bindgen crate `packages/bm25-rerank-wasm` for client-side BM25 scoring on fetched result sets
- FR-4.2: Accept a list of documents + query, return re-ranked scores
- FR-4.3: Wire into the search view for interactive re-ranking without round-trip

#### FR-5: WASM Extended — Markdown/Frontmatter Parsing
- FR-5.1: Create a wasm-bindgen crate `packages/md-parse-wasm` for client-side markdown frontmatter extraction and rendering
- FR-5.2: Parse YAML frontmatter + markdown body from raw text
- FR-5.3: Wire into page view for client-side rendering of wiki content

#### FR-6: Layout Cleanup
- FR-6.1: Remove `POST /api/graph/layout` route and handler from `wm-server/src/routes/layout.rs`
- FR-6.2: Remove `GET /api/graph/layout/{job_id}/events` SSE stub from `wm-server`
- FR-6.3: Remove `layout.rs` file entirely
- FR-6.4: Remove `computeLayout()` method from `api.service.ts` / `EnginePort`
- FR-6.5: Remove layout mock mappings + JSON stub files from `packages/wm-mock-server/mappings/`
- FR-6.6: Remove `layout` route registration from `wm-server/src/routes/mod.rs`

### Non-Functional Requirements
- NFR-1: All WASM crates must be fs-free, tokio-free, and rayon-optional (follow fjadra profile)
- NFR-2: EnginePort must not increase initial bundle size beyond +2KB (it's an interface + existing impl)
- NFR-3: WASM crates must be lazy-loaded (dynamic `import()`) — not blocking initial page load
- NFR-4: All existing e2e journeys must pass without modification
- NFR-5: All existing component unit tests must pass after migration to EnginePort

## Acceptance Criteria

- [ ] AC-1: `EnginePort` is defined as `InjectionToken<EnginePort>` with typed interfaces (zero `any` on public API)
- [ ] AC-2: `HttpEngineService` implements `EnginePort` and functions identically to current `ApiService`
- [ ] AC-3: `MockEngineService` returns typed canned responses usable in TestBed component tests
- [ ] AC-4: No production code imports `ApiService` — all consumers use `EnginePort` via `@Inject()`
- [ ] AC-5: `computeLayout()` removed from Angular, no compile errors
- [ ] AC-6: Layout route + SSE stub removed from server, server builds and runs
- [ ] AC-7: Layout mock JSON files removed from mock server, remaining e2e tests pass
- [ ] AC-8: `packages/graph-algo-wasm` compiles and exposes graph algorithms
- [ ] AC-9: `packages/bm25-rerank-wasm` compiles and exposes re-ranking
- [ ] AC-10: `packages/md-parse-wasm` compiles and exposes parsing
- [ ] AC-11: Each WASM crate lazy-loads via dynamic `import()` (no blocking)
- [ ] AC-12: Existing e2e journeys pass (navigation, search, graph, tasks, pages, memory, settings)

## Scenarios

### Scenario 1: Cleanup — Layout Endpoint Removal
**Given** a running wm-server
**When** a client sends `POST /api/graph/layout`
**Then** the server returns 404 (route no longer exists)

### Scenario 2: Cleanup — Frontend No Longer Calls Layout
**Given** the Angular app is running
**When** the graph view loads
**Then** no network request to `/api/graph/layout` is made

### Scenario 3: EnginePort — Component Test
**Given** a component that uses `EnginePort`
**When** a unit test provides `MockEngineService`
**Then** the component renders with canned data, no HTTP calls emitted

### Scenario 4: WASM — Graph Algorithms Client-Side
**Given** the user has fetched the full graph via HTTP
**When** they view a node's neighbors or path between nodes
**Then** the computation runs in-browser via WASM, no additional HTTP request

### Scenario 5: WASM — Lazy Loading
**Given** the user loads the Angular app
**When** they navigate to the graph view
**Then** WASM modules are loaded via dynamic `import()` (observable in Network tab)
**And** the search view does NOT load graph WASM

## Technical Notes

Oracle review reference: session ora-1 / ses_07622a88bffeXv84BfrxIaeowA
Reference implementation for WASM pattern: `packages/fjadra-wasm/`

## Open Questions

<none — all decisions locked during exploration>
