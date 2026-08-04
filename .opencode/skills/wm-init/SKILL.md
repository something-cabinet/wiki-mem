---
name: wm-init
description: Session initialization — load docs, learnings, memory, and current state
---

**CRITICAL:** Use `task` subagents for delegation. Do NOT create separate sessions or threads unless the user explicitly asks for one.

**IMPORTANT:** Use `wm_*` MCP tools (`wm_page.*`, `wm_search.*`, `wm_task.*`, `wm_memory.*`, etc.) for all operations.

# Session Initialization

**Announce:** "Using wm-init to initialize session."

**Core principle:** READ DOCS BEFORE DOING ANYTHING ELSE.

## Inputs

- Optional user focus such as a task ID, feature area, bug, or question
- Current project root already opened in the agent session

## Preflight

- Confirm the project root contains `.wm/` and `.wm/config.json`
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
wm_doc.list({"action": "list"})
```

## Step 3: Read Core Pages

```json
wm_doc.get({"action": "get", "id": "README"})
wm_doc.get({"action": "get", "id": "ARCHITECTURE"})
wm_doc.get({"action": "get", "id": "CONVENTIONS"})
```

For large pages, do not read the whole file:

```json
wm_doc.get({"action": "get", "id": "ARCHITECTURE"})
wm_doc.get({"action": "get", "id": "ARCHITECTURE"})
```

### Fallbacks

- If core docs are missing, say which docs were not found and which substitutes were used
- If task search/list is unavailable, state that clearly and continue with docs + codebase context

## Step 4: Check Current State

```json
wm_task.list({"status": "in-progress"})
wm_task.board()
```

## Step 5: Load Critical Learnings

Check for accumulated critical learnings from past work:

```json
wm_search.query({"q": "critical patterns", "type": "doc"})
```

If `learnings/critical-patterns` exists:

```json
wm_doc.get({"action": "get", "id": "wiki:learnings/critical-patterns"})
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


## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-init`, the key details should cover:
- project state summary
- available docs, memories, tasks
- current risks or gaps

## Related Skills

- `/wm-plan <task-id>` — Plan a task
- `/wm-research <query>` — Research a topic
- `/wm-spec` — Create a new spec

## Next Step Suggestion

After initialization, recommend one concrete next step:

```
/wm-plan <task-id>        — Plan a task
/wm-flow @page/<spec>     — Orchestrate an approved spec
/wm-research <query>      — Research a topic
/wm-spec                  — Create a new spec
```
