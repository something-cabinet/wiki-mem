---
title: WT: Wire From&lt;ToolError&gt; for ErrorData conversion in transport.rs
type: task
status: todo
priority: high
tags: [spec:wiki-tool-reliability, mcp, errors]
---

Wire the existing From&lt;ToolError&gt; for ErrorData impl (already in wm-error) into transport.rs so MCP error responses include code, message, and hint as structured JSON-RPC error.data. Currently only err.message is sent as text.