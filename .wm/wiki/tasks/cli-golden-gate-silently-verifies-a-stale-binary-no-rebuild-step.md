---
title: CLI golden gate silently verifies a stale binary (no rebuild step)
type: task
id: "wiki:tasks:cli-golden-gate-silently-verifies-a-stale-binary-no-rebuild-step"
status: in-review
priority: high
tags: [wm-cli, test-infra, honesty, gate]
acceptance_criteria:
  - text: "The CLI test harness fails with an actionable message when target/debug/wm-cli is older than the newest file under apps/wm-cli/src"
  - text: "The guard is skipped when TEST_BINARY is set, since that is an explicit external override"
  - text: "The guard is demonstrated to fire (touch a source file, run the golden test, observe the failure) and to pass after cargo build -p wm-cli"
  - text: "cli_test (17), cli_characterization_test and the mcp helper tests all stay green, and no comments or #[allow] are introduced"
implementation_notes: |2-

  ## Fixed and independently verified (2026-10-15)

  `apps/wm-core/tests/helpers/cli.rs` now resolves the binary and asserts it is not older than the newest file under `apps/wm-cli/src`, panicking with `stale wm-cli binary: run \`cargo build -p wm-cli\` before the CLI tests`. Skipped when `TEST_BINARY` is set (explicit external override). No nested cargo invocation, so no package-cache lock contention.

  Verified by the reviewer directly, not by the lane report: touching a source file and running the golden test reproduces the panic message exactly; `cargo build -p wm-cli` then makes it green; `cli_test` stays 17/17; `rg` finds zero comments and zero `#[allow]`/`#[expect]`/`#[doc]` in the helper.

  Affects 9 test targets that include the shared helper (`cli_characterization_test`, `cli_test`, `e2e_task`, `e2e_workflow`, `e2e_memory`, `e2e_graph`, `e2e_pages`, `e2e_search`, `path_resolution_test`). CI is unaffected because every job already runs a `Build wm-cli` step before tests; local runs now fail loudly instead of silently passing against stale code.
---

**Bug description:** `apps/wm-core/tests/cli_characterization_test.rs` (the behaviour gate for the entire `wm-cli` rewrite) spawns `target/debug/wm-cli` by path. `cargo test -p wm-core` does not rebuild `wm-cli` because `wm-cli` is not a `wm-core` dependency, and neither does `cargo check` or `cargo clippy`. A green golden run therefore proves nothing about the current source.

**Evidence (measured):** `stat` on `target/debug/wm-cli` gives an identical mtime before and after a golden-only run. Observed live: the batch-1 lane's first run passed against a pre-batch binary and only failed to notice because it re-checked afterwards; the same hazard applies to any local verification of the remaining step-3 commits.

**Why it matters now:** 17 per-command commits remain, each of which decomposes arm bodies so `git diff` byte-identity no longer covers the change — the goldens are the *only* behaviour gate. A stale pass would ship a regression silently. It also means ad-hoc verification (including review-time spot checks) can produce a false green.

**Not a problem in CI:** `.github/workflows/ci.yml` runs a `Build wm-cli (needed by CLI integration tests)` step in every job before tests, so CI is sound. This is a local/verification-hygiene defect.

**Fix direction:** make the harness fail loudly rather than silently. In `apps/wm-core/tests/helpers/cli.rs`, when resolving the binary, compare its mtime against the newest mtime under `apps/wm-cli/src`; if the binary is older, fail with a message naming the exact remedy (`cargo build -p wm-cli`). Skip the check when `TEST_BINARY` is set, since that is an explicit override. A guard is preferred over having the test shell out to `cargo build`, which would nest cargo and contend on the package-cache lock.

**Related:** the `#[allow]`-ban precedent is likewise a deterministic check rather than a lint; and rule `durable-write-or-error` argues a green result must not be able to lie.