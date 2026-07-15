---
title: Wire LSP and Git Tracking Config Consumers
type: task
status: todo
priority: medium
tags: [config, lsp, git]
---

## Overview

The `LspLanguageSettings` and `GitTracking` config structs exist in `config.rs` but may not be wired to actual behavior. Verify and complete the wiring.

## Requirements

- Verify LSP settings from config are loaded by code intel module
- Verify `GitTracking` toggles affect .gitignore generation
- Both should be exposed in `wm_project.status`

## Acceptance Criteria
- [ ] AC-1: `config.lsp` settings are readable from code intel
- [ ] AC-2: `config.git_tracking.memory` toggles memory gitignore
- [ ] AC-3: `wm_project.status` includes lsp and git_tracking fields
- [ ] AC-4: All tests pass
