---
title: 'Decision: Separate Service Ports over Monolithic EnginePort'
type: decision
id: wiki:decisions:separate-service-ports-over-monolithic-engineport
relates_to:
  - {type: references, target: wiki:specs:code-intel-search-ui}
---
id: wiki:decisions:separate-service-ports-over-monolithic-engineport

---
id: wiki:decisions:separate-service-ports-over-monolithic-engineport
title: Decision: Separate Service Ports over Monolithic EnginePort
type: decision
status: approved
tags: [decision, angular, architecture, services]
---
id: wiki:decisions:separate-service-ports-over-monolithic-engineport

## Context

The project has `EnginePort` as a single interface for all backend communication. As new feature domains (code intel search) were added, two approaches were considered:

**Option A:** Add code methods to the existing `EnginePort` interface and implement them in `HttpEngineService` and `MockEngineService`.

**Option B:** Create a separate `CodeIntelPort` with its own `InjectionToken`, `HttpCodeIntelService`, and `MockCodeIntelService`.

## Decision

**Choose Option B: Separate service ports.** Each distinct API domain gets its own port interface, injection token, and implementation pair.

## Rationale

- **Interface bloat.** `EnginePort` would grow unbounded as features are added — it already has 12 methods across search, pages, tasks, memory, graph.
- **Mock contamination.** Adding a method to `EnginePort` forces `MockEngineService` to implement it, even if those mocks aren't needed by code intel consumers.
- **Independent evolution.** Code intel and wiki pages have different API semantics, param shapes, and error modes.
- **Tree-shaking.** Components that only use code intel can inject `CodeIntelPort` without pulling in `EnginePort`.
- **Test isolation.** Code view tests only need `MockCodeIntelService`, not a full `MockEngineService`.

## Consequences

- More files per domain (port + HTTP impl + mock = 3 files).
- Consumers must inject the correct port.
- Pattern is now established; new domains should follow suit (e.g., `TaskPort`, `GraphPort`).

## Related
- @wiki/patterns:engine-port-backend-abstraction
- @wiki/specs:code-intel-search-ui