---
title: Don't fold training into index rebuild
type: memory
tags: [training, index-rebuild, architecture, decision-models]
status: active
---

Do NOT fold model training into `wm index rebuild`. Index rebuild is derived, cheap, idempotent state (graph/BM25/embeddings); training is costly (minutes-hours CPU), stochastic (breaks deterministic rebuilds), and artifact-producing (per-user, unversioned, breaks checksum-pinned manifests). Also: migrated records have `answers: {}` — no labels to train on; a self-training loop (answers->train->answers) amplifies errors; and it wouldn't fix the subjective/cross-domain mismatch. If training ever happens, it must be a separate, explicit, opt-in `wm model train` with a labeled corpus + versioned artifact.