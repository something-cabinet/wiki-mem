---
title: fjadra force-directed layout — implemented, works via server
type: task
status: done
spec: specs/webgl-graph-rendering
---

The Rust force-directed layout (fjadra) is spec'd but not implemented. Currently using d3-force for all graphs.

Action: Add fjadra to wm-tauri's Cargo.toml and create the \start_layout\ IPC command.

References: @wiki/specs/webgl-graph-rendering.md, @wiki/notes/session-handover-2026-07-17.md