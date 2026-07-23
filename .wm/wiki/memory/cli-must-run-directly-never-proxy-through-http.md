---
title: CLI must run directly, never proxy through HTTP
type: memory
tags: [cli, architecture]
status: active
---

wm-cli commands must execute in-process via create_engine(), never proxy through HTTP to wm-server. CLI tests, offline operation, and latency depend on this. Full reference: @wiki/decisions/cli-direct-execution-not-http-proxy