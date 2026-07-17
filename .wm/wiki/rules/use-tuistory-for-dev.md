---
title: Always use tuistory for dev commands
type: rule
status: active
---

## Rule: Use tuistory for dev commands

Always use `tuistory` to wrap long-running dev server and Tauri app commands instead of directly spawning processes with Start-Process or running them in the foreground.

### Why
- tuistory creates named background sessions that agents can read, wait on, snapshot, and type into
- Prevents duplicate servers (tuistory reuses existing sessions)
- Allows agents to inspect dev server output with `tuistory read -s <session>`
- Avoids zombie processes from direct process spawning

### How
Wrap any long-running command with `tuistory --`:

```bash
# Tauri dev (Angular + Tauri with hot-reload)
cd apps/wm-web && npm run tauri

# Angular dev server only
cd apps/wm-web && npm start

# Direct Tauri binary (no hot-reload)
tuistory -- ./target/debug/wm-tauri.exe
```

To inspect output:
```bash
tuistory read -s <session-name>
tuistory -s <session-name> wait "/ready/i" --timeout 30000
```

### Exceptions
- CI/CD environments where tuistory is not available
- One-shot commands that complete quickly
