---
title: Pattern: MCP-to-HTTP Proxy
page_type: pattern
status: draft
tags:
  - pattern
  - mcp
  - architecture
  - proxy
---

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

- Use `reqwest::blocking::Client` for synchronous MCP handlers, or async with `rmcp`'s `AsyncTool` trait
- Pre-register all tool handlers with a simple URL-to-tool mapping
- Health-check the HTTP server on startup and warn if unavailable
- Each tool's input params are forwarded as JSON request body; the HTTP server extracts them from the path or body
- Use a consistent URL namespace: `/api/<domain>/<action>` maps to `wm_<domain>.<action>`

## Related

- @doc/patterns: Blog pattern at https://rup12.net/posts/write-your-mcps-in-rust/
- @doc/patterns: The original wm-core ToolRegistry pattern
- @task: MCP proxy implementation in wm-mcp crate
