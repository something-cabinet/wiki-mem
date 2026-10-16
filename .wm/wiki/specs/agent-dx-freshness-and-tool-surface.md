---
title: Agent DX — Index Freshness, MCP Tool Surface, and Rebuild Guarantee
type: spec
id: "wiki:specs:agent-dx-freshness-and-tool-surface"
status: draft
tags: [agent-dx, freshness, mcp, tool-surface, index-integrity, doctor]
---

schema_version: 1
state: |-

  ## Overview

  Tier 1 of the agent-facing DX work identified from the peer-landscape research. Three related changes, all confined to surfaces WM already owns:

  1. **Index freshness is visible where it matters** — every read tool result says when the data it returned was indexed, and warns when it is stale.
  2. **The MCP tool surface gets a cost budget** — every `wm_*` tool is annotated, one composite entry point is added, and an opt-in lean profile advertises a curated core with a measured token report.
  3. **The index is provably a rebuildable projection** — a read-only `wm doctor` reports drift between disk and derived stores, and CI asserts a rebuild-from-pages is deterministic and complete.

  Problem being solved: an agent can currently receive tool output derived from a stale index with no signal in the response, pay the full schema cost of ~51 advertised tools every turn, and has no way to distinguish "the index is wrong" from "the wiki is wrong".

  Explicitly deferred to a later phase (not in this spec): hybrid retrieval fusion, non-destructive supersede / bi-temporal memory semantics, session-boot briefing, and interop/export formats.

  ## Locked Decisions

  - **D1** — Phased single spec. Tier 1 only: freshness surfacing (L5), tool surface cost (L4), rebuild guarantee (L2). Retrieval fusion and supersede semantics are a follow-up spec.
  - **D2** — Every search/code/retrieve/graph result carries freshness metadata (`indexed_at`, `stale_files`), with a warning only when something is stale. No refresh inside the request path.
  - **D3** — Annotate every `wm_*` tool; add a composite `wm_context` tool; add an opt-in `mcp.profile = lean` advertising a curated core with a published token measurement. Default remains `full` so no existing client breaks; any default flip is deferred and separately decided.
  - **D4** — `wm doctor` is read-only (drift detection with `file:line`, never repair). Repair stays an explicit `wm index` run. Plus a CI test asserting a rebuild-from-pages reproduces the same graph edges and BM25 results.

  Research framing: this spec names no external projects or sources. Supporting context lives in @wiki/reference/peer-landscape-2026

  ## Requirements

  ### Functional Requirements

  **Freshness**

  - **FR-1** — Every read tool whose output derives from an index returns a `freshness` object: `{ indexed_at: <timestamp|null>, stale_count: <int>, stale_files: [<path>...] }`. In scope: `wm_search.query`, `wm_search.retrieve`, `wm_search.resolve`, `wm_code_search`, `wm_code_symbols`, `wm_code_deps`, `wm_graph_neighbors`, `wm_graph_subgraph`, `wm_graph_affected`, `wm_graph_path`, `wm_page.get`, `wm_page.list`, `wm_task.list`, `wm_memory.list`.
  - **FR-2** — When `stale_count > 0`, the response also carries a warning naming up to 10 stale paths. Reuse the existing warning channel already used by search responses (the channel that reports degraded/semantic-unavailable today) rather than inventing a second one.
  - **FR-3** — When the freshness of a store cannot be determined, `indexed_at` is `null` and the warning states freshness is unknown. The tool still returns its normal results.
  - **FR-4** — Read tools must not mutate index state. The request path never triggers a rebuild.
  - **FR-5** — Freshness derives from metadata already persisted per derived store (index build timestamp + the set of source files changed since that build). It must not require re-walking the wiki on every call.

  **Tool surface**

  - **FR-6** — Every registered MCP tool declaration carries behavior annotations: read-only, destructive, idempotent, open-world. No tool may be registered without all four being explicitly set.
  - **FR-7** — A composite `wm_context` tool returns, within a caller-supplied token budget, one entity plus its typed neighbours and (for pages) matching code symbols. It is the single call an agent uses to orient on an entity. Whether it is a new tool or a distinct mode of the existing context-assembly path is an Open Question below — duplication of that path is not acceptable.
  - **FR-8** — Config supports `mcp.profile` with values `full` (default) and `lean`. `lean` advertises a curated core of at most 12 tools plus `wm_context` and `wm_help`; tool definitions absent from the advertised list remain callable by name for clients that already know them.
  - **FR-9** — A measurement path reports, per profile, the advertised tool count and the estimated schema token cost. The measured numbers are recorded in this spec before approval.
  - **FR-10** — Switching profiles is config-only: no rebuild, no restart requirement beyond re-listing tools.

  **Rebuild guarantee**

  - **FR-11** — `wm doctor` performs a read-only consistency check between the source of truth (wiki pages on disk) and the derived stores (wiki graph, BM25 index, code index). It reports, with `file:line` where applicable: pages on disk missing from the index, index entries with no page on disk, frontmatter/record parse failures, and code-index rows whose file no longer exists. It exits non-zero when it finds drift.
  - **FR-12** — `wm doctor` must not duplicate the existing lint and validate checks (orphans, missing ids, broken `wiki:*` refs). It reports drift between disk and derived state; it may reference the other checks in its output but must not reimplement them.
  - **FR-13** — `wm doctor` never writes. Repair is an explicit `wm index` invocation.
  - **FR-14** — A CI test rebuilds the derived stores from the wiki pages and asserts (a) two consecutive rebuilds produce identical graph edge sets and identical BM25 result sets for a fixed query corpus, and (b) every resolvable `@wiki/` reference in page bodies has a corresponding graph edge. It fails when a tree is edited in a way that changes derived state without a matching rebuild.
  - **FR-15** — Core architecture documentation states the invariant: the index is a derived, always-rebuildable projection of the pages, and no derived store is authoritative.

  ### Non-Functional Requirements

  - **NFR-1** — Freshness fields add under 1 KB to a response and no full-store scan per call.
  - **NFR-2** — `wm doctor` and the CI rebuild test run with no ONNX model, no network, and no model download.
  - **NFR-3** — No new runtime dependency, no daemon, no HTTP surface.
  - **NFR-4** — Full-profile tool count, names, and input schemas are unchanged; existing clients that call any current tool keep working.
  - **NFR-5** — Freshness metadata must never report success for a store that failed to build; an unbuilt or failed store reports unknown, not fresh.

  ## Acceptance Criteria

  - [ ] AC-1: A test enumerates every tool listed in FR-1 and asserts the response contains a `freshness` object with all three fields.
  - [ ] AC-2: With one page modified after the last index build, that tool's response contains a warning naming the modified path; against a fresh index no warning is emitted.
  - [ ] AC-3: Taking a hash of the derived stores before and after a sequence of read-tool calls yields no change (read tools do not mutate).
  - [ ] AC-4: `wm doctor` exits 0 on a clean tree; after deleting a page from disk while it remains indexed it exits non-zero and prints the page path with `file:line`; a snapshot of the working tree before/after the doctor run is identical.
  - [ ] AC-5: The CI rebuild test fails on a tree where a graph-producing edit was made without rebuilding, and passes on a clean tree.
  - [ ] AC-6: A registry test asserts every registered tool declares all four annotations; adding a tool without them fails the test.
  - [ ] AC-7: With `mcp.profile = lean`, `tools/list` returns at most 12 curated tools plus `wm_context` and `wm_help` (the exact list is frozen in a test); with `full`, the advertised set is byte-identical to today's.
  - [ ] AC-8: The measurement path prints tool count and estimated schema tokens for both profiles; the numbers are recorded in this spec.
  - [ ] AC-9: `wm_context` returns entity content plus typed neighbours within the requested budget, and the budget is respected within a stated tolerance.
  - [ ] AC-10: `core/ARCHITECTURE` documents the derived-index invariant (FR-15) and wiki lint stays clean.

  ## Scenarios

  ### Scenario 1: Fresh read (happy path)
  **Given** a wiki page was edited through a WM tool, so the index refreshed at the write path
  **When** an agent calls `wm_search.query`
  **Then** results return normally, `freshness.indexed_at` is recent, `stale_count` is 0, and no warning is emitted.

  ### Scenario 2: Stale read after an out-of-band edit
  **Given** a human edits a page with an external editor after the last index build
  **When** an agent calls `wm_search.query`
  **Then** results return, and the response carries a warning naming the edited path.

  ### Scenario 3: Unknown freshness
  **Given** a freshly cloned repository where no index has been built in this checkout
  **When** an agent calls `wm_code_search`
  **Then** the response reports `indexed_at: null` and warns that freshness is unknown, rather than claiming the results are current.

  ### Scenario 4: Doctor finds drift without touching anything
  **Given** a page was deleted from disk while its node remains in the graph
  **When** `wm doctor` runs
  **Then** it reports the stale index entry with its path, exits non-zero, and leaves every file unchanged.

  ### Scenario 5: Lean profile orientation
  **Given** a client configured with `mcp.profile = lean`
  **When** the agent asks "what is this project and what is in flight?"
  **Then** `tools/list` advertised a small curated set, and one `wm_context` call returns the project overview plus the in-flight tasks within the token budget.

  ### Scenario 6: Today's failure, for contrast
  **Given** the current build (no freshness metadata)
  **When** an agent trusts a code-search hit produced from an index older than the working tree
  **Then** the agent has no signal in the response — which is the defect this spec removes.

  ## Technical Notes

  - Freshness assembly belongs next to the existing per-store index metadata; the index tool already computes a code-index age, so that value is the natural seed rather than a new scan.
  - Warning output should reuse the response field already used to report degraded search modes.
  - Annotations attach at tool registration, so the registry is the single choke point for FR-6 and for the profile filter in FR-8.
  - `wm doctor` should be a thin composition over existing per-store read paths; its value is the disk↔derived comparison, not new parsing.
  - The CI test needs a fixed query corpus; keep it small and deterministic (NFR-2).
  - Config key lives beside the other MCP settings in the project config file.

  ## Open Questions

  - [ ] Is `wm_context` a new tool, or a distinct mode of the existing context-assembly path (which already does token-budgeted context assembly)? Duplicating that path would violate DRY. Resolve at plan time.
  - [ ] Which exactly are the ≤12 tools in the lean core? The list needs FR-9's measured token numbers before it can be frozen in AC-7.
  - [ ] Does the existing warning channel on search responses suffice for FR-2, or does the freshness warning need to be distinguishable from the degraded-mode warning? Pick one before implementation.
  - [ ] Which derived stores carry `indexed_at` — wiki graph and code index only, or also memory/task stores?
  - [ ] Should the full profile emit freshness metadata too, or only the lean profile? (Assumed: both.)
  - [ ] Does the default flip to `lean` in a later release, and what measurement would justify it?

  ## Related

  - @wiki/reference/peer-landscape-2026 — the research this spec is derived from
  - @wiki/core/ARCHITECTURE — receives the derived-index invariant (FR-15)
  - @wiki/rules/durable-write-or-error — grounds FR-3 and NFR-5 (no success without verified durable state)
  - @wiki/rules/no-compensating-layers — grounds FR-12 and D4 (detect and report; do not paper over)
  - @wiki/rules/tdd-red-green-refactor — the CI rebuild test and registry tests are the red step
  - @wiki/concepts/mcp-tool-unavailability-fallback — existing tool-surface constraint
- @wiki/tasks/code-index-goes-stale--affected-answers-from-a-10-day-old-codedb — the cancelled task this partially re-opens as a detectable condition