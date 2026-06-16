---
name: wm-init
description: Initialize session — read wiki context, load graph, check stale sources
---

# Session Init

**Announce:** "Using wm-init to initialize session."

**Core principle:** READ THE WIKI BEFORE DOING ANYTHING ELSE.

## Steps

### 1. Call initial state
Call `wm_initial` to get project state:
- Page count and types
- Source counts per state (pending/processing/done/error/stale)
- Graph health (nodes, edges, cycles)

### 2. Read AGENTS.md
Read `.wm/AGENTS.md` for wiki conventions and workflow instructions.

### 3. Check stale sources
Call `wm_source.verify` on recent sources. If any are stale, report to user.

### 4. Check recent activity
Read `.wm/wiki/log.md` last 10 lines to see what happened since last session.

### 5. Load critical patterns
Search for pages with `tags: [critical]` — these are the most important learnings.
