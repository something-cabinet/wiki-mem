---
title: WM Leapfrog — Replace Knowns with Complete Memory Layer
type: spec
tags: [spec, approved, roadmap, replacement, leapfrog]
---

# Spec: WM Leapfrog — Replace Knowns with Complete Memory Layer

## Overview

WM currently operates as a memory layer augmenting Knowns. This spec defines the incremental roadmap for WM to **leapfrog Knowns** — surpassing it by shipping features users assume Knowns already has (session memory, skill execution, tree-sitter) while matching Knowns' existing strengths (web UI, templates, ADRs, inline references).

The end state: WM is the sole memory layer. Knowns is deprecated and removed from this repo. No compatibility shims remain.

Core principle: **do not match Knowns' reality. Build what people assumed Knowns should have been.**

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | **Leapfrog Knowns** — build the fantasy | WM architected better (ArcSwap, graph, search, parallel). Ship the features Knowns never did. |
| D2 | **Incremental delivery** — each gap standalone | Lowest risk. Ship the smallest highest-impact item first. |
| D3 | **Rip bandaid** — fix `wm_doc.*` path immediately | Doc tools pointing at `.knowns/docs/` instead of `.wm/wiki/` is data corruption. Fix first. |
| D4 | **Web UI** — Angular + Sim UI + Tailwind CSS | Last priority (largest effort). Full web parity with Knowns. |
| D5 | **First gap = bug cleanup sprint** — 3 small items | Doc fix + session memory + appendNotes. Highest impact/effort ratio. |
| D6 | **Template engine = full Knowns parity** | Handlebars-style: control flow (if/unless/each/with), file ops (add, modify, append), case helpers, dry run. |
| D7 | **Tree-sitter code intelligence** | AST-aware, 100+ languages, offline. Knowns removed theirs. WM ships it. |
| OQ-1 | **FSRS-based session memory** | No TTL. FSRS scores entries, evicts by forgetting curve when capacity exceeded. |
| OQ-2 | **Skill execution = dispatch to agent** | Structured instructions returned to calling agent. Sub-agent spawner as future upgrade. |
| OQ-3 | **Tree-sitter languages: Rust, TS, Python, Go, Angular, Svelte** | Covers WM's own code + user projects + web UI + Svelte repos. |
| OQ-4 | **Axum HTTP server** (not Rocket) | Tower middleware composability with existing tokio + rmcp stack. Rocket syntax is cleaner but doesn't compose. |
| OQ-5 | **Migration via WM tools** | Run `.knowns/docs/` files through `wm_page.create` for clean re-import with correct frontmatter. |

## Requirements

### Functional Requirements

| FR | Description | Gap | Sprint |
|----|-------------|-----|--------|
| FR-1 | `wm_doc.*` reads/writes to `.wm/wiki/` not `.knowns/docs/` | Doc path fix | Sprint 0 |
| FR-2 | Session memory layer returns in-memory store instead of error | Session memory | Sprint 0 |
| FR-3 | `wm_task.update` supports `mode: "append"` for progress notes | appendNotes | Sprint 0 |
| FR-4 | `wm_skill.*` tools execute instructions instead of returning static text | Skill execution | Sprint 1 |
| FR-5 | Inline `@doc/`, `@task/`, `@memory/` references parse and resolve in body text | Reference resolution | Sprint 2 |
| FR-6 | Template engine supports if/unless/each/with blocks, file operations, case helpers, dry-run | Templates | Sprint 3 |
| FR-7 | Code intelligence uses tree-sitter for AST-aware symbol extraction across 10+ languages | Code intelligence | Sprint 4 |
| FR-8 | Web UI with Angular + Sim UI: search, graph, tasks, docs, memory, settings | Web UI | Sprint 5 |
| FR-9 | First-class ADR entity type with lifecycle (draft/accepted/superseded/rejected/archived) | ADRs | Sprint 3 |
| FR-10 | Task model supports subtasks (parent field + subtask tool) | Subtasks | Sprint 3 |
| FR-11 | Memory has semantic search via ONNX embeddings (not just BM25) | Memory search | Sprint 2 |

### Non-Functional Requirements

| NFR | Description |
|-----|-------------|
| NFR-1 | All existing WM tools continue to work throughout incremental delivery |
| NFR-2 | No data loss during Knowns → WM migration (`.knowns/docs/` → `.wm/wiki/`) |
| NFR-3 | Web UI build time adds ≤30s to CI |
| NFR-4 | Session memory survives individual tool calls within same MCP session |
| NFR-5 | Tree-sitter queries return results within 2s on repos up to 50k files |

## Acceptance Criteria

- [ ] AC-1: `wm_doc.get("specs/foo")` returns the same result as `wm_page.get("specs/foo")` (they share a store)
- [ ] AC-2: Agent calls `wm_memory.add({layer:"session", ...})` and can `wm_memory.list({layer:"session"})` in the same MCP session
- [ ] AC-3: `wm_task.update({id, appendNotes:"progress"})` appends to existing notes instead of replacing
- [ ] AC-4: `wm_skill.wm-plan` actually dispatches workflow instructions (triggers a real action, doesn't just return text)
- [ ] AC-5: Inline `@wiki/learnings/foo` in body text resolves to a link/reference
- [ ] AC-6: Template `{{#each items}}{{name}}{{/each}}` renders correctly
- [ ] AC-7: `wm_code.symbols` returns functions/classes/types parsed via tree-sitter, not regex
- [ ] AC-8: Web UI renders search results, graph visualization, task board, and doc viewer
- [ ] AC-9: Task can be created with `parent: "task-xxx"` and appears as subtask
- [ ] AC-10: `wm_memory.add({content, layer: "project"})` creates embedding vector for semantic search
- [ ] AC-11: Page can be created with `type: "decision"` and transitions through ADR lifecycle states
- [ ] AC-12: Knowns does not appear in any config file or tool output after migration

## Scenarios

### Scenario 1: Agent starts new session with WM
**Given** WM MCP server is running
**When** an agent calls `wm_initial.conventions` then creates a task with `wm_task.create`
**Then** the task appears in `.wm/wiki/tasks/` AND `wm_doc.get` can read it
**And** session-scoped context is preserved across tool calls

### Scenario 2: Agent executes a skill workflow
**Given** `wm_skill.wm-implement` is registered as an MCP tool
**When** an agent calls `wm_skill.wm-implement` with a task ID
**Then** the skill engine executes the workflow (dispatches instructions to agent or runs sub-actions)
**And** the agent receives actionable steps, not just static text

### Scenario 3: Migration from Knowns
**Given** the repo has existing `.knowns/docs/` content
**When** migration runs via `wm_page.create` for each file
**Then** all content is imported to `.wm/wiki/` with correct directory mapping and frontmatter
**And** `.knowns/` is either removed or ignored
**And** all existing `@doc/` references still resolve

### Scenario 4: Template-based code generation
**Given** a template with `{{#each fields}}{{pascalCase name}}{{/each}}`
**When** `wm_template.run` is called with field data
**Then** output contains correctly rendered casing for each field

### Scenario 5: Search across all entity types with semantic relevance
**Given** memory entries with semantic content exist
**When** `wm_search.query({query:"async error handling", type:"all"})` is called
**Then** results include both wiki pages AND memory entries
**And** memory entries have semantic (vector) scoring, not just BM25

## Delivery Sequence

Ship in this order. **Each sprint ends with a review gate** — spawn all relevant reviewers in parallel before merging.

| Sprint | Items | Effort | Review gate |
|--------|-------|--------|-------------|
| **Sprint 0** | FR-1 (doc path), FR-2 (session memory), FR-3 (appendNotes) | ~3 days | rust-reviewer + oracle |
| **Sprint 1** | FR-4 (skill execution) | 2-3 weeks | rust-reviewer + oracle + architect |
| **Sprint 2** | FR-5 (reference resolution), FR-11 (memory search) | 1 week | rust-reviewer + oracle |
| **Sprint 3** | FR-6 (templates), FR-10 (subtasks), FR-9 (ADRs) | 2 weeks | rust-reviewer + oracle + architect |
| **Sprint 4** | FR-7 (tree-sitter) | 2-3 weeks | rust-reviewer + oracle + architect |
| **Sprint 5** | FR-8 (web UI) | 2-3 months | designer + rust-reviewer + oracle + architect |
| **Final** | Migration + remove Knowns + remove shims | 1 week | oracle |

### Review Gate Process

After all sprint tasks are implemented and tests pass:

1. **Spawn reviewers in parallel**: rust-reviewer (code quality, safety), oracle (architecture, design decisions), architect (system design, trade-offs), designer (UI/UX, only Sprint 5+)
2. Each reviewer inspects the diff and produces findings with severity (error/warning/info)
3. Fix all errors before merging sprint
4. Fix warnings within same sprint if trivial, otherwise defer to tech debt log
5. Only then start the next sprint

## Technical Notes

- **Session memory**: Pure in-memory `DashMap<String, MemoryEntry>` scoped to the MCP server process. No persistence. FSRS scores entries, evicts by forgetting curve when capacity exceeded.
- **Skill execution**: Build on existing `TriggerConfig` + `fire_event()` infrastructure. Add a `SkillExecutor` trait that dispatches structured instructions back to the calling agent. The trigger system already parses lifecycle events (SessionStart, PageCreate, etc.) — wire them to the executor.
- **Doc path fix**: Change the hardcoded `.knowns/docs/` paths in `code/mcp/tools/doc.rs` to `.wm/wiki/`. Run migration via `wm_page.create` for each `.knowns/docs/` file.
- **Tree-sitter**: Use `tree-sitter` crate + language grammars as feature-gated deps. Start with 6 languages: Rust, TypeScript, Python, Go, Angular (HTML + TS), Svelte. Build queries for functions, classes, imports, calls.
- **Web UI**: Angular standalone + Sim UI + Tailwind CSS. Axum HTTP server embedded in Rust binary. `rust-embed` for Angular `dist/` assets. SSE for real-time sync. MCP bridge pattern stays for CLI/agent usage, Axum is only for the web UI.
- **appendNotes**: Add `mode: "append" | "replace"` parameter to `wm_page.update` and `wm_task.update`. Default remains "replace" for backward compatibility.
- **Migration**: `.knowns/docs/` files re-imported via `wm_page.create`. Map knowns doc types → WM page types: knowns `concepts/` → WM `concepts/`, knowns `tasks/` → WM `tasks/`, etc.

## Open Questions

> All open questions resolved. See Locked Decisions table above for OQ-1 through OQ-5.


## Migration Dir Mapping

WM infers page type from the first path segment under `.wm/wiki/`. **No new page types.** Knowns-only directories map to closest existing WM type:

| Knowns dir | WM dir | Type | Rationale |
|---|---|---|---|
| `concepts/` | `concepts/` | concept | Direct match |
| `specs/` | `specs/` | spec | Direct match |
| `learnings/` | `concepts/` | concept | Learnings are wiki concepts |
| `handover/` | `reference/` | reference | Handovers are reference docs |
| `knowns/` | `reference/` | reference | Knowns docs are reference |

During migration, `wm_page.create` receives the WM dir as path (e.g., `concepts/learning-foo`, not `learnings/foo`). Original Knowns dir name is not preserved as type.