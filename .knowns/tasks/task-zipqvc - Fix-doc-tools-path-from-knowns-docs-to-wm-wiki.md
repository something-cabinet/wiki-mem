---
id: zipqvc
title: Fix doc tools path from .knowns/docs/ to .wm/wiki/
status: done
priority: high
labels:
  - sprint-0
  - bug
  - p0
createdAt: '2026-07-10T10:14:39.722Z'
updatedAt: '2026-07-10T10:19:32.347Z'
timeSpent: 174
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-1
---
# Fix doc tools path from .knowns/docs/ to .wm/wiki/

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Change wm_doc.* hardcoded paths from .knowns/docs/ to .wm/wiki/. This is a P0 bug — doc tools currently read/write a different store than page tools, causing silent data splits.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Change hardcoded .knowns/docs/ paths in mcp/tools/doc.rs to .wm/wiki/
- [x] #2 Verify wm_doc.get returns same result as wm_page.get for same path
- [x] #3 Update doc tool descriptions that reference .knowns/docs/
- [x] #4 Add migration note: existing .knowns/docs/ content must be re-imported via wm_page.create
- [x] #5 Tests pass: wm_doc.* and wm_page.* share the same store
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented: Changed wm_doc.* hardcoded paths from .knowns/docs/ to .wm/wiki/. Added ensure_md_ext helper for .md extension handling matching page.rs convention. Added wiki_docs_dir helper function. Updated all descriptions. All 42 tests pass.
<!-- SECTION:NOTES:END -->

