---
title: golden_eval skips embedding yet asserts semantic recall when a model happens to be installed
type: task
id: "wiki:tasks:golden-eval-skips-embedding-but-asserts-semantic-recall"
status: in-progress
tags: [bugfix, test, search, hermeticity]
priority: medium
acceptance_criteria:
  - {text: "golden_query_recall_at_5 passes both with and without an embedding model installed in $HOME/.wm/models", checked: false}
  - {text: "The semantic recall floor is still asserted when a model is available, rather than skipped", checked: false}
  - {text: "No production code changes", checked: false}
---

**Finding.** `packages/wm-tools/tests/golden_eval.rs` branches on `engine.embedder.is_loaded()` to decide
whether to also evaluate semantic recall, but it builds the corpus with `wm_index_rebuild { skip_embed: true }`
unconditionally. When an embedding model is installed the embedder reports loaded, the semantic branch runs,
and `recall_for_mode(&registry, "semantic")` calls `wm_search.query` against an empty vector store, which fails
with `No embeddings indexed. Run index rebuild first.` and panics in `call_ok`.

**Reproduced 2026-10-19** after installing bge-small-en-v1.5 into `~/.wm/models`: `cargo test --workspace`
reports `golden_query_recall_at_5 ... FAILED` at `packages/wm-tools/tests/helpers/inproc.rs:43`. Without the
model installed the test passes, because the `!is_loaded()` branch returns early. So the suite's result
depended on the developer's global model cache even though the test pins keyword mode for its main measurement.

**Fix.** Build embeddings when a model is actually available: pass `skip_embed: !engine.embedder.is_loaded()`
to the rebuild, so the semantic assertion runs against a populated index. That makes the precondition match
the assertion instead of weakening the floor.