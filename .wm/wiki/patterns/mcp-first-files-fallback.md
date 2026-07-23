---
title: "MCP-first, Files-fallback"
type: pattern
status: active
tags: ["pattern", "workflow", "tooling"]
relates_to:
  - {type: references, target: wiki:specs:wiki-rules-auto-load}
  - {type: references, target: wiki:tasks:update-wm-init-load-rules}
---

## Problem

Agent workflows depend on MCP tools (e.g., `wm_page.*`, `wm_search.*`) for structured operations. But MCP tools require a running server binary — when the binary isn't built, disconnected, or misconfigured, workflows halt without a fallback.

## Solution

For every MCP-dependent operation, always provide a direct filesystem fallback:

**Primary — MCP tools:**
```json
wm_page.list({"type": "rule"})
```

**Fallback — direct file read:**
```bash
ls .wm/wiki/rules/*.md
```

The pattern applies to:
- **Discovery** — `wm_page.list` → `ls` / `glob`
- **Reading** — `wm_page.get` → `read_file`
- **Searching** — `wm_search.query` → `grep`

The primary path assumes MCP is healthy. The fallback uses basic I/O tools (bash, read_file, grep, glob) that are always available.

## When to Use

- Any agent skill or workflow that relies on MCP tools for data access
- CI/automation contexts where MCP may not be running
- Session init steps that must succeed even with degraded tooling

## When Not to Use

- Write operations (create, update, delete) — these should block until MCP is available to maintain graph consistency
- Operations where the file format is non-trivial and MCP provides parsing (e.g., versioned entities)

## Related

- @wiki/specs/wiki-rules-auto-load — first spec to formalize this pattern
- @wiki/tasks/update-wm-init-load-rules
