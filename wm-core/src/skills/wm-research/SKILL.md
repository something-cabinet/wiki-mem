---
name: wm-research
description: Research project context, code, and relevant sources
---

# Research

**Announce:** "Using wm-research for [query]."

**Core principle:** SEARCH FIRST → READ → SYNTHESIZE.

## Step 1: Cross-Entity Search

Search both wiki pages and memory entries simultaneously (default type: "all"):

```json
wm_search.query({ "query": "<topic>", "mode": "hybrid" })
```

Use `wm_search.retrieve` for assembled context packs with citations.

Filter by type when needed:
- `type: "all"` — pages + memory (default, omit type param)
- `type: "page"` — wiki pages only
- `type: "memory"` — memory entries only
- `type: "task"` — task pages only

```json
wm_search.query({ "query": "<topic>", "type": "memory" })
```

## Step 2: Read Relevant Pages

```json
wm_page.get({ "id": "<page-id>" })
```

## Step 3: Graph Exploration

```json
wm_graph.neighbors({ "id": "<page-id>" })
```

Follow related pages through typed edges (extends, depends_on, relates_to, implements).

## Step 4: Synthesize

Present findings in a structured summary with page references.
