---
title: SRV: Move MCP transport layer from wm-core to wm-cli
type: task
status: todo
priority: high
tags: [spec:wm-server, refactor, mcp]
---

Move serve_rmcp and ServerHandler impl for ToolRegistry from apps/wm-core/src/mcp/transport.rs to apps/wm-cli/src/mcp_transport.rs (new file). Keep ToolRegistry as public re-export from wm-core. This enables wm-server to exist without dragging in rmcp deps.