---
title: "Web UI: Dark Mode + Toasts + Polish"
type: task
status: done
tags: [web-ui, sveltekit]
priority: low
id: 94qxox
acceptance_criteria:
  - text: "Dark mode works via a prefers-color-scheme media query plus a manual toggle in the nav"
  - text: "console.error calls are replaced with on-screen toasts for errors and successes"
  - text: "vis-network is lazy-loaded (no longer a 514KB chunk on every page), and the sources page exposes reprocess/delete actions"
---

# Web UI: Dark Mode + Toasts + Polish

> *Imported from Knowns task `94qxox`*

# Web UI: Dark Mode + Toasts + Polish

## Description


(1) Dark mode — CSS variables already ready, add prefers-color-scheme media query + manual toggle in nav, (2) Toast/notification system — replace console.error with on-screen toasts for errors/successes, (3) Lazy-load vis-network (514KB chunk currently loaded on every page), (4) Source management actions (reprocess, delete) in sources page.


## Acceptance Criteria
