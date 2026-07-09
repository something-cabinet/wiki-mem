---
name: wm-research
description: Research project context, code, and relevant sources using search and graph traversal
---

# Research

**Announce:** "Using wm-research for [query]."

**Core principle:** SEARCH FIRST → READ → SYNTHESIZE.

## Inputs

- Natural language research query or topic
- Optional type filter: page, memory, task, or all
- Optional graph starting point for related-page exploration

## Step 1: Cross-Entity Search

Search both wiki pages and memory entries simultaneously:

```json
wm_search_query({ "query": "<topic>", "mode": "hybrid" })
```

Filter by type when narrowed focus is needed:

```json
wm_search_query({ "query": "<topic>", "type": "memory" })
wm_search_query({ "query": "<topic>", "type": "page" })
wm_search_query({ "query": "<topic>", "type": "task" })
```

## Step 2: Retrieve with Context

When search hits need assembled context with citations:

```json
wm_search_retrieve({ "query": "<topic>" })
```

## Step 3: Read Relevant Pages

```json
wm_page_get({ "id": "<page-id>" })
```

## Step 4: Graph Exploration

Follow related pages through typed edges:

```json
wm_graph_neighbors({ "id": "<page-id>" })
```

Typed edges: `extends`, `depends_on`, `relates_to`, `implements`.

For broader exploration:

```json
wm_graph_path({ "from": "<start-id>", "to": "<target-id>" })
wm_graph_subgraph({ "id": "<page-id>", "depth": 2 })
```

## Step 5: Synthesize

Present findings in a structured summary with page references, key insights, and actionable next steps.

## Checklist

- [ ] Searched with correct mode and type filters
- [ ] Retrieved assembled context when needed
- [ ] Read relevant pages
- [ ] Explored graph connections
- [ ] Findings synthesized with page references

## Red Flags

- Searching without narrowing type when result set is too large
- Ignoring graph connections — neighboring pages often contain critical context
- Not using `retrieve` when implementation needs organized context packs

## Next Step Suggestion

```
/wm-spec              — Create a spec based on findings
/wm-plan <task-id>    — Plan implementation with researched context
/wm-go @page/<spec>   — Full pipeline with researched knowledge
```
