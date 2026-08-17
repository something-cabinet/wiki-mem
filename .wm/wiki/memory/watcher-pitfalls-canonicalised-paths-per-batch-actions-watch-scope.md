---
title: Watcher pitfalls — canonicalised paths, per-batch actions, watch scope
type: memory
tags: [watcher, notify, macos, code-intel, performance]
status: active
---

Two traps when extending the notify watcher in `engine/main_engine_factory.rs`, both hit on 2026-08-17.

1. Never prefix-match event paths against a configured directory. macOS delivers events under `/private/var/...` for a `/var/...` temp root, so `path.starts_with(wiki_dir)` silently matched nothing and both live watcher tests in `file_watcher_test.rs` timed out. The original code was immune because it filtered on extension alone. Canonicalising the event path is not an option — `Remove` events name paths that no longer exist. Fix: canonicalise the watched directory once at setup and accept either prefix.

2. Act once per debounced batch, not once per event. The handler receives `Vec<DebouncedEvent>`; calling a rebuild inside the per-event loop ran it once per changed file (50 files → 50 rebuilds, 49 no-ops that each still walked and hashed every source file). Fix: set a flag in the loop, act after it.

Also load-bearing: watch specific top-level source directories, never the project root recursively. Recursive root registration puts `target/` and `node_modules/` under OS-level watch, and a cargo build then floods the channel. Filtering after receipt does not help — the events still arrive. Belt-and-braces: also filter every path component against `is_skipped_dir`, because nested `node_modules/` under a watched root will still deliver events.

And the standing hazard from wiki:core:critical-patterns still applies: no bare `tokio::spawn` on that std thread.