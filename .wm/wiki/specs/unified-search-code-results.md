---
title: Unified Search — code results alongside docs, without blending retrieval models
type: spec
id: "wiki:specs:unified-search-code-results"
status: draft
tags: [search, code-intel, mcp-tooling, spec]
---

schema_version: 1
state: |-
  ## Goal

  Give agents one entry point for "where is X handled?" that returns both wiki docs and source code, **without** blending two different retrieval models. Reviewer decision 2026-10-15: route code through `code.db` as a separate search leg fused into the result list — do not put code symbols into the BM25 corpus.

  ## Verified current state (2026-10-15)

  | Fact | Evidence |
  |---|---|
  | `wm_search.query` returns docs only | `SearchType` is `All | Page | Task | Memory` — no `Code` (`apps/wm-core/src/mcp/tools/search.rs:7-12`) |
  | The search path has no code integration at all | zero references to code in the search tool |
  | BM25 corpus is wiki sections only | `build_sections_from_wiki` (`apps/wm-core/src/mcp/tools/index.rs:159-166`) |
  | A code index already exists with symbols and resolved edges | `.wm/state/code.db`, five tables, materialised `resolved_edges` (`packages/wm-code-intel/src/services/code_index_db.rs`) |
  | A staleness gate already exists but covers only the graph path | `refresh_if_stale` called from `apps/wm-core/src/graph/code_edges.rs:38` only |
  | Code search is a separate tool today | `wm_code.search` (regex, plus AST-aware `edge_type`) |

  Consequence: an agent must know two tools exist, and the code one is feature-gated.

  ## Relationship to existing work

  - `@wiki/specs/agent-guidance-wm-mcp-search` (draft, untracked) solves the **discoverability** half by documenting that agents should use the `wm_code.*` tools, plus freshness semantics. That spec is **extended, not superseded** by this one: guidance becomes "search covers code, and here is how freshness works" rather than "there is also a code tool somewhere".
  - `@wiki/specs/code-index-cache-consumption` (draft) makes `code.db` the read path for code queries. This spec depends on it: a code search leg that reads the cache is only as good as the cache being read consistently.
  - Cross-entity search across pages, tasks, memory and docs already exists from `@wiki/tasks:4hk4kz` and `@wiki/tasks:jinq2x`; this adds a fourth leg rather than a new pipeline.

  ## Functional requirements

  - **FR-1** A `code` search type returns results from `code.db`: symbol matches (name and path), and resolved-edge matches where a symbol has typed edges.
  - **FR-2** Code results carry their own shape — file, symbol, kind, line, and edge kind plus provenance where applicable — not a page id and section.
  - **FR-3** Code results are a **separate retrieval leg** fused into the ranked list, labelled by source. Code symbols must **never** enter the BM25 corpus or its field weights.
  - **FR-4** `all` includes the code leg **only when the code index is available and fresh**. A stale or missing index must be surfaced explicitly in the response; silently omitting code is not acceptable.
  - **FR-5** The freshness contract is explicit: any code-bearing response states whether the staleness gate ran and the index age.
  - **FR-6** Clean degradation when built without the `code-intel` feature, or before any index exists: docs are returned and the absence of code is stated.
  - **FR-7** `@wiki/specs/agent-guidance-wm-mcp-search` is updated so the guidance describes unified search plus the freshness model, instead of pointing at a second tool.

  ## Non-functional requirements

  - **NFR-1** The code leg runs only for `type: code` or `type: all`; a doc-only query must not pay for it.
  - **NFR-2** Existing result shapes are preserved; new fields are additive only.
  - **NFR-3** `SearchType` gaining a variant changes the MCP tool schema. The schema must remain a valid top-level object; this project has already had a client-rejected `tools/list` from an invalid `input_schema`, so verify the emitted schema explicitly.
  - **NFR-4** Project rules: no comments, no `#[allow(...)]`, functions at most 20 lines, TDD with the failing test first.

  ## Design sketch

  The search router gains a fourth leg. For `type: code`, or for `all` when the index is fresh, it queries `code.db` through the existing helpers and the `refresh_if_stale` gate, producing code results with a `source` marker. Fusion reuses the existing rank-merge so legs stay independent and scores are not compared across models. Symbol matching is exact-prefix oriented, not fuzzy prose matching — that is the whole reason for keeping it out of BM25.

  ## Out of scope

  - Embedding/semantic search over code, AST-aware queries (that remains `wm_code.search`), and any change to BM25 field weights.
  - Progress output for indexing, and the other `index code` enhancements, which belong to `@wiki/specs/code-index-cache-consumption`.

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | Code noise in a blended ranked list | doc results degraded for everyone | separate leg, labelled source, NFR-2 |
  | Stale code served through search | the 10-day-old index class of bug, now inside the most-used tool | FR-4 and FR-5 make staleness explicit; gate before serving |
  | Schema change breaks a client | `tools/list` rejection | NFR-3 explicit schema verification |
  | Feature-gated build returns confusingly empty code results | agents conclude the codebase has no such symbol | FR-6 requires a stated reason |

  ## Verification

  TDD per requirement; a test that code results never appear in the BM25 corpus; a test that a stale index is reported rather than silently dropped; a test for the no-feature and no-index paths; emitted-tool-schema check for the new `SearchType` variant; before/after timing for a doc-only query to prove NFR-1.

  ## References

  - `@wiki/specs/agent-guidance-wm-mcp-search`
  - `@wiki/specs/code-index-cache-consumption`
  - `@wiki/specs/code-index-cache`
  - `@wiki/decisions/code-index-cache-architecture`
  - `@wiki/tasks:4hk4kz`, `@wiki/tasks:jinq2x` (cross-entity search)
  - `@wiki/reference:typed-decision-record-schema`
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
