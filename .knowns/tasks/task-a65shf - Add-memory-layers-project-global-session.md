---
id: a65shf
title: Add memory layers — project, global, session
status: done
priority: high
labels:
  - feature
  - memory
  - knowns-parity
createdAt: '2026-07-08T11:16:24.189Z'
updatedAt: '2026-07-09T07:54:57.735Z'
timeSpent: 0
---
# Add memory layers — project, global, session

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
WM stores all memory flat in .wm/memory/. Knowns has 3 layers: project (project-scoped), global (~/.knowns/memory/), session (ephemeral working memory). Add layer parameter to wm_memory.add/get/list, add promote action to move project→global.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 wm_memory.add accepts layer parameter (project/global/session)
- [x] #2 Global memory stored at ~/.wm/memory/
- [x] #3 Session memory is ephemeral (not persisted)
- [ ] #4 promote action moves project→global memory
<!-- AC:END -->

