---
title: Prune 16 edge types to 9 — remove unused, consolidate overlapping
type: task
status: done
relates_to:
  - {type: implements, target: wiki:specs:edge-type-pruning}
---

# Prune 16 edge types to 9 — remove unused, consolidate overlapping

**Severity:** P2 (medium) — taxonomy hygiene. Not user-facing broken, but every stale type is documented in 6+ places, confuses skill authors, and inflates the priority model. Compounds over time.

**Spec:** @wiki/specs/edge-type-pruning (review notes and final decisions live there — read before implementing)

**Estimate:** ~0.5 day (code) + 0.5 day (docs/frontmatter)

## Context

Audit (corrected during review — see spec): of 16 built-in edge types, only 7 have any real frontmatter usage; the graph has 13 edges across 318 nodes (~4% connectivity). 9 types have zero edges. The proposal collapses to 9 survivors: `extends`, `implements`, `example_of`, `part_of`, `supersedes`, `depends_on`, `answers`, `references`, `relates_to` (+ `custom`).

## Acceptance Criteria

- [ ] `EdgeType` enum in `packages/wm-engine/src/models/edge_type_model.rs` reduced to the 9 survivors + `Custom(String)`; `priority()` re-mapped per spec
- [ ] Lenient parser `parse_edge_type_flexible` in `packages/wm-engine/src/helpers/relation_helper.rs` updated; pruned names fall through to `Custom` (graceful, no crash on stale frontmatter)
- [ ] Strict parser `parse_edge_type` in `packages/wm-parser/src/lib.rs` updated to match
- [ ] Tests updated: priority assertions in `packages/wm-engine/src/lib.rs` and `apps/wm-core/src/engine/mod.rs` (`DependsOn == RequiredBy` assertion removed)
- [ ] Policy decision on inverse edges executed: either rewrite the 2 `implemented-by` frontmatter edges as `implements` from the opposite side, or register `implemented-by` in `.wm/config.json` — and close/re-scope `wiki:tasks:task-edge-type-implemented-by` accordingly
- [ ] Skills updated: `wm-doc/SKILL.md` (drop `questions` row, fix "16-type" ref), `wm-extract/SKILL.md` (fix "full 16-type reference"), `wm-research/SKILL.md` (drop `similar_to`)
- [ ] Wiki docs updated: `concepts/edge-types.md` (canonical table + usage-by-skill table), `concepts/graph-edge-types-traversal.md` (priority table, "17 types" claim)
- [ ] Project docs updated: `spec.md` (D12, FR-9, FR-27, edge table ~L658, examples L982-983), `docs/README.md` (edge table ~L136)
- [ ] `cargo build` + `cargo test` green
- [ ] `wm-cli lint check` clean (no new warnings from unregistered edge types)
- [ ] Follow-up question recorded: `supersedes` edge vs `superseded_by` frontmatter field overlap

## Affected Files

| File | Change |
|---|---|
| `packages/wm-engine/src/models/edge_type_model.rs` | Enum + priority |
| `packages/wm-engine/src/helpers/relation_helper.rs` | Lenient parser + YAML serializer |
| `packages/wm-parser/src/lib.rs` | Strict parser |
| `packages/wm-engine/src/lib.rs`, `apps/wm-core/src/engine/mod.rs` | Tests |
| `apps/wm-core/src/skills/wm-doc/SKILL.md` | Edge table |
| `apps/wm-core/src/skills/wm-extract/SKILL.md` | Edge table + ref |
| `apps/wm-core/src/skills/wm-research/SKILL.md` | Typed-edges list |
| `.wm/wiki/concepts/edge-types.md` | Canonical reference |
| `.wm/wiki/concepts/graph-edge-types-traversal.md` | Priority table |
| `spec.md`, `docs/README.md` | Root docs |
| `.wm/config.json` | Possibly register `implemented-by` (policy decision) |

## Notes

- No frontmatter migration needed for pruned types: all 7 pruned types have **zero** real edges. The "replacement mappings" in the original proposal (`contradicts → references + status tag`, etc.) are documentation guidance only, not data migrations.
- `wm-flow/SKILL.md` does **not** currently document edge types (proposal claimed it lists `implements/depends_on/required_by` — that table only exists in `concepts/edge-types.md`). Nothing to change there; optionally add a correct edge list.