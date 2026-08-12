---
title: 'Decision: Direct MCP handlers over proxy'
type: decision
id: wiki:decisions:mcp-direct-handlers-over-proxy
status: active
relates_to:
  - {type: relates_to, target: wiki:decisions:mcp-error-iserror-true}
---
id: wiki:decisions:mcp-direct-handlers-over-proxy

---
id: wiki:decisions:mcp-direct-handlers-over-proxy
{}
relates_to:
  - {type: references, target: wiki:tasks:853217}
  - {type: references, target: wiki:specs:mcp-direct-handlers}
  - {type: references, target: wiki:learnings:proxy-architecture-single-entrypoint}
  - {type: references, target: wiki:patterns:mcp-proxy-singleton}
  - {type: references, target: wiki:tasks:22ed6a}
---
id: wiki:decisions:mcp-direct-handlers-over-proxy
---
id: wiki:decisions:mcp-direct-handlers-over-proxy

title: Decision: Direct MCP handlers over proxy
type: decision
status: approved
tags: [decision, good-call, mcp, proxy, architecture]
---
id: wiki:decisions:mcp-direct-handlers-over-proxy

## Context

`wm-cli mcp` registered proxy handlers that routed every tool call to `wm-server` via HTTP. This required wm-server to be running, added latency, and introduced a hardcoded `STATIC_TOOLS` list that drifted from the actual registered tools. An oracle review found that ~26 of 50 advertised proxy tools no longer existed in the engine registry, and ~25 real tools were unreachable. The proxy also served empty descriptions and schemas for all tools.

## Decision

Replace the proxy with direct in-process handler registration. `wm-cli mcp` now creates an `EngineState` from the project root, calls `register_all_tools()` on the registry, and serves stdio MCP directly — matching Knowns' architecture (`internal/mcp/server.go`).

## Rationale

- Eliminates the network hop and wm-server dependency
- Removes the `STATIC_TOOLS` drift problem permanently — tools are discovered from the actual registry
- Enables proper error semantics (`isError: true` instead of transport failures)
- Makes `tools/list` serve real descriptions and schemas (previously empty)
- Self-contained process — no startup race, no server-death-kills-MCP problem

## Consequences

- `mcp_proxy.rs` deleted (162 lines)
- `wm-cli mcp` is now self-contained — no wm-server needed
- `wm-cli mcp` and wm-server produce identical `tools/list` (same `register_all_tools()` source)
- `wm-cli serve` renamed to `wm-cli web` (clarifies: web UI only, no MCP)

## Implementation (2026-08-12)

Reinstated and implemented by @wiki/tasks/cli-mcp-in-process-refactor: `mcp_proxy.rs` deleted (314 lines, 2× the original 162); `wm-cli mcp` now serves rmcp stdio in-process against `register_all_tools` via the new `apps/wm-cli/src/mcp_server.rs` (94 lines). `wm-cli serve` → `wm web` renaming confirmed. The proxy reversal is superseded by this implementation.

## Related

- @wiki/tasks/853217
- @wiki/tasks/cli-mcp-in-process-refactor
- @wiki/specs/mcp-direct-handlers