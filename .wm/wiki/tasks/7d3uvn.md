---
title: CLI E2E Integration Tests
type: task
status: done
tags: [test, cli, integration]
priority: high
id: 7d3uvn
acceptance_criteria:
  - text: "wm-core/tests/cli_test.rs covers CLI smoke tests: init, create pages for all 7 types, search (keyword/semantic/hybrid), graph operations (neighbors/stats/path/subgraph), task board, time tracking, lint/validate, and index rebuild"
  - text: "All commands are exercised with the --json flag"
---

# CLI E2E Integration Tests

> *Imported from Knowns task `7d3uvn`*

# CLI E2E Integration Tests

## Description


Create wm-core/tests/cli_test.rs with CLI smoke tests: init project, create pages (all 7 types), search (keyword/semantic/hybrid), graph operations (neighbors/stats/path/subgraph), task board, time tracking, lint/validate, index rebuild. Test with --json flag on all commands.


## Acceptance Criteria
