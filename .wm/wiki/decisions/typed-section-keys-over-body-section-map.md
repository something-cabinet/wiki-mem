---
title: 'Decision: Typed section keys over a body section map'
type: decision
id: wiki:decisions:typed-section-keys-over-body-section-map
status: approved
tags: [decision, parser-format, storage, schema, search-retrieval]
relates_to:
  - {type: references, target: wiki:specs:structured-all-doc-types}
  - {type: relates_to, target: wiki:decisions:all-wiki-pages-structured-records}
---

schema_version: 1
state: |-
  ## Context

  A recurring question about the page format: the file is `.md`, the frontmatter is YAML, and for 760 of 779 pages the body is also a YAML record (`schema_version`, `state: |-`, `questions`, `answers`) — only the `state` scalar is markdown. The natural follow-up was whether to go further: make the page a pure YAML document with a `body` field whose content is split into **headed section fields**, e.g.

  ```yaml
  body:
    - {heading: Context, level: 2, content: "|- prose"}
    - {heading: Decision, level: 2, content: "|- prose"}
  ```

  and whether JSON would be more optimal than YAML for that body.

  ## Decision

  Keep the current split and do **not** convert the body into a headed-section map. Specifically:

  1. The body stays a YAML record whose prose lives in the single `state` block scalar.
  2. Headed-section structure stays **derived**, not canonical — `apps/wm-core/src/graph/sections.rs` (`split_sections` → `SectionDoc`) builds the section corpus and BM25 index consumed by search and retrieval.
  3. Where a section has **fixed semantics**, promote it to a **typed frontmatter key** instead of a free heading: spec `functional_requirements`/`non_functional_requirements`/`general_goals`, decision `decision.context/options/rationale/outcome`, task `acceptance_criteria`.
  4. JSON is rejected for the body: it has no block scalars, so prose would become a single `\n`-escaped line — the exact legacy format the migration `normalize_state`/`ESCAPED_NEWLINE` handling has to decode.

  ## Rationale

  - **Prose is relocated, not removed.** A `body` section map still stores each section's text as a multiline string; it is a markdown AST re-encoded in YAML, with more ceremony and no less prose.
  - **Order is lost by default.** YAML mappings are unordered, so a section keyed by heading cannot preserve document order; a sequence with `level` is required, which is strictly more machinery than the markdown already present.
  - **Hierarchy is a tree.** H2/H3 nesting needs explicit levels and tree code once flattened into fields.
  - **Section granularity already exists for free.** Search/retrieval read sections built from `state` (spec FR-11: extraction reads `state`, not raw YAML); the machine benefit of structured sections is available from the derived index without paying the authoring cost.
  - **Round-trip safety.** NFR-2 of `@wiki/specs/typed-decision-doc-format` forbids whole-block YAML round-trip; more structure in the body widens the surface for the documented data-loss classes (`@wiki/concepts/frontmatter-corruption-sci-notation-id`, `@wiki/concepts:failure-bulk-frontmatter-repair-data-loss`).
  - **Ergonomics and the source-of-truth rationale.** `.md` + YAML frontmatter + a markdown `state` scalar keeps files git-diffable, editor-previewable, and human-editable; a pure YAML body reduces preview quality and diff readability for prose. The locked convention allows SQLite only for **local/derived** state, not the canonical store.

  ## Consequences

  - The `.md` extension is a container/frontmatter convention, not a promise the body is CommonMark; the meaningful markdown is the `state` scalar and its derived sections. This is accepted and documented here.
  - Future "more structured body" requests should be answered by extending typed per-type keys, not by restructuring `state`.
  - If a section's semantics become fixed and queried, add a typed key (and a validator rule) for it rather than parsing headings at query time.

  ## Related

  - @wiki/specs/structured-all-doc-types (D3/D4: idempotent migration, frontmatter preserved, `index.md` generated)
  - @wiki/specs/typed-decision-doc-format (D-D frontmatter stays YAML; NFR-2 no whole-block round-trip)
  - @wiki/decisions/all-wiki-pages-structured-records
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