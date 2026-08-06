---
title: Wire up fix modes — delete, stub, ref removal
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: in-progress
priority: high
acceptance_criteria:
  - text: "--fix mode deletes stale empty task pages"
  - text: "--fix mode stubs active task pages with a description"
  - text: "--fix mode removes broken relates_to entries from YAML frontmatter"
---

Implement --fix mode: delete stale empty task pages, stub active ones with description, remove broken relates_to entries from YAML frontmatter.