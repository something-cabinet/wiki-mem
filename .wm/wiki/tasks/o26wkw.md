---
title: Add wm_template.create tool
type: task
status: done
tags: [feature, templates, knowns-parity]
priority: medium
id: o26wkw
---

# Add wm_template.create tool

> *Imported from Knowns task `o26wkw`*

# Add wm_template.create tool

## Description


WM's template system supports list/get/run but not create. Templates must be manually placed as JSON files in .wm/templates/. Add a create tool that accepts name, description, content with {{variable}} placeholders.


## Acceptance Criteria

- [x] #1 wm_template.create accepts name, description, content
- [x] #2 Template written to .wm/templates/<name>.json
- [x] #3 Created template appears in wm_template.list
