---
title: Unify the YAML frontmatter serializers
type: spec
id: "wiki:specs:yaml-frontmatter-serializer-unification"
status: draft
tags: [spec, yaml, frontmatter, serializer, migration]
acceptance_criteria:
  - checked: false
    text: 'AC-1: Exactly one YAML frontmatter serializer exists workspace-wide, asserted by grep count in CI.'
  - checked: false
    text: 'AC-2: Pinned round-trip tests cover all six shapes, written RED before the unification.'
  - checked: false
    text: 'AC-3: The affected pages (the five timer-recovery task pages and any block-sequence users) migrate with no data loss.'
  - checked: false
    text: 'AC-4: `cargo build`, `test` and `clippy --workspace` are green.'
---

schema_version: 1
state: |-
  ## Overview

  Unify the three YAML frontmatter serializers in the workspace into one conformant writer, so a single implementation owns the byte output for every frontmatter shape.

  Split out of @wiki/specs/wm-core-layer-extraction as D-10: the consolidation was attempted there and correctly stopped because it requires a type change, a byte-output decision, and a page migration - not a step-0 code move.

  ## Evidence

  - `frontmatter_to_yaml` is a **live writer**, reached through `page/services/timer_recovery_service.rs` (task pages whose `time_started` is older than 24 hours); five such pages exist today. It is not a dormant second copy.
  - Six shapes would change bytes, each with a live example: forced double-quoting vs `yaml_scalar`; multi-line literal blocks (`|-`) emitted instead as a newline inside `"..."` (malformed YAML); block sequences vs flow maps for `acceptance_criteria` and `relates_to`; flow vs block `tags`; unknown keys such as `spec:`; and quoted ids.
  - `FrontmatterValue` cannot express block sequences at all - it is `Scalar | Id | Int | List(Vec<String>) | Nested(Vec<(&str, FrontmatterValue)>)`, so two of the six shapes cannot be produced without extending the value model.
  - `graph/export.rs` is a different job (a merge that preserves unknown keys) and stays out of scope.
  - **Measured 2026-10-16 (EXTRACT-02 parity harness, `scripts/verify-frontmatter-byte-parity.sh`):** whole-frontmatter idempotence is already broken on the pre-refactor serializer - `serialize(parse(serialize(parse(x))))` differs from `serialize(parse(x))` on 57 of 372 live task pages, **identically on both revisions**. `implementation_plan`/`implementation_notes` are emitted as a double-quoted scalar containing raw newlines; re-parsing folds the breaks, and when the text itself contains a `"` (for example a literal `type: "spec"` inside a plan) the emitted block does not re-parse at all. This is the malformed-YAML shape named below, now with a count; it is reported by the harness, not asserted in the suite, because it is pre-existing rather than a regression.

  ## Locked Decisions

  - D-1: The canonical byte output is the **owner's** decision, not the implementer's. This spec carries the six shapes as open options, not resolutions.

  ## Requirements

  ### Functional Requirements

  - FR-1: One implementation owns YAML frontmatter serialization for every writer in the workspace.
  - FR-2: The `FrontmatterValue` model can express every shape that exists in the live wiki, including block sequences.
  - FR-3: Every affected page round-trips byte-identically or is migrated in the same change.

  ### Non-Functional Requirements

  - NFR-1: No page's bytes change without an explicit, recorded migration.

  ## Acceptance Criteria

  - [ ] AC-1: Exactly one YAML frontmatter serializer exists workspace-wide, asserted by grep count in CI.
  - [ ] AC-2: Pinned round-trip tests cover all six shapes, written RED before the unification.
  - [ ] AC-3: The affected pages (the five timer-recovery task pages and any block-sequence users) migrate with no data loss.
  - [ ] AC-4: `cargo build`, `test` and `clippy --workspace` are green.

  ## Scenarios

  ### Scenario 1: round-trip a page with block-sequence ACs
  **Given** a task page whose `acceptance_criteria` is a block sequence
  **When** the unified serializer writes it
  **Then** the bytes are identical to the input.

  ## Open Questions

  - [ ] Byte-output decision: for each of the six shapes, which rendering wins? (forced double-quoting vs `yaml_scalar`; `|-` block vs escaped newline; block sequence vs flow map for `acceptance_criteria`/`relates_to`; flow vs block `tags`; unknown-key passthrough; quoted id). Owner decides - do not resolve in implementation.
  - [ ] Migration mechanism: one-shot rewriter vs serializer-version bump? Recommendation: one-shot rewriter with pinned tests.
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