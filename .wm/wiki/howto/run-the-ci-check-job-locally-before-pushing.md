---
title: 'Howto: run the CI check job locally before pushing'
type: howto
id: "wiki:howto:run-the-ci-check-job-locally-before-pushing"
status: reviewed
tags: [howto, ci, release, gates]
relates_to:
  - {type: references, target: wiki:concepts:failure-pushed-a-release-without-running-the-ci-check-gates}
  - {type: relates_to, target: wiki:howto:check-ci-and-npm-status}
---

schema_version: 1
state: |-
  ## When

  Before every push to master and before moving or pushing a release tag. The CI check job runs more than the test suites, and a red check job blocks publish and publish-npm.

  ## Steps

  Run from the repository root. Use a separate target directory when the repo target holds artifacts from another rustc.

  ```bash
  cargo build -p wm-cli --target-dir /tmp/wm-test-target
  find apps/wm-cli/src -name '*.rs' -exec wc -l {} + | awk '$1>300 && $2!="total"'
  wc -l apps/wm-cli/src/main.rs
  bash scripts/check-crate-sizes.sh
  bash scripts/check-dependency-layers.sh
  bash scripts/check-singleton-helpers.sh
  OMP_NUM_THREADS=1 cargo test --no-default-features --features "code-intel,lsp" -p wm-core --lib -p wm-search -p wm-engine -p wm-constants -p wm-code-intel --target-dir /tmp/wm-test-target -- -q
  cargo test --workspace --no-fail-fast --target-dir /tmp/wm-test-target
  cargo audit
  ```

  The first awk line must print nothing. main.rs must stay at 80 lines or fewer. The three scripts print one pass line each.

  ## Gotchas

  - Formatting a wm-cli source file updates its modification time. The CLI tests then stop with "stale wm-cli binary". Rebuild wm-cli and run them again.
  - Lint with cargo clippy --no-deps -p <crate> -- -D warnings. Without --no-deps, an existing warning in a dependency crate hides your own.
  - Fish has no heredocs and no var=value assignment. Write scripts to a file and run them, and use set name value.
  - cargo audit reads the live advisory database, so it can fail on a day when nothing in the repo changed.

  ## Verification

  Every command exits 0 and the gate scripts report passed. Then push, and read the run with gh run list and gh run view.
questions:
  - id: task_kind
    type: choice
    instructions: What kind of task does this howto cover?
    options:
    - setup
    - development
    - testing
    - release
    - debugging
    - operations
    - integration
  - id: prerequisites_complete
    type: noul
    instructions: All prerequisites for this howto are fully documented.
  - id: difficulty
    type: score
    instructions: What skill level does this howto require?
    levels:
    - beginner
    - intermediate
    - advanced
    - expert
  - id: has_verification
    type: noul
    instructions: This howto includes a verification step.
answers:
  task_kind: release
  prerequisites_complete: true
  difficulty: intermediate
  has_verification: true