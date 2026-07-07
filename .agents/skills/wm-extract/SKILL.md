---
name: wm-extract
description: Extract reusable patterns, decisions, and failures into wiki docs and memory
---

# Extracting Knowledge

**Announce:** "Using wm-extract for [pattern/decision]."

**Core principle:** IF IT COST TIME TO LEARN, SAVE IT FOR LATER.

## Inputs

- Source task ID, spec ref, or area of work
- Type of extraction: pattern, decision, failure, or convention

## Step 1: Review Source Material

```json
wm_task.get({ "taskId": "$ARGUMENTS" })
wm_log.recent({ "limit": 20 })
```

Review the task, recent logs, and changes to identify what is worth capturing.

## Step 2: Quick Memory (fast recall)

For concise insights that should surface quickly in future sessions, save as a memory entry:

```json
wm_memory.list({ "category": "pattern", "tag": "<domain>" })
```

Check existing memory first to avoid duplicates. Add a new memory entry by creating a page under `memories/`:

```json
wm_page.create({
  "id": "memories/<topic-slug>",
  "title": "<pattern/decision name>",
  "tags": ["<domain>", "<category>"],
  "content": "<2-3 sentence summary>"
})
```

## Step 3: Detailed Learning (full page)

For topics that need long-form explanation:

```json
wm_page.create({
  "id": "learnings/<topic-slug>",
  "title": "<Learning: Topic>",
  "tags": ["learning"],
  "content": "## Problem\n\n...\n\n## Root Cause\n\n...\n\n## Signal\n\n...\n\n## Fix\n\n..."
})
```

### Learning Doc Template

```markdown
## Problem
What was the issue or pattern being solved?

## Root Cause
What caused it? Include diagnostic steps.

## Signal
How to recognize this in the future (error messages, patterns, smells).

## Fix / Solution
The implemented solution or pattern.

## Related
- @page/...
- @task-...
```

## Step 4: Promote to Critical

If the knowledge would save ≥15 minutes for future agents, promote it to critical-patterns:

```json
wm_page.create({
  "id": "learnings/critical-patterns",
  "title": "Critical Patterns",
  "tags": ["critical"],
  "content": "## <Pattern Name>\n\n<Description>\n"
})
```

Or update if the page already exists:

```json
wm_page.update({ "id": "learnings/critical-patterns", "appendContent": "\n## <Pattern Name>\n\n<Description>\n" })
```

## Step 5: Rebuild Index

```json
wm_index.rebuild({})
```

## Checklist

- [ ] Source material reviewed
- [ ] Checked for existing memory/patterns to avoid duplicates
- [ ] Existing memory checked to avoid duplicates
- [ ] Quick memory page created under `memories/` for concise insights
- [ ] Long-form learning page created for complex topics
- [ ] Promoted to critical if high-value
- [ ] Index rebuilt

## Red Flags

- Saving incomplete or vague memories — future agents can't use them
- Duplicating existing knowledge — always search first
- Saving implementation details that will quickly become stale
- Not tagging memories — they won't surface in search
- Saving personal preferences as project-wide patterns

## Next Step Suggestion

```
/wm-plan <task-id>     — Continue with next task
/wm-commit             — Commit extracted docs
/wm-go                 — Continue pipeline
```
