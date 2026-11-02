---
title: Reference extractor must not capture trailing periods
type: spec
id: "wiki:specs:reference-extractor-must-not-capture-trailing-periods"
status: draft
tags: [references, graph, parser, spec]
acceptance_criteria:
  - text: A sentence-final @wiki reference resolves to the page it names
  - text: A name containing a period inside it still resolves
  - text: Unit tests cover sentence-final, comma-followed, parenthesised and dotted-name references
---

schema_version: 1
state: |-
  ## Goal

  An @wiki reference that ends a sentence must resolve to the page it names, not to the page name plus a period.

  ## Verified current state

  - REFERENCE_RE in apps/wm-core/src/reference_constant.rs captures the target with the class [A-Za-z0-9_./-], which includes the period, so a sentence-final reference keeps its trailing period.
  - graph::parse_page turns every extracted reference into a References edge, so the target becomes wiki:rules:durable-write-or-error. and the graph logs unresolved relates_to.
  - Seen in ~/projects/eightcap-new-portal (5 unresolved targets, all ending in a period) and in this wiki (the persist spec).

  ## Change

  Strip trailing periods (and other sentence punctuation that cannot end a page name) from the captured target before building the reference. A period inside a name stays.

  ## Verification

  - Unit tests in reference_service.rs: sentence-final reference, reference followed by a comma, reference in parentheses, name containing a period.
  - The wiki-mem graph rebuild no longer reports the unresolved target for the persist spec.
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: local
  needs_tasks: true
