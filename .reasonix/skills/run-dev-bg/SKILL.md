---
name: run-dev-bg
description: Run a long-lived dev command (e.g. tauri dev, ng serve) in background so the agent doesn't block. Use for anything that starts a watcher/server.
---

# run-dev-bg

Run a long-lived dev command in background so the agent doesn't get stuck.

## When to use

- `tauri dev` / `npm run tauri` — hot-reload dev server
- `ng serve` — Angular dev server
- `cargo watch` / any watcher
- Any server that runs until killed

## How it works

The tool `bash` has a `run_in_background` parameter. When `true`, it starts the
command in a detached process group, returns a job ID immediately, and the
agent continues its work. Use the other tools to monitor and clean up.

## Pattern

### Step 1 — Start

```bash
# Returns immediately with a job_id like "bash-1"
bash(command="npm run tauri", run_in_background=true)
# -> job_id: "bash-1"
```

Use absolute paths or `cd` inside the command if needed:

```bash
bash(command="cd apps/wm-web && npm run tauri", run_in_background=true)
```

### Step 2 — Read output (non-blocking)

```bash
# Returns all new output since last call
bash_output(job_id="bash-1")

# Filter for a specific line
bash_output(job_id="bash-1", filter="ready|compiled successfully|listening")
```

Keep calling `bash_output` to check progress — it never blocks.

### Step 3 — Wait for a condition (optionally block)

If you need to wait until the server is ready before interacting:

```bash
# Loop: read output until you see the ready message
output = bash_output(job_id="bash-1", filter="compiled successfully")
if "compiled successfully" in output: proceed
else: sleep a few seconds, retry
```

There is no `sleep` tool, but you can use `wait()` to approximate timing, or
just keep checking `bash_output` between your other work steps. The framework
will return control to you naturally on each turn.

### Step 4 — Interact while it runs

The dev server keeps running in background. You can:

- Run `tauri-pilot` commands against it
- Run tests against it
- Make API calls to it
- Do anything else — the background process is independent.

### Step 5 — Clean up

```bash
kill_shell(job_id="bash-1")
```

Kills the entire process group (dev server + all children + watchers). Always
do this when done, especially if the agent session may end.

### Step 6 — Block until done (only for finite commands)

If the command **will exit** (e.g. `cargo build`, a test run), you can block:

```bash
wait(job_ids=["bash-1"], timeout_seconds=120)
```

Do NOT use `wait` for dev servers — they never exit and the agent will hang.

## This project's specific dev commands

From `justfile`:

| Target | Command | Blocks? | Use background? |
|--------|---------|---------|-----------------|
| `tauri-watch` | `apps/wm-web; npm run tauri` | Yes (dev server) | Yes |
| `dev` | `cargo run -- web & ... ng serve` | Yes (ng serve) | Yes |
| `tauri-build` | `ng build + cargo build` | No (exits) | No, just `bash` |
| `tauri-run` | Start pre-built binary | Yes (binary runs) | Yes |
| `serve` | `cargo run -- web --port {{port}}` | Yes (server) | Yes |

## Notes

- `run_in_background` works with any command, not just dev servers
- The background job survives across agent turns — you start it, do other work,
  check on it, kill it later
- Multiple independent background jobs can run at once (each gets its own ID)
- If the agent crashes, the background process **will** be killed by the
  framework — so it won't leak forever
