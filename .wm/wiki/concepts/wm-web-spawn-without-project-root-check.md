---
title: 'Failure: wm-cli web spawns wm-server without project-root check'
id: wiki:concepts:wm-web-spawn-without-project-root-check
type: concept
relates_to:
  - {type: references, target: wiki:tasks:bundle-angular-frontend-with-wm-server-for-npm-distribution}
---

---
title: Failure: wm-cli web spawns wm-server without project-root check
type: concept
id: wiki:concepts:wm-web-spawn-without-project-root-check
tags: [failure, cli, wm-server, process-spawn]
---

# Failure: Spawning wm-server without project-root detection causes silent crash

## What went wrong
`wm-cli web` spawned `wm-server` and printed "Starting wm-server on port 4090..." — then nothing. The user saw a hanging terminal with no indication the server had crashed. From a directory without a `.wm/` project, wm-server always exits immediately, but the error (`No .wm directory found`) only appears on the child's stderr, which the user doesn't see before the parent waits.

## Root cause
`wm-cli web` passed `--port` and spawned the server binary without first resolving the project root. `wm-server` calls `detect_project_root()` from its inherited CWD, fails when no `.wm/` project exists, and exits non-zero. The parent process's `child.wait()` swallows the failure silently.

## Prevention
- Validate prerequisites in the launcher BEFORE spawning: `wm-cli web` now calls `detect_project_root()` first and prints a clear message (`No wiki-mem project found. Run 'wm init'...`) instead of spawning a doomed child.
- Set the child's `current_dir()` to the resolved project root so the server always finds it regardless of where the user invoked the command.
- For spawned daemons, surface child stderr to the parent's error path rather than relying on inherited stdio.

## Time lost
~20-30 min of confused debugging across the session (server "never started" from npm install, which masked this as a packaging bug).

## Related
- @wiki/tasks/bundle-angular-frontend-with-wm-server-for-npm-distribution
- @wiki/memory/wm-cli-web-must-bundle-wm-server-in-npm-package