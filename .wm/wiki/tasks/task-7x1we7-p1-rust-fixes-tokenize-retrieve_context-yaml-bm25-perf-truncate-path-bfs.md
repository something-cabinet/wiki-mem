---
title: P1 Rust fixes: tokenize, retrieve_context, YAML, BM25 perf, truncate, path BFS
type: task
status: done
tags: [review, rust, p1]
priority: high
knowns_id: 7x1we7
spec: specs/p1-rust-fixes-tokenize-retrieval-yaml-bm25-perf-truncate-bfs
relates_to:
  - {type: implements, target: wiki:specs:p1-rust-fixes-tokenize-retrieval-yaml-bm25-perf-truncate-bfs}
---

# P1 Rust fixes: tokenize, retrieve_context, YAML, BM25 perf, truncate, path BFS

> **Spec:** `specs/p1-rust-fixes-tokenize-retrieval-yaml-bm25-perf-truncate-bfs`

> *Imported from Knowns task `7x1we7`*

# P1 Rust fixes: tokenize, retrieve_context, YAML, BM25 perf, truncate, path BFS

## Description


Fix P1 items from rust-reviewer:

1. **tokenize excess String clones** (search.rs:237,241) — Avoid double allocation: change `let w = word.as_str().to_string()` + `tokens.push(w.clone())` to push only once.

2. **retrieve_context rebuilds full BM25** (search.rs:364-378) — When exact ID isn't in index, don't rebuild full BM25 from all graph nodes. Accept pre-built bm25_index as parameter.

3. **page.rs fragile YAML manipulation** (page.rs:312-374) — Replace line-based string matching with `serde_yaml` parse-modify-serialize for `set_yaml_field`, `ac_set_checked`, `remove_yaml_block`.

4. **Bm25Index score_doc O(n×m)** (search.rs:139) — Pre-compute term frequencies per document field to avoid linear scan per query term.

5. **rebuild_snapshot reads config from disk** (graph.rs:174-182) — Accept `custom_types: &[String]` parameter instead of re-reading config.json.

6. **Duplicate BFS path-finding** (tools.rs:984-1043, main.rs:1533-1558) — Extract shared `graph::find_path()` function.

7. **Duplicate truncate functions** (search.rs:522-528 vs util.rs:7-13) — Remove duplicate in search.rs, ensure callers use util::truncate_str.

8. **UTF-8 index panic in truncate** (search.rs:526) — Replace byte index with char_indices-based approach.

9. **parse_duration_to_minutes silent errors** (tools.rs:12-24) — Add tracing::warn! on parse failure.

10. **sync_skills_to duplicate** (skill.rs vs main.rs) — Remove disk-based version in skill.rs, keep embed-based version in main.rs.


## Acceptance Criteria



## Implementation Notes


All 10 P1 Rust fixes implemented:
1. tokenize: removed clone, push w directly
2. retrieve_context: added optional bm25_index parameter
3. YAML: replaced line-based manipulation with serde_yaml parse-modify-serialize
4. BM25 score_doc: pre-computed term_freqs HashMap per field
5. rebuild_snapshot: accepts custom_types parameter instead of reading config from disk
6. BFS path-finding: extracted shared graph::find_path(), used by both CLI and MCP
7. Duplicate truncate: removed local fn truncate from search.rs
8. UTF-8 panic: fixed truncate_str to use char_indices-based approach
9. parse_duration: added tracing::warn! on parse failure
10. sync_skills_to: removed unused function from skill.rs
All 108 tests pass.
