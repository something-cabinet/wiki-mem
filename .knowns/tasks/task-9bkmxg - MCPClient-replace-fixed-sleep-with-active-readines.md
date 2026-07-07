---
id: 9bkmxg
title: 'MCPClient: replace fixed sleep with active readiness polling'
status: done
priority: high
labels:
  - review-fix
  - test-infra
createdAt: '2026-07-07T08:50:58.670Z'
updatedAt: '2026-07-07T08:50:58.670Z'
timeSpent: 0
---
# MCPClient: replace fixed sleep with active readiness polling

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
P1 from rust-reviewer. MCPClient::start() used fixed 500ms sleep. Replaced with retry initialize() with 100ms backoff and 10s deadline. Removes flaky startup on slow CI runners.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

