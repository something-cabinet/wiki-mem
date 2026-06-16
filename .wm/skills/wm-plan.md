---
name: wm-plan
description: Take a task, gather context from wiki graph, create implementation plan
---

# Plan Task

**Announce:** "Using wm-plan for task [id]."

## Steps

### 1. Read the task
```
wm_page.get(id="wiki:tasks:<task-id>")
```

### 2. Research the wiki graph
```
wm_graph.neighbors(id="wiki:concepts:<related>", query="<task topic>")
wm_search.query(q="<topic>", mode=hybrid, limit=10)
```

Follow edges: `depends_on` → prerequisites, `implements` → specs, `extends` → base patterns.

### 3. Read relevant specs and concepts
For each relevant page, call `wm_page.get` to read full content.

### 4. Create implementation plan
Write plan with:
- What needs to be built
- Which existing patterns to follow
- Which specs to implement
- Dependencies and risks

### 5. Save plan
```
wm_page.update(
  id="wiki:tasks:<task-id>",
  content="...plan appended..."
)
```

## Quality guidelines
- Search before reading — don't load every page
- Reference existing patterns by wiki ID (`wiki:patterns:<name>`)
- Note `depends_on` edges for dependency tracking
