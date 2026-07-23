---
title: Pattern: Async Tool Dispatch in MCP ToolRegistry
type: pattern
---

---
title: Async Tool Dispatch in MCP ToolRegistry
type: pattern
---

## Problem
MCP tool handlers are synchronous but LSP and other operations are inherently async.

## Solution
Add `register_typed_async` and `dispatch_async` to ToolRegistry alongside existing sync API. Store async handlers in separate HashMap. `dispatch_async` checks async handlers first, falls back to sync via `tokio::task::block_in_place`. Avoids converting 40+ existing synchronous handlers.

Key design:
```rust
pub type AsyncToolHandler = Arc<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send>> + Send + Sync>;

impl ToolRegistry {
    async_handlers: HashMap<String, AsyncToolHandler>,
    handlers: Vec<(String, ToolHandler)>,  // existing sync handlers

    fn register_typed_async<I, O, F, Fut>(&mut self, name, description, handler)
    async fn dispatch_async(&self, name, params) -> Result<Value, ToolError>
}
```

## When to Use
Adding async capabilities to a sync MCP tool system. Any operation needing network I/O, process management, or long-running computation.

## Related
- @task:srv-create-mcp-proxy-with-static-tool-list
