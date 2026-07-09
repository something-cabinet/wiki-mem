---
name: wm-extract
description: Extract reusable patterns, decisions, and failures into wiki pages
---

# Extracting Knowledge

**Announce:** "Using wm-extract for [pattern/decision]."

**Core principle:** IF IT COST TIME TO LEARN, SAVE IT FOR LATER.

## Inputs

- Source task ID, spec ref, or area of work
- Type of extraction: pattern, decision, failure, or convention

## Wiki Page Type Mapping

| Extraction Type | Wiki Subdirectory | PageType |
|----------------|-------------------|----------|
| Pattern | `patterns/` | Pattern |
| Decision | `decisions/` | Decision |
| Convention | `patterns/` | Pattern |
| Failure / Learning | `concepts/` | Concept |
| How-to | `howto/` | Howto |
| Reference | `reference/` | Reference |

Wiki pages are stored as `.wm/wiki/<subdir>/<slug>.md` and accessible via `wm_wm_page_get({"id": "<subdir>/<slug>"})`.

## Step 1: Review Source Material

```json
wm_wm_task_get({ "taskId": "$ARGUMENTS" })
wm_wm_log_recent({ "limit": 20 })
```

Review the task, recent logs, and changes to identify what is worth capturing. Determine the extraction type.

## Step 2: Check for Duplicates

Search existing wiki pages and Knowns memory to avoid duplicating knowledge:

```json
wm_wm_search_query({ "query": "<topic>", "type": "all", "mode": "keyword" })
wm_wm_memory_list({ "category": "pattern", "tag": "<domain>" })
```

If the topic already exists, skip or update instead of creating a duplicate.

## Step 3: Create Wiki Page

Create a wiki page in the appropriate subdirectory based on extraction type:

```json
wm_wm_page_create({
  "id": "<subdir>/<topic-slug>",
  "title": "<Pattern/Decision Name>",
  "page_type": "<pattern|decision|concept|howto|reference>",
  "tags": ["<search-keyword>"],  # Specific search keyword (e.g., "arc-swap", "rrf-fusion"), not domain/category
  "content": "<markdown content>"
})
```

### Pattern Template (for reusable solutions)

```markdown
## Problem
What problem does this pattern solve?

## Solution
The reusable approach or implementation.

## When to Use
Signals that indicate this pattern applies.

## When Not to Use
Contexts where this pattern adds unnecessary complexity.

## Related
- @page/patterns/...
- @task-...
```

### Decision Template (for architectural choices)

```markdown
## Context
What situation led to this decision?

## Decision
What was chosen.

## Rationale
Why this option over alternatives (trade-offs considered).

## Consequences
What this decision means for future work.

## Related
- @page/decisions/...
- @task-...
```

### Learning/Failure Template (for hard-won lessons)

```markdown
## Problem
What was the issue?

## Root Cause
What caused it? Include diagnostic steps.

## Signal
How to recognize this in the future.

## Fix
The implemented solution.

## Related
- @page/...
- @task-...
```

## Step 4: Save Quick Memory (recall aid)

For concise insights that should surface quickly in future sessions, also save as a Knowns project memory:

```json
wm_wm_memory_add({ "title": "<Pattern Name>",
  "content": "<2-3 sentence summary>",
  "category": "<pattern|decision|failure>",
  "tags": ["<domain>"]
})
```

Check existing memory first to avoid duplicates.

## Step 5: Promote to Critical

If the knowledge would save ≥15 minutes for future agents, add it to `learnings/critical-patterns` (Knowns doc):

```json
wm_wm_page_update({
  "id": "learnings/critical-patterns",
  "appendContent": "\n---\n\n## [<date>] <Pattern Name>\n**Category:** <type>\n\n<2-3 sentence description>\n"
})
```

If the page doesn't exist yet, create it instead.

## Step 6: Rebuild Index

```json
wm_wm_index_rebuild({})
```

## Checklist

- [ ] Source material reviewed
- [ ] Extraction type determined (pattern/decision/failure/convention)
- [ ] Checked for existing wiki pages and memory to avoid duplicates
- [ ] Wiki page created in correct subdirectory (`patterns/`, `decisions/`, `concepts/`, etc.)
- [ ] Used appropriate template for the extraction type
- [ ] Quick memory entry created
- [ ] Promoted to critical-patterns if high-value
- [ ] Index rebuilt

## Red Flags

- Saving incomplete or vague knowledge — future agents can't use it
- Duplicating existing knowledge — always search first
- Saving implementation details that will quickly become stale
- Not tagging pages — they won't surface in search
- Creating pages in wrong wiki subdirectory (use the type mapping)
- Saving personal preferences as project-wide patterns

## Next Step Suggestion

```
/wm-plan <task-id>     — Continue with next task
/wm-commit             — Commit extracted docs
/wm-go                 — Continue pipeline
```
