---
title: Per-type schema enforcement — required/optional typed frontmatter keys + mandatory ACs for task and spec
type: task
id: "wiki:tasks:per-type-schema-enforcement--requiredoptional-typed-frontmatter-keys--mandatory-acs-for-task-and-spec"
status: done
priority: high
tags: [schema, validation, record-format, enforcement, spec-required]
acceptance_criteria:
  - text: "A required/optional typed-key matrix is declared per page type and approved as a wiki decision before implementation"
  - text: "Enforcement rejects missing required keys at create and update via a single shared invariant helper, using ToolError::invalid_params"
  - text: "wm_validate.check reports missing required keys in BOTH entity and all scope, and wm_lint.check reports them consistently"
  - text: "Task and spec require at least one acceptance criterion (the earlier reviewer directive)"
  - text: "Every key in the six typed data models is explicitly decided as required, optional, or removed — no key remains defined-but-unenforced (dead schema)"
  - text: "Existing corpus is either compliant or remediated by a counted, reversible migration whose page count is reported before and after"
  - text: "Tests cover: each required key enforced on both write paths, each optional key accepted, validation and lint reporting, and the no-AC rejection"
relates_to:
  - {type: depends_on, target: wiki:decisions:per-type-frontmatter-schema}
implementation_notes: 'Implemented and green. Corpus: 150/150 specs and 400/400 tasks carry acceptance_criteria (scripted migration, 118 from checkboxes, 31 boilerplate; 4 tasks fixed). Enforcement: one shared wm-engine validate_required_keys called from create and update; validator entity+all scope and lint use the same rule/message. CLI got --acceptance-criteria; wm_doc and wm_task.subtask can supply ACs; health stub adds an AC so legacy empty tasks remain repairable. Commits c0b46f6, 01fde3f, e4e9c91, 9c407da; CI run 39681617660 green. Gates: wm-engine 122+3, wm-core 226+8 AC tests+all integration suites, mcp_test 124, cli_test 20, clippy clean, Gate 1 12,970.'
---

schema_version: 1
state: |-

  **Directive (reviewer, 2026-10-15):** 'each file type should have their own specific schema too - not just a loose body'. Direction chosen: **enforce the existing typed frontmatter keys** — not a body restructure. This also closes the earlier directive 'spec and task should be forced to have ac', which is the same root cause.

  **Findings that motivate this:**

  1. **Per-type schemas already exist in code** — six data models built from typed frontmatter keys (`packages/wm-engine/src/models/page_data/`): `TaskData` (acceptance_criteria, estimate, prerequisites, difficulty, implementation_plan, implementation_notes, time_*), `SpecData` (functional_requirements, non_functional_requirements, general_goals, stakeholders), `DecisionData` (context, options, rationale, outcome, consequences), `PatternData` (when_to_use, example), `RuleData` (category, rationale, example, anti_pattern), `MemoryData` (layer, ttl_days, last_verified, merged_into, rejected_reason). Populated from frontmatter in `apps/wm-core/src/parser/mod.rs:364-425`.

  2. **They are dead in practice.** Counted across `.wm/wiki`: `when_to_use` **0** pages, `stakeholders` **0**, `estimate` **0**, `prerequisites` **0**, `difficulty` **0**; `functional_requirements` / `non_functional_requirements` / `general_goals` **3 of 127** specs; `implementation_plan` **4 of 321** tasks; `rationale` 12, `category` 11, `anti_pattern` 1. A live pattern page (`patterns/arc-swap-graph.md`) carries no `when_to_use` key — its 'When to use' is prose inside `state`.

  3. **Nothing enforces them.** No per-type required-field enforcement exists for task or spec. `has_ac` and `needs_tasks` are declared-only booleans in the canonical question sets, never checked against real ACs or linked tasks (`packages/wm-engine/src/decision/canonical_questions.rs:312-353`; `packages/wm-engine/src/decision/record_validator.rs:162-175` checks question shape only).

  4. **The architecture is already decided.** `wiki:decisions:typed-section-keys-over-body-section-map` (**approved**) states the body stays uniform prose in `state` and 'where a section has fixed semantics, promote it to a typed frontmatter key'. `wiki:decisions:all-wiki-pages-structured-records` (draft) mandates the uniform envelope. So this task implements the approved direction and must NOT reverse it.

  5. **Partial enforcement exists but is in the wrong scope.** `apps/wm-core/src/mcp/tools/validate.rs:282-293` already emits 'Task should have at least one acceptance criterion' as an error — but only in `validate_all_scope`; `validate_single_entity` (`validate.rs:50`) skips it. Lint reports missing ACs for tasks as a **warning** (`lint.rs:57-64`) and never for specs.

  **Enforcement points (from recon, no single shared site today):**
  - Create: `apps/wm-core/src/page/services/page_crud_service.rs:162` `create_page_with_repo` (covers `wm_task.create`, `wm_task.subtask`, `wm_page.create`, `wm_doc.create`, `wm_decision`, `wm_memory`, migration).
  - Update: `apps/wm-core/src/page/services/page_update_builder_service.rs:102` `write_frontmatter_changes` (covers `wm_task.update`, `wm_page.update`, `wm_doc.update`, `check_ac`).
  - These share no common function above `PageRepo::write` (byte-level, no type context), so a shared invariant helper is required rather than one call site.
  - Rejection error: `ToolError::invalid_params` (`apps/wm-core/src/error/mod.rs:53`), matching existing status-membership rejections; surfaces as MCP `isError`.

  **Known blast radius if required keys are enforced naively:** of ~117 specs only **1** carries frontmatter ACs and the `wm-spec` skill writes ACs into the **body**, so 'every spec needs >=1 AC' would invalidate essentially the whole spec corpus; ~10 passing tests create no-AC tasks and expect success (`mcp_test.rs:1679, 1720, 1387, 1433, 1447, 1766, 957`). A counted, reversible remediation is mandatory.


  ## 2026-10-15 addendum - four live instances, detected only at commit time

  A commit attempt surfaced 4 AC-less tasks via `wm_validate.check` (status `fail`, 4 errors): `agent-handbooks-reference-nonexistent-cli-commands` (committed) plus `missing-license-file-and-cli-naming-drift`, `shim-template-command-parity-guard` and `agent-dx-freshness-tool-surface-and-rebuild-guarantee` (all untracked). All four predate 2026-10-15.

  **Detection is late.** They sat in the wiki indefinitely and were reported only when a commit was attempted. Nothing in the create path and nothing in the agent loop surfaced them earlier - which is the actual gap this task should close, not just the schema check.

  **Coordination constraint - this fix and the `extra_frontmatter` fix must land together.** `extra_frontmatter` is silently discarded whenever `content` is supplied in the same call (`wiki:tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content`, reproduced 2 for 2). A caller can therefore pass `acceptance_criteria` at creation today and receive an AC-less page with a success response. If per-type enforcement lands first, that same caller is **rejected for a task whose ACs they did supply** - converting a silent failure into a loud wrong one. Enforcement is only correct once the discard is fixed; sequence them deliberately.

  **Not established:** that these four instances came from that discard. Their version records contain no `acceptance_criteria` field, so there is no evidence an AC write was ever attempted for them. The mechanism is proven; these four being its product is not.

  **Related:** @wiki/concepts/failure-mcp-task-missing-acceptance-criteria, enriched 2026-10-15 with the discard as root cause.
**Honesty note:** the three pages authored in this session (howto/pattern/concept) all used prose `## When to use` sections instead of the typed keys — contributed to the problem, and should be remediated by whatever this task decides.