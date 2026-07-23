---
title: Add ServerHandler impl for Arc<ToolRegistry>
type: task
status: cancelled
priority: high
tags: [from-spec, mcp]
spec: specs/mcp-tool-registry-unification
---

Add `impl ServerHandler for Arc<ToolRegistry>` in transport.rs that delegates all trait methods to the inner ToolRegistry. This is the prerequisite for sharing a single registry between MCP and HTTP paths.