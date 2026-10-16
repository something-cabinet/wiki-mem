---
title: 'Failure: MCP wm_task.create missing acceptance_criteria field'
type: concept
id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria
relates_to:
  - {type: references, target: wiki:tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content}
---

schema_version: 1
state: |-
  id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria

  ---
  id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria
  title: Failure: MCP wm_task.create missing acceptance_criteria field
  type: concept
  tags: [failure, mcp, task, validation, rust]
  ---
  id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria

  ## What went wrong

  wm_task.create and wm_task.update did not accept an acceptance_criteria parameter. This caused 153 validation errors ("tasks without acceptance criteria") across the entire wiki. Skills that created tasks (wm-plan, wm-flow, wm-review, wm-debug) could not set ACs because the MCP tool simply did not expose the field.

  ## Root cause

  The WmTaskAction enum variants Create and Update in action.rs were missing the acceptance_criteria field. The tool handler had no code path to write ACs into the YAML frontmatter.

  ## Prevention

  - When adding a new field to a page model, add it to ALL tool handlers that touch that page type (Create AND Update AND the frontmatter builder)
  - Verify with wm_validate.check after adding new task/create tool support
  - Cross-check skill docs against tool API — if a skill describes creating tasks but the tool cannot set ACs, the skill docs are misleading

  ## Time lost

  ~5 minutes to fix the tool handler. ~30+ minutes of validation noise that should not have existed.

  ## 2026-10-15 — root cause found: `extra_frontmatter` is discarded when `content` is supplied

  The cause is not specific to the task path. `extra_frontmatter` is **silently ignored whenever `content` is supplied in the same call**, on both `wm_page.create` and `wm_page.update`.

  Reproduced in one pass: creating a task with an envelope `content` **plus** `extra_frontmatter` carrying `acceptance_criteria` and `priority` produced a page whose frontmatter held only `title/type/id/status/tags`. Applying `extra_frontmatter` alone in a second call (no `content`) added both keys successfully.

  Consequence: **acceptance criteria cannot be set at task-creation time through any WM tool path.** Every task requires a create-then-rewrite pair — which is precisely the two-call shape that increases exposure to the concurrent-write lost-update defect recorded in @wiki/patterns/atomic-file-writes-unique-temp-and-path-lock The two defects compound.

  Source: @wiki/tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}