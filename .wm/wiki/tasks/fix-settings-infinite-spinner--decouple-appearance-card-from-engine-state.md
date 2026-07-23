---
title: Fix Settings infinite spinner + decouple Appearance card from engine state
type: task
status: todo
priority: high
tags: [bug, web-ui, settings]
---

From @designer review H1: settings-view.component.ts:142-145 — on !res.success neither state nor error is set → spinner renders forever. Also Appearance card is gated behind @if (state) — backend outage removes theme control. Fix: move Appearance out of state guard, set error on !res.success.