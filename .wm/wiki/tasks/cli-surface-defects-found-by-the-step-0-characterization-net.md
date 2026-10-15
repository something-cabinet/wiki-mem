---
title: CLI surface defects found by the step-0 characterization net
type: task
id: "wiki:tasks:cli-surface-defects-found-by-the-step-0-characterization-net"
status: todo
priority: medium
tags: [cli, determinism, bugfix, test-infra]
acceptance_criteria:
  - text: "page list output order is deterministic (sorted at the listing boundary) and a multi-page golden can be added to the characterization net without flaking"
  - text: "wm-cli agents --sync either honours the flag or the flag is removed — decided explicitly, since the CLI surface is frozen during the rewrite"
  - text: "The onnx semantic tests either pass on a machine with a valid model cache or fail with a message naming the model path attempted"
---

Findings surfaced while building the `wm-cli` characterization net (step 0 of `wiki:tasks:controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review`). All are CLI-surface or determinism issues, none blocked step 0.

1. **`page list` output order is filesystem-dependent.** The graph/walk layer uses unsorted `walkdir` (`apps/wm-core/src/graph/mod.rs:65`), so page listing order varies with directory entry order. Consequence: any multi-page golden would flake, which is why the characterization net deliberately uses a single-page fixture; and users see unstable ordering. Fix direction: sort by page id (or path) at the listing boundary, then a multi-page golden becomes possible.

2. **`wm-cli agents --sync` is accepted and ignored.** The arm destructures the flag and discards it (`sync: _sync`) while always performing a sync, so the flag is a lie. Either honour it (skip the sync when absent) or remove the flag as dead surface. Note this is inside the clap surface the rewrite is freezing — changing it is a behaviour change and must be an explicit, individual decision, not a drive-by.

3. **`semantic_test` onnx tests fail on a developer machine whose model cache is broken.** `onnx::semantic_search_with_loaded_model` and `onnx::hybrid_search_rrf_fusion_or_fallback` fail with `tokenization error: tokenizer load: expected value at line 1 column 1` because `~/.wm/models/bge-small-en-v1.5` does not load. The suite is therefore not green out of the box. Fix direction: diagnose whether the cache is corrupt or the loader is wrong, and make the failure message name the model path it tried.

Also recorded: `code_index_watcher_test::watcher_indexes_new_source_file` flaked once and passed on rerun — a flake worth watching, not yet diagnosed.