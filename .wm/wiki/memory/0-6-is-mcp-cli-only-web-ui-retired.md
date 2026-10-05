---
title: 0.6 is MCP + CLI only (web UI retired)
type: memory
tags: ['0.6', mcp-only, retirement, wm-server, wm-web]
status: active
---

0.6 ships MCP + CLI only: wm-web, wm-server, `wm web`, the HTTP MCP transport, and the browser-only WASM crates were deleted. MCP is in-process stdio (wm-cli depends only on wm-core). Nothing compiled against wm-server/wasm, so removal was a clean deletion. Full: @doc/decisions/retire-web-ui-ship-mcp-only-0-6