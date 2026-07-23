---
title: Web Server Build & Serve
type: spec
status: superseded
tags: [web, build, server, infrastructure, justfile]
---

> **⚠️ SUPERSEDED** — This spec is superseded by [@wiki/specs/single-http-server](../specs/single-http-server.md).  
> The project has moved from `wm-cli web` (embedded web server in CLI) to a standalone `wm-server` daemon. The build-and-serve details here are no longer current.

## Overview

Upgrade wm-cli web to match the quality of Knowns' rowser command: embed the Angular web UI into the Rust binary for release builds, add auto-port-increment when the default port is busy, and orchestrate the build pipeline with a justfile.

## Locked Decisions

- **D1:** Embed Angular dist into the Rust binary via ust-embed at build time (CI/release builds only). Local development uses 
g serve + proxy.conf.json.
- **D2:** Adopt justfile to orchestrate the build pipeline: just build-web (ng build + cargo build), just serve (run wm-cli web), just dev (ng serve + wm-cli web in parallel).
- **D3:** Default port 3000, auto-increment if busy (3001, 3002... up to 10 tries).

## Requirements

### Functional Requirements

- **FR-1:** wm-cli web --port 3000 tries port 3000. If busy, tries 3001, 3002, up to +10. Prints the actual port used.
- **FR-2:** Release builds embed the Angular dist (pps/wm-web/dist/) into the binary via ust-embed. The embedded assets are served as fallback when no dist directory exists on disk.
- **FR-3:** Local dev mode (
g serve) proxies API calls to the Rust server via proxy.conf.json — no embed needed, no rebuild on frontend changes.
- **FR-4:** justfile at repo root with targets: uild-web (ng build then cargo build), dev (concurrent ng serve + cargo run -- web), serve (cargo run -- web --port 3000).
- **FR-5:** When dist is missing from disk AND no embedded version exists, wm-cli web prints a clear error: "Web UI not built. Run 'just build-web' or use 'ng serve' for development."

### Non-Functional Requirements

- **NFR-1:** No Node.js dependency for Rust compilation. 
g build is a separate step or CI-only.
- **NFR-2:** Binary size increase from embedded web assets: expected ~2-5MB for the Angular dist (compressed).
- **NFR-3:** Port increment prints a warning: "Port 3000 in use, trying 3001..."

## Acceptance Criteria

- [ ] **AC-1:** wm-cli web without --port uses 3000. If 3000 is busy, increments to first available port.
- [ ] **AC-2:** just build-web runs 
g build then cargo build --release. The resulting binary serves the web UI without any dist directory on disk.
- [ ] **AC-3:** just dev starts both 
g serve (port 4200) and wm-cli web (port 3000) concurrently. Opening http://localhost:4200 works with API proxied to :3000.
- [ ] **AC-4:** Without dist directory AND without embedded assets, wm-cli web prints a helpful error message and exits.
- [ ] **AC-5:** The justfile is at the repo root and is checked into git.

## Scenarios

### Scenario 1: Release Build
**Given** a CI machine with Node.js + Rust installed
**When** just build-web runs
**Then** 
g build produces pps/wm-web/dist/, then cargo build --release compiles WM with the dist embedded via ust-embed, then the CI publishes the single binary

### Scenario 2: User Downloads Release Binary
**Given** a user downloads the release binary
**When** they run wm-cli web --port 3000
**Then** the server starts on port 3000 with the web UI served from the embedded assets, no dist directory needed

### Scenario 3: Port 3000 Busy
**Given** port 3000 is already in use by another process
**When** the user runs wm-cli web
**Then** the server tries 3001, finds it free, starts on 3001 and prints "Port 3000 busy, using 3001"

### Scenario 4: Developer Workflow
**Given** a developer working on the frontend
**When** they run just dev
**Then** 
g serve starts on 4200 with hot reload, and wm-cli web starts on 3000 as the API backend. Changes to Angular code auto-reload without rebuilding Rust.

## Technical Notes

- Use ust-embed (already a dep in wm-core) to embed the dist directory.
- In wm-server, try to use embedded assets first via ust-embed, fall back to disk ServeDir. This way dev mode (disk) works without rebuilding.
- Port probing: bind to 127.0.0.1:{port}, if EADDRINUSE increment and retry, up to 10 attempts.
- Install just via cargo install just or the recommended platform-specific method.

## Open Questions

- [ ] Should just dev run both processes in the same terminal (via just --shell / backgrounding) or use a tool like concurrently / 	mux?
- [ ] For the embedded assets path in rust-embed: should it point to pps/wm-web/dist/browser/ or should the Angular build output be flattened?

## Related

- @wiki/concepts/specs/web-ui-polish-production-readiness — existing UI production readiness spec
- Knowns rowser command (v0.20.5) — reference implementation
