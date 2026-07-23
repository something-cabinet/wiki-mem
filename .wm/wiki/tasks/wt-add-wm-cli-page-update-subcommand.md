---
title: WT: Add wm-cli page update subcommand
type: task
status: todo
priority: high
tags: [spec:wiki-tool-reliability, cli]
---

Add PageAction::Update variant to wm-cli main.rs. Accepts stdin JSON with optional fields: title, content, status, tags, type. Mirrors MCP wm_page.update tool.