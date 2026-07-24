---
name: wm-research
description: Research project context, code, and relevant sources using search and graph traversal
---

# Research

**Announce:** "Using wm-research for [query]."

**Core principle:** UNDERSTAND WHAT EXISTS BEFORE ADDING NEW CODE.

## Inputs

- Natural language research query or topic
- Optional type filter: page, memory, task, or all
- Optional graph starting point for related-page exploration

## Search Order

1. Project docs and memories (unified search)
2. Expand context via structural relations (if spec/doc found)
3. Completed or related tasks (keyword search for gaps)
4. Existing code paths and implementations
5. Adjacent tests, templates, and validation logic

## Step 1: Search Documentation and Memory

```json
wm_search.query({"q": "<topic>", "type": "doc"})
wm_search.query({"q": "<topic>", "type": "memory"})
```

Use `search` for discovery-first research. Use `retrieve` when the next consumer needs assembled context with citations:

```json
wm_search.retrieve({"q": "<topic>"})
```

If relevant memories appear, include them in findings and note whether they're still current.

## Step 2: Expand Context via Relations

If Step 1 found a spec or doc relevant to the topic, use structural resolve to discover related tasks, dependencies, and implementation status **before** searching tasks by keyword:

```json
// Found specs/ai-permission-model in Step 1 → find all tasks implementing it
wm_search.resolve({"q": "<spec-path>"})

// Found a doc that others depend on → find what depends on it
wm_search.resolve({"q": "<spec-path>"})
```

Skip this step only if Step 1 returned no relevant docs or specs.

## Step 3: Search Completed Tasks

```json
wm_search.query({"q": "<keywords>", "type": "all"})
wm_task.get({"id": "<id>"})
```

If Step 2 already found related tasks via structural resolve, focus keyword search on gaps — tasks that might be related but not formally linked.

## Step 4: Graph Exploration

Follow related pages through typed edges:

```json
wm_graph.neighbors({"id": "<page-id>"})
```

Typed edges: `extends`, `depends_on`, `relates_to`, `implements`.

For broader exploration:

```json
wm_graph.neighbors({"id": "<start-id>", "depth": 1})
wm_graph.neighbors({"id": "<page-id>", "depth": 2})
```

## Step 5: Synthesize Findings

```markdown
## Research: [Topic]

### Existing Implementations
- `src/path/file.ts`: Does X

### Patterns Found
- Pattern 1: Used for...

### Related Docs
- @doc/path1 - Covers X

### Recommendations
1. Reuse X from Y
2. Follow pattern Z
```

Present findings with page references, key insights, and actionable next steps.

## Knowledge Spillover Rule

If the research surface becomes too large for one response or one task:

- Create or update a wiki page for the reusable/domain knowledge
- Reference that doc from the current task or plan with `@doc/<path>`
- Keep the research summary short and point to the canonical doc instead of repeating everything inline

If the research uncovers a broad follow-up topic that should be tracked independently:

- Create a task for that general knowledge or follow-up work
- Reference it with `@task-<id>` from the current context
- Do not silently expand the original task with unrelated background work

## Fallbacks

- If search is noisy, narrow by file type, feature folder, or known reference IDs
- If no existing pattern is found, state that explicitly rather than implying one exists
- If docs and code disagree, call out the mismatch

## Checklist

- [ ] Searched documentation and memory first
- [ ] Expanded context via structural resolve (if spec/doc found)
- [ ] Reviewed similar completed tasks
- [ ] Explored graph connections
- [ ] Found existing code patterns
- [ ] Identified reusable components
- [ ] Findings synthesized with page references
- [ ] Knowledge spillover applied if surface too large

## Red Flags

- Diving into code search before checking docs — docs first, code second
- Ignoring graph connections — neighboring pages often contain critical context
- Not using `retrieve` when implementation needs organized context packs
- Silently expanding scope with background research not related to the query
- No response when no pattern exists — state explicitly

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-research`, the key details should cover:
- research findings, sources consulted, confidence level, actionable recommendations

## Related Skills

- `/wm-spec` — Create a spec based on findings
- `/wm-plan <task-id>` — Plan implementation with researched context
- `/wm-flow @page/<spec>` — Full pipeline with researched knowledge


## Next Step Suggestion

```
/wm-spec              — Create a spec based on findings
/wm-plan <task-id>    — Plan implementation with researched context
/wm-go @page/<spec>   — Full pipeline with researched knowledge
```
