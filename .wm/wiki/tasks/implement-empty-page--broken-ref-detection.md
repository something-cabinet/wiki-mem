---
title: Implement empty page + broken ref detection
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: done
priority: high
acceptance_criteria:
- text: Health audit scans wiki pages and detects empty tasks (zero parseable sections)
- text: Health audit scans YAML frontmatter and reports relates_to targets that don't exist in the graph
- text: Detection results are surfaced in the health audit output with affected page references
---

Implement health audit detection logic: scan wiki pages for zero parseable sections (empty tasks), and scan YAML frontmatter for relates_to targets that don't exist in the graph.