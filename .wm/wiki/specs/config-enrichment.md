---
title: Config Enrichment
type: spec
status: draft
tags: [config, knowns-parity, lsp, git]
---

## Overview

Enrich `config.json` with Knowns-level settings: customizable status colors, visible task board columns, per-language LSP configuration, per-section git tracking, and runtime memory injection.

## Locked Decisions

- D15: All five categories in scope
- D21: Status colors, columns, LSP, git tracking, runtime memory

## Requirements

### FR-1: Status colors
Add to `ProjectSettings`/`config.json`:
```json
{
  "settings": {
    "statusColors": {
      "todo": "gray",
      "in-progress": "blue",
      "done": "green",
      "blocked": "red",
      "in-review": "violet"
    },
    "visibleColumns": ["todo", "in-progress", "done"],
    "lsp": {
      "rust": { "command": "rust-analyzer" },
      "typescript": { "command": "typescript-language-server", "args": ["--stdio"] }
    },
    "gitTracking": {
      "memory": true,
      "versions": false
    },
    "runtimeMemory": {
      "maxEntries": 1000,
      "recencyStabilityDays": 7
    }
  }
}
```

### FR-2: Backward compatibility
Adding new fields to config with `#[serde(default)]` — existing config files without these fields continue to work.

### FR-3: CLI exposure
All settings readable via `wm_project.status` and individually configurable via `wm_project.configure`.

## Acceptance Criteria
- [ ] AC-1: `statusColors` map renders in task board display
- [ ] AC-2: `visibleColumns` controls which columns appear in `wm_task.board`
- [ ] AC-3: LSP settings are exposed in config (consumed by code intel module)
- [ ] AC-4: Git tracking toggles affect `.gitignore` generation
- [ ] AC-5: Runtime memory settings control DashMap eviction
- [ ] AC-6: Existing config files without new fields parse successfully
