---
title: Code Index Consumption — serve queries from the cache and surface what it drops
type: spec
id: "wiki:specs:code-index-cache-consumption"
status: draft
tags: [code-intel, index, performance, observability, spec]
---

schema_version: 1
state: |-
  ## Goal

  Make the existing `code.db` index actually get used, and make what it drops visible. Today the cache is built and maintained but only half-consumed, and every failure mode in the pipeline is silent.

  ## Verified current state (2026-10-15, checked against stable sources)

  | Tool / path | Reads `code.db`? | Evidence |
  |---|---|---|
  | `wm_code.search` with `edge_type` | yes, staleness-gated | `apps/wm-core/src/graph/code_edges.rs:38` calls `refresh_if_stale` |
  | `wm_code.symbols` | **no** — walkdir + re-parse per call | `apps/wm-core/src/mcp/tools/code.rs:379` |
  | `wm_code.deps` | **no** — walkdir + re-parse, resolution discarded | `apps/wm-core/src/mcp/tools/code.rs:602, 709` |
  | `wm_code.file` | no — raw read | `apps/wm-core/src/mcp/tools/code.rs:756` |

  Other verified facts: `refresh_if_stale` is called from exactly one site, so the staleness gate covers only the graph/edge path. `CodeIndexStats.errors` is hardcoded `Vec::new()` (`packages/wm-code-intel/src/services/ingest_service.rs:143`). Unresolvable edges are dropped with no count and no reason (`packages/wm-code-intel/src/services/graph_resolver.rs:147`). `materialize_resolved_edges` runs unconditionally every run (`ingest_service.rs:132`). `wm index rebuild` rebuilds graph, sections, BM25 and embeddings and never the code index (`apps/wm-core/src/mcp/tools/index.rs:138,157-174`). `wm_code.deps` accepts `--depth` and discards it (`code.rs:555` `let _depth = ...`).

  ## Relationship to existing decisions

  This spec **completes** accepted work rather than reversing it:

  - `@wiki/specs/code-index-cache` FR-1 and FR-2 require `wm_code.symbols` and `wm_code.deps` to query the turso cache instead of walking the filesystem. They are only partially wired.
  - `@wiki/specs/code-index-cache` FR-5 requires the code index to be part of `wm index rebuild`. Currently absent.
  - `@wiki/decisions/code-index-cache-architecture` (approved) chose turso-backed `code.db`, hash-skip, and a staleness signal; D3 specified a `stale` hint on query.
  - `@wiki/concepts/inert-cli-flags-silent-noop` documents this exact failure class for `--skip-hash-check`; `--depth` on `wm_code.deps` repeats it.

  ## Functional requirements

  - **FR-1** `wm_code.symbols` serves from `code.db` through SQL, behind the staleness gate. walkdir plus tree-sitter stays only as the no-index fallback.
  - **FR-2** `wm_code.deps` serves from the materialised `resolved_edges` table behind the same gate, instead of re-parsing and resolving on the fly and discarding the result.
  - **FR-3** Both responses carry an explicit staleness indicator whenever the served index may not match disk, so no path can silently answer from a mismatched index.
  - **FR-4** Edge resolution reports how many raw edges were dropped and by which reason; `CodeIndexStats` gains those counts and `wm-cli index code` prints them.
  - **FR-5** `CodeIndexStats.errors` is populated from per-file read and parse failures instead of being hardcoded empty.
  - **FR-6** The `wm index rebuild` code-step question is decided explicitly: either the code index becomes part of the rebuild, or the opt-in behaviour is recorded as a decision so the drift from FR-5 stops being accidental.

  ## Non-functional requirements

  - **NFR-1** A cached `wm_code.symbols` / `wm_code.deps` call must be measurably faster than the walkdir path on this repository; record before and after timings.
  - **NFR-2** Behaviour freeze: existing JSON shapes are preserved; new fields are additive only.
  - **NFR-3** Project rules apply: no comments, no `#[allow(...)]`, functions at most 20 lines, TDD with the failing test written first.
  - **NFR-4** The no-index case must still work end to end, since a fresh clone has no `code.db`.

  ## Design sketch

  Read path: reuse the existing `CodeIndexSnapshot::from_db` plus the `query_symbols` / `query_edges` helpers already in `packages/wm-code-intel/src/services/code_index_db.rs`, and call `crate::engine::code_index_refresh_service::refresh_if_stale` before serving — the same sequence the graph path already uses at `graph/code_edges.rs:38`. Fall back to the current walkdir implementation when the DB is absent or empty.

  Observability: have `resolve_code_edges` return a per-reason drop tally alongside the resolved set (the `_ => None` arm at `graph_resolver.rs:147` becomes a counted skip), extend `CodeIndexStats`, and print the tally in `print_code_index`.

  ## Out of scope

  - Progress output for long parses, a read-only `--status` or `--file` query, incremental (non-global) edge re-resolution, and folding `wm_lsp.*` into the index path. These are candidates for a later spec; none blocks this one.
  - Changing `--skip-hash-check` or any other existing `index code` flag.

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | Cached reads serve a stale index | wrong answers, the 10-day-old `code.db` class | FR-3 gate, plus a test asserting the gate runs before serving |
  | SQL result differs from the parse result | silent behaviour change | a test asserting identical symbol and edge output for the same file from both paths |
  | Drop tallies are computed but not surfaced | another silent metric | FR-4 acceptance requires the CLI to print them |
  | Touching `services/index/**` conflicts with the in-flight migration lane | merge churn | implement after migration step 3 completes |

  ## Verification

  TDD per requirement; before/after timings for NFR-1; `cli_test`, `cli_characterization_test` and the code-intel test suites stay green; the no-index path covered by an explicit test (NFR-4).

  ## References

  - `@wiki/specs/code-index-cache`
  - `@wiki/decisions/code-index-cache-architecture`
  - `@wiki/patterns/hash-skip-rebuild`
  - `@wiki/patterns/cli-delta-vs-total-reporting`
  - `@wiki/concepts/inert-cli-flags-silent-noop`
  - `@wiki/tasks/research-graphify-code-intel-edge-extraction-for-wm-adoption`
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: component
  status_class: draft
  needs_tasks: true
