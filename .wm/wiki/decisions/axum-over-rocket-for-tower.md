---
id: wiki:decisions:axum-over-rocket-for-tower
title: Decision: Axum over Rocket for Web UI Backend
type: decision
tags: [decision, web, architecture]
status: reviewed
relates_to:
  - {type: references, target: wiki:specs:single-http-server}
---
id: wiki:decisions:axum-over-rocket-for-tower

## Context

WM needed an HTTP server to serve the Angular web UI and REST API. Two candidates: Rocket.rs (simpler route syntax) and Axum (Tower middleware composability).

## Decision

**Axum**, despite more verbose route syntax. WM already uses tokio + Tower patterns via rmcp. Axum composes with the existing stack: middleware (auth, logging, CORS, rate limiting) can be shared between MCP and HTTP paths. Rocket has its own middleware system (fairings) that doesn't compose with Tower.

## Rationale

- WM's existing tokio + Tower + rmcp stack is Tower-based
- Axum middleware composes cleanly (CorsLayer, TraceLayer, etc.)
- Rocket would require adapting between fairings and Tower
- The simpler Rocket syntax wasn't worth breaking composability

## Consequences

- Web server integrated cleanly with existing EngineState via Arc
- Axum handlers call wm-core directly (no MCP bridge needed)
- Angular static files served via rust-embed
- Single binary deploy: `wm serve` starts everything

> **📌 Context update:** This Axum decision was originally made for `wm-cli web` (the embedded web server in the CLI tool). It now applies to `wm-server` (the new standalone daemon), with the same reasoning holding — Axum's Tower composability remains the right choice for the consolidated HTTP server. See [@wiki/specs/single-http-server](../specs/single-http-server.md).