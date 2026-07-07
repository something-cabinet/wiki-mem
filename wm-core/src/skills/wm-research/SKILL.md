---
name: wm-research
description: Research project context, code, and relevant sources
---

# Research

**Announce:** "Using wm-research for [query]."

**Core principle:** SEARCH FIRST → READ → SYNTHESIZE.

## Step 1: Search Wiki

```json
wm_search.query({ "query": "<topic>", "mode": "hybrid" })
```

Use `wm_search.retrieve` for structured context with citations when broad understanding is needed.

## Step 2: Search Memory

```json
wm_search.query({ "query": "<topic>", "type": "memory" })
```

## Step 3: Read Relevant Pages

```json
wm_page.get({ "id": "<page-id>" })
```

## Step 4: Graph Exploration

```json
wm_graph.neighbors({ "id": "<page-id>" })
```

Follow related pages through typed edges (extends, depends_on, relates_to, implements).

## Step 5: Synthesize

Present findings in a structured summary with page references.
