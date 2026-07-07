---
name: wm-extract
description: Extract reusable patterns, decisions, and failures into wiki docs
---

# Extracting Knowledge

**Announce:** "Using wm-extract for [pattern/decision]."

**Core principle:** IF IT COST TIME TO LEARN, SAVE IT FOR LATER.

## Quick Memory (fast recall)

```json
wm_memory.add({ "title": "<pattern>",
  "content": "<2-3 sentence summary>",
  "layer": "project",
  "category": "<pattern|decision|failure>",
  "tags": ["<domain>"] })
```

## Detailed Learning (full doc)

```json
wm_doc.create({ "title": "Learning: <topic>",
  "folder": "learnings",
  "tags": ["learning"],
  "content": "## Problem\n\n...\n\n## Root Cause\n\n...\n\n## Signal\n\n...\n\n## Fix\n\n..." })
```

Promote to critical-patterns if it would save ≥15 minutes for future agents.
