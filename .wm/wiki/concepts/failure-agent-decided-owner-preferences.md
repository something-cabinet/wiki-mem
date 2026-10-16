---
title: 'Failure: An agent decided the owner''s preferences'
type: concept
id: "wiki:concepts:failure-agent-decided-owner-preferences"
status: reviewed
tags: [failure, process, authority]
relates_to:
  - {type: references, target: wiki:specs:global-wiki-layer}
---

schema_version: 1
state: |-
  An agent decided the owner's preferences for them.

  ## What went wrong

  Given the requirement that shared rules be usable across projects, an agent produced a curated global rule set - deciding, for all 46 rule pages across four projects, which rules become global, which stay project-local, which retire, and which existing variant becomes the canonical text for each cluster. Every one of those calls is a statement of the owner's preferences. The owner's response was immediate: "why are you doing that? global rule should be for user to decide."

  ## Root cause

  A question of **authority** was framed as a question of **technical convergence**. Convergence problems have a best answer, so the agent optimised for one - pick the stack-agnostic variant as the base, retire the thin stubs, trim the set. Preference problems have no best answer, only the owner's. The framing error was invisible because the work looked like analysis.

  It is sharper because the agent had already written the correct rule twice: the spec it authored said "Curation is human, tooling only reports" and "owner-led", and its escalation brief quoted "Curation decides. Do not add fuzzy auto-merge." Then it did the work anyway. **Writing a rule is not the same as obeying it.**

  ## Prevention

  - When a deliverable encodes the owner's preferences - which rules, which conventions, which variant is canonical, what is shared versus local - produce **options with a recommendation and blank decision fields**, never resolutions. See @wiki/patterns:preference-artifacts-present-options-not-decisions.
  - Treat "which of these is better?" as the owner's question even when one answer is technically defensible.
  - If you notice you are about to *decide* the very thing the artifact exists to let the owner decide, that is the signal - not a detail to get right.

  ## Time lost

  A full design chain: four external review rounds, four specifications, thirteen tasks, one curation pass - all reversed in a single message. The reversal was cheap; noticing before the first specification would have been cheaper.

  ## Related

  - @wiki/tasks:curate-01-canonical-global-rule-set
  - @wiki/specs:global-wiki-layer
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}