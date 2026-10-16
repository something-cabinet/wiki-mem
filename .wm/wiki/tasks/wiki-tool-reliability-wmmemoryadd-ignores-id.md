---
title: 'Wiki Tool Reliability: wm_memory.add ignores the id parameter'
type: task
id: "wiki:tasks:wiki-tool-reliability-wmmemoryadd-ignores-id"
status: todo
tags: [bug, tool-reliability, mcp, memory]
acceptance_criteria:
  - text: wm_memory.add honours the requested id, or returns an explicit error when id and title slug disagree
  - text: If id is advisory by design it is removed from the schema and the title-slug behaviour is documented
  - text: 'Regression test: add with an explicit id, then read back and assert the stored page id'
priority: medium
relates_to:
  - {type: relates_to, target: wiki:tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content}
---

schema_version: 1
state: |-
  ## Bug description

  wm_memory.add **silently ignores the `id` parameter** and derives the page id from a slug of `title` instead. The call returns success and the caller cannot learn which id was actually used.

  ## Tool name

  `wm_memory` (action `add`)

  ## Reproduction - 2 for 2

  **Call 1:** id `wiki:memory:preference-artifacts-are-the-owners-call`, title "Preference artifacts are the owner's call - emit options, not decisions".
  Observed id: `wiki:memory:preference-artifacts-are-the-owner-s-call-emit-options-not-decisions` (title slug).

  **Call 2:** id `wiki:memory:no-global-rule-layer-per-project`, title "No global rule layer - rules stay per-project".
  Observed id: `wiki:memory:no-global-rule-layer-rules-stay-per-project` (title slug).

  Both confirmed by reading graph edges back: the decision page links to the *derived* id, not the requested one.

  ## Why it matters

  A caller who requests a specific id - to reference the entry later, to make the call idempotent, or to match an existing convention - gets a different page and a success response. This is the same class as the `extra_frontmatter` discard and the issue-#126 `r#type` field: a declared input the handler never consumes, shipping silently. The repo rule is explicit - if the schema accepts it, the handler must consume it: wire it or delete it.

  ## Counter-evidence / narrowing

  Not established whether title-slugging is deliberate and `id` merely advisory. `wm_page.create` does honour an explicit path, so the two write paths differ. A fix can either honour `id` (erroring on mismatch) or drop `id` from the schema.

  ## Workaround

  Read the returned `id` from the response and use *that* in any later link or reference; never assume the requested id survived. Both memory entries extracted in this session were reconciled this way.

  ## Related

  - @wiki/tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content - same defect class
  - @wiki/core/critical-patterns - the extra_frontmatter entry
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