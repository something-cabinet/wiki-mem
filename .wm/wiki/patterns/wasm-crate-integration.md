---
id: wiki:patterns:wasm-crate-integration
title: "Pattern: WASM Crate Integration (fjadra profile)"
type: pattern
status: draft
tags: [pattern, wasm, angular, build, integration]
relates_to:
  - {type: references, target: wiki:tasks:wasm-graph-algorithms--client-side-graph-operations}
  - {type: references, target: wiki:tasks:wasm-bm25-re-scoring--client-side-search-re-ranking}
  - {type: references, target: wiki:tasks:wasm-markdown-parsing--client-side-wiki-content-rendering}
  - {type: example_of, target: wiki:patterns:canvas2d-wasm-graph}
  - {type: references, target: wiki:specs:http-wasm-architecture-cleanup}
  - {type: references, target: wiki:patterns:critical-patterns}
---
id: wiki:patterns:wasm-crate-integration

## Problem

Adding Rust/WASM computation to an Angular app requires boilerplate: crate setup, wasm-bindgen bindings, build pipeline, lazy-loading in the browser. Each new WASM crate reinvents this setup.

## Solution

A repeatable 5-step template called the **fjadra profile** — named after the first crate that established it.

### The fjadra profile checklist

A crate is a good WASM candidate ONLY if it meets all criteria:
- **No filesystem I/O** — no `std::fs`, no file paths
- **No tokio** — no async runtime needed
- **No rayon** — or rayon is feature-gated and optional (browser has no threads without SharedArrayBuffer + COOP/COEP headers)
- **No C deps** — no native C/C++ libraries (ONNX, SQLite, etc.)
- **Pure computation**: data in → computation → data out. No side effects.
- **Chatty enough to justify WASM**: if you call it once per page load, HTTP is fine. If you call it every frame or on every user interaction, WASM wins.

### Step-by-step template

#### Step 1: Create the crate

```
packages/<name>-wasm/
├── Cargo.toml      # cdylib + wasm-bindgen + serde
└── src/
    └── lib.rs      # #[wasm_bindgen] structs and functions
```

`Cargo.toml`:
```toml
[package]
name = "<name>-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# Only add deps that compile to wasm32-unknown-unknown
```

`src/lib.rs` pattern:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn my_function(input: &str) -> Result<String, JsValue> {
    // pure computation only
    Ok(serde_json::to_string(&result).unwrap())
}
```

#### Step 2: Register in workspace

Add to `Cargo.toml` workspace members.

#### Step 3: Build with wasm-pack

```bash
wasm-pack build packages/<name>-wasm --target web \
  --out-dir ../../apps/wm-web/src/assets/wasm/<name>
```

#### Step 4: Create Angular lazy-loading service

```typescript
@Injectable({ providedIn: 'root' })
export class MyWasmService {
  private wasm: any = null;
  private loaded = false;

  async load(): Promise<void> {
    if (this.loaded) return;
    const wasmModule = await import('../../assets/wasm/<name>/<name>_wasm.js');
    await wasmModule.default();
    this.wasm = wasmModule;
    this.loaded = true;
  }

  myFunction(input: MyType): Promise<OutputType> {
    if (!this.wasm) return fallback();
    return Promise.resolve(JSON.parse(this.wasm.my_function(JSON.stringify(input))));
  }
}
```

#### Step 5: Wire into component (consumer calls load())

```typescript
@Component(...)
export class MyComponent {
  private wasmService = inject(MyWasmService);

  async ngOnInit() {
    await this.wasmService.load();
    // now use wasmService methods
  }
}
```

### WASM cleanup discipline

When a WASM crate replaces an HTTP endpoint, delete the HTTP endpoint. Every WASM addition should shrink the HTTP surface. Evidence of violation: `layout.rs` SSE stub that persisted for months after fjadra-wasm replaced it.

## When to Use

- Adding a pure-compute Rust crate to an Angular app
- Computation is CPU-bound, fs-free, and called frequently (per-frame or per-interaction)
- You already have the fjadra-wasm build pipeline set up

## When Not to Use

- Crate needs filesystem, threads, async runtime, or native C deps
- Computation runs once per page load (HTTP is simpler)
- Target browser doesn't support WASM (none in practice)

## Related

- @wiki/patterns:canvas2d-wasm-graph — first implementation of this pattern
- @wiki/tasks:wasm-graph-algorithms--client-side-graph-operations
- @wiki/tasks:wasm-bm25-re-scoring--client-side-search-re-ranking
- @wiki/tasks:wasm-markdown-parsing--client-side-wiki-content-rendering
- @wiki/specs/http-wasm-architecture-cleanup
- @wiki/patterns/critical-patterns
