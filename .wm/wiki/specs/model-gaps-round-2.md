---
title: Model Gaps Round 2
type: spec
status: draft
tags: [models, knowns-parity, gap-filling]
---

## Overview

Fill remaining Knowns model gaps: task enrichment (time tracking, plan/notes), decision enrichment, docs (ordering, imports), workspace, chat, versions, templates, and config.

## Already Locked
## Locked Decisions

- D6: Split implementation_notes into implementation_plan + implementation_notes
- D7: Add order: Option<i32> to WikiPageMeta
- D8: Same order field for task kanban and doc sidebar
- D9: Workspace deferred
- D10: Chat deferred
- D11: Version history IN SCOPE
- D12: Field-level diffs (TaskChange/DocChange style)
- D13: FSRS-driven version compaction
- D14: Template prompt/action system in scope
- D15: Config enrichment in scope (status colors, columns, LSP, git tracking, runtime memory)
- D16: Memory becomes a Page variant — Memory { meta, data: MemoryData }`n- D17: Reference format @wiki/{type}/{name} replaces all @doc/, @task/, @memory/, @decision/ formats (status colors, columns, LSP, git tracking, runtime memory)

- D1: `time_entries: Vec<TimeEntry>` in `TaskData` — struct exists, frontmatter parsing stubbed (`Vec::new()`)
- D2: `consequences: Option<String>` on `DecisionData` — field exists, frontmatter parsing missing
- D3: MemoryStatus enum — implemented
- D4: Spec/fulfills/supersedence = typed edges — implemented
- D5: Canonical data stays as files — no database

## Remaining Gray Areas

### GA1: Implementation Plan vs Notes
Should WM split `implementation_notes` into `implementation_plan` and `implementation_notes` (like Knowns), or keep a single field?

### GA2: Task Ordering
Knowns has `order: Option<int>` for kanban ordering. Should WM add it?

### GA3: Doc Ordering
Knowns docs have `order: Option<int>` for sidebar ordering. Worth adding?

### GA4: Workspace
Knowns has agent execution contexts with git worktrees, phases (research/plan/implement/review). WM has nothing. Scope? Storage format?

### GA5: Chat
Knowns persists conversations with token counting, cost tracking. Scope for WM?

### GA6: Versions
Knowns has full task/doc version history with change diffs. Snapshot or delta?

### GA7: Templates
Knowns has code scaffolding with prompts, actions, destinations. WM has string interpolation only. Scope?

### GA8: Config
Knowns has status colors, visible columns, LSP settings, git tracking. Scope?

## Requirements

### FR-1: Time entry frontmatter parsing
Parse `time_entries` from task YAML frontmatter. Format:
```yaml
time_spent: 2h 30m
time_entries:
  - started_at: "2026-07-14T10:00:00Z"
    ended_at: "2026-07-14T12:00:00Z"
    duration_s: 7200
    note: "Fixed auth bug"
```

### FR-2: Decision consequences frontmatter
Parse `consequences` from decision body or frontmatter.

### FR-3 through FR-8: Gray area resolutions
Wait for exploration.

## Acceptance Criteria

- [ ] AC-1: task YAML with `time_entries` parses into `TaskData.time_entries`
- [ ] AC-2: decision YAML with `consequences` parses into `DecisionData.consequences`
- [ ] AC-3+: Per resolved gray area


