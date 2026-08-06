---
title: 'Pattern: notify + notify-debouncer-full for Rust File Watching'
type: pattern
id: wiki:patterns:rust-file-watcher-stack
relates_to:
  - {type: references, target: wiki:tasks:57bca4}
---
id: wiki:patterns:rust-file-watcher-stack

---
id: wiki:patterns:rust-file-watcher-stack
title: Pattern: notify + notify-debouncer-full for Rust File Watching
type: pattern
tags: [pattern, rust, file-watcher]
---
id: wiki:patterns:rust-file-watcher-stack

## Problem

Need to watch a directory for file changes in a Rust application. The standard library has no filesystem watcher, and raw platform APIs (inotify, kqueue, FSEvents) require platform-specific code.

## Solution

Use the `notify` crate (108M+ downloads, 3400+ stars) for cross-platform file events, paired with `notify-debouncer-full` for debouncing:

```toml
[dependencies]
notify = "8.2.0"
notify-debouncer-full = "0.7.0"
```

```rust
use notify_debouncer_full::{notify::*, new_debouncer, DebounceEventResult};
use std::time::Duration;

let (tx, rx) = std::sync::mpsc::channel();
let mut debouncer = new_debouncer(
    Duration::from_millis(500),
    None,
    tx,
)?;
debouncer.watch(&path, RecursiveMode::NonRecursive)?;

std::thread::spawn(move || {
    for result in rx {
        match result {
            Ok(events) => {
                for event in events {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => handle_change(path),
                        EventKind::Remove(_) => handle_delete(path),
                        _ => {}
                    }
                }
            }
            Err(errors) => { /* log */ }
        }
    }
});
```

## Why notify-debouncer-full

Text editors generate 3–5 raw events per save (temp file → rename → modify). Without debouncing you'd reprocess the same file multiple times. `notify-debouncer-full` deduplicates, tracks renames, and merges events within the configurable window.

## Cross-platform

| Platform | Backend |
|----------|---------|
| macOS | FSEvents |
| Linux | inotify |
| Windows | ReadDirectoryChangesW |
| All (fallback) | PollWatcher |

## Related
- @wiki/specs/graph-connectivity-fix
- @wiki/tasks/57bca4