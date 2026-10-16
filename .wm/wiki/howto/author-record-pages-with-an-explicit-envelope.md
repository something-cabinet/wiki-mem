---
title: 'Howto: Author record pages with an explicit envelope'
type: howto
id: "wiki:howto:author-record-pages-with-an-explicit-envelope"
status: reviewed
tags: [howto, record-format, mcp, tooling]
relates_to:
  - {type: references, target: wiki:tasks:wiki-tool-reliability-wmtaskcreate-writes-prose-bodies--record-format-migration-missed-the-task-create-path}
---

schema_version: 1
state: |-
  ## Overview

  Every wiki page type is record-bearing (`WIKI-MEM.md` section *Page Bodies*), but `wm_page.create` writes whatever `content` it receives **verbatim**. Passing plain markdown therefore produces a **prose body** with no `schema_version`, `state`, `questions` or `answers` — and the tool returns success, with no warning observed in any case tried.

  ## Prerequisites

  None. This applies to every page type.

  ## Steps

  1. Build the envelope: `schema_version: 1`, then `state: |-` with the prose indented by two spaces, then the type's canonical `questions:` block, then `answers: {}`.
  2. Create the page with `content` set to that envelope.
  3. **Read the file back** and confirm `schema_version` is present. Do not trust the success response.

  ## Getting the canonical questions

  Do not guess and do not hand-write option lists. Read an existing page of the same type and copy its `questions:` block verbatim — a transcribed list can drift from `canonical_questions.rs`, a copied one cannot.

  ## Verification

  `wm_page.get` returning **no `record_format_hint` warning** means `parse_record` accepted the body. Treat that as the pass condition, since the create response itself carries no verdict.

  ## Caveats

  - `extra_frontmatter` is **silently discarded when `content` is supplied**, on both `create` and `update`. Acceptance criteria and priority therefore need a second, separate call with no `content`.
  - Never issue two writes to the same page concurrently — one can be silently lost while both calls report success, and one may report a false `IO_ERROR` on a write that landed.
  - `wm_page.create` writes prose for `spec` and `task` as well as for every other record-bearing type, despite `is_record_bearing()` returning true for all of them.

  ## Related

  - @wiki/tasks:wiki-tool-reliability-wmtaskcreate-writes-prose-bodies--record-format-migration-missed-the-task-create-path
questions:
  - id: task_kind
    type: choice
    instructions: What kind of task does this howto cover?
    options:
    - setup
    - development
    - testing
    - release
    - debugging
    - operations
    - integration
  - id: prerequisites_complete
    type: noul
    instructions: All prerequisites for this howto are fully documented.
  - id: difficulty
    type: score
    instructions: What skill level does this howto require?
    levels:
    - beginner
    - intermediate
    - advanced
    - expert
  - id: has_verification
    type: noul
    instructions: This howto includes a verification step.
answers: {}