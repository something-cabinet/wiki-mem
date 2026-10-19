---
title: wm model download writes to the wrong directory and installs a 404 tokenizer as if it succeeded
type: task
id: "wiki:tasks:model-download-writes-to-the-wrong-directory-and-fetches-a-404-tokenizer"
status: done
tags: [bugfix, cli, model, tool-reliability, durable-write]
priority: high
acceptance_criteria:
  - {text: "wm model download places the model where the engine reads it ($HOME/.wm/models/<name>), and wm model list reports that same path", checked: false}
  - {text: "All model path helpers in apps/wm-cli/src/model/model_service.rs resolve one directory, consistent with the engine and with remove_model", checked: false}
  - {text: "The tokenizer is fetched from the repository root, not from the onnx/ subdirectory, and a real 711 KB tokenizer is installed for bge-small-en-v1.5", checked: false}
  - {text: "A non-2xx response is never written to disk as a model file; the download fails with an error naming the status instead", checked: false}
  - {text: "Regression tests cover the path resolution, the tokenizer URL derivation, and the status check", checked: false}
implementation_notes: |-
  Fixed in commits e2d5054 and 0f32590.

  Path: models_dir_for_root/models_dir in apps/wm-cli/src/model/model_service.rs resolve
  $HOME/.wm/models, matching main_engine_factory.rs:28; the download, cached listing and removal call sites
  all route through it. Test: models_dir_resolves_under_hidden_wm_dir.

  Tokenizer: ModelEntry gained an explicit tokenizer_url; ensure_success rejects any non-2xx before bytes are
  written, on both the weights and tokenizer paths. Tests: every_registry_entry_points_at_a_root_tokenizer and
  non_success_status_is_rejected (both need --features onnx).

  Verified live: download installs to ~/.wm/models/bge-small-en-v1.5 with model.onnx 133,093,490 B, tokenizer
  711,396 B of real JSON, and manifest.json. The previous 15-byte 'Entry not found' tokenizer is gone.

  Separate concern recorded in the task body: eager model loading costs every CLI command about 3.46s.
---

**Two defects found while confirming the model download end-to-end** on 2026-10-16 at HEAD 5f9f7be, after
the nested-runtime panic in @wiki/tasks/wm-model-download-panics-with-nested-runtime-error was fixed.

**1. The download writes to a directory the engine never reads.**
`apps/wm-cli/src/model/model_service.rs` resolves the model root two different ways: `run_onnx_download`
(:51) and `print_cached_models` (:128) use `models_root().join(MODELS_SUBDIR)`, which is `$HOME/models`,
while `remove_model` (:172) uses `models_root().join(WM_DIR).join(MODELS_SUBDIR)`, which is
`$HOME/.wm/models`. The engine reads `$HOME/.wm/models` (`apps/wm-core/src/engine/main_engine_factory.rs:28`).
Observed: a successful 133 MB download landed at /Users/nhkhanh/models/bge-small-en-v1.5, and the engine still
logged 'No ONNX model found at /Users/nhkhanh/.wm/models/bge-small-en-v1.5'. So `wm model download` cannot
make semantic search work, and `wm model list` never reports the model it just installed.

**2. The tokenizer URL 404s and the error body is written to disk as the model file.**
`packages/wm-embed/src/services/onnx/mod.rs:488` derives the tokenizer URL with
`entry.url.replace("model.onnx", "tokenizer.json")`. For bge-small the registry URL is
`https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/onnx/model.onnx`, so the derived URL becomes
`.../onnx/tokenizer.json`, which does not exist. Measured with the client's own user agent:
`.../onnx/tokenizer.json` returns 404 with a 15-byte body, while `.../tokenizer.json` returns 200 with 711,396
bytes. The downloader performs no status check, so it wrote those 15 bytes - the literal text 'Entry not
found' - into `<model_dir>/tokenizer.json` and reported success.

**Impact.** The installed model is unusable: the loader needs a real tokenizer, so
packages/wm-tools/tests/semantic_test.rs cannot pass, and any task requiring a green workspace suite is
blocked. The silent-success half is the same durable-write class as @wiki/rules/durable-write-or-error - the
tool reported a verified SHA-256 for the weights while installing a corrupt tokenizer beside them.