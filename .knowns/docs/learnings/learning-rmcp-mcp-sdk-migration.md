---
title: 'Learning: rmcp MCP SDK Migration'
description: ''
createdAt: '2026-07-09T17:16:00.122Z'
updatedAt: '2026-07-09T17:16:00.122Z'
tags:
  - learning
  - mcp
  - rust
  - rmcp
---

## Patterns

### Use rmcp for MCP Server Implementation
- **What:** Replace hand-rolled JSON-RPC MCP transport with `rmcp` (official MCP Rust SDK). Handles protocol negotiation, content[] wrapping, error codes, tool discovery, and server info automatically.
- **When to use:** Any new Rust MCP server. rmcp fixes spec compliance (content[] wrapper, inputSchema, protocol versioning) that hand-rolled implementations miss.
- **How:** `impl ServerHandler for ToolRegistry` — the existing tool registration system can be preserved as-is, with rmcp wrapping the dispatch.
- **Dependency:** `rmcp = { version = "2", features = ["server", "transport-io"] }` (no HTTP, no SSE, min deps)
- **Source:** @task-8wqqm8 (indirect — the transport refactor)

## Decisions

### Migrate to rmcp over alternatives
- **Chose:** `rmcp` (official MCP Rust SDK, v2.2.0, 15M downloads)
- **Over:** `mcpserver` (too low-level), `model-context-protocol` (609 total downloads), `fastmcp-rust` (requires nightly Rust)
- **Tag:** GOOD_CALL
- **Outcome:** 302 lines of hand-rolled transport code replaced with ~100 lines of ServerHandler impl. All 148 tests pass. Fixed the content[] wrapper compliance issue that caused blank tool responses in OpenCode.
- **Recommendation:** Use rmcp for any future Rust MCP server work.

### Keep ToolRegistry, wrap with ServerHandler
- **Chose:** Kept existing ToolRegistry and `register_with_schema()` pattern, added `impl ServerHandler for ToolRegistry` on top
- **Over:** Rewriting all 16 tool modules to use rmcp macros
- **Tag:** GOOD_CALL
- **Outcome:** Zero changes needed in any tool module. The migration was purely in transport.rs.

## Failures

### OpenCode Doesn't Surface Persistent MCP Tools
- **What went wrong:** After migrating to rmcp and confirming the server responds correctly via bash JSON-RPC, WM tools still don't appear as callable functions in OpenCode sessions. mcp-jq tools work because they're process-per-call. knowns tools work because they're native OpenCode functions.
- **Root cause:** OpenCode generates the agent's tool list at session boot using oh-my-opencode-slim's preset. Only explicitly integrated MCP servers (knowns) and process-per-call servers (mcp-jq) get surfaced. Persistent MCP server tools are connected but not exposed to the agent.
- **Time lost:** ~2h debugging MCP connections, restarting sessions, checking configs
- **Prevention:** Test MCP tool visibility in the target platform first (OpenCode, Claude Code, Kiro) before committing to an MCP-based tool architecture. WM tools work correctly through direct JSON-RPC, bash piping, and likely in Claude Code — only OpenCode has this limitation.
