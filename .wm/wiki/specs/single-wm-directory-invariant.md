---
title: Single .wm/ Directory Invariant
type: spec
status: approved
tags: [spec, architecture, invariant, enforcement]
relates_to:
  - {type: references, target: wiki:specs:engine-explicit-project-root}
  - {type: references, target: wiki:tasks:engine-explicit-project-root}
  - {type: references, target: wiki:tasks:task-cli-page-update-missing}
---

# Single `.wm/` Directory Invariant

**Status:** draft

## Overview

There must be exactly **one** `.wm/` directory in the entire repository — at the project root. No code in any crate may create, write to, or resolve path references to any `.wm/` directory other than `{project_root}/.wm/`.

This invariant was systematically violated: `EngineState::new()` used `current_dir()` as the project root, so running CLI commands from any subdirectory created ghost `.wm/` copies. At peak, 6 `.wm/` directories existed across the repo.

## Locked Decisions

- D1: **Only `detect_project_root()` may locate the `.wm/` directory.** Every crate that needs `.wm/` must either call `detect_project_root()` or receive the root from `EngineState.project_root`.
- D2: **No crate may construct a `.wm/` path from `current_dir()` or hardcoded relative paths.** Every `.wm/` access must go through `project_root.join(".wm/...")`.
- D3: **Ghost `.wm/` directories are deleted and gitignored.** The `.gitignore` lists known crate-local `.wm/` paths; any new ghost `.wm/` must be removed and its root cause fixed.

## Requirements

### Functional Requirements

- FR-1: All `.wm/` path construction must use `project_root.join(".wm")`, never relative paths
- FR-2: `EngineState` must never use `current_dir()` for project root — it receives it as a parameter
- FR-3: No crate may create a `.wm/` directory at its own local path
- FR-4: A `wm lint check` rule should flag any `.wm/` directory outside the project root
- FR-5: CI must verify only one `.wm/` exists (or at least none outside expected path)

### Non-Functional Requirements

- NFR-1: Zero performance impact — `detect_project_root()` is called once at startup
- NFR-2: Backward compatible — existing projects are unaffected (the root `.wm/` never moves)

## Acceptance Criteria

- [ ] FR-1: All code uses `project_root.join(".wm")` — verified by code search
- [ ] FR-2: `EngineState::new()` takes explicit `project_root` — ✅ done
- [ ] FR-3: No crate creates `.wm/` locally — verified by `find` across workspace
- [ ] FR-4: `wm lint check` detects rogue `.wm/` directories — task created
- [ ] FR-5: CI step checks exactly one `.wm/` exists — task created

## Scenarios

### Scenario 1: New developer runs CLI from subdirectory
**Given** a developer runs `wm-cli page list` from `apps/wm-core/`
**When** the engine starts
**Then** `detect_project_root()` walks up from CWD to find project root
**And** all `.wm/` paths resolve to `{project_root}/.wm/`
**And** no `.wm/` is created in `apps/wm-core/`

### Scenario 2: Code review catches violation
**Given** a new crate is added to the workspace
**When** its code constructs `Path::new(".wm")` relative to an unknown base
**Then** review must flag it and redirect to `project_root.join(".wm")`

## Technical Notes

### Current enforcement

Gitignore entries for ghost dirs:
```
apps/wm-core/.wm/
apps/wm-cli/.wm/
apps/wm-web/.wm/
apps/wm-web/src-tauri/.wm/
```

These should never contain files. If they reappear, it means a new `current_dir()`-based path bug was introduced.

### Known violations (all fixed)

| Location | Root cause | Fix |
|----------|-----------|-----|
| `apps/wm-core/` | Tests ran from crate dir | EngineState now explicit root |
| `apps/wm-cli/.wm/` | CLI ran from crate dir | ✅ fixed + gitignored |
| `apps/wm-web/.wm/` | Tauri build ran from crate dir | ✅ fixed + gitignored |
| `apps/wm-web/src-tauri/.wm/` | Tauri build from `src-tauri/` | ✅ fixed + gitignored |
| `.wm/wiki/.wm/` | CLI commands from wiki dir | ✅ fixed + gitignored |

### Legacy patterns found and patched

| File | Pattern | Fix |
|------|---------|-----|
| `source_service.rs` | `Path::new(".wm/sources/")` | `root.join(".wm").join("sources")` |
| `source_service.rs` | `".wm/wiki/log.md"` | `root.join(".wm").join("wiki").join("log.md")` |
| `engine_state_mediator.rs` | `current_dir()` | Explicit `project_root` param |

## Open Questions

- [ ] Should there be a CI script that runs `find . -not -path './.wm' -name '.wm' -type d` and fails if anything found?
- [ ] Should `wm lint check` get a new `--strict` mode that checks this invariant?
