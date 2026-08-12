---
title: 'Fix: wire anchored path resolution into update_page_with_repo'
type: task
id: wiki:tasks:fix-update-page-anchored-path
status: todo
---

## Finding (2026-08-12, gate 1 deferral)

`update_page_with_repo` resolves wiki-relative `meta.path` values against the process CWD, while the `anchored_page_path` confinement fix in page_crud_service.rs is not wired into update. Phase 1 masked this with a chdir-to-project-root in wm-cli's `engine_handle()` + `Commands::Mcp`. Root-cause fix: use anchored resolution in the update path; keep the chdir as belt-and-suspenders.

## Acceptance criteria

- [ ] update_page_with_repo resolves wiki-relative meta.path against the project root, not process CWD
- [ ] anchored_page_path (or equivalent) is wired into the update path
- [ ] path_resolution_test::cli_page_crud_from_wiki_dir_cwd_resolves_meta_path passes WITHOUT the wm-cli chdir (temporarily disabled to prove the fix)
- [ ] zero warnings; suite green

