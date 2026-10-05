---
title: 'Model-backed MCP tools: async + cached backend'
type: memory
tags: [mcp, async, caching, model-runtime]
status: active
---

Model-backed MCP tools must be async (register_typed_async + spawn_blocking), cache the backend once (keyed by dir+revision), stream-hash downloads, and error on a missing backend result instead of fabricating labels[0]. Applied to wm_decision (TD-13/14/15). Full: @doc/patterns/model-backed-mcp-tool-async-and-cached