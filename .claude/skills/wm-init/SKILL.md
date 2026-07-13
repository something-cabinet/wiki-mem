---
name: wm-init
description: Session initialization — load docs, learnings, memory, and current state
---

**CRITICAL:** Use `task` subagents for delegation. Do NOT call `kimaki send` or `kimaki session` to create separate sessions unless the user explicitly asks for a separate thread.

# Session Initialization

**Announce:** "Using wm-init to initialize session."

**Core principle:** READ DOCS BEFORE DOING ANYTHING ELSE.

## Inputs

- Optional user focus such as a task ID, feature area, bug, or question
- Current project root already opened in the agent session

## Preflight

- Confirm this is a Knowns project
- Prefer wiki docs over guessing from code structure
- If `README`, `ARCHITECTURE`, or `CONVENTIONS` do not exist, choose the closest equivalents from the docs list
- If a doc is large, read its TOC first and only open the relevant sections
- Do not invent project conventions that were not found in docs or code

## Step 1: Runtime Bootstrap

```json
wm_initial({})
```

Summarize project state, available tools, domains, active timer, and any warnings.

## Step 2: List Docs

```json
wm_docs.list()
```

## Step 3: Read Core Pages

```json
wm_docs.get({"path": "README", "smart": true})
wm_docs.get({"path": "ARCHITECTURE", "smart": true})
wm_docs.get({"path": "CONVENTIONS", "smart": true})
```

For large pages, do not read the whole file:

```json
wm_docs.get({"path": "ARCHITECTURE", "toc": true})
wm_docs.get({"path": "ARCHITECTURE", "section": "<heading-or-number>"})
```

### Fallbacks

- If core docs are missing, say which docs were not found and which substitutes were used
- If task search/list is unavailable, state that clearly and continue with docs + codebase context

## Step 4: Check Current State

```json
wm_tasks.list({"status": "in-progress"})
wm_tasks.board()
```

## Step 5: Load Critical Learnings

Check for accumulated critical learnings from past work:

```json
wm_search.search({"query": "critical patterns", "type": "doc", "tag": "critical"})
```

If `learnings/critical-patterns` exists:

```json
wm_docs.get({"path": "learnings/critical-patterns", "smart": true})
```

These are promoted learnings that cost the most to discover and save the most by knowing. Include a brief summary in the session context if any exist.

## Step 6: Load Project Memory

```json
wm_memory.list({"layer": "project"})
```

Project memories contain accumulated patterns, decisions, and conventions from past work. Include key entries in the session context summary. Prioritize by recency and relevance to the user's stated focus.

## Step 7: Load Global Memory

```json
wm_memory.list({"layer": "global"})
```

Global memories contain cross-project knowledge — tooling config, universal conventions, personal preferences, and patterns applicable to any project. Always include these in the session context as they may affect how work is done. If there are many entries, prioritize by recency and relevance.

## Step 8: Summarize

```markdown
## Session Context
- **Project**: [name]
- **Key Docs**: README, ARCHITECTURE, CONVENTIONS
- **Critical Learnings**: [count, or "none yet"]
- **Project Memories**: [count, or "none yet"]
- **Global Memories**: [count, or "none yet"]
- **In-progress tasks**: [count]
- **Current risks / gaps**: [missing docs, unclear conventions, broken search, etc.]
- **Ready for**: tasks, docs, questions
```

## Checklist

- [ ] Runtime bootstrap called
- [ ] Docs listed
- [ ] Core pages read (README, ARCHITECTURE, CONVENTIONS)
- [ ] Fallbacks applied if core docs missing
- [ ] Task board and in-progress checked
- [ ] Critical learnings loaded
- [ ] Project memory loaded
- [ ] Global memory loaded
- [ ] Session context summary provided

## Red Flags

- Skipping `wm_initial({})` — critical runtime bootstrap
- Reading full large pages without checking TOC first
- Inventing project conventions not found in docs or code
- Failing to report missing core docs
- Skipping global memory load — may miss cross-project preferences

## Next Step Suggestion

After initialization, recommend one concrete next step:

```
/wm-plan <task-id>        — Plan a task
/wm-flow @page/<spec>     — Orchestrate an approved spec
/wm-research <query>      — Research a topic
/wm-spec                  — Create a new spec
```
