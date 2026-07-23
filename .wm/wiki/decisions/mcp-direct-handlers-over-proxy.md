---
{}
relates_to:
  - {type: relates_to, target: wiki:decisions:mcp-error-iserror-true}
---

---
{}
relates_to:
  - {type: references, target: wiki:tasks:mcp-direct-t1-replace-proxy}
  - {type: references, target: wiki:specs:mcp-direct-handlers}
  - {type: references, target: wiki:learnings:proxy-architecture-single-entrypoint}
  - {type: references, target: wiki:patterns:mcp-proxy-singleton}
  - {type: references, target: wiki:tasks:srv-create-mcp-proxy-with-static-tool-list}
---
---

title: Decision: Direct MCP handlers over proxy
type: decision
status: approved
tags: [decision, good-call, mcp, proxy, architecture]
---

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

## Related

- @wiki/tasks/mcp-direct-t1-replace-proxy
- @wiki/specs/mcp-direct-handlers