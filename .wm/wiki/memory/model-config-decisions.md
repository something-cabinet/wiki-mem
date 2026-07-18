---
title: "OpenCode Model Config Decisions"
type: memory
status: active
tags: [opencode, config, models, mcp]
---

## Summary

Configured the opencode-go preset in `oh-my-opencode-slim.json` with model changes and MCP access patterns.

## Changes Made

### Model Upgrades
- **Oracle**: deepseek-v4-pro → kimi-k3 (frontier reasoning for code review)
- **Designer**: kimi-k2.6 → kimi-k3 (vision + 1M context for UI work)
- **Observer**: kimi-k2.6 → kimi-k3 (browser automation gets frontier model)

### MCP Access Pattern
- Subagents (oracle, designer) have `"mcps": []` — they don't read files directly
- Orchestrator delegates file reading to explorer, then passes context to subagents
- Explorer has full MCP access for file discovery

### Key Models Available
- `opencode-go/deepseek-v4-flash` — fast/cheap, used by orchestrator, librarian, explorer, fixer
- `opencode-go/deepseek-v4-pro` — strong reasoning at $0.435/$0.87 per 1M, best value
- `opencode-go/kimi-k3` — frontier tier at $3/$15 per 1M, used for deep reasoning
- `opencode-go/kimi-k2.7-code` — coding specialist, 256K context, $0.95/$4.00

### Cost Awareness
- K3 is ~7-17x more expensive than V4 Pro
- Oracle is only called occasionally, not on every turn
- Keep K3 for the calls where deep reasoning matters most

### UI Review
- Created task `wiki/tasks/ui-review-findings` with 15 findings (P0-P3)
- Covers wm-web Angular 22 app: CSS duplication, missing features, dark mode, a11y
