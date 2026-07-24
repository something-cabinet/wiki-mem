---
title: "Architectural refactors: tools.rs split, skill dependency, method extraction"
type: task
status: done
tags: [review, architect, refactor]
priority: medium
id: uc9ioi
spec: specs/architectural-refactors-toolsrs-split-dependency-inversion-extraction
relates_to:
  - {type: implements, target: wiki:specs:architectural-refactors-toolsrs-split-dependency-inversion-extraction}
---

# Architectural refactors: tools.rs split, skill dependency, method extraction

> **Spec:** `specs/architectural-refactors-toolsrs-split-dependency-inversion-extraction`

> *Imported from Knowns task `uc9ioi`*

# Architectural refactors: tools.rs split, skill dependency, method extraction

## Description


Apply architect-recommended refactors:

1. **Split mcp/tools.rs** (1969 lines → domain modules) — Create mcp/tools/ directory with per-domain modules: search.rs, page.rs, source.rs, graph.rs, lint.rs, validate.rs, index.rs, task.rs, log.rs, time.rs, model.rs, project.rs, misc.rs. Each 100-250 lines. Keep tools.rs as ~80-line delegator. This is the single highest-value refactor.

2. **Invert skill → mcp dependency** (skill.rs:145-166) — Replace `register_mcp_tools()` with `tool_specs()` data method returning Vec<SkillToolSpec>. Wire MCP registration in tools.rs instead.

3. **Extract rebuild_memory_index to search.rs** (engine.rs:527-569) — Move BM25-building logic to `search::rebuild_memory_index_from_dir()`. Keep EngineState wrapper that calls it and stores via ArcSwap.

4. **Move recover_orphan_timers from source.rs** — It operates on task pages, not sources. Move to page.rs or its own module.

5. **Extract duplicate BFS to graph::find_path()** — tools.rs:984-1043, main.rs:1533-1558 both implement identical BFS path-finding. Extract to shared function.

6. **Add ScorcingConfig unit tests** (config.rs) — Verify all default values match expected. Test ProjectConfig::default() deserializes from valid JSON.

7. **Add PageType tests** — Test priority_rank() returns expected values. Add unit tests for page.rs YAML operations.


## Acceptance Criteria



## Implementation Notes


Architectural refactors implemented:
- tools.rs split: 14 domain modules in mcp/tools/ directory (search, page, source, graph, lint, validate, index, task, log, model, time, project, skills)
- tools.rs: ~30-line delegator
- Skill dependency inversion: tool_specs() data method, registration in tools/skills.rs
- rebuild_memory_index extraction: moved BM25 logic to search::rebuild_memory_index_from_dir()
- recover_orphan_timers: moved from source.rs to page.rs
- Duplicate BFS: already done in P1
- ScoringConfig + PageType unit tests added
All 120 tests pass.
