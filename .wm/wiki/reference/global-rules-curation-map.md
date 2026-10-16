---
title: Global rules curation map
type: reference
id: "wiki:reference:global-rules-curation-map"
status: reviewed
tags: [reference, rules, curation, global]
relates_to:
  - {type: references, target: wiki:specs:global-wiki-layer}
---

schema_version: 1
state: |-
  **Status: advisory only (2026-10-15).** The global wiki layer was removed by owner decision - the global rule set is the owner's call, and the mechanism pushed that judgement somewhere it does not belong. Nothing consumes this map and no promotion will occur; wiki:tasks:curate-01b is cancelled. Retained as a survey of the four projects' rule sets and as evidence of the divergence. See the Superseded note on wiki:specs:global-wiki-layer.

  Deliverable of wiki:tasks:curate-01-canonical-global-rule-set (phase A). Decides the curated global rule set **before** the composition mechanism lands; wiki:tasks:curate-01b-populate-global-root materialises it into the global rules root once gwl-01/02/04/05 exist.

  Measured 2026-10-15 across the four projects that have rules.

  ## Measured baseline

  | Project | rule pages | record bodies | prose bodies | duplicated frontmatter |
  |---|---|---|---|---|
  | wiki-mem | 18 | 18 | 0 | 0 |
  | eightcap-new-portal | 16 | 0 | 16 | 0 |
  | axon-ai-course | 6 | 0 | 6 | 0 |
  | fish-deckbuilding | 6 | 0 | 6 | 6 |

  **Format divergence is as severe as naming divergence.** 28 of 46 pages are prose bodies; only wiki-mem's 18 carry the record envelope. All six of fish-deckbuilding's pages carry **duplicated frontmatter blocks** - the same defect as the open task core-pages-carry-duplicated-frontmatter-blocks-and-escaped-newlines, live in a second project. Fish's pages are otherwise one-line stubs (401-508 B).

  **Decision on format.** The canonical global set is authored as **records** (WIKI-MEM.md: every page type is record-bearing). The prose copies are therefore shadowed or retired rather than promoted as-is - promoting a prose body into the global root would ship the very defect this repository has a rule against.

  ## A. Cluster resolutions

  **no-comments-in-code** - 4 names: axon no-comments-naming-only (2361 B), eightcap no-code-comments (2051 B), fish no-comments-in-code (498 B), wiki-mem no-comments-in-code (2149 B).
  Canonical slug: `no-comments-in-code`. Base: **wiki-mem** (richest: bans inline/block, rustdoc, `#[doc]`; names allowed exceptions). Delta: fold in eightcap's names-carry-intent framing. Resolution: fish's 498 B stub contains nothing the base lacks -> retire; the same-name pair resolves in favour of the base.

  **tdd-red-green-refactor** - 3 names: axon tdd-red-green-refactor (1996 B), fish `tdd` (1537 B), wiki-mem tdd-red-green-refactor (3361 B).
  Canonical slug: `tdd-red-green-refactor`. Base: **wiki-mem** (adds the RED-step-proof rationale and enforcement). Delta: none identified. Resolution: fish's `tdd` is a pure rename -> retire.

  **spec-first** - 5 names across 4 projects: axon `spec-first` (2437 B), eightcap spec-first-workflow (1983 B), fish spec-before-action (401 B) **and** spec-driven-development (476 B), wiki-mem has none.
  Canonical slug: `spec-first`. Base: **axon** (largest). Resolution: fish's two pages are intra-project near-duplicates and both retire into one canonical rule. Note: wiki-mem's `findings-first-task-spec` is **not** part of this cluster - it governs findings becoming a task plus spec, a different obligation, and stays global under its own slug.

  **no-else** - wiki-mem `no-else` (4077 B) vs fish no-else-prefer-early-return (471 B). Canonical `no-else`; base wiki-mem; fish retires.

  **no-magic-values** - wiki-mem (3190 B) vs fish (508 B). Canonical `no-magic-values`; base wiki-mem; fish retires.

  **parallel-execution** - axon parallel-execution-first (2816 B) vs eightcap parallel-first-execution (2509 B). Canonical slug `parallel-execution`. Base: **axon** - it is stack-agnostic (lanes, dependency analysis, ownership bounding, reconcile). Delta: fold in eightcap's explicit **parallel gate** (dependencies satisfied; write scopes disjoint; no shared runtime contract touched) and its **serialize runs, not authoring** distinction. Resolution: eightcap's page becomes **project-local**, shadowing the global rule, because it embeds content genuinely specific to that project - mock-server state on :8081, dev servers on :4300/:4400/:4310/:4410, and apps/eightcap-e2e/codecept.conf.ts.

  ## B. Scope decision for all 46 pages

  **wiki-mem (18)** - global: check-git-remote-for-pat, check-wm-tool-health-before-work, deepwork-must-follow-wm-workflows, durable-write-or-error, findings-first-task-spec, no-comments-in-code (base), no-compensating-layers, no-dead-code-clone-scanning, no-else (base), no-magic-values (base), no-warnings, tdd-red-green-refactor (base), tool-reliability-bug-tracking, use-tuistory-for-dev = 14. project-local: check-github-issue-board (names this repository's issue board), rust-anti-patterns (Rust-specific; flagged for review since the owner writes Rust across projects). retire: doc-comment-convention (superseded), no-allow-attributes (draft - not binding, so not promotable).

  **eightcap-new-portal (16)** - global: commit-message-style, language-english-only, no-commit-agentic-files, open-questions-evidence-based, agent-execution (delegation and approval pacing, which the other rules declare they extend) = 5. project-local: the five e2e-* rules (~16 KB of suite specifics: conventions, lint-and-gates, locators, mock-server-ownership, run-and-debug), app-code-conventions (Angular), prefer-nested-scss, respect-file-tree-and-barrel-files, parallel-first-execution (embeds this project's ports and paths, so it shadows the global rule) = 9. retire: no-code-comments, spec-first-workflow (both absorbed) = 2.

  **axon-ai-course (6)** - global: spec-first (base), parallel-execution-first (cluster), designer-scope-only (orchestration preference) = 3. project-local: ui-changes-design-system-first = 1. retire: no-comments-naming-only, tdd-red-green-refactor (absorbed) = 2.

  **fish-deckbuilding (6)** - all retire: the five absorbed stubs plus `tdd`.

  **Net global set: ~22 rules.** That is a large set to bind every project with on day one, and it is the main thing to trim during review - prefer promoting the rules a project would actually be surprised by over everything the owner happens to have written.

  ## C. AGENTS.md port map

  /users/nhkhanh/projects/AGENTS.md (5,907 B) is the de-facto cross-project rulebook and is read by no tool. Section mapping:

  | AGENTS.md section | Disposition |
  |---|---|
  | Preparation (tooling availability: fd, rg, fzf) | global - owner-environment preference |
  | Key principles - skill-first, one-question-at-a-time, multiple-choice, explore-alternatives | global, split into discrete rules |
  | Key principles - YAGNI, DRY, SOLID, design patterns, scalability, separation of concerns | fold into existing no-* rules or one new `design-principles` rule |
  | Language & Implementation Standards (Airbnb JS, functional JS, strict TS, NgRx reducers/actions/selectors) | global but stack-specific - owner's stack, not this repository's; review |
  | Additional implementation check (type safety, functions over 20 lines, TSDoc) | global - **except the TSDoc clause, see conflict below** |
  | Delegation Standards (specialist expectations table) | global |
  | Approval Gates (before design / implementation / between tasks / completion) | global |
  | Family Branches (S3 bucket, CodeBuild project names) | project-local - organisation-specific infrastructure |
  | Workflow Instruction + Core Rules (ByteRover query-first/curate-later) | project-local - tool integration for one MCP server |

  **Conflict the port must resolve.** AGENTS.md requires *"Use TSDoc for exported functions, explaining the Why"*, while this repository's `no-comments-in-code` bans **all** comments including rustdoc `///` and `#[doc = "..."]`. These cannot both bind. The canonical `no-comments-in-code` (which is the wiki-mem base, the richest of the four variants) wins on the evidence that it is enforced by CI, but the owner must confirm - the alternative is to scope the AGENTS.md clause to non-Rust or drop it.

  ## D. Ordering and staging decision

  Approved 2026-10-15 and recorded in wiki:specs:global-wiki-layer as an amendment: the composition mechanism ships with an **empty** global rules root, which binds nothing, and curation populates it afterwards. This reverses the superseded cascade text requiring curation to land by the time the cascade ships; the empty-first order satisfies that intent more safely, since the risk only materialises if the mechanism ships with uncurated content already in the global root.

  Staging therefore lives **inside this repository** for now, because confinement blocks writes below `~/.wm/` until the composition surface exists. This page is the staging artifact.
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