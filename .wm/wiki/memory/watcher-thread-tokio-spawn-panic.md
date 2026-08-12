---
title: 'Watcher thread panic: tokio::spawn from std thread'
type: memory
id: wiki:memory:watcher-thread-tokio-spawn-panic
status: active
tags: [failure, watcher, tokio, concurrency]
---

## Pattern

Calling `tokio::spawn` from a plain `std::thread` panics with "there is no reactor running, must be called from the context of a Tokio 1.x runtime" — and the panic kills that thread.

## Failure

`EngineState::notify_file_changed` (lsp feature) called `tokio::spawn` directly. The notify file watcher thread in `MainEngine::with_root` is a `std::thread::spawn` loop — so the FIRST external file change with `lsp` enabled panicked the watcher thread. The daemon (watcher-backed since Phase 1) would silently lose external-edit refresh. Unit tests that called `handle_file_change` from `tokio::test` contexts never hit it — only the real watcher test (`test_watcher_picks_up_disk_delete`, a delete after a create) caught it, because the create path updated the graph before the panic killed the thread, while the subsequent delete was never processed.

## Fix

Runtime-agnostic dispatch: `tokio::runtime::Handle::try_current()` → `tokio::spawn` when on a runtime; otherwise spawn a `std::thread` running a one-shot current-thread runtime that `block_on`s the notify.

## Lesson

- The watcher path was previously untested (file_watcher_test only called `handle_file_change` directly) — write watcher-level tests that exercise the real thread (MainEngine::with_root + direct disk write + deadline poll).
- `tokio::spawn` in code reachable from non-tokio threads (watchers, sync callers) needs the `try_current` guard.

