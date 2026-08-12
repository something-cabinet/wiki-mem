---
title: 'Spec: CLI + MCP in-process architecture'
type: spec
id: wiki:specs:cli-mcp-in-process-refactor
status: draft
---

## Approach

1. **CLI commands in-process**: replace `call_tool()` HTTP dispatch with direct `wm_core` calls via the existing `create_engine()` path (the TUI already proves this works). Delete `http_client.rs`, `server_discovery.rs`, `post_json_with_retry`, credential cell. 29 call sites in `main.rs` become typed calls.
2. **MCP in-process**: `wm-cli mcp` serves rmcp stdio directly against `register_all_tools` — no daemon, no tokens, no readiness races. Delete `mcp_proxy.rs` (314 lines).
3. **Daemon slims to web-only**: keep Axum `wm-server` for the Angular UI with web-token + reject_cross_site (real threat model). Delete `/api/mcp/*` routes, mcp-token generation, exemption list. Keep a minimal `wm web` spawner.
4. **Watcher in the daemon**: route daemon engine through the notify debouncer (MainEngine::with_root or move debouncer into EngineState::new) so disk edits refresh without write-path pokes; delete dead staleness code.
5. **Tests**: minimal surgery to compile+pass in this phase; deep simplification is Phase 2 (@wiki/tasks/test-suite-simplification).
6. **Docs**: reconcile the two approved decisions' status; update ARCHITECTURE.md deployment-modes table.

## Constraints

- No external deps; ureq removed from wm-cli.
- Zero warnings (`cargo check --workspace`, `cargo clippy --workspace`).
- Keep web security intact (web-token, cross-site rejection, SPA serving).
- Write-path refresh stays (still correct; watcher is belt-and-suspenders for daemon).

