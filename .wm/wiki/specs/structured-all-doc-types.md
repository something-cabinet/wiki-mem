---
title: Make All Page Types Structured (Record Bodies)
type: spec
id: "wiki:specs:structured-all-doc-types"
status: draft
tags: [structured, typed-decisions, migration, all-types, wiki]
---

schema_version: 1
state: |-
  ## Overview

  Extend the typed-decision record body from the 5 descriptive types to **all** page types (`rule`, `core`, `memory`, `task`, `spec`, `note`), and **rewire prose consumers to read the record's `state`**. Ships uniform, machine-checkable docs across the whole wiki. Readability loss is accepted; the `state` field carries the original prose. This is **independent of any model runtime** (the decision model is non-viable; the value here is uniform structure/validation).

  ## Locked Decisions

  - **D1**: All page types use the `{schema_version, state, questions, answers}` envelope, with **per-type question sets** for `rule`/`core`/`memory`/`task`/`spec`/`note`.
  - **D2**: Prose consumers read **`state`** via the shared record-state extractor; `wm_page.get` returns `state` as the readable **content** for record pages (plus the parsed `record`).
  - **D3**: `wm page migrate-records` extends to all types; **one-shot**, **idempotent**, frontmatter byte-preserved.
  - **D4**: `index.md` is generated — **not** converted (regenerated).
  - **D5**: No model/runtime dependency.
  - **D6**: Readability loss accepted.

  ## Requirements

  ### Functional
  - **FR-1**: Extend `is_record_bearing` + `canonical_questions` to `rule`/`core`/`memory`/`task`/`spec`/`note`.
  - **FR-2**: `wm_page.get` returns `state` as `content` (rendered) for record pages, so agents reading a page get the prose.
  - **FR-3**: Rewire every prose consumer to read `state`: **memory read** (`mcp/tools/memory.rs`), **task description** (`mcp/tools/task/mod.rs`), the **memory branch of `wm_search.query`**, and any other body reader.
  - **FR-4**: Graph refs / inline tags / sections stay record-aware (extend the existing FR-11 extractor to all types).
  - **FR-5**: Validation is **strict for all types** (prose body = error).
  - **FR-6**: Migration converts all remaining pages (except `index.md`), reports per-type counts, skips ambiguous pages, is idempotent.
  - **FR-7**: **Agent rule-loading still yields the rule text** (wm-init's rule read returns `state`).

  ### Non-Functional
  - **NFR-1**: No data loss — frontmatter byte-for-byte; `state` carries the normalized prose.
  - **NFR-2**: No regression in page read, graph edges, or search.
  - **NFR-3**: Zero compiler/clippy warnings; rules honored (no comments/else/magic).
  - **NFR-4**: No new comments introduced (comment ban in force).

  ## Acceptance Criteria
  - [ ] AC-1: Every page type parses as a record; canonical question sets exist for all types.
  - [ ] AC-2: `wm_page.get` returns `state` as content; agents receive rule/memory/task prose.
  - [ ] AC-3: Migration converts all remaining pages; 0 parse failures; frontmatter preserved; re-run is a no-op.
  - [ ] AC-4: No regression in search/graph/validate; **rule loading and memory recall still work**.
  - [ ] AC-5: `cargo check`/`clippy` clean.

  ## Scenarios
  ### Scenario 1: Convert a rule
  **Given** a `rule` page
  **When** migration runs
  **Then** its body is a record whose `state` holds the rule text, and the agent still receives that text when loading rules.

  ### Scenario 2: Memory recall intact
  **Given** a converted `memory` page
  **Then** `wm_memory.get/list` returns the `state` text as content, and `wm_search.query`'s memory branch injects `state`.

  ### Scenario 3: Task description intact
  **Given** a converted `task`
  **Then** the task description is derived from `state`.

  ## Technical Notes
  - `record_state_text` already exists (FR-11) and is used by graph/sections/parser — extend its application to all types.
  - Watch the migration pre-pass: duplicated frontmatter / escaped newlines.
  - `index.md` must be regenerated, never converted.
  - Risk: rule loading is the agent's instruction channel — validate explicitly.

  ## Open Questions
  - [ ] `note` pages: convert or leave prose?
  - [ ] Keep a rendered human view (CLI) of `state`?
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}
