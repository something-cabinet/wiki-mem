---
title: Fix run_cli() timeout — spawn + try_wait poll loop
type: task
status: done
tags: [review-fix, test-infra]
priority: high
id: rb1jdx
---

# Fix run_cli() timeout — spawn + try_wait poll loop

> *Imported from Knowns task `rb1jdx`*

# Fix run_cli() timeout — spawn + try_wait poll loop

## Description


P0 from rust-reviewer. run_cli() claimed 60s timeout but used Command::output() which blocks indefinitely. Replaced with spawn + try_wait() poll loop with 60s deadline and cross-platform kill_process().


## Acceptance Criteria
