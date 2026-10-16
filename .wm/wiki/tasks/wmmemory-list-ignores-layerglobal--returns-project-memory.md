---
title: Remove the unimplemented global memory layer (project + session only)
type: task
id: "wiki:tasks:wmmemory-list-ignores-layerglobal--returns-project-memory"
status: todo
priority: high
acceptance_criteria:
  - text: wm_memory list and add with layer="global" return a structured validation error naming the allowed values (project, session) — no silent fallback to project
  - text: Add response layer equals the layer actually persisted (no echo of an unrouted value)
  - text: rg -n 'Global|Promote|global_memory|resolve_global' apps/wm-core packages/wm-engine -g '*.rs' returns zero hits
  - text: MEMORY_LAYER_OPTIONS in canonical_questions.rs offers only project and session; no registered tool schema advertises global
  - text: No shim or skill copy instructs loading global memory — verified across embed_files/ and the repo-root twins, including both wm-init copies
  - text: 'cargo build and cargo clippy pass with zero warnings; no new #[allow(dead_code)]'
  - text: wm-cli validate is clean
  - text: Regression test asserts an unsupported layer yields an error payload and never a success-on-no-op
tags: [bug, tool-reliability, memory, mcp, dead-code]
relates_to:
  - {type: depends_on, target: wiki:specs:remove-global-memory-layer}
---

schema_version: 1
state: |-
  ## Why this task was re-scoped (2026-10-15)

  The original task assumed the global memory layer worked and merely had a broken filter: its ACs asked that `list(layer=global)` read "the global store (`~/.wm/memory` or the `global_memory_path` target)". Investigation this session showed the premise was wrong on both counts — the path is `$HOME/.wm/wiki/memory/`, not `~/.wm/memory`, and the layer was **never implemented at all**.

  The work is therefore no longer "make the filter work". Per spec @wiki/specs/remove-global-memory-layer (oracle-reviewed, verdict: DELETE), the global layer is removed and the layer contract becomes **project + session only**.

  ## Evidence that redefined the scope

  - The `List` handler branches only on `is_session` (`apps/wm-core/src/mcp/tools/memory.rs:220`); every other layer reads the project's memory pages (line 233). `list(layer="project")` and `list(layer="global")` returned byte-identical 50 entries.
  - `Add` writes `memory/{slug}` into the project regardless of layer (line 345) and then echoes the requested layer back (line 370) — a false success, breaching `durable-write-or-error`.
  - No memory page persists a `layer` value: 0 of 134 files carry a `layer:` key, so a correct filter would still have nothing to filter on.
  - `resolve_global_memory_dir()` (line 193) has exactly one call site — `Promote` (line 406). Nothing reads the global store; `promote` is a write-only dead end.
  - The store directory has never existed: `~/.wm` holds only `bin/`, `logs/`, `models/`.
  - `global_memory_path()` (line 429) keeps only the file name, so the same slug promoted from two repositories silently overwrites.
  - The engine is rooted in one `project_root` (`engine_state_mediator.rs:35`), so a store outside the root cannot be indexed, graphed, or searched.

  ## Consequences of the decision beyond the filter bug

  The 3-layer contract is shipped to every project through RustEmbed'd shims and the `wm-init` skill, both of which instruct loading global memory. Removal therefore touches code, embedded assets, their repo-root twins, two wiki pages, and the skill in both copies. See the spec's Change Inventory.

  ## Note on the original AC text

  The original AC-1 named `~/.wm/memory`; the code produces `~/.wm/wiki/memory`. `wiki:concepts:memory-system` states the path correctly. The ACs below replace the originals.
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