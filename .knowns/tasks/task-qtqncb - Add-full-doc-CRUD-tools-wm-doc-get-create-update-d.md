---
id: qtqncb
title: Add full doc CRUD tools (wm_doc.get/create/update/delete)
status: done
priority: high
labels:
  - feature
  - docs
  - knowns-parity
createdAt: '2026-07-08T11:16:25.058Z'
updatedAt: '2026-07-09T07:55:08.727Z'
timeSpent: 0
---
# Add full doc CRUD tools (wm_doc.get/create/update/delete)

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
WM only has wm_doc.list for .knowns/docs/. Need get, create, update, delete to fully manage docs via MCP. Currently users must use wm_page.* for wiki pages, but there's no bridge to Knowns docs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 wm_doc.get exists (read doc content by path)
- [x] #2 wm_doc.create exists (create new doc)
- [x] #3 wm_doc.update exists (update existing doc)
- [x] #4 wm_doc.delete exists (remove doc)
<!-- AC:END -->

