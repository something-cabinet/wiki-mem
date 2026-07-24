---
title: "MCPClient: replace fixed sleep with active readiness polling"
type: task
status: done
tags: [review-fix, test-infra]
priority: high
id: 9bkmxg
---

# MCPClient: replace fixed sleep with active readiness polling

> *Imported from Knowns task `9bkmxg`*

# MCPClient: replace fixed sleep with active readiness polling

## Description


P1 from rust-reviewer. MCPClient::start() used fixed 500ms sleep. Replaced with retry initialize() with 100ms backoff and 10s deadline. Removes flaky startup on slow CI runners.


## Acceptance Criteria
