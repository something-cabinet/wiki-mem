---
title: Resolve three-way deployment architecture contradiction (Tauri vs wm-server)
type: task
status: todo
priority: urgent
tags: [architecture, decision, urgent]
---

From @oracle design review S-1: Three documents contradict each other on deployment model — enterprise-grade.md says "Tauri v2 primary, all-in", an approved ADR (wm-server-overrides-tauri-primary) says "wm-server overrides Tauri", and ARCHITECTURE-SPEC.md describes a phase-gated migration. No wm-server crate exists on disk. Angular is hard-wired to Tauri IPC. Pick one, close the contradictory docs, align the ADR.