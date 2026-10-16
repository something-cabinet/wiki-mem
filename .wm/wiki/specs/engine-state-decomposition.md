---
title: "Decompose EngineState into per-domain state handles"
type: spec
id: "wiki:specs:engine-state-decomposition"
status: draft
tags: [spec, architecture, domain-driven, engine, wm-core]
relates_to:
  - {type: depends_on, target: "wiki:specs:wm-core-layer-extraction"}
---

schema_version: 1
state: |-
  ## Overview

  Break the `EngineState` god-object into per-domain state handles so the domain modules get explicit boundaries, then use those boundaries to decide which domains genuinely earn their own crate.

  `EngineState` (`apps/wm-core/src/engine/engine_state_mediator.rs`) is a flat bag of 21 public fields with 143 references workspace-wide; `page`, `graph`, `engine` and `search` all reach each other through it. That cyclic mesh is why module-level crate splits fail (D-11 in @wiki/specs/wm-core-layer-extraction, and the `e688f0` revert before it): no module has a closed interface while it can see every field. Decomposing the state is the precondition D-11 named for revisiting domain crates.

  ## Evidence

  Measured 2026-10-16, post-extraction (master `8636c2a`).

  - **21 public fields** on `EngineState`: `graph`, `page_contents`, `section_corpus`, `bm25_index`, `vector_registry`, `source_registry`, `content_hashes`, `stale_flag`, `config`, `audit_sender`, `audit_drops`, `started_at`, `project_root`, `embedder`, `vector_store`, `skill_engine`, `write_channel`, `index_scheduler`, `session_memory`, `lsp`, `tool_list`.
  - **143 `EngineState` references** workspace-wide: 19 files under `packages/wm-tools/src/tools`, 5 in wm-core page services, 4 in engine, 2 in search, 1 each in version, wm-code-intel and the tests.
  - The fields group into natural bounded contexts: graph snapshot; index (bm25 + vectors + embedder + vector store); page content + content hashes; sources; memory; config + project root + staleness; audit; skill + write channel + index scheduler; and the two odd ones (`lsp`, `tool_list`).

  ## Locked Decisions

  - **D-1 (shape):** `EngineState` becomes a **struct of about eight domain handles**. Call sites keep `engine.graph`-style access; the god-object stops being a flat bag but stays a facade. Chosen over explicit per-function dependencies (which rewrite 143 call sites at once) and over trait-based accessors (ceremony that would also delay any crate decision).
  - **D-2 (migration):** migrate **one domain at a time**, each an independently revertible change with the suites green before the next.
  - **D-3 (goal):** this is a **seam change, not a file move**. Whether any domain then earns a crate is decided after the handles exist, against D-11's (a)/(b)/(c) rubric - not assumed now.

  ## Requirements

  ### Functional Requirements

  - FR-1: Each domain's state is owned by exactly one handle type; no handle reaches into a sibling handle's fields.
  - FR-2: Cross-domain reads go through explicit methods on the owning handle.
  - FR-3: The observable behaviour of every MCP tool and CLI command is unchanged.

  ### Non-Functional Requirements

  - NFR-1: The `ArcSwap` snapshot pattern is preserved for read-hot state (graph, index) - no new locks on the read path.
  - NFR-2: No new `#[allow(dead_code)]` and no new compiler or clippy warnings.

  ## Acceptance Criteria

  - [ ] AC-1: `EngineState` exposes no flat public fields for the migrated domains; access is via the handle types.
  - [ ] AC-2: Each migrated domain has a unit test proving the handle owns its state and does not reach into a sibling handle.
  - [ ] AC-3: `page`, `graph` and `search` no longer reference each other's fields directly; the sibling-module import count drops measurably.
  - [ ] AC-4: `cargo build`, `test` and `clippy --workspace` are green and the existing suites are unchanged.
  - [ ] AC-5: The spec records, per domain, whether it now clears D-11's (a)/(b)/(c) for a crate, with the measurement.

  ## Scenarios

  ### Scenario 1: adding a domain without touching the others
  **Given** the handles exist
  **When** a new domain handle is added
  **Then** no existing handle's fields change and no consumer is edited.

  ## Open Questions

  - [ ] Migration order: candidates are index, then graph, then page (highest coupling first). Confirm before the first change.
  - [ ] Do `tool_list` and `lsp` belong to a domain handle at all, or to the application layer (`wm-tools`)?
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
answers: {}
