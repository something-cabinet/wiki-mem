---
title: Add full doc CRUD tools (wm_doc.get/create/update/delete)
type: task
status: done
tags: [feature, docs, knowns-parity]
priority: high
id: qtqncb
---

# Add full doc CRUD tools (wm_doc.get/create/update/delete)

> *Imported from Knowns task `qtqncb`*

# Add full doc CRUD tools (wm_doc.get/create/update/delete)

## Description


WM only has wm_doc.list for .knowns/docs/. Need get, create, update, delete to fully manage docs via MCP. Currently users must use wm_page.* for wiki pages, but there's no bridge to Knowns docs.


## Acceptance Criteria

- [x] #1 wm_doc.get exists (read doc content by path)
- [x] #2 wm_doc.create exists (create new doc)
- [x] #3 wm_doc.update exists (update existing doc)
- [x] #4 wm_doc.delete exists (remove doc)
