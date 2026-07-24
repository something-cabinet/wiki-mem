---
id: wiki:specs:lsp-git-config-wiring
title: LSP and Git Tracking Config Wiring
type: spec
status: draft
tags: [config, lsp, git, code-intel]
---
id: wiki:specs:lsp-git-config-wiring

## Overview

Verify and complete the wiring of `LspLanguageSettings` and `GitTracking` config structs that were added to `config.rs` but may not have consumers yet.

## Background

Two config structs were added:
- `LspLanguageSettings { command: String, args: Option<Vec<String>> }`
- `GitTracking { memory: Option<bool>, versions: Option<bool>, state: Option<bool> }`
- Both stored in `ProjectConfig` with `#[serde(default)]`

These need to be wired to actual behavior or removed if not needed.

## Requirements

### FR-1: LSP config load
In `code_intel.rs`, read `config.lsp` to configure language server commands per language.

### FR-2: Git tracking config load
In the `.gitignore` generation code, read `config.git_tracking` to control per-section ignore rules.

### FR-3: Project status exposure
Both should be exposed in `wm_project.status` output (already partially done).

## Acceptance Criteria

- [ ] AC-1: `code_intel.rs` reads LSP config and makes commands available
- [ ] AC-2: `.gitignore` generation respects `git_tracking.memory` toggle
- [ ] AC-3: `.gitignore` generation respects `git_tracking.versions` toggle
- [ ] AC-4: `wm_project.status` returns `lsp` and `git_tracking` fields when configured
- [ ] AC-5: Backward compatible — config without these fields loads successfully
- [ ] AC-6: All existing tests pass
