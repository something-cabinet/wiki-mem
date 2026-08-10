---
title: 'Decision: MCP stdio→HTTP proxy with privileged /api/mcp channel + token split'
type: decision
id: wiki:decisions:mcp-proxy-privileged-channel-token-split
status: approved
tags:
- decision
- architecture
- mcp
- proxy
- security
relates_to:
  - {type: supersedes, target: wiki:decisions:wm-server-overrides-tauri-primary}
---

## Context

wiki-mem moved from a multi-EngineState Tauri architecture to a single `wm-server` daemon (see decision wm-server-overrides-tauri-primary). MCP clients (Claude, opencode, editors) speak stdio JSON-RPC, but the engine now lives in one HTTP daemon. Two prior decisions collided: D1 (MCP becomes a stdio→HTTP proxy to the daemon) and D2 (delete the generic `/api/tools/{name}` endpoint — its allowlist names never matched registered dotted tools, e.g. `wm_codesymbols` vs `wm_code.symbols`, and blocked tools returned 200+FORBIDDEN instead of 404).

## Decision

MCP becomes a thin **stdio→HTTP proxy** that targets a **privileged `/api/mcp/*` channel** with a **separate credential** from the read-only web API:

- `POST /api/mcp/tools/list` — returns `ToolRegistry::list_tools()` verbatim (name, description, inputSchema) for dynamic tool discovery. **No STATIC_TOOLS compile-time array** — the registry is the single source of truth.
- `POST /api/mcp/tools/call` — body `{name, arguments}`, dispatches via `registry.dispatch_async`. Tool-level errors are HTTP 200 with `{success:false, error, code}` (mapped to MCP `isError:true`); only auth/transport failures are non-200.
- **Token split**: `.wm/state/web-token` (read-only web surface) and `.wm/state/mcp-token` (privileged MCP surface), both 0600. The web token lives in browser context, so it must never authorize writes; a separate prefix + credential makes read-only-web vs full-MCP structural.
- Proxy (`wm-cli mcp`): ureq inside `tokio::task::spawn_blocking` (never blocks the runtime; no reqwest), reads `.wm/server.json` (fallback 127.0.0.1:4090), health-checks `/api/health`, spawns the co-located daemon if down (parallel with the rmcp handshake), and on 401 re-reads the token file and retries once (daemon rotates tokens on restart).

## Rationale

- Single engine, single writer — the proxy adds no second `EngineState` (avoids split-brain graph/BM25/vector-store state that the old in-process MCP path had).
- Writes through MCP hit the daemon → fire its SSE `/api/events` → the Angular UI live-updates.
- The credential split upgrades D2's "read-only web API" from convention to structure: the web token cannot authorize writes because writes live behind a different credential on a different prefix.
- Dynamic `tools/list` never goes stale when a new tool is registered.

## Consequences

- One extra HTTP hop per tool call + a token-lifecycle dance (re-read + retry on 401).
- `wm_initial`'s first-call runtime-context block must be emitted daemon-side (a dumb proxy can't append it) — moved into the `wm_initial` handler.
- Normalize `wm_index_*` → `wm_index.*` dotted naming is a follow-up (breaking rename, low risk since clients discover dynamically).
- Same channel serves CLI-over-HTTP (#109) — `wm-cli search/page/...` uses the same client.

## Related

- @task-22ed6a (MCP proxy)
- @wiki/decisions/wm-server-overrides-tauri-primary
- @wiki/specs/single-http-server