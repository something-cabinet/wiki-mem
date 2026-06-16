---
name: wm-research
description: Search the wiki graph, explore neighbors, find related patterns
---

# Research

**Announce:** "Using wm-research."

## Steps

### 1. Search the wiki
```
wm_search.query(q="<topic>", mode=hybrid, limit=20)
```

### 2. Read top matches
```
wm_page.get(id="<result-id>")
```

### 3. Explore the graph
```
wm_graph.neighbors(id="<page-id>", query="<topic>")
```

### 4. Follow interesting edges
- `extends` → base concepts
- `implements` → concrete patterns  
- `part_of` → parent systems
- `contradicts` → alternative approaches

### 5. Retrieve context pack
For complex topics, use `wm_search.retrieve` with a token budget:
```
wm_search.retrieve(q="<topic>", token_budget=8192)
```

## Efficiency rules
- Search before reading — don't read all docs hoping to find info
- Use `wm_graph.neighbors` with query for topic-aware sorting
- Read selectively — only fetch pages you need
