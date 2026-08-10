---
title: wm_task NOT_FOUND after index rebuild — task store loses tasks created via API
type: task
tags:
- bug
- tool-reliability
- task-store
status: done
priority: high
implementation_notes: |-
  Reproduced 2026-07-31 via wm-extract: task created via wm_task.create (id wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse) resolved fine pre-rebuild (wm_task.update/check_ac worked). After an index rebuild, wm_task.get by that id → NOT_FOUND; wm_task.list now returns short hash ids (98a7ff-style). File on disk .wm/wiki/tasks/wm-index-code-output-misleading-*.md had frontmatter with ONLY status + implementation_notes — id:/title:/type: absent — so the store re-derived ids and lost the API-created mapping. Workaround applied: rewrote the file via wm_page.update embedding full frontmatter (id/title/type/status/tags/implementation_notes) in content. Recommend: wm_task.create must always write id:/title:/type: into the task file frontmatter.
  Root cause family resolved by task 7ce26d (2026-08-07): shared graph-index-first/disk-fallback resolver (resolve_page_meta) wired into wm_task get/update/delete + update_page_with_repo. Tasks created via API that exist on disk are now resolvable even when the index is stale — no NOT_FOUND. See @wiki/patterns/stale-index-disk-fallback. Verify ACs against current behavior before closing.
acceptance_criteria:
- text: 'wm_task.create always writes id:/title:/type: into the task file frontmatter'
- text: Tasks created via the API remain resolvable (wm_task.get/update/check_ac) after wm_index.rebuild
- text: On-disk acceptance_criteria persist their checked state across index rebuilds
---

A task created via wm_task.create (id: wiki:tasks:bundle-angular-frontend-with-wm-server-for-npm-distribution) was fully usable (get/update/check_ac worked, plan saved). After running wm_index.rebuild (skip_embed=true), wm_task.get and wm_task.update return NOT_FOUND for the same ID even though the file exists at .wm/wiki/tasks/bundle-angular-frontend-with-wm-server-for-npm-distribution.md with correct frontmatter. wm_search finds the page (as a page), but the task store cannot resolve it. check_ac returned success earlier but the on-disk acceptance_criteria still show checked: false — AC state did not persist to the file.

Impact: cannot transition task to done, cannot append implementation notes, AC check state lost. Task store appears to cache task metadata at startup; index rebuild drops tasks not present at cache-build time, or the task store's id_index is stale.

Repro:
1. wm_task.create → OK
2. wm_task.update (status in-progress, assignee) → OK
3. wm_task.check_ac ×4 → OK (returns checked arrays)
4. wm_index.rebuild (skip_embed=true)
5. wm_task.get → NOT_FOUND (file still on disk)

Expected: task store reads from disk or rebuilds its index after wm_index.rebuild so tasks created via API remain resolvable.