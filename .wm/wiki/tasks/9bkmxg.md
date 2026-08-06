---
title: "MCPClient: replace fixed sleep with active readiness polling"
type: task
status: done
tags: [review-fix, test-infra]
priority: high
id: 9bkmxg
acceptance_criteria:
  - text: "MCPClient::start() no longer uses a fixed 500ms sleep — readiness is polled via retry initialize() with 100ms backoff"
  - text: "Startup succeeds within a 10s deadline on slow CI runners without flaky failures"
---

# MCPClient: replace fixed sleep with active readiness polling

> *Imported from Knowns task `9bkmxg`*

# MCPClient: replace fixed sleep with active readiness polling

## Description


P1 from rust-reviewer. MCPClient::start() used fixed 500ms sleep. Replaced with retry initialize() with 100ms backoff and 10s deadline. Removes flaky startup on slow CI runners.


## Acceptance Criteria
