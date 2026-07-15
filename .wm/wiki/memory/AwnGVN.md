---
title: Generic /api/tools dispatch pattern
type: memory
tags: [mcp, api, dispatch]
created_at: "2026-07-14T04:41:47.529Z"
updated_at: "2026-07-14T04:41:47.529Z"
---

Single POST /api/tools endpoint dispatches all 78+ tools via ToolRegistry. Proxy registers handlers dynamically, each forwarding to /api/tools. No need for per-tool REST routes. Full reference: @doc/learnings/proxy-architecture-single-entrypoint