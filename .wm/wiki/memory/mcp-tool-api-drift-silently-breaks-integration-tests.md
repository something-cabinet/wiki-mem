---
title: MCP tool API drift silently breaks integration tests
type: memory
tags: [failure, testing, mcp]
status: active
---

Integration tests that launch wm-cli as a subprocess silently rot when the MCP tool surface evolves (action enums, tool renames). No compiler catches this. Fix: run full test suite in CI, update test fixtures in the same PR as the tool refactor. Full entry: @wiki/concepts/test-rot-mcp-api-drift