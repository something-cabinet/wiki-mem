---
id: wiki:patterns:e2e-mock-scenarios
title: Pattern: E2E Mock Scenarios for Isolated Testing
type: pattern
---
id: wiki:patterns:e2e-mock-scenarios

---
id: wiki:patterns:e2e-mock-scenarios
title: E2E Mock Scenarios for Isolated Testing
type: pattern
---
id: wiki:patterns:e2e-mock-scenarios

## Problem
E2E tests depend on real server data, making them flaky and hard to reproduce.

## Solution
Use a WireMock-compatible mock server with scenario switching. Mock server serves static JSON stubs from mappings/ directory. Admin API (`/__admin/scenarios/`) switches between data states at test runtime.

Key components:
- MockManager Helper with resetScenarios() and setupSessionFor(name)
- Scenario directories (empty/, error/) with override stubs
- resetScenarios() in _before() hook for isolation

## When to Use
Any E2E test depending on API data. Testing error/empty states in CI without real backend.

## Related
- @task:setup-codeceptjs-e2e-tests-following-gehenna-app-pattern
