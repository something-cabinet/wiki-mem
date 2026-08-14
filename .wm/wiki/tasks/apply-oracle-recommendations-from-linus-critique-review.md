---
title: Apply Oracle recommendations from Linus critique review
type: task
id: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review
status: todo
priority: high
tags: [review, refactor, cleanup, from-oracle, simplification]
acceptance_criteria:
  - text: "Reciprocal edge storage removed: graph/mod.rs body-ref reciprocal pass deleted, one edges_undirected helper serves all 5 consumer sites (query.rs:67, affected.rs:75, export.rs:66, graph tools, UI)"
  - text: "Single frontmatter builder choke point: all 7 string-built create paths routed through one typed build_frontmatter (task create, task subtask, page, doc, memory, decision, lint auto-fixer)"
  - text: "Ranking measured then reduced: golden-query eval (30-50 queries, recall@5 in CI, non-blocking), pre-normalize rerank deleted, dead enrich_search_results_from_graph deleted"
  - text: "retire-wm-doc executed: wm_doc becomes deprecated alias over wm_page path, doc.rs + parity tests deleted (~650 lines)"
  - text: "MCP SDK conformance smoke test in CI: initialize, tools/list, tools/call from official TS SDK; rmcp re-evaluated at next protocol bump"
  - text: "All deletions verified: cargo build, clippy -D warnings, targeted suites green"
---

Apply the Oracle's verdict on the Linus critique (ora-1 analysis of "layers of stupidity stacked, each compensating the other"). Verified findings with evidence:

Claim 1 (LANDED): reciprocal backlink edges stored at graph/mod.rs:150-163 tagged Derived, but the reverse view is one iterator call away (query.rs:67-70, affected.rs:75, export.rs:66-68 already use edges_directed Incoming). Stored transpose drags: double-counted degree in exports, 0.5 ranking weight, UI legend tier, phantom-source bug (graph/mod.rs:186-199), and only covers body @wiki refs not frontmatter relates_to (incoherent semantics). Fix: delete the reciprocal pass, one edges_undirected helper.

Claim 2 (LANDED on structure, grep justified): 7 call sites build string frontmatter with inconsistent quoting — task create (task/mod.rs:329, tags/assignee/parent/spec raw), task subtask (task/mod.rs:729-734), page (page/mod.rs:154), doc (doc.rs:492-499), memory (memory.rs:310), decision (decision.rs:73), and the lint auto-fixer itself (lint.rs:35 — repair tool can author corruption). yaml_helper adoption is opt-in; no create-path choke point. NOTE: the CI grep ban on allow(dead_code) is NOT redundant — allow_attributes_without_reason enforces reasons, not a ban; keep the grep (minor: CI-only scope, whitespace-evadable).

Claim 3 (LANDED): pre-normalize boosts (+8/+4/+2/+7/+3) duplicate post-RRF intent (bm25_index_service.rs:354-355 admits it); enrich_search_results_from_graph still exported and uncalled (search/mod.rs:6, query.rs:396 comment) — live wiring copied its comparator (query.rs:405-414 vs 93-102). Zero measurement infrastructure (no recall@k eval anywhere). Tie-break tiers are near-inert and harmless (keep, they're display metadata). Fix: golden-query eval harness, then delete pre-normalize rerank + dead comparator, measuring each deletion.

Claim 4 (LANDED on wm_doc, partially on MCP): doc.rs duplicates parse_frontmatter (doc.rs:459-486 vs crate::parser used at :253) and build_markdown exists solely to byte-imitate wm_page (re-introducing unquoted-tags bug at :497); parity tests pin byte-identity of two writers (mcp_test.rs:407-429) making retire-wm-doc spec harder. MCP hand-rolled by spec tradeoff (NFR-3.1 no-new-deps) — clean impl, but no SDK interop test (P3 m-7).

Sources: ora-1 session analysis (2026-08-14), spec retire-wm-doc (draft, already written), decisions clippy-lint-curated-list-not-all (2026-08-14 amendment).