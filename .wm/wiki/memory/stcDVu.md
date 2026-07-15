---
title: Failure: reqwest::blocking panics inside tokio runtime
type: memory
tags: [rust, tokio, reqwest, ureq]
created_at: "2026-07-14T04:41:47.527Z"
updated_at: "2026-07-14T04:41:47.527Z"
---

reqwest::blocking::Client::new() panics when called inside #[tokio::main] because it creates/drops its own tokio runtime. Use ureq (pure blocking, no tokio dep) instead, or create via std::thread::spawn. Full reference: @doc/learnings/proxy-architecture-single-entrypoint