---
id: okkl5q
title: Knowns migration — per-doc classification and re-import
status: todo
priority: medium
labels:
  - final
  - migration
  - cleanup
createdAt: '2026-07-10T10:15:55.671Z'
updatedAt: '2026-07-10T10:15:55.671Z'
timeSpent: 0
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-12
---
# Knowns migration — per-doc classification and re-import

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Walk all .knowns/docs/ files. Per-doc AI classification to determine best WM page type. wm_page.create re-import with correct type, frontmatter, and path. Map dirs: concepts→concepts, specs→specs, learnings→concepts, handover→reference, knowns→reference. Remove .knowns/ directory. Remove Knowns shims from configs. Ensure all @doc/ refs still resolve.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

