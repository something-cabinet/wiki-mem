---
title: Typed-Decision Doc Format — System One Records for All Wiki Pages
type: spec
id: "wiki:specs:typed-decision-doc-format"
status: approved
tags: [typed-decisions, system-one, doc-format, migration, runtime, wiki, approved]
---

## Overview

Replace the human-readable body of the **convertible** wiki doc pages with a **System One typed-decision record** — a uniform `{ schema_version, state, questions, answers }` envelope whose questions are **type-appropriate** (`choice` / `score` / `noul`). Records are the **canonical doc format** and the **runtime input** an off-the-shelf local model answers over (`gliner-rs`). **No training is in scope.** Human readability is not a requirement.

The normative schema contract lives in `@doc/reference/typed-decision-record-schema` (v1). Built on `@doc/specs/system-one-model-application`, `@doc/concepts/system-one-feasibility`, and `@doc/concepts/typed-decision-application-map`.

## Locked Decisions

- **D-A**: The typed record **replaces the page body** (not an add-on).
- **D-B**: A **uniform envelope** for every converted page, with **per-page-type question sets** (fixed vocabularies; see schema doc).
- **D-C**: Records are the **canonical format** and the **runtime input** only. **Training is out of scope.**
- **D-D**: Keep YAML **frontmatter** (`id`/`title`/`type`/`status`/`tags`/`relates_to`); the **body is a strict machine record**. Readability not required.
- **D-E (revised)**: Convert only `decision`, `pattern`, `concept`, `howto`, `reference` (~160 pages: 37/61/46/10/6). **One-shot**.
- **D-F (revised)**: Runtime is `gliner-rs` + `GLiNER2.5-small-v1` (Apache-2.0); `score`/`noul` are **emulated** over its choice head. No training.
- **D-G**: The model is a **checksum-pinned download** (manifest in `.wm/models.json`), never committed.
- **D-H (blast radius)**: **Excluded from conversion** — `rule` (binding instructions read at session start), `core` (project conventions/architecture), `memory` (recall content injected into context), `task`, `spec`, `index.md`, `note`, and the steering files (`WIKI-MEM.md`, `AGENTS.md`, shims, `.wm/AGENTS.md`). These keep prose bodies.

## Requirements

### Functional Requirements

- **FR-1**: Define the record envelope per `@doc/reference/typed-decision-record-schema` v1 (`schema_version`, `state`, `questions[]`, `answers`), primitives `choice`/`score`/`noul`.
- **FR-2**: Preserve frontmatter unchanged.
- **FR-3**: Per-page-type canonical question sets (fixed vocabularies) in a registry (`canonical_questions(page_type)`), enforced by the validator.
- **FR-4**: Parser exposes the record through a typed accessor (`wm_page.get` returns a parsed `record` field, **no inference**); round-trip without corruption using line-based helpers.
- **FR-5**: `wm_validate` rejects malformed records (missing `state`; bad question `id`/`type`/`instructions`; `choice` <2 or >20 options; `score` outside 2–10 levels; answer key/value invalid; canonical-set mismatch). Answers are scalars/bool/lists by **label**; probabilities are runtime-only.
- **FR-6**: One-shot migration of the **convertible** pages, preserving frontmatter and id, reporting a per-type count; skip+report ambiguous pages.
- **FR-7**: Map `state`+`questions` into the gliner-rs wire format (no corpus export).
- **FR-8**: Runtime path: `wm_decision` MCP tool (`answer`/`status`) + CLI (`wm decide`), offline, via gliner-rs.
- **FR-9**: Migration idempotent (first body line `schema_version: 1` = no-op).
- **FR-10**: Obtain the model via checksum-pinned manifest (`.wm/models.json`), never committed; load by explicit path with gliner-rs offline mode.
- **FR-11 (new)**: **Record-aware extraction** — section splitting, inline tags, and `@wiki/` refs/wikilinks are extracted from `state` (not the raw YAML body) so search/graph do not regress.
- **FR-12 (new)**: Migration **pre-pass** for the duplicated-frontmatter/escaped-newline bug (first block wins; `relates_to` union; scalar conflicts → skip+report).

### Non-Functional Requirements

- **NFR-1**: Machine-first; readability not required.
- **NFR-2**: Round-trip safety — never whole-block YAML round-trip; `id` double-quoted; line-based helpers.
- **NFR-3**: No data loss — frontmatter byte-for-byte; `state` carries the normalized original prose.
- **NFR-4**: Local, offline, no hosted API, no training.
- **NFR-5**: Deterministic validation, covered by tests.
- **NFR-6**: No regression in page read, graph edges, or search (see FR-11).
- **NFR-7**: Only the model manifest is version-controlled, not the artifact.

## Acceptance Criteria

- [ ] AC-1: Envelope + all six per-type question sets documented and enforced by the canonical registry.
- [ ] AC-2: All **convertible** pages converted; 0 parse failures; per-type count reported; **excluded types untouched**.
- [ ] AC-3: `wm_validate` catches each FR-5 malformed case (tests).
- [ ] AC-4: Round-trip test: parse → write → parse identical (record + frontmatter).
- [ ] AC-5: `state`/`questions` map into the gliner-rs wire format (fixture test).
- [ ] AC-6: `wm_decision.answer` returns typed answers with probabilities via gliner-rs, offline.
- [ ] AC-7: No converted page alters its frontmatter fields.
- [ ] AC-8: Migration re-run is a no-op.
- [ ] AC-9: No regression in page read, graph edges, or search — **including wikilink/`@wiki/` edges and BM25 ranking** (FR-11 tests).
- [ ] AC-10: Model fetched via checksum-pinned manifest, not committed; runtime uses it with no training.
- [ ] AC-11: Excluded types (`rule`/`core`/`memory`/`task`/`spec`/`index`/`note`/steering) retain prose bodies and remain loadable by the agent workflow.

## Scenarios

### Scenario 1: Convert a decision page
**Given** a decision page with prose
**When** migration runs
**Then** the body becomes a record whose `state` carries the normalized prose and `answers` the recorded `outcome`/`reversibility`/`confidence`/`impact` (fixed vocabularies); frontmatter untouched.

### Scenario 2: Reject a malformed record
**Given** a `choice` with one option, or an answer key not matching any question
**When** `wm_validate` runs
**Then** a deterministic error names the page and field.

### Scenario 3: Runtime decision query
**Given** a `state` and a page type's question set
**When** `wm_decision.answer` runs
**Then** gliner-rs returns typed answers with probabilities, offline, no training.

### Scenario 4: Model fetched, not committed
**Given** a fresh checkout with no model
**When** the runtime is first used
**Then** WM downloads the pinned model, verifies its checksum, and proceeds; the artifact is absent from VCS.

### Scenario 5: Excluded docs untouched
**Given** `rule`/`core`/`memory` pages
**When** migration runs
**Then** their bodies remain prose and the agent rule/recall workflow is unaffected.

## Technical Notes

- Full contract: `@doc/reference/typed-decision-record-schema`.
- Reuse existing frontmatter parser + line-based helpers (`set_yaml_field`/`remove_yaml_block`/`ac_set_checked`); never whole-block YAML round-trip.
- Search/graph consumers read the raw body today (`graph/sections.rs`, `graph/mod.rs`, `parser/mod.rs`); FR-11 is required to keep them correct.
- Model: `gliner-rs` (candle) + `GLiNER2.5-small-v1`; long `state` (>512 tokens) → chunked classify with mean-per-label aggregation.
- Migration phase 1 is **code-only** (deterministic skeleton; `answers: {}` allowed); optional phase 2 agent backfill.
- Residual risk: gliner zero-shot classification is weak without calibration — measure ~30 labeled pages before committing the default.

## Open Questions

- [ ] Does the GlINER2.5-small `state` prose classification beat a trivial baseline on our pages (measured eval)?
- [ ] Should phase-2 agent answer backfill ship now or later?
- [ ] Where exactly `wm_decision` sits in the tool registry/docs (naming/parity).
