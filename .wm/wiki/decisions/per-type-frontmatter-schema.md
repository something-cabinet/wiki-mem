---
title: 'Decision: Per-type frontmatter schema — required vs optional keys'
type: decision
id: "wiki:decisions:per-type-frontmatter-schema"
status: approved
tags: [decision, schema, record-format, validation]
implementation_notes: 'Approved 2026-10-22. Migration executed: 150/150 specs now carry acceptance_criteria (118 extracted from body checkboxes, 31 boilerplate review criteria, 1 pre-existing); the 4 AC-less tasks were fixed via wm_task.update; wm_validate.check reports 0 errors. Script: scripts/migrate-spec-acs.py (idempotent, dry-run default, --rollback available).'
---

schema_version: 1
state: |-
  ## Context

  Six typed frontmatter models exist (`packages/wm-engine/src/models/page_data/`), but almost none of their keys are
  enforced, and the corpus does not populate them. Counted 2026-10-22 over `.wm/wiki`:

  | Type | Pages | Typed-key population |
  |---|---|---|
  | task | 400 | acceptance_criteria 396; implementation_notes 83; plan/time_spent 5 |
  | spec | 150 | acceptance_criteria 1; functional/non-functional/goals 3 each; stakeholders 0 |
  | decision | 53 | rationale 1; context/options/outcome/consequences 0 in frontmatter |
  | pattern | 81 | when_to_use 0; example 0 |
  | rule | 20 | category 6; rationale 6; example 1; anti_pattern 1 |
  | memory | 160 | layer 0 in frontmatter (layer lives in the record answers) |

  Current enforcement is partial and split: `validate_all_scope` reports missing task ACs, `validate_single_entity`
  skips it, and lint reports it as a warning. There is no shared required-key check on create or update.

  The approved decision @wiki/decisions/typed-section-keys-over-body-section-map says fixed-semantics sections should
  be promoted to typed keys (task ACs, spec FR/NFR/goals, decision context/rationale/outcome). This decision defines
  which of those are *required* now, which stay optional, and how the corpus is remediated.

  ## Decision matrix

  Universal required for every record-bearing page (already structurally enforced): `title`, `type`, `id`,
  `status`, and the record envelope `schema_version/state/questions/answers`.

  | Page type | Required now | Optional (explicitly unenforced) | Removed |
  |---|---|---|---|
  | task | `acceptance_criteria` with at least one item | estimate, prerequisites, difficulty, time_spent, time_entries, implementation_plan, implementation_notes | none |
  | spec | `acceptance_criteria` with at least one item, after a counted migration | functional_requirements, non_functional_requirements, general_goals, stakeholders | none |
  | decision | none beyond universal | context, options, rationale, outcome, consequences | none |
  | pattern | none beyond universal | when_to_use, example | none |
  | rule | none beyond universal | category, rationale, example, anti_pattern | none |
  | memory | none beyond universal | layer, ttl_days, last_verified, merged_into, rejected_reason, metadata | none |
  | core/howto/reference/concept/note | none beyond universal | n/a | n/a |

  Rationale for the optional rows: their semantics are already carried by the record `answers` or by headed prose in
  `state`, and enforcing them would invalidate 50-149 pages per type for no demonstrated agent benefit. They stay
  defined and documented as optional rather than becoming accidental dead schema. Candidate removal (not now):
  pattern `when_to_use`/`example`, spec `stakeholders`, decision `options`.

  ## Enforcement

  - One shared helper `validate_required_keys(page_type, frontmatter) -> Result<(), ToolError>` called from both
    `create_page_with_repo` and `write_frontmatter_changes`; missing keys -> `ToolError::invalid_params` naming the
    key and page type.
  - Validator and lint call the same helper, so entity scope and all scope report identically.
  - Task ACs are enforced immediately (4 known violations). Spec ACs are enforced only after the migration below.
  - `has_ac`/`needs_spec` question answers are NOT cross-checked in this phase; they remain descriptive.

  ## Spec AC migration (counted and reversible)

  149 of 150 specs lack frontmatter ACs; 119 of those contain body checkboxes. Plan:
  1. Dry-run pass writes a report (page count before, per-page source of the AC, after count) to a versioned artifact.
  2. For specs with checkbox items, copy the unchecked/checked items into `acceptance_criteria` verbatim.
  3. For specs with no checkboxes, add one explicit criterion: "Spec reviewed and its requirements are implemented",
     unchecked. This is boilerplate but visible and honest, not a silent pass.
  4. Each migrated page gets a version record; re-running is idempotent; a rollback restores from versions.
  5. Enable spec-AC enforcement for create and update only after the after-count is 150/150.

  ## Consequences

  - Tasks become uniformly closeable (ACs always present); specs become closeable in the same way after migration.
  - 4 task pages and 149 spec pages need remediation; the migration is counted, reversible, and reported.
  - The typed models stop being dead schema: every key is explicitly required, optional, or a named removal candidate.
  - No body restructure; the uniform record envelope and headed prose stay.

  ## References

  - @wiki/tasks/per-type-schema-enforcement--requiredoptional-typed-frontmatter-keys--mandatory-acs-for-task-and-spec
  - @wiki/decisions/typed-section-keys-over-body-section-map
  - @wiki/reference/typed-decision-record-schema
  - @wiki/specs/record-format-wave
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers:
  outcome: adopted
  reversibility: false
  confidence: medium
  impact: system