---
id: wiki:patterns:task-subagents-for-delegation
title: Pattern: Use Task Subagents for Delegation
type: pattern
tags: [pattern, workflow, delegation]
status: draft
relates_to:
  - {type: references, target: wiki:patterns:run-clippy-before-rust-reviewer}
---
id: wiki:patterns:task-subagents-for-delegation

## Problem

Spawning separate Discord threads via `kimaki send --thread` for delegating work within the same project causes loss of control, untracked progress, and context switching.

## Solution

Use the `task` tool to spawn subagents instead. Subagents run in their own context window, return results to the orchestrator, and keep the user in a single thread. The orchestrator can parallelize independent work by spawning multiple `task` subagents simultaneously.

```
Orchestrator              task subagent
    │                          │
    ├── task("review X") ──────┤
    │                          ├── reads files
    │                          ├── runs checks
    │                          └── returns results
    │←──── results ────────────┤
    │                          │
    └── presents to user ──────┘
```

## When to Use

- Delegating work within the same project
- Parallelizing independent review/implementation tasks
- Any time you need context isolation with results returned

## When Not to Use

- The user explicitly asks for a separate thread
- Work needs to happen in a different project repo
- Sending a notification-only ping (use notification tools instead)

## Related

- patterns/run-clippy-before-rust-reviewer