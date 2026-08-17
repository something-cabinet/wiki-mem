---
title: wm_memory list ignores layer=global — returns project memory
type: task
id: "wiki:tasks:wmmemory-list-ignores-layerglobal--returns-project-memory"
status: todo
priority: high
tags: [bug, tool-reliability, memory, mcp]
acceptance_criteria:
  - text: "wm_memory list with layer=global returns entries from the global store (~/.wm/memory or the global_memory_path target), not the project's .wm/wiki/memory pages"
  - text: "layer=project and layer=global return different result sets when a global entry exists that the project does not have"
  - text: "An unknown/unsupported layer value is rejected or documented rather than silently falling back to project"
  - text: "Regression test covers list(layer=global) vs list(layer=project) divergence"
---

Reproduction (session 2026-08-14, wm-init): wm_memory list with layer=project and layer=global both returned the identical 50 entries. Root cause in apps/wm-core/src/mcp/tools/memory.rs:175-190 — the List handler only branches on is_session(&layer); every non-session layer falls through to page::list_pages(PageType::Memory), i.e. the project wiki memory dir. The global layer is never read even though the add/promote path has a global_memory_path (memory.rs:407). Effect: agents following wm-init Step 7 (load global memory) silently get project memory and never see cross-project preferences. Also note total memory pages is 105 (wm_initial) while list caps at limit=50 with no pagination signal.