---
title: "Web UI: Page Editing + Task Interactions"
type: task
status: done
tags: [web-ui, sveltekit]
priority: medium
id: umpd47
acceptance_criteria:
  - text: "A /page/[id]/edit route exists with title/type/status/content fields that POST to wm_page.update"
  - text: "Delete page button works with a confirmation step"
  - text: "Task cards cycle todo→in-progress→done on click or via kanban drag-and-drop; keyboard shortcuts (/ focus search, n/p pagination, ? help overlay) work"
---

# Web UI: Page Editing + Task Interactions

> *Imported from Knowns task `umpd47`*

# Web UI: Page Editing + Task Interactions

## Description


(1) Edit page — add /page/[id]/edit route with title/type/status/content fields that POST to wm_page.update, (2) Delete page — button with confirmation, (3) Task status — click cards to cycle todo→in-progress→done, or drag-and-drop between kanban columns, (4) Keyboard shortcuts — / to focus search, n/p for pagination, ? for help overlay.


## Acceptance Criteria
