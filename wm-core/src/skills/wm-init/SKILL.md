---
name: wm-init
description: Use at the start of a new session to read project docs, understand context, and see current state
---

# Session Initialization

**Announce:** "Using wm-init to initialize session."

**Core principle:** BOOTSTRAP WITH MCP INITIAL → DISCOVER WITH HELP → READ ONLY RELEVANT DOCS.

## Inputs

- Optional user focus such as a task ID, feature area, bug, or question
- Current project root already opened in the agent session

## Preflight

- Call `initial({})` first — it is the runtime bootstrap
- Use `help({})` when an action schema or workflow route is not visible
- Prefer wiki docs over guessing from code structure
- If a page is large, read its TOC first and only open the relevant sections

## Step 1: Runtime Bootstrap

```json
initial({})
```

Summarize project state, available tools, domains, active timer, and any warnings.

## Step 2: List Docs

```json
doc.list({})
```

## Step 3: Read Core Pages

```json
page.get({ "id": "README", "smart": true })
```

For large pages, do not read the whole file:

```json
page.get({ "id": "ARCHITECTURE", "toc": true })
page.get({ "id": "ARCHITECTURE", "section": "<heading-or-number>" })
```

## Step 4: Check Current State

```json
task.board({})
task.list({ "status": "in-progress" })
```

## Step 5: Load Critical Learnings

Check for accumulated critical learnings from past work:

```json
search.query({ "query": "critical patterns", "type": "page", "tag": "critical" })
```

If `learnings/critical-patterns` exists:

```json
page.get({ "id": "learnings/critical-patterns", "smart": true })
```

Include a brief summary in the session context if any exist.

## Step 6: Load Project Memory

```json
memory.list({ "layer": "project" })
```

Project memories contain accumulated patterns, decisions, and conventions from past work. Include key entries in the session context summary. Prioritize by recency and relevance to the user's stated focus.

## Step 7: Summarize

```markdown
## Session Context
- **Project**: [name]
- **Key Docs**: README, ARCHITECTURE, CONVENTIONS
- **Critical Learnings**: [count, or "none yet"]
- **Project Memories**: [count, or "none yet"]
- **In-progress tasks**: [count]
- **Current risks / gaps**: [missing docs, unclear conventions, broken search, etc.]
- **Ready for**: tasks, docs, questions
```

## Checklist

- [ ] Runtime bootstrap called
- [ ] Docs listed
- [ ] Core pages read (README, ARCHITECTURE, CONVENTIONS)
- [ ] Task board and in-progress checked
- [ ] Critical learnings loaded
- [ ] Project memory loaded
- [ ] Session context summary provided

## Red Flags

- Skipping `initial({})` — critical runtime bootstrap
- Reading full large pages without checking TOC first
- Inventing project conventions not found in docs or code
- Failing to report missing core docs

## Next Step Suggestion

After initialization, recommend one concrete next step:

```
/wm-plan <task-id>        — Plan a task
/wm-flow @page/<spec>     — Orchestrate an approved spec
/wm-research <query>      — Research a topic
/wm-spec                  — Create a new spec
```

