---
title: MCP E2E Integration Tests
type: task
status: done
tags: [test, mcp, integration]
priority: high
id: s2ff4x
acceptance_criteria:
  - text: "wm-core/tests/mcp_test.rs spawns wm serve and passes JSON-RPC tests for initialize, tools/list (45+ tools returned), wm_initial, and wm_search.query"
  - text: "Error handling is covered for invalid params and missing fields"
  - text: "Performance is measured for query latency and index rebuild time"
---

schema_version: 1
state: |-
  # MCP E2E Integration Tests

  > *Imported from Knowns task `s2ff4x`*

  # MCP E2E Integration Tests

  ## Description


  Create wm-core/tests/mcp_test.rs with JSON-RPC protocol tests: spawn wm serve, test initialize, tools/list (45+ tools returned), wm_initial, wm_search.query, error handling (invalid params, missing fields), performance (query latency, rebuild time). Follow Knowns pattern from tests/e2e_mcp_test.go.


  ## Acceptance Criteria
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
