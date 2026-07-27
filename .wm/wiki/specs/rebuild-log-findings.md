---
title: Rebuild Log Findings Cleanup
type: spec
tags:
- health
- cleanup
- rebuild
- broken-refs
- empty-tasks
status: reviewed
---

## Overview

During a full `wm index rebuild` on 2026-07-27, the rebuild log surfaced three categories of wiki health issues: 27 empty/unparseable task pages, 50+ broken `relates_to` references to non-existent pages, and 1 graph cycle from mutual links. This spec defines automated cleanup tooling to resolve all three findings and produce a summary report.

## Locked Decisions

- D1: All three findings are in scope — empty tasks, broken refs, graph cycles.
- D2: Each empty task page is audited — stale pages are deleted, active-but-content-poor pages are stubbed with a description.
- D3: Broken `relates_to` entries are removed from their source pages.
- D4: An automated script handles detection, cleanup decisions, and report generation.

## Requirements

### Functional Requirements

- FR-1: Tool detects all wiki pages that exist on disk but yield zero parseable sections.
- FR-2: Tool detects all `relates_to` entries in YAML frontmatter that reference a page ID not present in the wiki.
- FR-3: For each empty task page, the tool reports its ID, title (if available), `relates_to` edges, and inbound reference count so a human can decide stale vs. stub.
- FR-4: Tool supports a `--dry-run` / `--check` mode that only reports findings without any modifications.
- FR-5: Tool removes broken `relates_to` entries from source pages (with `--fix` or confirmation), preserving all valid edges.
- FR-6: Tool produces an end-of-run summary: pages deleted, pages stubbed, refs removed, graph cycle count, and any remaining issues.
- FR-7: Tool runs as a CLI subcommand (`wm health audit` or equivalent) and can be re-run idempotently.

### Non-Functional Requirements

- NFR-1: Idempotent — re-running against an already-clean wiki produces no changes and confirms clean state.
- NFR-2: Safe — default mode is dry-run; destructive operations require explicit `--fix` flag.
- NFR-3: Fast enough to run on every index rebuild (~5s overhead max for a 500-page wiki).

## Acceptance Criteria

- [ ] AC-1: Running `wm health audit --dry-run` against the current wiki identifies exactly the 27 empty task pages and 50+ broken refs reported in the index rebuild, without modifying any files.
- [ ] AC-2: After audit and `--fix`, running the tool again reports zero empty task pages and zero broken refs.
- [ ] AC-3: The tool produces a machine-readable report (JSON) plus a human-readable summary.
- [ ] AC-4: No valid `relates_to` edges are removed by the tool.
- [ ] AC-5: The tool does not modify pages outside the `tasks/` directory for the empty-page audit (deletion/stub scope).

## Scenarios

### Scenario 1: Happy Path — Full Cleanup
**Given** a wiki with 27 empty tasks and ~55 broken refs
**When** the operator runs `wm health audit` (dry-run by default) and reviews the report
**Then** no files are modified and the report categorizes each empty task as stale or active, lists all broken refs, and flags inbound reference counts
**When** the operator re-runs with `wm health audit --fix`
**Then** stale tasks are deleted, active ones are stubbed with a brief description, all broken refs are removed, and a summary is printed
**And** a subsequent `wm index rebuild` runs cleanly with zero warnings for sections or unresolved targets
**And** `wm_validate.check` reports zero broken wiki:* refs

### Scenario 2: Idempotent Re-run
**Given** a clean wiki with no empty tasks and no broken refs
**When** the operator runs `wm health audit --dry-run`
**Then** the tool reports "wiki is healthy: 0 empty pages, 0 broken refs" and makes zero modifications

### Scenario 3: Broken Ref with Multiple Valid Edges
**Given** a page with `relates_to: [{type: extends, target: "wiki:concepts:real"}, {type: depends_on, target: "wiki:tasks:deleted"}]`
**When** the operator runs `wm health audit --fix`
**Then** the broken `wiki:tasks:deleted` entry is removed
**And** the valid `wiki:concepts:real` edge is preserved

### Scenario 4: Empty Task That Is Referenced Elsewhere
**Given** an empty task page that is referenced by 6 other pages via `relates_to`
**When** `wm health audit --dry-run` is run
**Then** the tool flags "6 inbound refs" so the human can assess impact before deletion

## Technical Notes

- The tool should parse YAML frontmatter directly from `.wm/wiki/**/*.md` files, not via MCP tools, for speed.
- Use the existing graph or `wm_graph_stats` to cross-reference resolved vs. unresolved nodes.
- Graph cycles are expected (mutual references) and already handled by BFS visited tracking — no code change needed, just report the count for awareness.
- Implementation as a new CLI command under `wm-cli`: `wm health audit [--dry-run] [--fix] [--format json|text]`. Default mode is dry-run.

## Open Questions