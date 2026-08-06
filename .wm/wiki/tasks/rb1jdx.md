---
title: Fix run_cli() timeout — spawn + try_wait poll loop
type: task
status: done
tags: [review-fix, test-infra]
priority: high
id: rb1jdx
acceptance_criteria:
  - text: "run_cli() enforces a 60s deadline via a spawn + try_wait() poll loop with cross-platform kill_process()"
  - text: "run_cli() no longer blocks indefinitely on Command::output()"
---

# Fix run_cli() timeout — spawn + try_wait poll loop

> *Imported from Knowns task `rb1jdx`*

# Fix run_cli() timeout — spawn + try_wait poll loop

## Description


P0 from rust-reviewer. run_cli() claimed 60s timeout but used Command::output() which blocks indefinitely. Replaced with spawn + try_wait() poll loop with 60s deadline and cross-platform kill_process().


## Acceptance Criteria
