---
title: 'Spec: wm_page.link merge + serialized page writes'
type: spec
id: wiki:specs:wmpagelink-merge-and-write-serialization
status: draft
tags: [bugfix, tool-reliability, graph, frontmatter, concurrency]
relates_to:
  - {type: references, target: wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber}
  - {type: implements, target: wiki:rules:durable-write-or-error}
functional_requirements:
  - {id: FR-1, description: "FsPageRepo::write uses a temp name unique per call (pid + AtomicU64 counter) so concurrent writes cannot collide on rename"}
  - {id: FR-2, description: "The page read-modify-write path is serialized per page so concurrent link/unlink cannot lose updates"}
  - {id: FR-3, description: "wm_page.link appends to relates_to, preserving existing entries; it dedupes identical (type, target) pairs"}
  - {id: FR-4, description: "wm_page.link re-reads the page and returns an error if the edge is absent after the write"}
  - {id: FR-5, description: "wm_page.update keeps replace semantics for relates_to; only link merges"}
  - {id: FR-6, description: "Regression tests cover sequential double-link preserving both edges and N parallel links producing N edges with no IO_ERROR"}
non_functional_requirements:
  - {id: NFR-1, description: "No new comments; zero compiler warnings"}
  - {id: NFR-2, description: "No workspace-wide cargo fmt; scope to -p wm-core"}
  - {id: NFR-3, description: "Round-trip safety: line-based frontmatter edits, never whole-block YAML round-trip"}
general_goals:
  - description: "Stop silent edge loss and IO_ERROR failures in wm_page.link"}
  - description: "Make concurrent page writes safe within a single process"}
---

schema_version: 1
state: |-
  # Spec: wm_page.link merge + serialized page writes

  ## Problem

  `wm_page.link` has two reproduced defects (task wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber):
  4 of 5 concurrent links on one source page failed with `IO_ERROR "No such file or directory (os error 2)"` while the same calls ran sequentially; and `link` replaces the page's `relates_to` block instead of appending, silently dropping existing edges — including edges authored directly in frontmatter.

  ## Root causes

  - **Clobber** — `WmPageAction::Link` (apps/wm-core/src/mcp/tools/page/mod.rs:396-402) passes a single-element `relates_to`. `apply_relates_to` (apps/wm-core/src/page/services/page_update_builder_service.rs:169-186) calls `remove_yaml_block(new_fm, "relates_to")` then appends only that list, destroying prior edges.
  - **Race** — `FsPageRepo::write` (apps/wm-core/src/page_repo/mod.rs:20-34) uses temp name `.{file}.tmp-{pid}`, not unique per call. Same-process concurrent writes share the temp path; the first `std::fs::rename` consumes it, the second fails `ENOENT`.

  ## Approach

  FR-1: give each write a unique temp path (`pid` + a monotonic `AtomicU64` counter) so `write`+`rename` is independent per call.
  FR-2: serialize the read-modify-write in the page update path with a per-path lock so a concurrent pair cannot both read the same base and clobber each other.
  FR-3: in `handle_link`, read the page's current `relates_to`, append the new `(type, target)` if not already present, and pass the merged list to the update.
  FR-4: after the write, re-read the page and return `ToolError::internal` if the edge is not present (rule `@wiki/rules/durable-write-or-error`).
  FR-5: `wm_page.update`'s `relates_to` replacement semantics are unchanged; merge is a `link` behaviour.

  ## Files

  - apps/wm-core/src/page_repo/mod.rs
  - apps/wm-core/src/mcp/tools/page/mod.rs
  - apps/wm-core/src/page/services/page_update_builder_service.rs
  - apps/wm-core/tests/mcp_test.rs

  ## Verification

  `cargo test -p wm-core` (scoped). New tests: sequential double-link keeps both edges; N parallel links yield N distinct edges on disk with no IO_ERROR. Manual: rebuild and repeat the 5-way parallel repro.

  ## Risks

  FR-2 touches the shared page-write path. Keep the lock scope to the update read-modify-write; the existing suite covers regressions. Single-process assumption holds for the local MCP server.
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
