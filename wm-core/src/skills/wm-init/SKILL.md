---
name: wm-init
description: Use at the start of a new session to read project docs, understand context, and see current state
---

# Session Initialization

**Announce:** "Using wm-init to initialize session."

**Core principle:** READ DOCS BEFORE DOING ANYTHING ELSE.

## Inputs

- Optional user focus such as a task ID, feature area, bug, or question
- Current project root already opened in the agent session

## Step 1: Check Wiki State

```json
wm_initial({})
```

Review: project state, conventions, recent log entries.

## Step 2: List Docs

```json
wm_doc.list({})
```

Open relevant docs (README, ARCHITECTURE, CONVENTIONS if they exist).

## Step 3: Check Tasks

```json
wm_task.board({})
wm_task.list({ "status": "in-progress" })
```

## Step 4: Load Memory

```json
wm_memory.list({ "layer": "project" })
```

## Step 5: Summarize

Report: project purpose, key docs, current in-progress work, ready for next steps.
