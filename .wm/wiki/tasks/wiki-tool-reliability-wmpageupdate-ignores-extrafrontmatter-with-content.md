---
title: 'Wiki Tool Reliability: wm_page.update ignores extra_frontmatter when content is supplied'
type: task
id: "wiki:tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content"
status: todo
tags: [bug, tool-reliability, mcp, update, frontmatter]
acceptance_criteria:
  - text: wm_page.update adds a new extra_frontmatter key when content is supplied in the same call, or returns an explicit error
  - text: wm_page.create honours extra_frontmatter when content is supplied, so ACs and priority can be set at creation time
  - text: 'Regression test: content + extra_frontmatter together on both create and update, asserting the new key is present on read-back'
  - text: The hypothesis (supplying content rebuilds frontmatter, so extra_frontmatter can only overwrite existing keys) is confirmed or refuted by a probe and documented
priority: high
---

schema_version: 1
state: |-
  ## Bug description

  `wm_page.update` **silently ignores `extra_frontmatter` when `content` is supplied in the same call**, for a key that does not already exist in the page's frontmatter. No error, no warning — the caller believes frontmatter was set.

  ## Tool name

  `wm_page` (action `update`)

  ## Reproduction — two calls, same session, same page

  **Call 1 (fails silently).** `content` **and** `extra_frontmatter` together:

  ```json
  {
    "action": "update",
    "id": "wiki:tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success",
    "content": "<full record envelope, ~7 KB>",
    "extra_frontmatter": { "acceptance_criteria": [ {"text": "…"}, {"text": "…"} ] }
  }
  ```

  Result: `{"status":"updated"}`. Read-back shows **`acceptance_criteria` absent from the frontmatter** — the key was never added. The `content` write itself landed correctly.

  **Call 2 (succeeds).** `extra_frontmatter` alone, no `content`:

  ```json
  {
    "action": "update",
    "id": "wiki:tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success",
    "status": "todo",
    "extra_frontmatter": { "priority": "high",
                           "acceptance_criteria": [ {"text": "…"} ] }
  }
  ```

  Result: both `acceptance_criteria` (4 items) **and** `priority: high` present in the frontmatter on read-back. No `content` was supplied.

  ## Counter-evidence / narrowing

  The same `content` + `extra_frontmatter` pattern **did** work earlier in the same session on `wiki:tasks:wmmemory-list-ignores-layerglobal--returns-project-memory` — but that page **already had** an `acceptance_criteria` key in its frontmatter, so the merge updated an existing key rather than adding a new one.

  **Hypothesis (needs one confirming probe):** when `content` is supplied, the update path rebuilds the frontmatter and `extra_frontmatter` can only overwrite keys that already exist — it cannot introduce new ones. The counter-example is consistent with this; a clean probe on a scratch page would settle it.

  ## Impact

  ``` 
  Silent parameter discard is the same class as the `wm_doc.create` `r#type` defect recorded in `@wiki/core/critical-patterns` (issue #126): a declared input that the handler never consumes, shipping silently. There, the field was dead in the handler; here, the parameter is ignored only under a specific co-occurrence, which is harder to spot in review.

  Concretely: an agent that adds acceptance criteria while writing a task body gets a success response and a task with **no ACs** — violating the "mandatory ACs for task and spec" expectation and leaving `has_ac` questions unanswerable.

  ## Workaround

  Set `extra_frontmatter` in a **separate** `wm_page.update` call from the one that writes `content`, and read the frontmatter back afterwards to confirm (rule `durable-write-or-error`).

  ## 2026-10-15 addendum — the defect also affects `wm_page.create`

  Batch task creation (13 tasks in one pass, session 2026-10-15) confirmed the same silent discard on the **create** path. `wm_page.create` called with both `content` and `extra_frontmatter` produced a page carrying the record body but **no `acceptance_criteria` and no `priority`** — verified by read-back on `wiki:tasks:cwr-01-state-root-resolution-and-config-read`, whose frontmatter contained only `title/type/id/status/tags`.

  Applying `extra_frontmatter` in a subsequent `update` call (no `content`) then added both keys successfully — the same workaround the update path needs.

  **Consequence:** acceptance criteria cannot be set at task-creation time through any WM tool path. Every task must be created and then re-written to carry its ACs, which is precisely the two-call pattern that invites the concurrent-write lost-update defect recorded on `wiki:tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success`. The two defects compound: the workaround for this one increases exposure to that one.
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