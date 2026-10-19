---
title: 'Backfill edges for orphaned learnings: 557 orphans, 145 with no relates_to'
type: task
id: "wiki:tasks:backfill-edges-for-orphaned-learnings-pages"
status: todo
tags: [docs, wiki-health, graph, consolidation]
priority: medium
acceptance_criteria:
  - {text: "Every pattern, concept and decision page carries at least one edge to its source task, spec or parent concept", checked: false}
  - {text: "Edge types are the strongest applicable one, not references by default", checked: false}
  - {text: "wm-cli lint check orphan count drops measurably from the 557 baseline, reported per batch", checked: false}
  - {text: "wm-cli validate stays green after each batch", checked: false}
---

**Consolidation sweep, 2026-10-19.** Full `wm-extract --consolidate` across the wiki. Measured state:
941 nodes, 894 edges, **557 orphans** (pages with no edges at all), 0 broken references (`wm-cli validate`
returns pass). Page counts: 79 patterns, 56 concepts, 52 decisions, 12 howto, 11 reference, 4 core, 156 memory.

**Duplication is not the problem.** No two patterns or concepts share a slug, and no near-duplicate titles
were found across the two directories. The consolidation backlog is edge coverage, not merged content.

**Edge coverage is the problem.** 63 of 79 patterns, 43 of 56 concepts and 39 of 52 decisions carry no
`relates_to` frontmatter at all. Those pages are invisible to graph traversal, so `wm_search` cannot surface
them through neighbours and a future session cannot discover that prior work already answered a question.

**Why it was not fixed in this pass.** Adding an edge requires knowing the correct target and type per page -
145+ judgements, not a mechanical rewrite. Fabricating links to hit a number would poison the graph, which is
worse than leaving it sparse.

**Plan.** Batch by directory, highest-value first: decisions (they answer recurring 'why' questions), then
patterns, then concepts. For each page: find its source task or spec, pick the strongest edge type from the
nine (`implements`, `answers`, `example_of`, `part_of`, `supersedes` before `references`), and link it. Track
the orphan count from `wm-cli lint check` as the metric, in batches of about 20 pages per change so each is
reviewable.