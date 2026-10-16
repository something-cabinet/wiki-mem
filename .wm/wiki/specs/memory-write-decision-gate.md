---
title: Memory Write Decision Gate — Heuristic Answering of the Memory Question Set
type: spec
id: "wiki:specs:memory-write-decision-gate"
status: approved
tags: [spec, approved, memory, decision-gate, heuristics, typed-decisions]
---




schema_version: 1
state: |-
  ## Overview

  Add a deterministic, heuristic decision gate to `wm_memory.add` so a memory write is evaluated **before** a page is created — should it be stored, is it a near-duplicate of existing memory, which layer, and how confident is the recorded knowledge. This replaces the local model runtime for the memory question set: the gliner typed-decision runtime is declared non-viable (`@wiki/decisions:gliner-decision-runtime-not-viable-keep-heuristics`) and the `decision` feature stays off by default.

  Architecture (per oracle review): a **peer answerer** in `wm-core` that produces `DecisionResult`-shaped answers directly from engine context. `DecisionBackend`/`DecisionRuntime` in `wm-engine` are **not** modified — they are text-classification shaped and have no corpus/graph access, so routing relational memory decisions through them would be a compensating layer.

  ## Locked Decisions

  - **D1**: `wm_memory.add` is a **real gate**. On a near-duplicate it returns `{status:"skipped", reason, nearest:{id,score}}` and writes nothing; otherwise it stores and fills the record answers. This is a behavior change; the tool description must document the new `skipped` status.
  - **D2**: v1 answers the **memory set only**. `dedup_action` emits only `new`|`skip`; `merge`/`supersede` are deferred until eval evidence exists. Their absence in v1 is intentional, not a bug.
  - **D3**: amend `@wiki/specs:typed-decision-doc-format` **D-H** to reflect reality — all page types are record-bearing; only some have an answerer. (Verified: 18/18 rule, 4/4 core, 129/146 memory pages already carry envelopes.)
  - **D4**: **one** conservative, config-overridable skip threshold. The gate returns `nearest:{id,score}` evidence even when it stores. An evaluation step is required before the threshold may be lowered.
  - **D5**: `confidence` is **provenance-categorical**, no numeric β. v1 emits `high` when content cites concrete anchors (`wiki:` id, file path, or eval number — checkable by existing parser utilities), else `medium`. The classifier is a **total function** with a named catch-all predicate (`unverifiable provenance`) currently mapped to `medium`; the `low` mapping is **reserved and documented**, not emitted in v1.
  - **D6**: the peer answerer lives in `wm-core` (in-process with graph/search/config). Relational questions use a memory-scoped `run_unified_search`. Thresholds live in `wm-constants` + the existing config schema. `wm-engine`'s `decision` module is unchanged.
  - **D7**: the **session layer is exempt** from the gate in v1 — FSRS eviction (`evict_lowest_fsrs`) already bounds it.
  - **D8**: **no** heuristic mode on `wm_decision.answer` in v1 — one consumer, one code path.

  ## Requirements

  ### Functional Requirements

  - **FR-1**: New `wm-core` module exposing `evaluate_memory_write(engine, title, content, layer_hint) -> Result<MemoryWriteDecision, ToolError>`.
  - **FR-2**: The gate runs before `create_page_through_builder` on the project layer.
  - **FR-3**: Near-duplicate (top-1 memory score >= threshold) → `store=false`, `dedup_action=skip`, response status `skipped` with `reason` and `nearest{id,score}`; **no file is written**.
  - **FR-4**: Otherwise → page created; record `answers` populated with `layer`, `store_or_skip=true`, `dedup_action=new`, and `confidence` from D5.
  - **FR-5**: `nearest{id,score}` evidence is returned in **both** outcomes.
  - **FR-6**: The skip threshold is a named, config-overridable constant (no magic values).
  - **FR-7**: `confidence` = `high` iff content cites concrete anchors; else `medium` (D5); the unverifiable case is a named predicate reserved for the future `low` mapping.
  - **FR-8**: Session-layer `add` bypasses the gate unchanged.
  - **FR-9**: Amend `typed-decision-doc-format` D-H (D3, doc change).

  ### Non-Functional Requirements

  - **NFR-1**: Deterministic — no model, no network, fully offline.
  - **NFR-2**: No new index plumbing — reuses the existing memory BM25 / unified search.
  - **NFR-3**: Answers written via line-based record/frontmatter helpers, never a whole-block YAML round-trip.
  - **NFR-4**: Zero new compiler warnings; no `#[allow(...)]`; no comments (production code).
  - **NFR-5**: No magic values — enums and named constants only.
  - **NFR-6**: TDD — failing test first for each behavior.

  ## Acceptance Criteria

  - [ ] AC-1: A near-duplicate `add` returns status `skipped` and writes **no** file (test).
  - [ ] AC-2: A non-duplicate project `add` creates the page with all four answers populated (test).
  - [ ] AC-3: The response carries `nearest{id,score}` in both outcomes (test).
  - [ ] AC-4: Session-layer `add` still inserts into the session store and is unaffected (test).
  - [ ] AC-5: `confidence=high` only when content cites concrete anchors; `medium` otherwise (table test).
  - [ ] AC-6: Overriding the threshold config changes skip behavior (test).
  - [ ] AC-7: `dedup_action` never emits `merge`/`supersede` in v1 (test).
  - [ ] AC-8: `low` is accepted as a `confidence` answer value by the record layer, even though the heuristic does not emit it (test) — reservation for D5.
  - [ ] AC-9: `typed-decision-doc-format` D-H updated to state all types are record-bearing (doc check).
  - [ ] AC-10: The `wm_memory` tool description documents the new `skipped` status.

  ## Scenarios

  ### Scenario 1: Novel memory stored
  **Given** an empty memory corpus
  **When** `wm_memory.add` is called with a new title/content
  **Then** a `type: memory` page is created, `answers` contains `{layer: project, store_or_skip: true, dedup_action: new, confidence: <high|medium>}`, and the response contains `nearest` (likely empty/null).

  ### Scenario 2: Near-duplicate rejected
  **Given** a memory page already exists
  **When** `add` is called with content whose top-1 memory score meets the threshold
  **Then** no page is written and the response is `{status:"skipped", reason, nearest:{id,score}}`.

  ### Scenario 3: Session add is not gated
  **Given** `layer:"session"`
  **When** `add` is called
  **Then** the entry is inserted into the session store exactly as today, with no gate evaluation.

  ### Scenario 4: Unverifiable content
  **Given** content with no `wiki:` id, file path, or eval number
  **When** the heuristic assigns confidence
  **Then** it returns `medium` (the reserved `low` predicate is not yet wired).

  ## How the heuristic works

  **Overview.** The gate runs on the project layer *before* the page is written: it slugifies the title, checks the existing memory corpus for a duplicate, derives the four canonical answers, and then either refuses the write (`skipped`) or creates the page with those answers baked into the record envelope. Every decision is deterministic and returned as auditable evidence.

  | Question | Rule |
  |---|---|
  | `layer` | From the caller's layer hint; defaults to `project`. The **session** layer bypasses the gate entirely (FSRS eviction already bounds it). |
  | `store_or_skip` | Two duplicate checks, OR-ed: (1) **exact** — the target `memory/<slug>` path already exists; (2) **near-duplicate** — tokenize `title` + `content`, score every memory document with the **raw** BM25 score, take the top-1, and skip when it is `>= memory_dedup_threshold`. |
  | `dedup_action` | Derived from the store decision: `new` when stored, `skip` when not. `merge`/`supersede` are deferred (D2). |
  | `confidence` | Provenance-categorical: `high` when the content cites a concrete anchor — a `wiki:` / `@wiki/` reference, a file path, or a measurement carrying a unit or `%` — else `medium`. A named predicate reserves the `low` seat (D5). |

  **Why `low` is not emitted in v1.** `low` is already part of the canonical set, so the record layer accepts it (test `memory_record_layer_accepts_the_reserved_low_confidence`). It is withheld from the heuristic because there is no evidence-backed predicate for it yet: the only candidate — "unverifiable provenance" — would fire on any content lacking a concrete anchor, which today would silently downgrade genuine knowledge. Emitting a value that nothing consumes would also add noise without changing behaviour. The predicate is named and currently mapped to `medium`; the `low` mapping is reserved, so adopting it later is one branch plus an eval step — no schema, validator, or migration change (D5).

  On store, all four answers are written into the record envelope (`build_record_body` then `write_record_answers`), so the created page body is a canonical memory record rather than prose.

  **Why raw BM25 and one threshold.** `Bm25Index::search` normalizes the top-1 score to 1.0, so it cannot express an absolute threshold; the gate uses the raw per-document score instead. The threshold is a single config value (`search.scoring.memory_dedup_threshold`, default `100.0`) set conservatively so only near-duplicates trip it. Evidence (`nearest` id + score) is returned in **both** outcomes so every decision is auditable.

  **What it does not do.** No merge/supersede, no model path (`wm_decision` unchanged), no effect on the session layer.

  ## Technical Notes

  ```rust
  // apps/wm-core/src/memory_gate/mod.rs  (new)
  pub struct GateEvidence { pub nearest_id: Option<String>, pub nearest_score: Option<f32> }
  pub struct MemoryWriteDecision {
      pub store: bool,
      pub dedup_action: DedupAction, // New | Skip   (Merge/Supersede deferred, D2)
      pub layer: MemoryLayer,
      pub confidence: Confidence,    // High | Medium (Low reserved, D5)
      pub evidence: GateEvidence,
  }
  pub fn evaluate_memory_write(engine: &EngineState, title: &str, content: &str,
                              layer_hint: Option<&str>) -> Result<MemoryWriteDecision, ToolError>;
  ```

  `WmMemoryAction::Add` calls this before `create_page_through_builder` (`apps/wm-core/src/mcp/tools/memory.rs:361`).

  ## Open Questions

  - [ ] Confirm a memory-scoped `run_unified_search` (`apps/wm-core/src/search/query.rs:92`) is cheap without new index plumbing.
  - [ ] Grep whether any `wm-extract` consumer parses `wm_memory.add`'s response shape (a new `skipped` status could confuse success-assuming callers).
  - [ ] When `low` is adopted (D5), should it **couple** to the gate (e.g. low confidence → not stored, or flagged), or stay decorative? Decide in a follow-up with eval evidence.
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