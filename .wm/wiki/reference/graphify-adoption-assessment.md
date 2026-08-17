---
title: Graphify Adoption Assessment
type: reference
id: wiki:reference:graphify-adoption-assessment
status: draft
tags: [reference, graphify, code-intel, edges, tree-sitter, assessment]
---

# Graphify Adoption Assessment

Verified against the working tree on branch `feat/graphify-gaps` (2026-08-14). Companion to @wiki/reference/graphify-architecture (which describes Graphify) — this page records what wiki-mem **actually has today** and what is worth adopting next, with evidence.

## 1. Edge taxonomy: what wm emits vs Graphify's 12 relations

wm-code-intel emits exactly 3 code edge types (`packages/wm-code-intel/src/models/code_edge_model.rs`): `calls`, `imports`, `inherits`. `CodeEdge::is_break_sensitive` gates all three for blast radius.

| Graphify relation | wm today | Adoption cost |
|---|---|---|
| imports_from (file→file) | covered by `imports` | none — same semantics |
| imports (file→symbol) | partial: `target_symbol` holds the raw path, not a resolved symbol node | low |
| calls | present but only bare-identifier callees (see §2) | — |
| indirect_call | absent | high (needs dispatch analysis) |
| inherits | Rust `impl Trait for T`, TS `extends`, Python superclass | present |
| implements | folded into `inherits` (Rust impl-trait is really `implements`) | low — split the edge type |
| references (field/param/return/generic/attribute/value) | absent | medium — biggest structural gain |
| re_exports | not an edge type; re-export chasing exists but only stamps `Derived` provenance on the `imports` edge (`graph_resolver::chase_reexport`, depth 2) | low — promote to a first-class edge |
| contains (file→symbol) | implicit via `CodeIntelSymbol.file`, not an edge | low |
| method (class→method) | absent | low |
| case_of (enum→case) | absent | low |
| decorates | absent | low (matters for Angular/Python) |

## 2. Verified defect: call edges miss the dominant call forms

`engine_service::extract_edges` uses `(call_expression function: (identifier) @name)` for Rust/TS/TSX/Go and `(call function: (identifier) @name)` for Python. In tree-sitter, `x.method()` is a `field_expression`/`member_expression` callee and `Type::assoc()` is a `scoped_identifier` — neither matches an `identifier` pattern, so neither produces an edge.

Measured on this repo (`rg -o` over `apps/` + `packages/`, `*.rs`):

- method-call sites (`.foo(`): 13,009
- path-call sites (`Foo::bar(`): 2,912
- bare-identifier call sites (heuristic, includes `fn` declarations): 3,838

So the call graph observes well under a quarter of Rust call sites, and in TypeScript (service/DI-heavy Angular code) member calls dominate even more heavily. This is why Graphify's **receiver-type inference** is not a refinement but a prerequisite: capturing member calls without it would emit mostly `Ambiguous` edges.

## 3. Resolution pipeline gap-by-gap

`packages/wm-code-intel/src/services/graph_resolver.rs` + `engine_service::resolve_import_candidates`.

| Graphify stage | wm status |
|---|---|
| Import path resolution | present per language: `resolve_rust_import` (progressive `::` prefix stripping so `crate::engine::run` → `src/engine.rs`), `resolve_ts_import` (relative paths), `resolve_python_import` |
| tsconfig alias / baseUrl | **absent** — no `tsconfig` read anywhere in the package |
| Workspace package globs (pnpm/npm/yarn) | **absent** — relevant here: this repo is a Cargo + npm workspace |
| Go imports | **`None` — deliberately deferred**, so Go import edges never resolve |
| Re-export chains | present, bounded at depth 2, stamps `Derived` + records `via` |
| Receiver-type inference | **absent** (see §2) |
| Disambiguation heuristic | **absent** — ties are resolved by sorted-first-wins (`defining_files[0]`), then flagged `Ambiguous`. Graphify's path-distance heuristic would pick the nearer candidate instead of alphabetical order |
| Provenance | present and 1:1 with Graphify: `Explicit`/`Derived`/`Ambiguous` (`wm_engine::models::edge_type_model::EdgeProvenance`), and unlike Graphify it is wired into search ranking (`search/query.rs::provenance_weighted_centrality`) |
| Wildcard imports (`use foo::*`) | dropped, documented as MVP-acceptable |

## 4. Language coverage

7 declared (`SupportedLanguage`: Rust, TypeScript, Tsx, Python, Go, Html, Svelte) but effective code-edge coverage is 5: HTML and Svelte return empty queries for imports/calls/inherits and are skipped outright in `CodeIndexSnapshot::collect_from_fs`. Go has calls but no import resolution. Inherits covers Rust/TS/TSX/Python only.

Conclusion on Graphify's 30+ languages: **not worth chasing breadth.** The languages that matter for this repo (Rust, TS/TSX) are already present; the deficit is depth per language, not language count. The only breadth item with real local value is Svelte/HTML template→component references (the Angular/Svelte UI is invisible to the graph today).

## 5. Analysis layer

Absent in wm: Leiden/Louvain community detection, god-node detection, surprising connections, import-cycle detection, graph diff, suggest_questions. The only cycle handling is an informational note in `graph/lint.rs` for the wiki graph (mutual `relates_to` is expected, BFS uses visited-tracking).

Present and arguably more valuable than clustering: `graph/affected.rs` blast radius (wiki `depends_on`/`extends` via incoming edges; code `calls`/`inherits`/`imports` behind the `code-intel` feature) with per-hop provenance and `file:line`, exposed via CLI `wm graph affected`, MCP `wm_graph`, and the HTTP route. Export breadth also landed: `graph/export.rs` does JSON, hand-written GraphML (no new dependency), and an Obsidian vault.

Assessment: **import-cycle detection** is the one analysis item with a clear payoff, and it is nearly free once member/path call edges exist — plus Graphify's `deferred` flag for dynamic `import()` is what makes cycle output trustworthy. Community detection is a visualization feature against a 668-node wiki graph; it would not change agent behavior. God-nodes are a one-line degree sort over an existing graph, cheap to add if wanted, low value while the call graph is incomplete.

## 6. Other adoptable patterns

- **Deferred imports**: absent. Cheap, and a precondition for honest cycle detection.
- **Extraction schema validation** (`validate.py`): absent. wm's equivalent safety net is the type system plus `tests/code_edges.rs`; a validation pass would mostly catch query-vs-grammar drift (exactly the §2 bug class), so the cheaper fix is fixture tests asserting each call form produces an edge.
- **MinHash dedup**: absent, and not needed — wm indexes incrementally by content hash (`wm-embed/vector_db.rs::content_hashes`), which solves the redundant-work problem MinHash solves for Graphify's batch runs.
- **Origin-file stamps**: partially present — `ResolvedCodeEdge.via` already carries traversed files for derived edges.
- **Performance note**: `wm_code.search` and the graph code path build the snapshot via `CodeIndexSnapshot::collect_from_fs` (full walk + reparse per call) even though `code.db` exists and `CodeIndexSnapshot::from_db` is implemented. Preferring `from_db` with an FS fallback would be a straight win.

## Recommendations (ranked)

1. **Capture member and path calls, then add receiver-type inference for Rust + TS.** Without it the call graph — and therefore blast radius — is unsound for the code this repo is written in. See @wiki/tasks/code-call-edges-miss-member-and-path-calls.
2. **Add path-distance disambiguation** to replace alphabetical first-wins in `resolve_symbol_edge`/`resolve_import`. Small, deterministic, immediately reduces `Ambiguous` volume (which now costs 0.25x ranking weight).
3. **tsconfig alias + workspace resolution.** This repo's Angular app uses path aliases; without them TS import edges silently drop.
4. **Split `implements` out of `inherits`** and add `references` edges with typed contexts (field/param/return/generic). Highest structural value after calls; enables "what uses this type" queries the LSP tools currently answer per-file only.
5. **Deferred-import flag + import-cycle detection.** Only after (1), otherwise cycles are computed over a fraction of the graph.
6. **Read code edges from `code.db`** instead of re-walking the filesystem per tool call.

Explicitly **not recommended**: Leiden clustering, MinHash dedup, 30+ language breadth, LLM-augmented extraction (deterministic-local posture is locked by NFR-2.1 of @wiki/specs/graphify-gap-closure; the LLM question is tracked separately in @wiki/tasks/research-graphify-local-llm-usage-for-wm-code-intel-augmentation).

## Related

- @wiki/reference/graphify-architecture — what Graphify does
- @wiki/specs/graphify-gap-closure — the six-gap roadmap (items 1, 2, 5, 6 have landed)
- @wiki/tasks/research-graphify-code-intel-edge-extraction-for-wm-adoption — this research task
- @wiki/concepts/edge-types — wiki-mem's 9 wiki edge types
