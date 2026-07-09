---
id: o26wkw
title: Add wm_template.create tool
status: done
priority: medium
labels:
  - feature
  - templates
  - knowns-parity
createdAt: '2026-07-08T11:16:25.713Z'
updatedAt: '2026-07-09T07:54:56.785Z'
timeSpent: 0
---
# Add wm_template.create tool

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
WM's template system supports list/get/run but not create. Templates must be manually placed as JSON files in .wm/templates/. Add a create tool that accepts name, description, content with {{variable}} placeholders.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 wm_template.create accepts name, description, content
- [x] #2 Template written to .wm/templates/<name>.json
- [x] #3 Created template appears in wm_template.list
<!-- AC:END -->

