---
id: we6gtd
title: MCP Bridge for Web UIs
layer: project
category: pattern
tags:
  - mcp
  - web-ui
  - bridge
createdAt: '2026-07-06T17:43:11.514Z'
updatedAt: '2026-07-06T17:43:11.514Z'
---

Web UI communicates with Rust engine via wm serve child process + JSON-RPC over stdin/stdout. wm-bridge.ts spawns the process, sends/receives JSON-RPC. SvelteKit API routes delegate to the bridge. No HTTP server crate needed in Rust. Full reference: @doc/learnings/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration
