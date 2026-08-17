---
title: Graphify Gap Closure
type: spec
id: wiki:specs:graphify-gap-closure
status: approved
tags: [spec, graph, provenance, code-intel, mcp, export, approved]
---

# Graphify Gap Closure

## Overview

Close the six capability gaps identified in the Graphify (Graphify-Labs/graphify) gap analysis: (1) edge provenance, (2) typed cross-file code edges, (3) shared-HTTP MCP server, (4) query-before-grep agent hooks, (5) export breadth, (6) blast-radius / impact analysis. All items target MVP depth (D3). The unifying principle borrowed from Graphify: make the graph's trust and structure explicit — where an edge came from, what it means, and what breaks — and feed that signal into the existing retrieval pipeline instead of bolting on new surfaces.

## Locked Decisions

- D1: Spec covers all six gaps as one roadmap (single spec, per-gap requirements).
- D2: Edge provenance semantics map 1:1 to Graphify (three levels), reworded for wiki-mem edge sources:
  - `explicit` — authored: `relates_to` frontmatter and `@wiki/` refs written by a human/agent in page content (Graphify `EXTRACTED`)
  - `derived` — engine-generated: reciprocal backlink edges and auto-created edges (Graphify `INFERRED`)
  - `ambiguous` — resolution hit multiple candidate targets; edge target uncertain (Graphify `AMBIGUOUS`)
- D2b: Provenance integrates into the existing scoring pipeline (docs/search-scoring-formula.md) as a signal, not display-only metadata. The provenance factor MUST affect live search ranking (`wm_search.query` / HTTP search), not only an enrichment helper.
- D3: All six gaps at MVP depth (see per-gap scope below).

## Requirements

### 1. Edge provenance (MVP)

#### Functional Requirements

- FR-1.1: Every graph edge carries a `provenance` attribute: `explicit | derived | ambiguous`, with source-based defaults (frontmatter `relates_to` and body `@wiki` refs → `explicit`; reciprocal/auto backlinks → `derived`; multi-candidate resolution → `ambiguous`). Unresolved refs (zero candidates) remain dropped with a warning — no edge is created without a target node (no phantom nodes). [Amended per P1 Oracle gate]
- FR-1.2: Provenance is persisted in the edge model (graph/mod.rs) and survives index rebuild (recomputed deterministically from the same sources).
- FR-1.3: Provenance is exposed read-only through graph MCP tools (`wm_graph.neighbors`, `wm_graph.full`, `wm_graph.subgraph`) and the HTTP graph routes.
- FR-1.4: Graph view (Angular, fjadra-wasm) renders provenance (e.g., line style/legend) without changing layout behavior.
- FR-1.5 (scoring integration, D2b): The graph-centrality term in the search pipeline (edge-type-weighted inbound, apps/wm-core/src/search/query.rs) multiplies edge weight by a provenance factor (`explicit` = 1.0, `derived` = 0.5, `ambiguous` = 0.25; values configurable). The factor MUST be wired into the live `run_unified_search` ranking sort so actual query results reflect provenance; a test asserts the live comparator consumes the weighted centrality. Documented in docs/search-scoring-formula.md. [Live-path wiring added per P1 Oracle gate — pre-P4 acceptance]

#### Non-Functional Requirements

- NFR-1.1: Zero change to markdown-first storage — pages remain source of truth; provenance is derived, never stored as graph.json.
- NFR-1.2: No regression in existing search behavior when provenance weights are neutral (all edges `explicit`), verified by existing search tests.
- NFR-1.3: Provenance derivation adds no file-watcher latency beyond current rebuild budget (recomputed in the same graph rebuild pass).

#### Acceptance Criteria

- [ ] AC-1.1: A fixture wiki with an authored `relates_to` edge, a reciprocal backlink, and an intentionally ambiguous ref shows correct provenance on all three edges in `wm_graph.full`.
- [ ] AC-1.2: Unit test: with identical docs, centrality contribution of an `ambiguous` edge is 0.25x an `explicit` edge, and search ranking reflects the difference — through the live search path (`wm_search.query`), not only the centrality helper.
- [ ] AC-1.3: `wm_validate.check` passes on the fixture wiki after provenance introduction (no new lint/validation failures).

### 2. Typed cross-file code edges (MVP)

#### Functional Requirements

- FR-2.1: Extend wm-code-intel (tree-sitter, existing 7 languages: Rust, TS, TSX, Python, Go, HTML, Svelte) to emit typed cross-file edges: `calls`, `imports`, `inherits` (where resolvable per language).
- FR-2.2: Code edges land in the same graph with provenance (`explicit` where the AST shows the direct reference, `derived` for resolution via re-exports/indirection, `ambiguous` on multi-candidate symbol resolution).
- FR-2.3: Exposed via `wm_code.deps` / `wm_code.search` and graph tools; source locations (`file:line`) on edges.

#### Non-Functional Requirements

- NFR-2.1: Extraction stays deterministic and local (no LLM calls), matching Graphify's code-only posture.
- NFR-2.2: Incremental — only changed files re-extract; no full re-index on single-file edits.

#### Acceptance Criteria

- [ ] AC-2.1: Rust + TS fixtures with a known call chain produce `calls` edges with correct file:line and provenance.
- [ ] AC-2.2: `wm_graph.neighbors` on a code symbol returns typed code edges alongside wiki edges.
- [ ] AC-2.3: Re-extraction after one file edit is incremental (only the edited file's edges change).

### 3. Shared-HTTP MCP server (MVP)

#### Functional Requirements

- FR-3.1: `wm mcp` gains an HTTP transport mode (`--transport http`, port configurable) serving the existing 50-tool surface over the wm-server axum runtime.
- FR-3.2: Auth via the existing shared token (`x-wm-token`) / API-key header; CSRF guard reused.
- FR-3.3: stdio mode unchanged (default), backward compatible.

#### Non-Functional Requirements

- NFR-3.1: No new dependencies beyond the existing axum stack (MCP Streamable-HTTP shape if feasible, else JSON-RPC over HTTP — decision in Open Questions).

#### Acceptance Criteria

- [ ] AC-3.1: An MCP client can connect over HTTP and call `wm_search.query` with the token.
- [ ] AC-3.2: Unauthenticated requests are rejected.
- [ ] AC-3.3: All existing stdio MCP tests pass unchanged.

### 4. Query-before-grep agent hooks (MVP)

#### Functional Requirements

- FR-4.1: `wm setup <platform>` emits hook/instruction config for supported platforms (opencode, claude, etc.) instructing the agent to query `wm_graph`/`wm_search` before falling back to file greps.
- FR-4.2: Optional `--strict` mode that blocks the first raw file read until a graph query has been issued in the session (per-platform capability permitting).

#### Non-Functional Requirements

- NFR-4.1: No changes to the MCP tool surface itself.

#### Acceptance Criteria

- [ ] AC-4.1: `wm setup opencode --strict` produces a config where the first raw read requires a prior graph query (verified via config inspection in a fixture agent run).
- [ ] AC-4.2: Non-strict setups emit the guidance as instructions only.

### 5. Export breadth (MVP)

#### Functional Requirements

- FR-5.1: `wm graph export` subcommands: `--json` (graph dump snapshot), `--graphml` (Gephi/yEd), `--obsidian` (vault with pages + link edges).
- FR-5.2: Exports are snapshots — never a storage format; markdown pages stay canonical (per earlier graph.json decision).

#### Acceptance Criteria

- [ ] AC-5.1: Exported JSON validates against the graph schema; GraphML opens in a standard reader.
- [ ] AC-5.2: `--obsidian` produces a vault where Obsidian-style double-bracket wikilinks match wiki-mem edges (provenance preserved as edge metadata where format allows).

### 6. Blast-radius / impact analysis (MVP)

#### Functional Requirements

- FR-6.1: `wm graph affected <node>` returns the transitive breakage set: nodes reachable via break-sensitive edges (`calls`, `inherits`, `imports` for code; `depends_on`/`extends` for wiki pages), with the path and provenance of each hop. `affected` operates per graph — the wiki and code graphs are disjoint, so a single query returns wiki hops or code hops, never a cross-domain chain. [Per-graph note added per P2 Oracle gate]
- FR-6.2: Exposed via graph MCP tool + HTTP route (LLM PR triage explicitly out of scope for MVP).

#### Acceptance Criteria

- [ ] AC-6.1: Fixture: removing a function node lists all transitively affected symbols with edge paths.
- [ ] AC-6.2: Wiki-page dependencies (`depends_on`, `extends`) are included in the affected set.

## Scenarios

### Scenario 1: Agent weighs ambiguous context (items 1+2b)
**Given** a wiki where a frontmatter `relates_to` target resolves ambiguously (two candidate pages — body `@wiki` refs are full-id exact matches and cannot be multi-candidate) and a search query matching both
**When** the agent runs `wm_search.query`
**Then** the ambiguous edge contributes 0.25x centrality vs an explicit one, and the live search result set reflects the provenance-weighted ranking — the agent sees grounded results first.

### Scenario 2: Blast radius before a refactor (items 2+6)
**Given** a Rust fixture with a call chain A→B→C
**When** the agent queries `wm graph affected A`
**Then** B and C are returned with `calls` edges, file:line, and provenance per hop.

### Scenario 3: Team shares the engine over HTTP (item 3)
**Given** `wm mcp --transport http` running on a dev machine with a token
**When** a second machine's MCP client connects with the token
**Then** all 50 tools respond; unauthenticated calls are rejected with 401.

## Technical Notes

- Provenance lives on edges in the existing typed-edge model (graph/mod.rs:84-153); no new page types.
- Scoring integration point: the graph-centrality term of the hybrid pipeline (apps/wm-core/src/search/query.rs) — provenance factor multiplies the existing edge-type weight, wired into the live `run_unified_search` comparator; formula update lands in docs/search-scoring-formula.md.
- Code-edge extraction extends packages/wm-code-intel (engine_service.rs) symbol/deps pass; reuse existing tree-sitter grammars.
- HTTP MCP reuses wm-server's axum runtime and token service (routes/mod.rs, web_token_service.rs).
- Exports read from the live graph snapshot (arc-swap) — no storage change.

## Open Questions

- [ ] Provenance factor values (1.0 / 0.5 / 0.25) — keep as defaults or expose per-editor config?
- [ ] Provenance tag naming: `explicit|derived|ambiguous` vs keeping Graphify's `EXTRACTED|INFERRED|AMBIGUOUS` in the API?
- [ ] HTTP MCP: Streamable-HTTP shape vs plain JSON-RPC over HTTP for MVP?
- [ ] Does item 2 (code edges) ship with provenance in the same release, or provenance-first then code edges?

## Related

- Gap analysis source: Graphify research (lib-2 session), capability inventory (exp-1 session)
- Existing pipeline doc: docs/search-scoring-formula.md
- Branch: feat/graphify-gaps
- P1 Oracle gate amendments: FR-1.1 wording, FR-1.5 live-path wiring, Scenario 1 premise (2026-08-13)
- P2 Oracle gate amendments: FR-6.1 per-graph note (2026-08-13)
