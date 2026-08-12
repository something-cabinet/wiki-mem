---
title: 'Refactor: CLI + MCP in-process, delete daemon hostage layers'
type: task
id: wiki:tasks:cli-mcp-in-process-refactor
status: done
acceptance_criteria:
- "wm-cli commands call wm_core directly (create_engine), no HTTP client layer (http_client.rs deleted)"
- "wm-cli mcp serves rmcp stdio in-process against register_all_tools (mcp_proxy.rs deleted)"
- "server_discovery.rs + /api/mcp/* routes + mcp-token half deleted; web-token keeps protecting the web surface"
- "401 retry loop (post_json_with_retry) + credential cell deleted"
- "wm web keeps a minimal spawner; daemon is opt-in for the web UI only"
- "File watcher (notify debouncer) wired into the daemon engine path; dead staleness machinery (check_external_staleness, wiki_dir_mtime, stale_flag partial) removed"
- "Workspace tests compile and pass (minimal surgery allowed)"
- "cargo check --workspace + cargo clippy --workspace: zero warnings"
- "Decision pages cli-direct-execution-not-http-proxy and mcp-direct-handlers-over-proxy reconciled (status reflects reality)"
---

## Finding

Oracle architecture review (2026-08-12): the daemon-annexed architecture is "layers of stupidity, each compensating the other" — CLI HTTP-client-to-itself, mcp-token theater, 401 retry loop, hand-rolled HTTP parsers, spawn/probe/port machinery, and an SSE endpoint returning `{"events": []}` as the stated justification. Two approved decisions (direct-execution, direct-handlers) were reverted; this refactor reinstates them.

## Files

- apps/wm-cli/src/mcp_proxy.rs (delete → in-process rmcp)
- apps/wm-cli/src/http_client.rs (delete)
- apps/wm-cli/src/server_discovery.rs (delete)
- apps/wm-cli/src/main.rs (rewire command dispatch, ~250 lines spawn/probe machinery, page_update_args)
- apps/wm-server/src/routes/mcp.rs (delete), web_token_service.rs (mcp half), routes/events.rs (9-line stub, verify)
- apps/wm-core engine path: wire notify debouncer into daemon (main_engine_factory.rs:198-272 exists; EngineState::new daemon path doesn't use it)
- Dead code: engine_state_mediator.rs:111-132 check_external_staleness/wiki_dir_mtime (zero callers)

## Severity

High — architectural, ~1,100-1,300 lines deleted, class of failure modes removed (port conflicts, 401 retries, readiness races, stale graphs).

## Related

- @wiki/specs/cli-mcp-in-process-refactor
- @wiki/decisions/cli-direct-execution-not-http-proxy
- @wiki/decisions/mcp-direct-handlers-over-proxy
- @wiki/tasks/test-suite-simplification (Phase 2, after this)

