---
title: Rust Learning Knowledge Capture
type: spec
id: "wiki:specs:rust-learning-knowledge-capture"
status: draft
tags: [docs, rust, learning, knowledge-capture]
acceptance_criteria:
  - checked: false
    text: Spec reviewed and its requirements are implemented
---

schema_version: 1
state: |-
  ## Goal

  Capture two research passes as durable wiki knowledge so agents and humans can reuse them instead of re-deriving them:

  1. **Borrowing semantics** — the *order / area / layout* framework for diagnosing any borrow-checker error, transcript-verified from RustCurious episode 10 and mapped to real ownership sites in this workspace.
  2. **Runtime inspection** — how to observe addresses, layout, and variable state in Rust, recorded together with the repo-specific constraints that invalidate the generic advice.

  ## Deliverables

  | Page | Type | Purpose |
  |---|---|---|
  | `@wiki/howto/rust-inspect-variables-and-layout` | howto | Tooling ladder: `{:p}` / `&raw const` / `addr_of!`; `size_of` / `align_of` / `offset_of!` / `Layout`; `-Zprint-type-sizes`; `cargo expand` / `cargo asm`; `tracing` + lldb / CodeLLDB / rust-analyzer; Miri; ASan / dhat. Plus the wiki-mem caveats. |
  | `@wiki/patterns/borrow-errors-order-area-layout` | pattern | The three approaches, when each is right, and the verified anchors in this repo. |
  | `@wiki/concepts/rust-learning-research-sources` | concept | Source register: course playlist, episode, exercise repo, coverage per episode, what is next in the series. |

  ## Design

  **One finding, one task, one spec, three artefacts.** The three pages are one body of work with a shared source and method; splitting them into three task/spec pairs would be triple the ceremony for the same traceability. They are split at the *type* boundary (howto / pattern / concept) because each answers a different question and carries a different canonical question set.

  **Page types are chosen against the canonical question vocabularies**, not by intuition:

  - `howto` — `task_kind: debugging`, `has_verification: true`. It is a procedure with a verification step.
  - `pattern` — `problem_kind: architecture`, `language_specific: true`. It is a reusable decision framework, and it is Rust-specific.
  - `concept` — `kind: research-report`, `category: process`, `maturity: exploratory`. `reference` was rejected: its canonical `kind` vocabulary (`api | cli | configuration | error-catalog | schema | scoring`) has no slot for a source register.

  **Every page body is a typed-decision record** (`schema_version: 1` + `state` + canonical `questions` + `answers`), per `@wiki/reference/typed-decision-record-schema`. No prose bodies, no duplicated frontmatter fields inside `state`.

  ## Sourcing and honesty rules

  - Claims about Rust tooling carry a stability note (stable vs nightly) and, where it matters, the release the item stabilised in.
   - Claims about *this* repository carry a `file:line` and were re-read during this work; the four ownership anchors were confirmed by direct file read, not taken from the recon summary.
  - Third-party research outputs are labelled as such. Snippets marked verified by a research subagent are **not** re-labelled as verified here.

  ## Out of scope

  - No code changes. This is knowledge capture only.
  - No new wiki `rule` pages. If the pattern proves load-bearing it can be promoted later via the `wm-extract` flow.
  - No vendoring of the video's exercise code into the repo.

  ## Risks

  - **Link rot** — the episode is on YouTube; the register page records the stable identifiers (channel, playlist id, episode number, upload date) so the source is findable if the URL dies.
  - **Staleness** — nightly-only tooling claims (`-Zprint-type-sizes`, Miri, sanitizers) will drift; the page states the toolchain observed at time of writing rather than implying permanence.

  ## References

  - `@wiki/reference/typed-decision-record-schema`
  - `@wiki/patterns/arc-swap-graph`
  - `@wiki/concepts/arc-swap-try-unwrap-race`
  - `@wiki/tasks/capture-rust-learning-research-borrowing-semantics--inspection-tooling-as-durable-wiki-knowledge`
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