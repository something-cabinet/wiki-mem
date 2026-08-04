---
type: pattern
id: wiki:patterns:mcp-http-proxy
title: 'Pattern: MCP-to-HTTP Proxy'
page_type: pattern
status: draft
tags:
  - pattern
  - mcp
  - architecture
  - proxy
relates_to:
  - {type: references, target: wiki:specs:web-server-build-serve}
---
id: wiki:patterns:mcp-http-proxy

# Pattern: MCP-to-HTTP Proxy

## Problem

An MCP server that embeds its own engine duplicates state (graph, BM25, embedder) when a separate HTTP server already owns the same data. Running two engine instances wastes memory (~200-500MB for ONNX models), causes startup latency, and creates consistency issues when the two graphs drift apart.

## Solution

Build the MCP server as a **thin HTTP proxy** — a protocol adapter that translates MCP tool calls into HTTP requests to a backend server. The MCP server has zero state, no embedded engine, no database. Each tool handler is a 3-line HTTP call:

```rust
// Tool handler: proxy to HTTP API
registry.register_with_desc("wm_page.get", "Get page content", Arc::new(move |params| {
    let resp = client
        .post("http://localhost:3000/api/pages/get")
        .json(&params)
        .send()?;
    Ok(resp.json()?)
}));
```

The backend HTTP server owns the engine (graph, BM25, memory, embedder). Other clients (Angular, curl, scripts) also talk to the same HTTP server — single source of truth.

### Architecture

```
MCP Client ──stdio──► MCP Proxy ──HTTP──► HTTP Server (engine)
Angular UI ────────────────────────► HTTP Server (engine)
curl ──────────────────────────────► HTTP Server (engine)
```

## When to Use

- You have an existing HTTP API that owns application state
- You need an MCP interface for AI agents but don't want to duplicate the engine
- Multiple clients need access to the same data (Angular, CLI, MCP, curl)
- Memory/startup overhead of a second engine instance matters

## When Not to Use

- The MCP server needs to work offline without the HTTP backend
- The HTTP API doesn't exist yet and would be too expensive to build
- The latency of localhost HTTP (~0.5-2ms per call) is unacceptable for high-frequency tools
- The tool handles require real-time streaming that HTTP doesn't support well

## Implementation Notes

- Use `ureq` (blocking HTTP client, no tokio dependency) for synchronous MCP proxy handlers
- Tool handlers are auto-discovered from `wm_core::mcp::tools::register_all_tools` and registered with HTTP-forwarding closures
- All tool calls are dispatched to a single `POST /api/tools` endpoint with `{"name": "<tool>", "arguments": {...}}`
- The HTTP server is started in-process by `wm-cli` on a random port — no separate binary needed
- The response `success` field is checked by the proxy: if `false`, the error is propagated as an MCP-level error (`isError: true`)
- Use a generic URL namespace: `POST /api/tools` maps any tool name to the matching handler

## Architecture (WM Implementation)

```
┌─────────────────────────────────────────────┐
│              wm-cli (single binary)           │
│                                               │
│  ┌──────────────┐    HTTP localhost:random    │
│  │  MCP Proxy   │ ────────────────────────►   │
│  │  (rmcp stdio)│                             │
│  │  78 handlers │     ┌──────────────────┐   │
│  │  ureq → POST │     │  wm-server       │   │
│  │  /api/tools  │     │  (axum HTTP)     │   │
│  └──────────────┘     │  /api/tools →    │   │
│                       │  ToolRegistry    │   │
│  Angular UI ──────────►  /api/search     │   │
│  curl ────────────────►  /api/pages/*    │   │
│                       └──────────────────┘   │
└─────────────────────────────────────────────┘
```

## Related

- patterns: Blog pattern at https://rup12.net/posts/write-your-mcps-in-rust/
- patterns: The wm-core ToolRegistry pattern
- specs/web-server-build-serve: Single binary build with embedded web UI
