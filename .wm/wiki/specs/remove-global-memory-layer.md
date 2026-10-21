---
title: Remove the unimplemented global memory layer
type: spec
id: "wiki:specs:remove-global-memory-layer"
status: approved
tags: [spec, memory, mcp, cleanup, dead-code]
acceptance_criteria:
  - checked: false
    text: 'AC-1: `wm_memory list` and `add` with `layer="global"` return a structured validation error naming the allowed values; no silent fallback to project.'
  - checked: false
    text: 'AC-2: `Add` response `layer` equals the layer actually persisted.'
  - checked: false
    text: 'AC-3: `rg -n ''Global|Promote|global_memory|resolve_global'' apps/wm-core packages/wm-engine -g ''*.rs''` returns zero hits.'
  - checked: false
    text: 'AC-4: `MEMORY_LAYER_OPTIONS` offers only `project` and `session`; no registered tool schema still advertises `global`.'
  - checked: false
    text: 'AC-5: No shim or skill copy instructs loading global memory — verified across `embed_files/` **and** the repo-root twins; the two `wm-init` copies are byte-identical in the memory section.'
  - checked: false
    text: 'AC-6: `cargo build` and `cargo clippy` pass with zero warnings; no new `#[allow(dead_code)]` (rule `no-warnings`, `no-dead-code-clone-scanning`).'
  - checked: false
    text: 'AC-7: `wm-cli validate` is clean.'
  - checked: false
    text: 'AC-8: Regression test asserts an unsupported layer yields an error payload and never a success-on-no-op (rule `durable-write-or-error`).'
  - checked: false
    text: 'AC-9: A session-init dry run against the updated shims loads project memory only and records "Global Memories" nowhere.'
---

schema_version: 1
state: |-
  ## Overview

  `wm` documents **three** memory layers — project / global / session — and ships that contract into every project through embedded shims and skills. The `global` layer was **never implemented**. It has no reader, its store directory has never been created on any known machine, and its only writer (`wm_memory.promote`) is a write-only dead end. Meanwhile `wm_memory` `list`/`add` with `layer="global"` **succeed silently while doing project work**.

  This spec removes the global layer entirely and makes the layer contract honest: **project and session only**.

  ## Context & Evidence

  All eight findings below were verified against the working tree and against live MCP tool calls in session 2026-10-15.

  - **E1 — The enum and data model exist and are otherwise sound.** `packages/wm-engine/src/models/memory/layer_model.rs:4-12` declares `enum MemoryLayer { #[default] Project, Global, Session }` with `as_str()`. `packages/wm-engine/src/models/page_data/memory_data_model.rs:6-14` declares `MemoryData { layer, ttl_days, last_verified, merged_into, rejected_reason, metadata }`, wired onto page meta at `packages/wm-engine/src/models/page/meta_model.rs:39`.
  - **E2 — `layer` is never persisted.** Zero of the 134 files in `.wm/wiki/memory/` contain a `layer:` key. It appears only inside the record `questions:` block with `answers: {}` empty. `MemoryData.layer` and the `layer` question are declared-but-never-wired.
  - **E3 — The read path has no global branch.** `apps/wm-core/src/mcp/tools/memory.rs:214-280` branches only on `is_session(&layer)` (line 220); every other layer falls through to `page::list_pages(&e, Some(&PageType::Memory))` (line 233) — the project's `.wm/wiki/memory/`. Observed: `list(layer="project")` and `list(layer="global")` returned byte-identical 50 entries.
  - **E4 — The write path returns a false success.** `memory.rs:297-372`: `Add` defaults `layer` to `"project"` (line 306); if not session it writes `format!("memory/{slug}")` into the project (line 345) **regardless of the requested layer**, then returns `"layer": layer` (line 370) — echoing back `"global"` for a project write. Violates `wiki:rules:durable-write-or-error`.
  - **E5 — The global directory is created lazily and has never existed.** `memory.rs:193-205` resolves `$HOME/.wm/wiki/memory/` (`WM_DIR=".wm"`, `WIKI_DIR="wiki"`) with a `HOME` → `USERPROFILE` → `"."` fallback that degrades to `./.wm/wiki/memory` — i.e. accidentally the project. `rg` confirms **exactly one call site**: line 406, inside `Promote`. On this machine `~/.wm` holds only `bin/`, `logs/`, `models/`.
  - **E6 — `promote` is a write-only dead end.** `memory.rs:402-419` is the only writer of the global directory. Nothing reads it. Promoting a page makes it invisible to every read surface, including `List`.
  - **E7 — No project namespacing in the global store.** `memory.rs:429-436`: `global_memory_path()` keeps only `project_path.file_name()`, so `my-title.md` promoted from two repositories collides at `~/.wm/wiki/memory/my-title.md` and silently overwrites. Two tests assert the flattening (`global_memory_path_appends_only_file_name`, `global_memory_path_never_double_appends_memory_segment`); the collision itself is untested.
  - **E8 — The engine is single-project-root, so a global store cannot be indexed.** `apps/wm-core/src/engine/engine_state_mediator.rs:35` holds `pub project_root: RwLock<PathBuf>`, constructed from one root (line 50); the embedder/vector store and write-channel consumer take the same root. Graph, BM25, section index and `wm_initial` counts all derive from `project_root/.wm/wiki/`. Any entry under `~/.wm/wiki/memory/` is outside that root and unindexable without a second root or a copy-on-read merge.

  ## Decision

  **D1 — Delete the global layer.** Remove the `Global` variant, the `promote` action, the global-path helpers, and every document, shim and skill instruction that promises the layer. Retain **project** and **session** memory.

  **D2 — Unknown or unsupported layers fail loudly.** `list`/`add` with `layer="global"` (or any unrecognised value) return a structured validation error naming the allowed values, rather than silently falling back to project.

  **D3 — `Add` reports the layer actually written.** The response `layer` field reflects the persisted layer, never the requested one.

  **D4 — No substitution for the removed session-start step.** The "load global memory" steps in the shims and the `wm-init` skill are deleted outright. Adding a tolerant branch would itself be a compensating layer.

  **D5 — Reintroduction goes through an RFC.** A one-line note records that cross-project memory is unimplemented and that any future effort starts from real requirements, not from this skeleton.

  ### Rejected alternatives

  - **Fix only the `List` filter (the original task premise).** Insufficient: E2 means there is nothing to filter on, E5/E6 mean the store is empty and unreadable, and E8 means a correct filter would return entries that `wm_search`, the graph and `wm_initial` cannot see. It would also leave E4's false success in place, breaching `durable-write-or-error`.
  - **Build the full layer.** Requires either a second engine root or copy-on-read merging, both of which breach `wiki:core:ARCHITECTURE`'s "single engine per process" stance and add drift semantics (`merged_into`, `rejected_reason`) to a store that has never held a byte. Demand signal is zero: no entry, no persisted `layer`, one lying write path — after the layer was documented, shipped in shims, and marked done in task `a65shf`.
  - **Keep the variant parked for later.** `no-dead-code-clone-scanning` forbids `#[allow(dead_code)]`, so there is no legal way to park it.

  ## Contract Delta

  - `wm_memory` `list`: `layer` accepts `project` | `session` (default `project`). `global` → validation error.
  - `wm_memory` `add`: same acceptance; response `layer` echoes the persisted layer.
  - `wm_memory` `promote`: **action removed** from the tool surface.
  - `schemars` descriptions at `memory.rs:23` and `:41` change from `project/global/session` to `project/session`.
  - MCP only: no CLI `memory` subcommand parity work is in scope here.

  ## Change Inventory

  **Code**
  - `packages/wm-engine/src/models/memory/layer_model.rs:4-12` — drop the `Global` variant.
  - `packages/wm-engine/src/decision/canonical_questions.rs:86` — `MEMORY_LAYER_OPTIONS` becomes `["project", "session"]`.
  - `apps/wm-core/src/mcp/tools/memory.rs` — remove the `Promote` action and handler (402-419); delete `resolve_global_memory_dir()` (193-205) and `global_memory_path()` (429-436) with its two tests (443-460); make `List` (214-233) and `Add` (297-372) reject unsupported layers; stop echoing the requested layer at line 370; update descriptions at lines 23 and 41.
  - `apps/wm-core/tests/mcp_test.rs:544` — `MEMORY_RECORD_BODY` question options drop `global`.

  **Embedded assets (RustEmbed'd — must change in lockstep with their repo-root twins)**
  - `apps/wm-core/src/embed_files/shims/WIKI-MEM.md:19` **and** repo-root `WIKI-MEM.md:19`.
  - `apps/wm-core/src/embed_files/shims/{AGENTS,CLAUDE,GEMINI,OPENCODE,REASONIX,copilot-instructions}.md` **and** the six repo-root twins.
  - `apps/wm-core/src/embed_files/skills/wm-init/SKILL.md:111-154` **and** `.opencode/skills/wm-init/SKILL.md:111-154` — delete "Step 7: Load Global Memory", the `Global Memories` summary line, the checklist item, and the "Skipping global memory load" red flag.

  **Wiki**
  - `wiki:core:CONVENTIONS` line 116 — replace the `layer="global"` guidance with the project/session statement plus the RFC note.
  - `wiki:concepts:memory-system` lines 19 and 73 — remove the global rows (note: this page states the path **correctly** as `$HOME/.wm/wiki/memory/`, unlike the original task premise).
  - `wiki:tasks:wmmemory-list-ignores-layerglobal--returns-project-memory` — re-scoped to this removal (see the linked task).
  - `wiki:tasks:a65shf` — annotate that the global layer it introduced was removed in follow-up; history records the reversal.

  ## Migration

  **None required.** Nothing persisted `layer` (E2), and the global directory does not exist (E5). The observable behaviour change is that calls using `layer="global"` now error instead of silently returning project memory — which is the intended correction. No byte of user data is lost.

  ## Acceptance Criteria

  - [ ] AC-1: `wm_memory list` and `add` with `layer="global"` return a structured validation error naming the allowed values; no silent fallback to project.
  - [ ] AC-2: `Add` response `layer` equals the layer actually persisted.
  - [ ] AC-3: `rg -n 'Global|Promote|global_memory|resolve_global' apps/wm-core packages/wm-engine -g '*.rs'` returns zero hits.
  - [ ] AC-4: `MEMORY_LAYER_OPTIONS` offers only `project` and `session`; no registered tool schema still advertises `global`.
  - [ ] AC-5: No shim or skill copy instructs loading global memory — verified across `embed_files/` **and** the repo-root twins; the two `wm-init` copies are byte-identical in the memory section.
  - [ ] AC-6: `cargo build` and `cargo clippy` pass with zero warnings; no new `#[allow(dead_code)]` (rule `no-warnings`, `no-dead-code-clone-scanning`).
  - [ ] AC-7: `wm-cli validate` is clean.
  - [ ] AC-8: Regression test asserts an unsupported layer yields an error payload and never a success-on-no-op (rule `durable-write-or-error`).
  - [ ] AC-9: A session-init dry run against the updated shims loads project memory only and records "Global Memories" nowhere.

  ## Verification

  1. `rg` sweeps from AC-3 and AC-5, plus `rg -in 'global' WIKI-MEM.md .opencode/skills/wm-init/ apps/wm-core/src/embed_files/` — hits allowed only in the new RFC note.
  2. Live MCP calls: `wm_memory list`/`add` with `layer="global"` and with `layer="bogus"` — both must error.
  3. `wm_memory promote` must be absent from the registered tool surface (schema check via `tools/list`).
  4. `cargo build --workspace` + `cargo clippy --workspace` (zero warnings), `cargo test -p wm-core`.
  5. Read-back of the two edited wiki pages to confirm the record body and frontmatter survived (rule `durable-write-or-error`).

  ## Rollback / Reintroduction

  Deletion is reversible in concept but intentionally not in code: the `layer` concept and the `promote` verb remain documented as a future RFC. Any reintroduction designs cross-project memory from requirements — cross-project collision semantics (E7), an indexing strategy compatible with the single-root engine (E8), and a write path that persists `layer` (E2) — rather than reviving this skeleton, which carries no logic worth preserving.

  ## Amendment (2026-10-15) — two deltas and a sibling spec

  This spec's decision (remove the global **memory** layer) stands unchanged. Two deltas follow from @wiki/specs/global-rules-cascade, which introduces a global **rules** layer — a different entity that reuses the same `~/.wm/wiki/` directory convention.

  1. **`resolve_global_memory_dir` is generalised, not deleted.** It becomes `resolve_global_wiki_dir(subdir)`, shared with the rules cascade and moved to the engine crate. Its `HOME` → `USERPROFILE` → `"."` fallback is replaced by an **error**, because a fallback pointing at the project would create global-looking writes inside a project root — the worst outcome for a cascade. See AC-10.
  2. **The wm-init session-start block is repurposed, not simply deleted.** This spec removes the global *memory* load instruction ("Step 7: Load Global Memory"); the rules-overlay step once planned for the same block was withdrawn when wiki:specs:global-wiki-layer was superseded on 2026-10-15. Only the deletion applies now; the earlier warning that — one spec deleting a step while the other adds its sibling.

  Everything else in the change inventory stands: removal of the `Global` memory variant, removal of the memory `promote` action, the `List`/`Add` validation errors, the shim and skill edits, the obsoletion of `wiki:tasks:wmmemory-list-ignores-layerglobal--returns-project-memory`, and the `a65shf` outcome note.

  **AC-10 (added):** `resolve_global_wiki_dir` errors, and does not fall back to the project, when both `HOME` and `USERPROFILE` are unset.
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