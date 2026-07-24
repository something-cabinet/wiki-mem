---
title: MCP E2E Integration Tests
type: task
status: done
tags: [test, mcp, integration]
priority: high
id: s2ff4x
---

# MCP E2E Integration Tests

> *Imported from Knowns task `s2ff4x`*

# MCP E2E Integration Tests

## Description


Create wm-core/tests/mcp_test.rs with JSON-RPC protocol tests: spawn wm serve, test initialize, tools/list (45+ tools returned), wm_initial, wm_search.query, error handling (invalid params, missing fields), performance (query latency, rebuild time). Follow Knowns pattern from tests/e2e_mcp_test.go.


## Acceptance Criteria
