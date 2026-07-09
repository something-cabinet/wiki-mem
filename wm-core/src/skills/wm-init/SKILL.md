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

- Call `wm_initial({})` first — it is the runtime bootstrap
- Use `wm_help({})` when an action schema or workflow route is not visible
- Prefer wiki docs over guessing from code structure
- If a page is large, read its TOC first and only open the relevant sections

## Step 1: Runtime Bootstrap

```json
wm_initial({})
```

Summarize project state, available tools, domains, active timer, and any warnings.

## Step 2: List Docs

```json
wm_wm_doc_list({})
```

## Step 3: Read Core Pages

```json
wm_wm_page_get({ "id": "README", "smart": true })
```

For large pages, do not read the whole file:

```json
wm_wm_page_get({ "id": "ARCHITECTURE", "toc": true })
wm_wm_page_get({ "id": "ARCHITECTURE", "section": "<heading-or-number>" })
```

## Step 4: Check Current State

```json
wm_wm_task_board({})
wm_wm_task_list({ "status": "in-progress" })
```

## Step 5: Load Critical Learnings

Check for accumulated critical learnings from past work:

```json
wm_wm_search_query({ "query": "critical patterns", "type": "page", "tag": "critical" })
```

If `learnings/critical-patterns` exists:

```json
wm_wm_page_get({ "id": "learnings/critical-patterns", "smart": true })
```

Include a brief summary in the session context if any exist.

## Step 6: Load Project Memory

```json
wm_wm_memory_list({ "layer": "project" })
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

- Skipping `wm_initial({})` — critical runtime bootstrap
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
