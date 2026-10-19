---
title: EngineState is a facade over seven domain handles
type: memory
tags: [architecture, engine, wm-core]
status: active
---

After specs/engine-state-decomposition, EngineState declares 11 fields: handles graph, page, index, source_registry, audit, lifecycle, session_memory, plus flat stale_flag, config, project_root, lsp. tool_list moved to ToolRegistry in wm-tools. Hot snapshots stay bare ArcSwap inside their handle; no consumer loads a snapshot directly. Measured D-11: no handle earns a crate. Full reference: @wiki/core/ARCHITECTURE