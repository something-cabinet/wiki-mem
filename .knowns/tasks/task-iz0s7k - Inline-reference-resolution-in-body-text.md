---
id: iz0s7k
title: 'Inline @reference resolution in body text'
status: done
priority: medium
labels:
  - sprint-2
  - feature
  - references
createdAt: '2026-07-10T10:15:45.037Z'
updatedAt: '2026-07-10T11:12:15.192Z'
timeSpent: 149
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-5
---
# Inline @reference resolution in body text

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Parse @doc/, @task/, @memory/, @decision/, @template/ references in body text. Resolve to rendered content or links. Respect code blocks (skip references inside ```). Reuse parser infrastructure for pages, memory, templates, and search.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented inline @reference resolution: new reference.rs module with extract_references() parsing @doc/, @task/, @memory/, @decision/, @template/ patterns. Code block awareness. 3 MCP tools: wm_ref.extract, wm_ref.resolve, wm_ref.resolve_all. 6 unit tests. 78/78 tests pass.
<!-- SECTION:NOTES:END -->

