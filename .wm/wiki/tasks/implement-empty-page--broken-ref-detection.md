---
title: Implement empty page + broken ref detection
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: in-progress
priority: high
---

Implement health audit detection logic: scan wiki pages for zero parseable sections (empty tasks), and scan YAML frontmatter for relates_to targets that don't exist in the graph.