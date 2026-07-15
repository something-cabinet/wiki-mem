---
title: Global OnceLock for axum state workaround
type: memory
tags: [axum, rust, state]
created_at: "2026-07-14T04:41:47.530Z"
updated_at: "2026-07-14T04:41:47.530Z"
---

When axum's Router<S> type constraints fight you (into_make_service not found), use a global OnceLock<Arc<T>> for complex state instead of putting it in AppState. Only for single-instance, once-initialized state. Full reference: @doc/learnings/proxy-architecture-single-entrypoint