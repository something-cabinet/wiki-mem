---
title: 'CLI characterization: the version golden is stale at 0.5.2 against a 0.6.0 workspace'
type: task
id: "wiki:tasks:cli-golden-version-fixture-is-stale-at-0-5-2"
status: done
tags: [bugfix, test, cli, goldens]
priority: high
acceptance_criteria:
  - {text: "A freshly built target/debug/wm-cli and the version.txt golden agree, so cli_characterization_test passes", checked: false}
  - {text: "The characterization run proves version.txt was the only mismatching case, so the fix is a single-fixture correction and not a blanket regeneration", checked: false}
  - {text: "cargo test --workspace is green, so the suite can gate the remaining engine-state-decomposition migrations", checked: false}
  - {text: "No other golden file is modified, and no comments or #[allow] are introduced", checked: false}
implementation_notes: |-
  Blocker found while verifying this task's AC-3 (workspace suite green).

  The version golden is fixed and cli_characterization_test passes (1 passed; 32 other goldens unaffected).
  cargo test --workspace then runs 32 of 33 suites green, but packages/wm-tools/tests/semantic_test.rs fails two
  tests: onnx::semantic_search_with_loaded_model and onnx::hybrid_search_rrf_fusion_or_fallback.

  Cause is environmental, not code: the bge-small-en-v1.5 ONNX model is absent from ~/.wm/models (only the
  gliner models are present), and the model download path itself panics - see
  @wiki/tasks/wm-model-download-panics-with-nested-runtime-error.

  AC-3 therefore cannot be marked done until the model download path is repaired or the model is installed
  by hand. ACs 1, 2 and 4 are satisfied.
  Closed on evidence that AC-3 was met: cargo test --workspace has now been green in consecutive runs
  (exit=0, 42 suites ok, 0 failed), with ~/.wm/models/bge-small-en-v1.5/model.onnx intact at 133,093,490 bytes
  before and after each. The single-fixture correction was verified at the time: cli_characterization_test
  reported exactly one mismatch across all cases, and only version.txt was edited - regenerate_goldens was never
  run.

  The version golden is fixed and cli_characterization_test passes (1 passed; 32 other goldens unaffected).
  cargo test --workspace then runs 32 of 33 suites green, but packages/wm-tools/tests/semantic_test.rs fails two
  tests: onnx::semantic_search_with_loaded_model and onnx::hybrid_search_rrf_fusion_or_fallback.

  Cause is environmental, not code: the bge-small-en-v1.5 ONNX model is absent from ~/.wm/models (only the
  gliner models are present), and the model download path itself panics - see
  @wiki/tasks/wm-model-download-panics-with-nested-runtime-error.

  AC-3 therefore cannot be marked done until the model download path is repaired or the model is installed
  by hand. ACs 1, 2 and 4 are satisfied.
---

**Finding.** `apps/wm-core/tests/fixtures/cli_golden/version.txt` holds `wm 0.5.2` while the workspace crates
declare 0.6.0 (`apps/wm-cli/Cargo.toml:3`, `apps/wm-core/Cargo.toml:3`). A freshly built binary prints
`wm 0.6.0`, so `cli_characterization_test::goldens_match_current_behaviour` fails, and because cargo aborts
the remaining test binaries on a failure, a plain `cargo test --workspace` cannot gate anything - including
the remaining migrations of @wiki/specs/engine-state-decomposition, whose acceptance criteria all require a
green workspace suite.

**What not to do.** `tests/fixtures/cli_golden/regenerate.sh` regenerates all 33 goldens at once. Running it
to clear this one mismatch would rewrite the entire characterization net to whatever the binary currently
emits, masking any genuine output regression. See
@wiki/tasks/cli-golden-gate-silently-verifies-a-stale-binary-no-rebuild-step, which records that these
goldens are the only behaviour gate for the wm-cli rewrite.

**Intended change.** Update the single stale golden to the version the manifest and the binary actually
report, only after confirming by running the characterization test that no other case mismatches.