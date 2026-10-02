---
title: 'Pattern: Carve out agent-instruction docs before a body-format migration'
type: pattern
id: "wiki:patterns:instruction-doc-carve-out-before-body-migration"
status: draft
tags: [pattern, migration, wiki, agent-workflow, safe-refactor]
relates_to:
  - {type: references, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  ## Problem

  Replacing doc page bodies with a machine format (e.g. typed-decision records) can silently break the agent workflow when instruction-bearing pages are converted: `rule` pages are read as binding instructions at session start, `core` pages as project conventions/architecture, and `memory` bodies as recall content injected into context.

  ## Solution

  Before any body-format migration, run a **blast-radius recon** and carve out the instruction/recall pages: `rule`, `core`, `memory`, plus `task`/`spec`/`index.md`/steering files. Convert only descriptive types (`decision`, `pattern`, `concept`, `howto`, `reference`). Also make search/graph **record-aware** — read the record's `state` for sections/tags/`@wiki/` refs — so BM25 ranking and body-link edges do not regress once bodies become YAML.

  ## When to Use

  Any bulk rewrite of wiki/doc bodies, a frontmatter schema change, or a migration that touches page content.

  ## When Not to Use

  Metadata-only migrations (tags/status) that leave bodies untouched.

  ## Evidence

  In the `typed-decision-doc-format` migration (~160 pages), converting `rule`/`core`/`memory` would have severed the agent instruction channel; the carve-out kept 564 pages byte-identical and the wiki intact.
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}