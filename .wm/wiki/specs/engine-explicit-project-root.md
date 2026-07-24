---
id: wiki:specs:engine-explicit-project-root
title: EngineState must use explicit project_root, not current_dir()
type: spec
status: approved
tags: [engine, bugfix, refactor]
relates_to:
  - {type: references, target: wiki:tasks:engine-explicit-project-root}
  - {type: references, target: wiki:tasks:bug-page-link-metapath-relative}
---
id: wiki:specs:engine-explicit-project-root

## Overview

EngineState::new() derived the project root from std::env::current_dir(), making the engine sensitive to the working directory of the calling process. This caused ghost .wm/ directories to be created when CLI commands were run from subdirectories.

## Background

The CLI correctly detected the project root via detect_project_root() (walking up from CWD looking for .wm/config.json). But it only passed the config to MainEngine::new(), not the root. EngineState then re-derived the root from current_dir() — which could be wrong if CWD was a subdirectory.

This was triggered when wm-cli commands were run from inside .wm/wiki/ (as a workaround for a separate P1 bug where page link/update/delete fails from project root — see @wiki/tasks/8b43fd).

## Locked Decisions

- D1: MainEngine owns project root detection — single source of truth
- D2: MainEngine::new() with no args auto-detects root (for most callers)
- D3: MainEngine::with_root(config, root) for callers that already know it (Tauri)
- D4: EngineState accepts project_root as a parameter, never uses current_dir()

## Requirements

### Functional Requirements

- FR-1: MainEngine::new() must auto-detect project root via detect_project_root()
- FR-2: EngineState must never use current_dir() for project root
- FR-3: All CLI callers must use MainEngine::new() (no manual root detection)
- FR-4: Ghost .wm/ directories must be gitignored to prevent re-commit

### Non-Functional Requirements

- NFR-1: cargo build + cargo test must pass
- NFR-2: No behavior change for normal usage (project root = CWD)

## Acceptance Criteria

- [x] MainEngine::new() auto-detects root
- [x] EngineState::new(config, root) uses passed root
- [x] 23 CLI callers updated
- [x] Tauri caller updated
- [x] Ghost dirs cleaned + gitignored
- [x] Build + test green
- [x] No ghost .wm/ reappears

## Scenarios

### Scenario 1: Normal CLI usage from project root
**Given** CWD is the project root
**When** user runs any wm-cli command
**Then** engine loads correctly from .wm/config.json
**And** .wm/state/vectors.db is created at the correct location

### Scenario 2: CLI run from subdirectory
**Given** CWD is inside apps/wm-cli/ or .wm/wiki/
**When** user runs a wm-cli command
**Then** engine still finds the project root via detect_project_root()
**And** no ghost .wm/ is created in the subdirectory
