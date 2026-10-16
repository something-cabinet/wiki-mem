---
title: 'Decision: No global rule layer - rules stay per-project'
type: decision
id: "wiki:decisions:no-global-rule-layer-per-project-rules"
status: approved
tags: [decision, rules, process]
relates_to:
  - {type: references, target: wiki:specs:global-wiki-layer}
---

schema_version: 1
state: |-
  ## Context

  Five projects each carry their own `.wm/wiki/rules/`. Measured 2026-10-15: 46 rule pages, the same concept under four names (no comments in code), five names for spec-first, and same-name content differing up to 4x between projects (`no-comments-in-code` at 498 B versus 2149 B). An agent proposed a global wiki layer - two composed source roots, project-wins shadowing, promote/demote - to converge them.

  ## Decision

  **No global layer.** Rules stay per-project, and the duplication is accepted deliberately rather than solved by composition. `MemoryLayer::Global` and the memory `promote` action are also removed (see @wiki/specs:remove-global-memory-layer), leaving project and session memory only.

  ## Rationale

  - **Authority.** A global rule set is a statement of the owner's preferences across projects. Which rules go global, and which variant is canonical, is the owner's call - and the mechanism's purpose was to push that judgement somewhere it does not belong. See @wiki/concepts:failure-agent-decided-owner-preferences.
  - **Cost.** Composition required a second engine root or copy-on-read merging, amending the single-engine-per-process invariant, plus shadow semantics, a trust gate, a global registry and an audit command.
  - **No demonstrated demand.** The store held nothing: `layer` was never persisted, the global directory never existed, and `promote` had one writer and no reader.

  The measured need is real, and is accepted as manual duplication rather than denied.

  ## Consequences

  - `wiki:specs:global-wiki-layer` is superseded and its tasks cancelled. The `ARCHITECTURE` single-root invariant stands unmodified - no amendment is needed at all.
  - `wiki:specs:configurable-wm-root` survives: relocating one project's state root is independent of cross-project sharing.
  - `wiki:decisions:scope-is-provenance-not-page-data` survives and remains the argument for deleting the wrong-typed `MemoryLayer::Global`.
  - If duplication starts to hurt, the cheap option is a **one-way sync** command that copies canonical rules into a project - no composition, no shadowing, no index change. Copy drift becomes ordinary, visible file drift.

  ## Related

  - @wiki/specs:global-wiki-layer
  - @wiki/decisions:global-wiki-multi-root-composition
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
answers: {}