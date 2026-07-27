---
{}
relates_to:
  - {type: references, target: wiki:specs:http-wasm-architecture-cleanup}
---

id: wiki:patterns:engine-port-backend-abstraction

## Problem

Angular frontends typically call a backend API directly via `HttpClient` or `fetch`. This couples every consumer to HTTP, making component tests impossible without a running mock server and making future transport changes (WASM, WebSockets, IPC) a full rewrite.

## Solution

Define an `EnginePort` — an `InjectionToken<EnginePort>` interface that abstracts all backend communication. Consumers depend on the interface, not the implementation.

### Structure

```
services/
├── engine-port.ts            # Interface + InjectionToken + typed response types
├── http-engine.service.ts    # Production: fetch-based implementation
├── mock-engine.service.ts    # Test: canned responses
└── api.service.ts            # (optional) re-export shim for backward compat
```

### engine-port.ts

```typescript
export const ENGINE_PORT = new InjectionToken<EnginePort>('ENGINE_PORT');

export interface EnginePort {
  getData(): Observable<DataResponse>;
  search(query: string): Observable<SearchResult[]>;
  // ... all backend methods with typed return types (no `any`)
}
```

### Registration (app.config.ts)

```typescript
providers: [
  { provide: ENGINE_PORT, useClass: HttpEngineService },
  // Use MockEngineService in TestBed:
  // { provide: ENGINE_PORT, useClass: MockEngineService },
]
```

### Consumer injection

```typescript
constructor(@Inject(ENGINE_PORT) private api: EnginePort) {}
// or with inject():
private api = inject(ENGINE_PORT);
```

### MockEngineService

Returns typed canned responses (zero `as any` casts). Used in component tests without a running backend:
```typescript
@Injectable()
export class MockEngineService implements EnginePort {
  getData(): Observable<DataResponse> {
    return of({ items: [] });  // typed, no `any`
  }
}
```

### Benefits

- **Component-testable**: provide `MockEngineService` in TestBed, no HTTP server needed
- **Transport-swappable**: WASM engine, WebSocket, or IPC are just new implementations of the same interface
- **Typed responses**: consumers get autocomplete and compile-time type checking (eliminates `any`)
- **Auditable**: logging/debugging wrapper (`LoggingEnginePort`) can wrap any implementation

## When to Use

- Any Angular frontend with a backend dependency
- Project with component-level tests
- Multiple transport options (HTTP + WASM + mock)

## When Not to Use

- Tiny apps with a single view and no tests
- Backend is third-party and won't change (still useful for testability though)

## Related

- @wiki/tasks:engineport--mockengineservice--typed-angular-backend-abstraction
- @wiki/specs:http-wasm-architecture-cleanup
- `reference/design-patterns`
- @wiki/patterns/critical-patterns