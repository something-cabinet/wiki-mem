---
title: 'Wiki Tool Reliability: wm_memory.add writes prose bodies — new memories are born old-style'
type: task
id: "wiki:tasks:wiki-tool-reliability-wmmemoryadd-writes-prose-bodies--new-memories-are-born-old-style"
status: todo
priority: high
tags: [tool-reliability, record-format, writer, migration]
acceptance_criteria:
  - text: "wm_memory.add emits a record body with the canonical memory question set (layer, store_or_skip, dedup_action, confidence)"
  - text: "wm_memory.add emits the record_format_hint warning for a prose body, matching wm_page behaviour"
  - text: "wm_validate.check flags a prose body under memory/ , or an explicit decision records why memory is exempt"
  - text: "wm_task.create and wm_page.create are fixed in the same change, since all three share the missing choke point"
  - text: "A test asserts a newly added memory parses as a record with the canonical question set"
  - text: "The six memories migrated during this session still validate and keep their content after the fix"
---

schema_version: 1
state: |-
  **Bug description:** `wm_memory.add` writes a page whose body is plain prose (frontmatter plus markdown), with **no record envelope** — no `schema_version`, `state`, `questions` or `answers`. Memory pages added through the documented tool are therefore old-style from birth, and `wm_validate.check` reports them as passing, so nothing surfaces the drift.

  **Tool name:** `wm_memory` (action `add`).

  **Measured:** **123 of 146** memory pages in `.wm/wiki/memory/` carry record bodies; the six created this session through `wm_memory.add` did not. After `wm page migrate-records --apply` on those six, the corpus reads **129 of 146** and all six validate with the record envelope present.

  **Full input parameters:** `wm_memory({"action":"add","title":"…","content":"…","layer":"project","category":"pattern","tags":[…]})`

  **Full output:** `{"id":"wiki:memory:…"}` — success, with no warning of any kind.

  **Counter-evidence that the page should have been a record:** `wm_validate.check` returns `pass, 0 errors` for the prose page both before and after migration, and `is_record_bearing` returns `true` for `PageType::Memory` (every page type is record-bearing). So the page type is record-bearing and the validation layer simply does not enforce it for this directory.

  **Why this matters — same class as an already-filed defect:** `@wiki/tasks/wiki-tool-reliability-wmtaskcreate-writes-prose-bodies--record-format-migration-missed-the-task-create-path` reports that `wm_task.create` writes prose bodies, so the task store also produces old-style pages. Together these two tools cover the two page types agents are told to write most often, which is the practical content of the standing complaint that AI still generates old-style wiki files. `wm_page.create` is the third in the class: it emits the `record_format_hint` warning but still does not build the envelope.

  **Fix direction:**
  1. Route `wm_memory.add` through the existing record body builder with the canonical memory question set (`layer`, `store_or_skip`, `dedup_action`, `confidence`), the way the migration planner already does.
  2. Emit the `record_format_hint` warning on the memory add path, matching `wm_page`.
  3. Extend `wm_validate.check` to flag a prose body under `memory/` (it currently does not), or record an explicit decision that memory is exempt — silence is the defect.
  4. Treat `wm_task.create`, `wm_memory.add` and `wm_page.create` as one fix, since they share the same missing choke point.

  **Workaround used to extract this session's memories:** create through `wm_memory.add`, then `wm page migrate-records --only memory/<slug>.md --apply`.
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
