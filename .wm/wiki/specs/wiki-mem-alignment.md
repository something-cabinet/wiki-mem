---
id: wiki:specs:wiki-mem-alignment
title: WIKI-MEM.md Alignment
type: spec
status: draft
tags: [docs, knowns-parity, wiki-mem]
---

schema_version: 1
state: |-
  id: wiki:specs:wiki-mem-alignment

  ## Overview

  Align `WIKI-MEM.md` with Knowns' `KNOWNS.md` structure and content.

  ## Gaps Identified

  | Section | KNOWNS.md | WIKI-MEM.md |
  |---|---|---|
  | TL;DR | Cleaner, more concise | Verbose, missing "Don't revert changes" |
  | References | ✅ `@task`, `@doc`, `@template` with line/range/heading | ❌ Missing |
  | Recommended File Roles | ✅ KNOWNS.md + shims + other docs | ❌ Missing |
  | Tool Matrix mentions `task` for delegation | ✅ | ❌ Missing |
  | Common Mistakes (CLI pitfalls) | ✅ `--plain` vs `--json` vs `--smart`, `-a` flag | ❌ Missing |
  | MCP preferred over CLI | ✅ "Use CLI only as fallback" | ❌ Not stated |
  | Self-contained? | Fallback when MCP unavailable | Also self-contained, but has wiki conventions inline |

  ## Requirements

  - FR-1: Update TL;DR to match KNOWNS.md conciseness
  - FR-2: Add References section (`@task`, `@doc`, `@template` with line/range/heading)
  - FR-3: Add Recommended File Roles section
  - FR-4: Add `task` to tool matrix for delegation
  - FR-5: Add Common Mistakes (CLI pitfalls)
  - FR-6: Add "prefer MCP, CLI as fallback" guidance
  - FR-7: Optionally strip wiki conventions (7 page types, frontmatter schema) to skills, keeping WIKI-MEM.md as rules-only
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
