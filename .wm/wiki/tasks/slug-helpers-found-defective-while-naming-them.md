---
title: Slug helpers found defective while naming them
type: task
id: "wiki:tasks:slug-helpers-found-defective-while-naming-them"
status: todo
tags: [bugfix, refactor, naming, wm-core]
acceptance_criteria:
  - text: 'task_page_id_slug no longer collides: distinct titles cannot produce the same page id or path, or the collision is documented and accepted with a migration note'
  - text: memory_entry_id_slug and task_page_id_slug either share a stated id convention or document why they differ
  - text: label_slug is given a production call site or deleted - no dead public API
  - text: Each helper keeps a pinning test from the naming pass
priority: high
relates_to:
  - {type: references, target: wiki:patterns:name-distinct-behaviours-do-not-converge-them}
---

schema_version: 1
state: |-
  Found on 2026-10-16 while giving three look-alike `slugify` copies honest names (@wiki/patterns:name-distinct-behaviours-do-not-converge-them). Naming exposed three defects that convergence would have hidden. **None of these was changed during the naming pass** - the lane reported them instead.

  1. **`task_page_id_slug` is lossy and collision-prone.** `apps/wm-core/src/mcp/tools/task/mod.rs`. It deletes every non-alphanumeric character and does not collapse runs, so `"my_task"` and `"mytask"` both produce `mytask` - two distinct titles map to one page id and one file path. It also emits `auth--security` for `"Auth & Security!"` and `hello--world` for `"Hello  World"`. It produces `wiki:tasks:{slug}` (`:327`), the path `tasks/{slug}` (`:382`) and subtask ids (`:825`).

  2. **Two id-producing helpers disagree.** `memory_entry_id_slug` (`mcp/tools/memory.rs`) keeps `_` and collapses runs, while `task_page_id_slug` deletes punctuation and does not collapse - so `"my_task"` becomes `my_task` as a memory id and `mytask` as a task id. One of the two, or the convention they both claim to implement, is wrong.

  3. **`label_slug` has zero production call sites.** `apps/wm-core/src/util/mod.rs` - only its own test calls it. A dead public function contradicts `@wiki/rules:no-dead-code-clone-scanning`; either give it a call site or delete it (do not suppress).

  **Not in scope:** `apps/wm-core/src/source_service.rs:73` is `Path::file_stem()` verbatim, not a slugify.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}