---
title: P1-P2 Pre-work: Fix 3 contradictory architecture docs
type: task
status: todo
priority: high
tags: [spec:wiki-tool-reliability, spec:wm-server, docs]
---

Fix 3 contradictory docs that conflict with wm-server architecture:
1. wiki:conventions:enterprise-grade — rewrite D1 from "Tauri primary" → superseded by single-http-server
2. wiki:specs:web-server-build-serve — mark superseded, point to single-http-server
3. wiki:decisions:axum-over-rocket-for-tower — annotate as now applying to wm-server