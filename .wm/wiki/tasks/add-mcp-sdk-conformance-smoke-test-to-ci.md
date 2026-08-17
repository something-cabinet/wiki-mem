---
title: Add MCP SDK conformance smoke test to CI
type: task
id: wiki:tasks:add-mcp-sdk-conformance-smoke-test-to-ci
status: done
priority: low
tags:
- from-oracle
- test
- mcp
- linus-remediation
parent: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review
acceptance_criteria:
- text: CI smoke test drives initialize, tools/list, tools/call through the official TS MCP SDK against the daemon
- text: m-7 (P3 Oracle) converted from open risk to a passing test
- text: rmcp dependency re-evaluated and decision recorded at next protocol bump
- text: cargo build + clippy + mcp_http suite green
implementation_notes: 'Wave 1 review gate: GO-with-findings (3xP2, all non-blocking). Conformance harness VERIFIED PASS end-to-end locally (Node v24.15.0, wm-server debug binary spawned on hermetic free port 56622, /api/health readiness, web-token read, initialize 2025-06-18 echoed, tools/list=51 tools, tools/call wm_search.query isError=false). CI step present in check job. protocolVersion-negotiation P2 (harness masks the 2024-11-05 fallback) already tracked in wiki:tasks:fix-mcp-protocolversion-negotiation--negotiate-highest-mutually-supported; add the negotiated-version assert there once the Rust fix lands. ACs satisfied.'
---

From wiki:tasks:apply-oracle-recommendations-from-linus-critique-review AC-5. Oracle verdict PARTIALLY LANDED: HTTP MCP transport hand-rolled (mcp.rs, 231 lines) by spec-level tradeoff (NFR-3.1 locked no new deps beyond axum) — implementation is clean (stateless subset is legal Streamable-HTTP; 202-on-notification correct), so the valid critique targets the constraint itself. P3 Oracle m-7 stands: no SDK interop test, no session-id/GET handling, protocol-version header unchecked. Fix: one CI smoke test driving initialize, tools/list, tools/call from the official TS MCP SDK; re-evaluate rmcp at the next protocol bump. Smallest item — do it while it's cheap.