---
title: P0: Wire body @wiki/ references into graph builder
type: task
status: todo
priority: high
tags: [from-spec, spec:graph-connectivity-fix, p0]
---

Implement FR-1 to FR-4. In build_graph_from_wiki, after collecting frontmatter relates_to entries, call reference_service::extract_references(body) to extract body @wiki/ refs, convert to edges, deduplicate. Add reciprocal references edges for each body-extracted ref.