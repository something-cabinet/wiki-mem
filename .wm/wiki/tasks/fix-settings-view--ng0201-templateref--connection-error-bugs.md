---
title: Fix Settings view — NG0201 TemplateRef + Connection Error bugs
type: task
status: todo
priority: high
tags: [bug, web-ui, settings]
---

Settings view has two bugs identified during playwriter testing:
1. NG0201: No provider for TemplateRef — BrnPopoverContent/Spartan UI component issue. Likely a popover used without proper structural directive context.
2. Connection Error — "Failed to load settings. Check that the server is running." Settings view makes an API call that fails or gets an unexpected response format.
These are pre-existing issues from the designer review (infinite spinner bug was already known) but need fixing.