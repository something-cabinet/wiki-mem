---
{}
---

id: wiki:patterns:mcp-proxy-singleton

## Problem

Multiple processes each create their own `EngineState` when connecting to the wiki engine. This causes duplicate memory (~500MB each), stale data across instances, and no single source of truth. The MCP server and Web UI need to share one engine instance.

## Solution

The **MCP proxy singleton** pattern: before starting any service that needs the engine, check if the engine server is already running via a health endpoint. If alive, connect as a client. If dead, start the server, wait for health, then connect.

```
┌── Client starts ──┐
│                    │
│ GET /api/health    │──→ 200 → connect as client
│                    │
│ GET /api/health    │──→ error → spawn server → wait for 200 → connect
└────────────────────┘
```

### Implementation

```rust
fn ensure_server(base_url: &str) -> Result<()> {
    if check_health(base_url) {
        return Ok(());  // already running, just connect
    }
    // Spawn server as child process (or in-process thread)
    let child = std::process::Command::new("wm-server")
        .spawn()?;
    // Wait for health with timeout
    wait_for_health(base_url, Duration::from_secs(10))?;
    Ok(())
}

fn check_health(base_url: &str) -> bool {
    reqwest::blocking::get(format!("{}/api/health", base_url))
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
```

### Health endpoint

The server itself uses port binding to prevent duplicates:

```rust
async fn try_bind(port: u16) -> Result<TcpListener> {
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(listener) => Ok(listener),   // first instance
        Err(_) => {
            if check_health(port).await {
                std::process::exit(0);  // already running, graceful exit
            }
            bail!("Port {port} in use by non-wm-server process");
        }
    }
}
```

## When to Use

- Any process that needs to use the local engine server
- `wm-cli mcp` on startup (before starting rmcp server)
- CLI commands that need engine data
- Secondary `wm-server` invocations

## When Not to Use

- Commands that don't need the engine (`wm-cli init`, `wm-cli --version`, `wm-cli help`)
- CI/CD where a fresh engine instance per run is desired
- Remote connections (use the IP directly, skip health check)

## Related

- @doc/specs/single-http-server — Single HTTP Server spec
- docker CLI → dockerd pattern (same principle)
- PostgreSQL `pg_ctl` → `postgresd` pattern