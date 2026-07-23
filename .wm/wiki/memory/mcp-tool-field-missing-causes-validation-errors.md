---
title: MCP tool field missing causes validation errors
type: memory
tags: [failure, mcp, task, validation]
status: active
---

When adding a new field to a page model, add it to ALL tool handlers (Create AND Update) or validation will break. wm_task.create was missing acceptance_criteria causing 153 false validation errors. Full reference: @wiki/concepts/failure-mcp-task-missing-acceptance-criteria