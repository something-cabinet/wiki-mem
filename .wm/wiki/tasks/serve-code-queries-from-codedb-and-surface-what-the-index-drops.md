---
title: Serve code queries from code.db and surface what the index drops
type: task
id: "wiki:tasks:serve-code-queries-from-codedb-and-surface-what-the-index-drops"
status: todo
priority: high
tags: [code-intel, index, performance, observability, spec-linked]
spec: specs/code-index-cache-consumption
acceptance_criteria:
  - text: "wm_code.symbols serves from code.db through SQL with the staleness gate, with walkdir plus tree-sitter retained only as the no-index fallback"
  - text: "wm_code.deps serves from the materialised resolved_edges table behind the same gate instead of re-parsing and discarding"
  - text: "A test asserts the cached path and the walkdir path return identical symbols and edges for the same file"
  - text: "A test asserts the staleness gate runs before serving, and that responses signal staleness when the index may not match disk"
  - text: "Edge resolution reports a per-reason drop tally, CodeIndexStats carries it, and wm-cli index code prints it"
  - text: "CodeIndexStats.errors is populated from per-file read and parse failures instead of hardcoded empty"
  - text: "The wm index rebuild code-step question is closed: either the code index is included, or the opt-in is recorded as an explicit decision"
  - text: "Before/after timings recorded for a wm_code.symbols and wm_code.deps call on this repository"
  - text: "cli_test, cli_characterization_test and the code-intel suites stay green, and no comments or #[allow] are introduced"
---

Implementation task for `@wiki/specs/code-index-cache-consumption` (draft, awaiting approval). Reviewer selected the top three `wm-cli index code` enhancements after a verified recon.

**Item 1 — the cache is only half-consumed (highest value).** `wm_code.symbols` (`apps/wm-core/src/mcp/tools/code.rs:379`) and `wm_code.deps` (`:602, 709`) re-walk the filesystem and re-parse on every call, and `wm_code.deps` resolves edges on the fly then discards the result. `refresh_if_stale` is invoked from exactly one place (`apps/wm-core/src/graph/code_edges.rs:38`), so only the graph/edge path is gated. This completes `@wiki/specs/code-index-cache` FR-1 and FR-2, which were accepted but not fully wired.

**Item 2 — the pipeline is silent about what it drops.** `packages/wm-code-intel/src/services/graph_resolver.rs:147` drops unresolvable edges with `_ => None` and no count or reason; `CodeIndexStats.errors` is hardcoded `Vec::new()` (`packages/wm-code-intel/src/services/ingest_service.rs:143`). An operator cannot tell why edges are missing. This is the maintainer-facing diagnostic that `@wiki/specs/code-edge-resolution` deferred.

**Item 3 — `wm index rebuild` never rebuilds the code index.** `apps/wm-core/src/mcp/tools/index.rs:138,157-174` rebuilds graph, sections, BM25 and embeddings only, contradicting `@wiki/specs/code-index-cache` FR-5. Today it is a deliberate opt-in; this item makes that a recorded decision or closes the gap, so it stops being accidental drift.

**Ordering constraint:** `apps/wm-cli/src/services/index/**` is being rewritten by migration step 3 of `wiki:tasks:controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review`. Implement this only after that step lands, or the two will collide.