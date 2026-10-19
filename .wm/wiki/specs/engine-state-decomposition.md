---
title: "Decompose EngineState into per-domain state handles"
type: spec
id: "wiki:specs:engine-state-decomposition"
status: approved
relates_to:
  - {type: depends_on, target: "wiki:specs:wm-core-layer-extraction"}
tags: [spec, approved, architecture, domain-driven, engine, wm-core]
---

schema_version: 1

schema_version: 1
state: |-
  ## Overview

  Break the `EngineState` god-object into per-domain state handles so the domain modules get explicit boundaries, then use those boundaries to decide which domains genuinely earn their own crate.

  `EngineState` (`apps/wm-core/src/engine/engine_state_mediator.rs`) is a flat bag of 21 public fields with 143 references workspace-wide; `page`, `graph`, `engine` and `search` all reach each other through it. That cyclic mesh is why module-level crate splits fail (D-11 in @wiki/specs/wm-core-layer-extraction, and the `e688f0` revert before it): no module has a closed interface while it can see every field. Decomposing the state is the precondition D-11 named for revisiting domain crates.

  This spec is the seam change D-11 deferred. It does not split a crate. The ordering rule it follows is recorded in @wiki/patterns/decompose-shared-state-before-splitting-crates (decompose shared state before splitting coupled modules into crates).

  ## Evidence

  Measured 2026-10-16 post-extraction; re-verified at HEAD `063c5d1`. The previously cited `8636c2a` is an ancestor of HEAD, five commits back, all wiki/version-record commits with no source drift.

  - **21 public fields** on `EngineState`: `graph`, `page_contents`, `section_corpus`, `bm25_index`, `vector_registry`, `source_registry`, `content_hashes`, `stale_flag`, `config`, `audit_sender`, `audit_drops`, `started_at`, `project_root`, `embedder`, `vector_store`, `skill_engine`, `write_channel`, `index_scheduler`, `session_memory`, `lsp`, `tool_list`.
  - **143 `EngineState` references** across 46 files: 19 files under `packages/wm-tools/src/tools`, 5 in wm-core page services, 4 in engine, 2 in search, 1 each in version, wm-code-intel and the tests.
  - The fields group into natural bounded contexts: graph snapshot; index (bm25 + sections + vectors + embedder + vector store); page content + content hashes; sources; memory; config + project root + staleness; audit; skill + write channel + index scheduler; and two outliers.
  - **The two outliers are not symmetric.** `tool_list` has three references, all in the tools layer: written by `packages/wm-tools/src/tools/mod.rs:55` and read by `tools/project.rs:123`. `lsp` has six call sites in `packages/wm-tools/src/tools/lsp.rs` plus `apps/wm-core/src/engine/engine_state_mediator.rs:158`, inside `notify_file_changed` - wm-core's own watcher owns that path.
  - **`section_corpus` is written and read with `bm25_index`**, not with page content: `engine/freshness_service.rs:21` stores both in one refresh, and `graph/mod.rs:308,354,404` plus `tools/index.rs:160` write them together.

  ## Locked Decisions

  - **D-1 (shape):** `EngineState` becomes a **struct of per-domain handles**. Call sites keep `engine.graph`-style access; the god-object stops being a flat bag but stays a facade. Chosen over explicit per-function dependencies (which rewrite 143 call sites at once) and over trait-based accessors (ceremony that would also delay any crate decision).
  - **D-2 (migration):** migrate **one domain at a time**, each an independently revertible change with the suites green before the next.
  - **D-3 (goal):** this is a **seam change, not a file move**. Whether any domain then earns a crate is decided after the handles exist, against D-11's (a)/(b)/(c) rubric - not assumed now.
  - **D-4 (unit):** a handle owns **state**, not consumers. Each field has exactly one owning handle; a module needing another domain's state calls a method on that handle (FR-2). State-centric handles were chosen over per-consuming-module handles, which would partition fields by design judgement rather than by the measured grouping, and over per-module facade traits, which is the ceremony D-1 rejected.
  - **D-5 (the nine handles):** `SourceState` is named `SourceRegistryState` because `wm_engine::SourceState` already owns that name in the `engine` barrel.

    | Handle | Owns |
    |---|---|
    | `GraphState` | `graph` |
    | `IndexState` | `section_corpus`, `bm25_index`, `vector_registry`, `embedder`, `vector_store` |
    | `PageState` | `page_contents`, `content_hashes` |
    | `SourceRegistryState` | `source_registry` |
    | `MemoryState` | `session_memory` |
    | `ConfigState` | `config`, `project_root`, `stale_flag` |
    | `AuditState` | `audit_sender`, `audit_drops`, `started_at` |
    | `LifecycleState` | `skill_engine`, `write_channel`, `index_scheduler` |
    | `LspState` | `lsp` |

  - **D-6 (two fields leave the decomposition):** `tool_list` **leaves wm-core**, becoming a field on the tool registry in `packages/wm-tools` - its only writer and reader are already there, and an MCP-protocol cache is application state, not domain state (@wiki/specs/wm-core-layer-extraction D-5). `lsp` **stays in wm-core** as the single-field `LspState` handle, because wm-core's file watcher owns the notification path. Folding it into `LifecycleState` was rejected as a grouping on the loose commonality of "long-lived subsystem".
  - **D-7 (order):** migrate **leaf-first**: `AuditState` -> `SourceState` -> `MemoryState` -> `LifecycleState` -> `IndexState` -> `GraphState` -> `PageState`. The 1-3 field handles with the fewest call sites prove the mechanics first, and the ArcSwap read-hot state (`graph`, `index`) is untouched until the pattern is validated - which is what NFR-1 protects. Hardest-first (index, graph, page) was rejected because a mistake there lands on the most load-bearing code.

  ## Requirements

  ### Functional Requirements

  - FR-1: Each of the 20 remaining fields is owned by exactly one handle type; no handle reaches into a sibling handle's fields.
  - FR-2: Cross-domain reads go through explicit methods on the owning handle.
  - FR-3: The observable behaviour of every MCP tool and CLI command is unchanged.
  - FR-4: `tool_list` is no longer a field on `EngineState`; the MCP tool cache lives in `packages/wm-tools`.
  - FR-5: The field-to-handle mapping in D-5 is exhaustive and non-overlapping: every one of the original 21 fields is either owned by exactly one handle or moved out per D-6.

  ### Non-Functional Requirements

  - NFR-1: The `ArcSwap` snapshot pattern is preserved for read-hot state (graph, index) - no new locks on the read path.
  - NFR-2: No new `#[allow(dead_code)]` and no new compiler or clippy warnings.

  ## Acceptance Criteria

  - [ ] AC-1: `EngineState` exposes no flat public fields for the migrated domains; access is via the handle types.
  - [ ] AC-2: Each migrated domain has a unit test proving the handle owns its state and does not reach into a sibling handle.
  - [ ] AC-3: `page`, `graph` and `search` no longer reference each other's fields directly; the sibling-module import count drops measurably.
  - [ ] AC-4: `cargo build`, `test` and `clippy --workspace` are green and the existing suites are unchanged.
  - [ ] AC-5: The spec records, per domain, whether it now clears D-11's (a)/(b)/(c) for a crate, with the measurement.
  - [ ] AC-6: The 21 original fields are each accounted for exactly once by D-5's table plus D-6's rulings - 20 owned by handles, 1 moved to `wm-tools` - asserted by a checked-in grep count, not by prose.
  - [ ] AC-7: The migrations land in D-7's order, each as its own revertible commit, and each leaves `cargo test --workspace` green.

  ## Scenarios

  ### Scenario 1: adding a domain without touching the others
  **Given** the handles exist
  **When** a new domain handle is added
  **Then** no existing handle's fields change and no consumer is edited.

  ### Scenario 2: a cross-domain read
  **Given** the `IndexState` handle owns `section_corpus` and `bm25_index`
  **When** `graph` needs the corpus to rebuild edges
  **Then** it calls an explicit method on `IndexState` and never loads `IndexState`'s fields directly.

  ### Scenario 3: the last migration
  **Given** seven domains are already handles
  **When** `PageState` is migrated
  **Then** the page services compile without naming any sibling handle's fields, and the suites green at change one are still green.

  ## Technical Notes

  - The wiki page body is a typed-decision record; `wm_page.update` writes the body verbatim and does **not** re-wrap it, so an edit must carry the whole record envelope rather than bare prose; the shape is given in @wiki/reference/typed-decision-record-schema
  - `IndexState` ends up owning both the corpus and the index over it, which is the point of the `section_corpus` ruling in D-5: corpus and index are written together in `freshness_service`, so splitting them would immediately need a shared accessor.

  ## Open Questions

  - [ ] Method names for the cross-domain reads are not fixed here; each is settled in the task that first needs it.
  - [ ] After the handles exist, which domains actually earn a crate? Decide with D-11; do not assume.
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
answers:
  kind: system
  scope: component
  status_class: approved
  needs_tasks: true