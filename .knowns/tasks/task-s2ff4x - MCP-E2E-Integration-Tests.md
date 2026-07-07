---
id: s2ff4x
title: MCP E2E Integration Tests
status: done
priority: high
labels:
  - test
  - mcp
  - integration
createdAt: '2026-07-06T17:40:16.384Z'
updatedAt: '2026-07-07T07:03:12.034Z'
timeSpent: 0
---
# MCP E2E Integration Tests

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create wm-core/tests/mcp_test.rs with JSON-RPC protocol tests: spawn wm serve, test initialize, tools/list (45+ tools returned), wm_initial, wm_search.query, error handling (invalid params, missing fields), performance (query latency, rebuild time). Follow Knowns pattern from tests/e2e_mcp_test.go.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

