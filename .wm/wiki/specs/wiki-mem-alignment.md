---
title: WIKI-MEM.md Alignment
type: spec
status: draft
tags: [docs, knowns-parity, wiki-mem]
---

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
