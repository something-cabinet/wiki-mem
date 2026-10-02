---
title: Typed-Decision Record Schema (v1)
type: reference
id: "wiki:reference:typed-decision-record-schema"
status: draft
tags: [typed-decisions, schema, system-one, gliner-rs, reference]
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  ## Purpose

  The finalized **v1 schema** for typed-decision record bodies, produced by TD-01. Normative contract for the parser, validator, migration, and runtime of `@doc/specs/typed-decision-doc-format`. Runtime: **`gliner-rs` + `GLiNER2.5-small-v1`** (no training). Read alongside `@doc/concepts/typed-decision-application-map` and the blast-radius findings below.

  ## Envelope

  Body is a strict YAML mapping with **exactly four top-level keys**: `schema_version`, `state`, `questions`, `answers`.

  ```yaml
  schema_version: 1
  state: |-
    (normalized original prose; headings preserved)
  questions:
    - id: outcome
      type: choice          # choice | score | noul
      instructions: What is the recorded outcome of this decision?
      options: [adopted, rejected, deferred, superseded, abandoned]
    - id: reversibility
      type: noul
      instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
    - id: confidence
      type: score
      instructions: How strong is the recorded justification for the selected outcome?
      levels: [low, medium, high]
  answers:
    outcome: adopted
    reversibility: true
    confidence: high
  ```

  | Field | Type | Req | Rules |
  |---|---|---|---|
  | `schema_version` | int | ✅ | exactly `1`; first key; unknown top-level keys rejected |
  | `state` | string | ✅ | non-empty; **literal block scalar** (`\|-`); soft cap 1 MiB (warn) |
  | `questions` | list | ✅ | 1–32; ids unique; must equal the canonical set for the page type (id, type, multi, options/levels, order) |
  | `answers` | map | ✅ | keys ⊆ question ids; may be `{}`; absent = unanswered (not an error) |

  Question fields: `id` (`^[a-z][a-z0-9_]{0,47}$`), `type` (`choice|score|noul`), `instructions` (single line, 1–240 chars; `noul` must be a positive declarative), `multi` (choice only, default false), `options` (choice, 2–20 unique), `levels` (score, 2–10 unique, **ascending order is semantic**).

  Answer values: `choice` → a label; `choice`+`multi` → list of labels; `score` → **the level label** (not index); `noul` → `true`/`false`. **Probabilities are never stored** — they are runtime output only.

  ## gliner-rs wire mapping

  | Primitive | `ClassificationSpec` | Decode |
  |---|---|---|
  | choice (single) | labels = options, prompt = instructions, softmax | argmax + distribution |
  | choice (multi) | `multi_label(true, 0.5)`, sigmoid | labels ≥ 0.5 |
  | noul | labels = `["false","true"]`, softmax | value = argmax=="true"; report P(true) |
  | score | labels = levels (in order), softmax | argmax → level label + distribution |

  `task` = question `id`; `prompt` = `instructions`; `labels` = options/levels — 1:1, no transformation. Per-label probabilities available (`-a/--all`). Long `state` (>512 tokens) → `classify_text_long` (384-word chunks, 64 overlap) with deterministic **mean-per-label** aggregation; result reports `chunks`/`truncated`.

  ## Per-type question sets (canonical, fixed vocabularies)

  **decision** — `outcome` choice [adopted,rejected,deferred,superseded,abandoned]; `reversibility` noul; `confidence` score [low,medium,high]; `impact` choice [local,component,system,project-wide]

  **rule** — `enforcement` choice [ci-enforced,tool-enforced,review-enforced,convention-only]; `severity` score [advisory,recommended,required,blocking]; `has_exception` noul; `applies_to` choice-multi [code,tests,docs,configuration,workflow,security]

  **pattern** — `problem_kind` choice [architecture,api-design,data-model,error-handling,performance,testing,ui,tooling,workflow]; `preconditions_required` noul; `complexity` score [trivial,simple,moderate,complex]; `language_specific` noul

  **concept** — `kind` choice [concept,failure-analysis,research-report,reference-note]; `category` choice [architecture,search-retrieval,graph,parser-format,mcp-tooling,cli,storage,embeddings,web-ui,process]; `maturity` score [raw,exploratory,established,stable]; `code_referenced` noul

  **howto** — `task_kind` choice [setup,development,testing,release,debugging,operations,integration]; `prerequisites_complete` noul; `difficulty` score [beginner,intermediate,advanced,expert]; `has_verification` noul

  **reference** — `kind` choice [api,cli,configuration,error-catalog,schema,scoring]; `surface` choice-multi [mcp-tool,cli-command,rust-api,http-api,config-file]; `stability` score [unstable,evolving,stable,frozen]; `has_examples` noul

  Registry lives in `packages/wm-engine` as `canonical_questions(page_type)` — single source of truth for parser, validator, migration, runtime.

  ## Open-question resolutions

  - **Model manifest** → committed `.wm/models.json` (logical name + immutable HF `revision` + per-file `sha256`), plus a non-committed cache manifest under `~/.wm/models/<name>/`. WM loads the verified cache dir by explicit path and sets `GLINER_OFFLINE=1` so gliner-rs never auto-downloads unverified. Extend the existing `wm_model` registry + SHA-256 verification (`packages/wm-embed/src/services/onnx/mod.rs`).
  - **Runtime surface** → new **`wm_decision`** MCP tool (`answer`, `status`) + CLI (`wm decide`); `wm_page.get` additionally returns a parsed `record` field with **no inference**. Do **not** load the model in the hot page-read path.

  Runtime result (never persisted): `{ page, model, answers: {id: {value, probability, distribution?}}, chunks, truncated }`.

  ## Migration edge cases

  - **Duplicated frontmatter** (pre-existing bug): pre-pass — first block authoritative; `relates_to` unioned; other scalar conflicts → skip + report.
  - **Escaped-newline body**: decode `\n` only in the unambiguous single-line case; else preserve + report. Block-scalar-only `state` makes the class unrepresentable.
  - **Long content**: full body stored in `state`; runtime chunks.
  - **No answers**: `answers: {}` valid; migration phase 1 is code-only (skeleton); optional phase 2 agent backfill.
  - **Search/graph non-regression (must-fix)**: record-aware extraction — de-indent `state` and run section splitting on it; extract wikilinks/inline tags from `state`. Otherwise BM25 IDF is polluted and body-link edges disappear.

  ## Exclusions (from TD-02 blast radius)

  Do **not** convert (bodies stay prose):
  - **`rule`** (17) — read as binding instructions every session (`wm-init` SKILL, `WIKI-MEM.md`).
  - **`core`** (4) — project-defining conventions/architecture read at init.
  - **`task`** (296), **`spec`** (117) — already out of scope.
  - **`memory`** (114) — body is the recall content injected into context.
  - **`index.md`**, **`note`**, steering files (`WIKI-MEM.md`, `AGENTS.md`, shims, `.wm/AGENTS.md`).
  - Non-standard dirs (`conventions/`, `learnings/`, `research/`) — classify explicitly first.

  **Convertible**: `decision`, `pattern`, `concept`, `howto`, `reference` — with `@wiki/` refs preserved (graph edges).

  ## Recommended spec changes (TD-01 flags)

  1. Pin runtime to `gliner-rs` + `GLiNER2.5-small-v1`; `score`/`noul` are emulations over the choice head.
  2. Answers are authored scalars/bool/list only; probabilities are runtime output.
  3. `state` is string block scalar; `schema_version: 1` required.
  4. Use fixed-vocab `outcome` (not page-specific `chosen_option`); amend Scenario 1.
  5. Allow empty `answers`; migration phase 1 code-only; AC-2 checks parse + frontmatter, not answer completeness.
  6. Add an explicit FR for record-aware section/tag/wikilink extraction with tests.
  7. Answer `score` by label, not index.
  8. Promote duplicated-frontmatter/escaped-newline handling to a migration pre-pass requirement.
  9. **Carve `rule`/`core`/`memory` out of scope** (this is the critical scope change).

  ## Residual risk

  GLiNER zero-shot classification on `state` prose is weak without calibration; the format's value is a uniform decision corpus, not an accuracy guarantee. Measure on ~30 hand-labeled pages before committing the default model.

  ## References

  - @doc/specs/typed-decision-doc-format
  - @doc/concepts/typed-decision-application-map
  - @doc/concepts/system-one-feasibility
questions:
  - id: kind
    type: choice
    instructions: What kind of reference material is this?
    options:
    - api
    - cli
    - configuration
    - error-catalog
    - schema
    - scoring
  - id: surface
    type: choice
    instructions: Which surfaces does this reference document?
    multi: true
    options:
    - mcp-tool
    - cli-command
    - rust-api
    - http-api
    - config-file
  - id: stability
    type: score
    instructions: How stable is the documented surface?
    levels:
    - unstable
    - evolving
    - stable
    - frozen
  - id: has_examples
    type: noul
    instructions: This reference includes usage examples.
answers: {}
