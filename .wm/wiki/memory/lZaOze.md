---
title: Single entry point — wm-cli is the only binary
type: memory
tags: [architecture, mcp, entry-point]
created_at: "2026-07-14T04:41:47.491Z"
updated_at: "2026-07-14T04:41:47.491Z"
---

wm-cli is the only standalone binary. wm-server, wm-vectors-bin are library crates. wm-cli mcp embeds the HTTP server in-process on a random port. wm-cli web embeds it on a user-specified port. No separate wm-mcp binary. Full reference: @doc/learnings/proxy-architecture-single-entrypoint