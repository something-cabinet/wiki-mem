---
name: wm-plan
description: Take a task, gather context, create implementation plan
---

# Planning a Task

**Announce:** "Using wm-plan for task [ID]."

**Core principle:** GATHER CONTEXT → DRAFT PLAN → USER APPROVES.

## Inputs

- Task ID
- Optional spec path

## Step 1: Read Task

```json
wm_task.get({ "taskId": "<id>" })
```

Check: spec refs, linked docs, existing notes.

## Step 2: Gather Context

Search wiki and memory for related concepts, patterns, specs:

```json
wm_search.query({ "query": "<task keywords>", "mode": "keyword" })
wm_search.retrieve({ "query": "<task context>" })
wm_search.query({ "query": "<task keywords>", "type": "memory" })
```

## Step 3: Check Memory

```json
wm_memory.list({ "category": "pattern", "tag": "<domain>" })
```

## Step 4: Draft Plan

Write implementation steps. Save as task plan:

```json
wm_task.update({ "taskId": "<id>", "plan": "1. ...\n2. ...\n3. Tests" })
```

## Step 5: Present for Approval

Show plan and wait for user confirmation before implementing.
