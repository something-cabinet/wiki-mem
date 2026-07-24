---
id: wiki:rules:deepwork-must-follow-wm-workflows
title: Deepwork sessions must follow WM workflow conventions
type: rule
status: active
tags: [rule, deepwork, workflow, session]
---
id: wiki:rules:deepwork-must-follow-wm-workflows

## Rule: Deepwork sessions must follow WM workflow conventions

When the `deepwork` skill is active, orchestrate through WM workflows, not ad-hoc. The deepwork skill manages phased execution with specialist delegation and Oracle review gates. WM workflows provide the task tracking, spec docs, and knowledge capture infrastructure.

### Requirements

1. **Phase tracking in WM tasks** — Each deepwork implementation phase should have a corresponding WM task for tracking and linking. Use `wm_task.create` before starting a phase, not just session-level todo lists.

2. **Specs go in wiki** — Specs created during deepwork sessions belong in `.wm/wiki/specs/` as proper typed pages, not as markdown files in `.slim/deepwork/`.

3. **Oracle findings → wiki tasks** — When Oracle review produces actionable findings, create WM tasks for them before starting remediation. Follow the findings-first pattern in `@wiki/rules/findings-first-task-spec`.

4. **Memory capture** — During deepwork, capture patterns and decisions to WM memory as they emerge (`.wm/memory/` JSON files, not just in the session progress file). Don't wait for a separate wm-extract pass.

5. **Progress file ≠ wiki** — `.slim/deepwork/` is for session-level orchestration state. Deliverables (specs, docs, ADRs) go in `.wm/wiki/`. Config files live in `.wm/config.json`.

6. **Validate before marking phase complete** — After each phase, run `wm_lint check` and `wm_validate check` to ensure no broken refs or stale state before advancing to the next Oracle review gate.

### Why

- Prevents knowledge loss when deepwork sessions overlap or are interrupted.
- Ensures deepwork output is discoverable by future agents via WM search and graph traversal, not buried in `.slim/deepwork/` files.
- Keeps Oracle reviews grounded in WM tasks with ACs, not ad-hoc phase descriptions.

### Related
- @wiki/rules/findings-first-task-spec