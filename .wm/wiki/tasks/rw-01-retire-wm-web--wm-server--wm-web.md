---
title: RW-01 Retire wm-web + wm-server + wm web
type: task
id: "wiki:tasks:rw-01-retire-wm-web--wm-server--wm-web"
status: done
priority: high
tags: [from-spec, spec:retire-web-ui-mcp-only, retirement, p0]
spec: specs/retire-web-ui-mcp-only
acceptance_criteria:
  - text: "wm-web + wm-server removed from workspace and lockfile"
  - text: "wm web / wm serve commands removed"
  - text: "browser-only WASM crates/assets removed if unused"
  - text: "grep-clean of removed crate/command references"
  - text: "cargo check/clippy clean; CLI + MCP still work"
---

Retire the web stack for 0.6 MCP-only: delete apps/wm-web + apps/wm-server, remove wm web/serve, retire browser-only WASM crates/assets, remove web e2e tests, update docs. Per specs/retire-web-ui-mcp-only.