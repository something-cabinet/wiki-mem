---
title: code_index_watcher_test fails on a fixed 20s deadline under parallel workspace load
type: task
id: "wiki:tasks:code-index-watcher-tests-have-a-load-dependent-deadline"
status: done
tags: [bugfix, test, flaky, watcher]
priority: high
acceptance_criteria:
  - {text: "code_index_watcher_test passes reliably under cargo test --workspace, not only in isolation", checked: false}
  - {text: "The wait no longer fails on elapsed wall-clock time alone; it either observes progress or scales the budget to actual load", checked: false}
  - {text: "Repeated workspace runs do not intermittently report watcher timeouts", checked: false}
implementation_notes: |-
  Fixed in the following commit. Root cause was not machine load: the poll opened the code-index DB read-write
  every 200ms, colliding with the watcher's 500ms debounce flush so refresh_code_index failed silently and lost
  the event. Reads are now gated on a .wm/state fingerprint and the watcher-waiting tests are serialized.

  Verified: 7/7 passed three times in a row (3.81s, 3.68s, 3.63s), versus failures at 20.51s in workspace runs.
  The budget is derived from the observed slowest step, clamped 20-45s, so a genuinely broken watcher still
  fails at the floor.
---

**Finding.** `apps/wm-core/tests/code_index_watcher_test.rs` waits with `poll_until` against a fixed 20-second
wall-clock deadline (`POLL_DEADLINE`), and the file panics at `:92` when it expires. Under
`cargo test --workspace` many watcher tests run concurrently, so the deadline expires for reasons unrelated
to the code under test.

**Observed 2026-10-19** twice: a workspace run failed `watcher_indexes_new_source_file` and
`skipped_dirs_never_reach_the_index` together, reporting `finished in 20.51s`, while the same file passes 7/7
in 1.23s when run alone. An earlier specialist independently saw 1 failure in 4 isolated runs.

**Why it matters.** A load-dependent deadline makes every green gate provisional and every red gate
ambiguous. It has already cost three gate runs in this session and once looked like a regression from the
LifecycleState migration, which it was not.

**Fix direction.** Poll on progress rather than wall-clock, or scale the deadline to the number of concurrent
test binaries, or serialize the watcher suites with a process-wide lock so their deadline is meaningful.