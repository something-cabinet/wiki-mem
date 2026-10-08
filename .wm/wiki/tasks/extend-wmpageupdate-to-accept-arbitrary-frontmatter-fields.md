---
title: Extend wm_page.update to accept arbitrary frontmatter fields
type: task
tags:
- mcp
- tool
- refactor
status: done
priority: medium
implementation_notes: 'Confirmed 2026-07-31: wm_page.update(id, title=..., type=..., status=..., tags=..., relates_to=...) persisted only status/tags/relates_to into frontmatter — title and type params are DROPPED (pattern page wiki:patterns:hash-skip-rebuild lost title/type after an update passing them as params; page reverted to concept parse). Frontmatter survives only when embedded verbatim in the content argument (worked for core:critical-patterns). Title/type must be written into the frontmatter block written by update.'
acceptance_criteria:
- text: wm_page.update accepts arbitrary frontmatter keys (an id parameter or generic frontmatter map) and persists them to the frontmatter block on disk
- text: title and type passed as parameters are written into frontmatter instead of being dropped, fixing the hash-skip-rebuild regression
- text: Frontmatter updates trigger appropriate cache invalidation (stale flag or incremental update)
---

schema_version: 1
state: |-
  id: wiki:tasks:extend-wmpageupdate-to-accept-arbitrary-frontmatter-fields

  `wm_page.update` currently supports only a fixed set of frontmatter fields: title, status, tags, type, relates_to, content, notes. It cannot update arbitrary YAML frontmatter keys, which means:

  1. The `knowns_id → id` rename (and any future frontmatter migration) requires bulk file-level sed instead of using the MCP tool
  2. No way to programmatically fix frontmatter issues without editing files directly

  **Scope:**
  - Add an `id` parameter to `wm_page.update` (or a generic `frontmatter: HashMap<String, Value>` field)
  - Write the updated frontmatter keys to disk
  - Trigger appropriate cache invalidation (stale flag or incremental update)
  - Consider exposing a generic `frontmatter` map for future flexibility

  Note: This is tool plumbing only — the incremental cache pipeline (graph/BM25/embeddings vs full rebuild) is tracked separately in @wiki/specs/graph-connectivity-fix.
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
