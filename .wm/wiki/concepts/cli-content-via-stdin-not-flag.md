---
title: 'Failure: CLI Page Create Uses Stdin, Not --content Flag'
type: concept
id: wiki:concepts:cli-content-via-stdin-not-flag
relates_to:
  - {type: references, target: wiki:tasks:p0-wire-body-wiki-references-into-graph-builder}
---
id: wiki:concepts:cli-content-via-stdin-not-flag

---
id: wiki:concepts:cli-content-via-stdin-not-flag
title: Failure: CLI Page Create Uses Stdin, Not --content Flag
type: concept
tags: [failure, cli, testing]
---
id: wiki:concepts:cli-content-via-stdin-not-flag

## What went wrong

Several E2E tests used `run_cli(&root, &["page", "create", ..., "--content", "text"])`, passing content via a `--content` flag. The CLI `page create` command has **never** supported a `--content` flag — content is always read from stdin. These tests were written against the wm-server HTTP API endpoint (which accepts content as a JSON body field) and never updated when the CLI was refactored.

## Root cause

Tests were authored for the HTTP endpoint (`POST /api/pages/create`) which accepts content as a JSON field. When the CLI was refactored to proxy through HTTP, the tests passed content via `--content` — but this flag was never added to the CLI's argument parser. The tests appeared to pass because the HTTP endpoint silently ignored unknown fields.

## Prevention

- CLI tests must use `run_cli_with_stdin(dir, args, stdin_content)` to pass content via stdin
- Integration tests should be reviewed when the transport layer changes (CLI ↔ HTTP ↔ MCP)
- Never assume a test passing means it's testing the right thing — the `--content` tests were returning error messages that weren't being checked

## Time lost

~30 minutes debugging test failures after CLI was restored to direct execution mode.

## Related
- @wiki/decisions/cli-direct-execution-not-http-proxy
- @wiki/tasks/fbe6a0