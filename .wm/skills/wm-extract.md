---
name: wm-extract
description: Extract patterns, decisions, and failures from completed work into wiki pages
---

# Extract Knowledge

**Announce:** "Using wm-extract for task [id]."

**Core principle:** EXTRACT PATTERNS + DECISIONS + FAILURES → COMPOUND INTO WIKI GRAPH.

## Steps

### 1. Review what was done
```
wm_page.get(id="wiki:tasks:<task-id>")
wm_log.recent(count=20)
```

### 2. Identify three categories

| Category | What to look for |
|----------|-----------------|
| **Patterns** | Reusable approaches, architecture decisions, integration techniques |
| **Decisions** | Good calls, bad calls, trade-offs, surprises |
| **Failures** | Bugs, wrong assumptions, time wasted, missing prerequisites |

### 3. Search for existing wiki pages
```
wm_search.query(q="<pattern>", mode=keyword, limit=5)
```
Don't duplicate — update existing pages when possible.

### 4. Create wiki pages

#### For patterns
```
wm_page.create(
  path="wiki/patterns/<slug>.md",
  title="Pattern: <Name>",
  content="## When to use\n\n## How it works\n\n## Example\n\n## Source\n@task-<id>"
)
```

#### For decisions
```
wm_page.create(
  path="wiki/decisions/<slug>.md",
  title="Decision: <Title>",
  content="## Context\n\n## Chosen approach\n\n## Alternatives considered\n\n## Outcome\n\n## Source\n@task-<id>"
)
```

### 5. Link into the graph
```
wm_page.update(
  id="wiki:patterns:<slug>",
  relates_to=[
    {type: example_of, target: "wiki:concepts:<related>"},
    {type: implements, target: "wiki:specs:<spec>"}
  ]
)
```

### 6. Validate
```
wm_validate.check()
wm_lint.check()
```
