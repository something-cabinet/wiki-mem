---
title: Edge Type Pruning: 16 → 9 types
type: spec
status: approved
---

# Edge Type Pruning: 16 → 9 types

**Status:** reviewed (oracle review incorporated) | **Task:** @wiki/tasks/edge-type-pruning

## Background

The engine defines 16 built-in edge types (`packages/wm-engine/src/models/edge_type_model.rs`). An audit claimed 5 are used; **re-verification during review found the audit table wrong in both directions.** Ground truth from real frontmatter (doc examples excluded — `concepts/graph-edge-types-traversal.md` and `specs/local-knowledge-engine-rust.md` contain yaml *illustrations*, not edges):

| Edge | Real edges | Pages | Audit claimed |
|---|---|---|---|
| `implements` | 7 | 4 | ✅ |
| `references` | 6 | 6 | ✅ |
| `answers` | 4 | 1 (`specs/design-pattern-alignment`) | ✅ |
| `example_of` | 2 | 2 | ✅ (1 page — actually 2) |
| `part_of` | 1 | 1 (`patterns/code-aware-tokenizer`) | ❌ never used — **wrong** |
| `extends` | **0** | — | ✅ used — **wrong** (only doc examples) |
| `depends_on` | **0** | — | ❌ (correct, but kept anyway) |
| `relates_to` | 1 (kebab `relates-to`) + wikilink-derived | — | "17 pages (all empty)" |
| `implemented-by` (custom, unregistered) | 2 | 2 — **dropped at graph build** | not in audit |
| `supports`, `contradicts`, `supersedes`, `required_by`, `questions`, `similar_to`, `causes`, `mitigates` | 0 | — | ❌ |

Graph stats confirm: **318 nodes, 13 edges (~4% connectivity)**. `.wm/config.json` has `custom_edge_types: []`, so the 2 `implemented-by` edges are silently skipped.

**Key insight the original audit missed:** usage alone is a weak pruning criterion. Edge types earn their keep through (a) distinct semantics tooling acts on, and (b) priority-driven retrieval weighting. `extends` (priority 10) has zero edges but is the structural backbone of concept hierarchy and the top retrieval weight. Prune on *semantic redundancy*, not just *statistics*.

## Decision

**Survivors (9 + custom):** `extends`, `implements`, `example_of`, `part_of`, `supersedes`, `depends_on`, `answers`, `references`, `relates_to`, `custom(...)`.

**Pruned (7):** `supports`, `contradicts`, `required_by`, `questions`, `similar_to`, `causes`, `mitigates`.

## Rationale per pruning (oracle review — not a rubber stamp)

| Pruned | Proposal mapping | Verdict | Reasoning |
|---|---|---|---|
| `required_by` | → `depends_on` (traverse inverse) | **AGREE** | Exact inverse; one canonical direction is correct. petgraph traverses incoming edges fine. ⚠️ But this creates a policy inconsistency: `tasks/task-edge-type-implemented-by` proposes *adding* `implemented-by` (inverse of `implements`). Decide once: **canonical single direction + reverse traversal**. Rewrite the 2 `implemented-by` edges as `implements` from the opposite side; don't register it. |
| `questions` | → `answers` | **AGREE with revised rationale** | The stated mapping is semantically wrong (a questioning page answers nothing). Real reason to cut: an *open* question is **state, not a relation** — you can't draw the edge until an answer exists, and once it exists, `answers` (Decision→Spec) captures it. Model open questions as task pages (actionable, trackable). Zero usage confirms. |
| `supports` | → `references` | **AGREE** | Evidence-vs-citation distinction earns nothing today (0 usage). `references` covers the wm-extract linking use case. Re-add later as a registered custom if evidence-tracking becomes real (e.g., test→spec verification). |
| `contradicts` | → `references` + status tag | **DISAGREE with mapping; AGREE with cut** | The mapping is a fiction: there is zero data to migrate, and `references` (priority 1) is *below* `graph_depth_retrieve_min_priority: 5` — a contradiction marked as `references` would be **invisible to retrieval**, destroying the one signal worth surfacing. `contradicts` is actually the most agent-actionable of the pruned types ("resolve before trusting either page"). But YAGNI: 0 usage → delete outright. Document `supersedes` as the path for version conflicts; if genuine conflicts need marking later, re-add deliberately. |
| `similar_to` | → `relates_to` | **AGREE** | Genuinely overlapping weak semantics; both low-priority. Lossless in practice. (Update wm-research docs — it lists `similar_to`.) |
| `causes` | → `relates_to` | **AGREE with cut; mapping irrelevant** | Causality is directional and strong; `relates_to` (priority 0) is the weakest type — the mapping is lossy but there is no data, so it costs nothing. Failure pages (wm-extract) already link via `references`. If failure→fix tracing matures, re-add as a pair with `mitigates`. |
| `mitigates` | → `relates_to` | **AGREE with cut; mapping irrelevant** | Same reasoning as `causes`. |

**Kept despite zero usage (deliberately):** `extends` (top retrieval weight, concept hierarchy backbone), `depends_on` (wm-flow task ordering / spec→task waves), `supersedes` (versioning must exist *before* you need it — you can't retroactively reconstruct what replaced what), `part_of` (1 real edge + SDD decomposition semantics). Pruning these would save nothing and damage the retrieval model.

## Migration plan

1. **Code** (`edge_type_model.rs`): delete 7 variants; re-map `priority()` (see below). Keep `Custom(String)`.
2. **Parsers**: update `parse_edge_type_flexible` (`relation_helper.rs`) — pruned names fall through to `Custom` → unregistered → skipped with warning at graph build. **No crash on stale frontmatter.** Update strict `parse_edge_type` (`wm-parser/src/lib.rs`) to match. Update `edge_type_to_yaml_str`.
3. **Frontmatter**: no data migration for pruned types (zero edges). Rewrite 2 `implemented-by` edges → `implements` from the opposite side (per inverse-edge policy); close `tasks/task-edge-type-implemented-by` as won't-do with a pointer here.
4. **Tests**: remove `DependsOn == RequiredBy` assertions (`wm-engine/src/lib.rs`, `apps/wm-core/src/engine/mod.rs`); keep Custom serde roundtrip test.
5. **Docs**: update all files listed below; fix "16-type" / "17 types" claims.
6. **Verify**: `cargo test` green; `wm-cli lint check` clean; graph rebuild shows no new unregistered-type warnings.

**Serde risk: low.** No binary persistence of `EdgeType` (no bincode/postcard); graph is rebuilt from frontmatter at startup; lenient parser degrades unknown strings to `Custom`. Removing variants is safe.

## Affected files

| File | Change |
|---|---|
| `packages/wm-engine/src/models/edge_type_model.rs` | Enum −7 variants, priority re-map |
| `packages/wm-engine/src/helpers/relation_helper.rs` | Parser + serializer |
| `packages/wm-parser/src/lib.rs` | Strict parser |
| `packages/wm-engine/src/lib.rs`, `apps/wm-core/src/engine/mod.rs` | Tests |
| `apps/wm-core/src/skills/wm-doc/SKILL.md` | Drop `questions` row; fix "16-type" ref |
| `apps/wm-core/src/skills/wm-extract/SKILL.md` | Fix "full 16-type reference" |
| `apps/wm-core/src/skills/wm-research/SKILL.md` | Drop `similar_to` from typed-edge list |
| `.wm/wiki/concepts/edge-types.md` | Canonical table (16→9) + usage-by-skill table |
| `.wm/wiki/concepts/graph-edge-types-traversal.md` | Priority table, "All 17 Edge Types" claim |
| `spec.md` | D12 (L57), FR-9 (L403), FR-27 (L437), edge table (~L658), examples (L982-983) |
| `docs/README.md` | Edge table (~L136) |
| `.wm/wiki/patterns/{mcp-response-format,platform-aware-mcp-config}.md` | Rewrite `implemented-by` edges |
| `.wm/wiki/tasks/task-edge-type-implemented-by.md` | Close per policy |

## Edge type priority re-evaluation

Current priorities have orphans after pruning (7, 4, 3 vanish). Gaps are harmless — priority is ordinal weighting, not a dense scale. **Do not renumber** (avoid invalidating `graph_depth_retrieve_min_priority: 5` semantics).

| Edge | Priority | Change |
|---|---|---|
| `extends` | 10 | — |
| `implements` | 9 | — |
| `part_of` | 8 | — |
| `supersedes` | 8 | — |
| `example_of` | 6 | — |
| `depends_on` | 5 | — |
| `answers` | 2 | **Recommend 2 → 5** (see below) |
| `references` | 1 | — |
| `relates_to` / `custom` | 0 | — |

**Open recommendation:** with `graph_depth_retrieve_min_priority: 5`, retrieval BFS never traverses `answers` edges — so a Decision that answers a Spec's question is invisible during context assembly from the Spec side. That undercuts the SDD payoff. Consider `answers: 2 → 5` (equal to `depends_on`). Not a blocker; decide during implementation.

## Updated skill references (post-prune)

| Skill | Edges | Notes |
|---|---|---|
| wm-extract | `references`, `example_of`, `supersedes`, `extends`, `relates_to` | Fix "16-type" mention |
| wm-doc | `answers`, `implements`, `depends_on`, `extends`, `part_of`, `references`, `supersedes`, `relates_to` | Drop `questions` row; open questions → task pages |
| wm-research | `extends`, `part_of`, `references`, `depends_on`, `relates_to` | Drop `similar_to` |
| wm-flow | none documented | Proposal's claim was stale — optionally add `implements`, `depends_on` docs |

## Missing docs / discrepancies found during review

1. **`spec.md` D12, FR-9, FR-27** still specify all 16 types + old priorities — must be updated (it's the root project spec).
2. **`concepts/graph-edge-types-traversal.md`** says "All 17 Edge Types" and documents pruned types with priorities.
3. **Inverse-edge policy is undocumented** — record the "canonical direction, traverse inverse" rule in `concepts/edge-types.md` once decided.
4. **`supersedes` edge vs `superseded_by` frontmatter field** (D11 in `specs/local-knowledge-engine-rust`) — potential duplicate versioning mechanisms. Follow-up question, not in scope.
5. **`relates_to` frontmatter key doubles as the generic edge type name** — confusing but out of scope.
6. **Low connectivity (13/318) is the real problem** — pruning types doesn't add edges. Consider a follow-up task: wm-extract/wm-doc should prompt for edges on every page create (checklist exists; enforcement doesn't).

## Rejected alternatives

- **Prune by usage only** (would cut `extends`, `depends_on`, `supersedes`): rejects the structural role of high-priority types in retrieval and SDD workflow.
- **Keep all 16 "just in case"**: taxonomy cost is real — 6+ doc locations, skill confusion, false precision in priority model.
- **Renumber priorities densely after pruning**: pointless churn; gaps are harmless.