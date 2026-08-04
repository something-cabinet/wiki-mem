---
title: Security Remediation
type: spec
status: approved
tags: [security, path-traversal, confinement, spec]
---

## Overview

Remediate six findings from `docs/security-review-2026-08-04.md` (WM-000…WM-005). Five are confirmed vulnerabilities, four exploited end-to-end; two are Critical.

All six share one root cause: request-derived input reaches the filesystem without confinement, because `Path::starts_with` is component-wise and does not resolve `..`. WM-001…WM-004 live in `wm-core` and are reachable via `wm mcp`, so they are not fixable by changing `wm-server`.

Scope: eliminate unconfined filesystem access from request-derived input, make the guard structurally unforgeable, reduce the HTTP surface to read-only, and repair a test that currently certifies the vulnerability as correct behaviour.

Out of scope: dependency bumps, CI hardening, release/disclosure mechanics, persisted audit logging. See Spillover.

## Locked Decisions

- D1: Web UI is a read-only viewer. `/api/pages/create|update|delete` are removed, not gated. `/api/tools/{name}` admits read-only tools only.
- D2: External source ingestion requires the directory in `source_dirs`. `add_source` enforces the same allowlist `discover_sources` already applies. No new config field.
- D3: Rejected paths emit `tracing::warn!`. Persisted audit is a separate task.
- D4: Web token stored at `.wm/state/web-token` mode 0600 and injected into `index.html`. `/api/health` stays unauthenticated.

## Requirements

### Functional Requirements

- FR-1: A single confinement helper is the only sanctioned way to derive a filesystem path from request input. It rejects `..` escapes, absolute paths outside the root, and symlinks resolving outside the root.
- FR-2: The helper offers a strict mode that additionally rejects any path component beginning with `.`, so `.git/config`, `.env`, and `.wm/config.json` are unreadable even inside the root.
- FR-3: The helper validates without absolutising. Callers that currently store or return relative paths continue to do so.
- FR-4: `wm_model` `remove` validates `name` against the existing `MODEL_REGISTRY` allowlist before `remove_dir_all`.
- FR-5: `add_source` accepts a path only if it confines under a configured `source_dirs` entry and matches `source_extensions`, using strict mode.
- FR-6: `wm_page` and `wm_doc` create/update/delete resolve paths through the helper.
- FR-7: `wm_template` confines all four write actions (`add`, `addMany`, `modify`, `append`) and validates `destination`.
- FR-8: Request-derived path input is carried in a type that cannot reach the filesystem without passing through the helper.
- FR-9: `/api/tools/{name}` dispatches only names on an explicit read-only allowlist; all others return 404.
- FR-10: `/api/pages/create|update|delete` are removed from the router.
- FR-11: The API is same-origin. The permissive CORS layer is removed and cross-site requests are rejected.
- FR-12: Every `/api/*` route except `/api/health` requires the token from `.wm/state/web-token`.
- FR-13: Model download verifies SHA-256 against a pinned hash; an empty expected hash is an error.
- FR-14: Path rejections emit `tracing::warn!` including the attempted path.

### Non-Functional Requirements

- NFR-1: No regression. The full pre-existing suite passes, verified against a baseline captured before any change using the CI invocation.
- NFR-2: Zero compiler and clippy warnings across the workspace.
- NFR-3: Test-first. RED is a separate verifiable step before GREEN.
- NFR-4: New code complies with the no-else, no-magic-values, no-comments-in-code, and rust-anti-patterns rules.
- NFR-5: File placement follows CONVENTIONS — one primary type per file, role-based suffixes and subdirectories, `mod.rs` barrel re-exports.
- NFR-6: No new required config field. Any config addition is `#[serde(default)]`.
- NFR-7: The rejection path allocates no attacker-controlled string into persisted state.

## Acceptance Criteria

- [ ] AC-1: A baseline of `cargo build -p wm-cli -p wm-server && cargo test --workspace -- -q` is recorded before the first change, and any pre-existing failure is listed by name.
- [ ] AC-2: `{"action":"remove","name":"../../../victim"}` on `wm_model` returns an error and the target directory still exists.
- [ ] AC-3: `{"action":"remove","name":"bge-small-en-v1.5"}` still removes that model directory.
- [ ] AC-4: `wm_source add` with `/etc/hosts` returns an error.
- [ ] AC-5: `wm_source add` with `.git/config` returns an error.
- [ ] AC-6: `wm_source add` with a `.md` file under a configured `source_dirs` entry succeeds, and `process` returns its content.
- [ ] AC-7: `wm_page create` with `path: "../../../x"` returns an error and no file is written outside the wiki directory.
- [ ] AC-8: `wm_doc delete` with a traversing path returns an error and deletes nothing.
- [ ] AC-9: `wm_template run` with `variables.name = "../../x"` returns an error and writes nothing outside the root.
- [ ] AC-10: `wm_template run` with `destination: "../../.."` returns an error.
- [ ] AC-11: `wm_template run` with benign variables produces byte-identical output to pre-change.
- [ ] AC-12: `graph_meta_path_is_relative_to_project_root` still passes — `meta.path` is relative and starts with `.wm/wiki/`.
- [ ] AC-13: `cli_page_crud_from_wiki_root_resolves_meta_path` still passes — full create/get/update/link/unlink/delete lifecycle with `:`-separated IDs.
- [ ] AC-14: `POST /api/tools/wm_model` returns 404; an allowlisted read-only tool returns 200.
- [ ] AC-15: `POST /api/pages/create` returns 404.
- [ ] AC-16: A request with `Origin: https://evil.com` is rejected, and no response carries `access-control-allow-origin: *`.
- [ ] AC-17: `GET /api/pages/list` without a token returns 401; with the token from `.wm/state/web-token` returns 200.
- [ ] AC-18: `GET /api/health` returns 200 without a token.
- [ ] AC-19: `.wm/state/web-token` is mode 0600 and git-ignored.
- [ ] AC-20: Model download with an empty expected hash returns an error; a hash mismatch removes the partial file.
- [ ] AC-21: Reintroducing an unconfined `.join()` on request input fails to compile or fails CI.
- [ ] AC-22: `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings.
- [ ] AC-23: Every rejection emits a `tracing::warn!` containing the attempted path.
- [ ] AC-24: The rewritten traversal test asserts `Err` and fails against the pre-fix code.

## Scenarios

### Scenario 1: Happy path — normal page lifecycle unaffected

**Given** a project with `.wm/wiki/`
**When** an agent runs `wm_page create` with `path: "concepts/my-topic"`, then get, update, link, unlink, and delete
**Then** every operation succeeds, `meta.path` is `.wm/wiki/concepts/my-topic.md` and relative

### Scenario 2: Traversal via page path

**Given** the MCP server is running
**When** `wm_page create` is called with `path: "../../../fakehome/.evilrc"`
**Then** an error is returned, nothing is written outside `.wm/wiki/`, and a warning is logged

### Scenario 3: Traversal via template variable

**Given** a template whose action path is `docs/{{name}}.txt`
**When** it runs with `variables.name = "../../fakehome/pwn"`
**Then** an error naming the offending variable is returned and no file is created outside the root

### Scenario 4: Dotfile read attempt inside the root

**Given** `.git/config` exists inside the project root and contains a credential
**When** `wm_source add` is called with `.git/config`
**Then** an error is returned — root-confinement alone would have permitted this

### Scenario 5: Grandfathered source still processable

**Given** a source registered before this change whose `original_path` is outside `source_dirs`
**When** `wm_source process` is called on it
**Then** it succeeds, reading the copy at `stored_path` inside `.wm/sources/`

### Scenario 6: Opt-in external ingestion

**Given** `source_dirs` includes `/Users/me/notes`
**When** `wm_source add /Users/me/notes/thing.md` is called
**Then** it succeeds

### Scenario 7: Cross-origin browser attack

**Given** `wm web` is running and the user visits a hostile page
**When** that page issues `fetch('http://127.0.0.1:4090/api/tools/wm_search')`
**Then** the request is rejected and no response body is readable cross-origin

### Scenario 8: Local process without token

**Given** `wm web` is running
**When** `curl http://127.0.0.1:4090/api/pages/list` runs with no token header
**Then** 401 is returned

### Scenario 9: Server readiness probe

**Given** `wm web` is starting
**When** `wm-cli` polls `/api/health`
**Then** 200 is returned without a token and startup completes

### Scenario 10: Symlink escape

**Given** a symlink inside `.wm/wiki/` pointing outside the project root
**When** a write is attempted through it
**Then** an error is returned

## Technical Notes

- Helper location: `apps/wm-core/src/shared/helpers/path_confine_helper.rs`; newtype at `apps/wm-core/src/shared/models/user_path_model.rs`. Barrel re-exports required.
- `..` must be resolved lexically, not via `canonicalize`, because create-paths do not yet exist. Symlink checking canonicalizes the deepest existing ancestor.
- FR-3 exists because `path_resolution_test.rs:38-46` asserts `meta.path` is relative and starts with `.wm/wiki/`. Returning a canonical absolute path breaks it.
- `add_source:24` is the sole arbitrary-read primitive. `claim_source_and_read_content:122` and `verify_source:214` both read `stored_path`; `original_path` is never re-read. This is why Scenario 5 needs no migration.
- `model.rs` and `doc.rs` are named in the rust-anti-patterns rule section 4 for blocking I/O in async context. Convert touched `std::fs` calls to `tokio::fs`.
- Changing tool input types will silently rot subprocess tests that call tools by name string. Per critical-patterns, grep tool names across `apps/wm-core/tests/` and update fixtures in the same change. `test_tools_list` is the smoke check.
- Sanctioned test changes, not regressions: the traversal test at `page/mod.rs:107-131` is inverted to assert `Err` (AC-24); `wm_cli_web_test.rs` gains token handling for non-health routes; MCP fixtures update alongside FR-8.
- Nine broken lexical guards to replace: `page_path_helper.rs:16`, `:35` (unguarded), `doc.rs:233`, `:293`, `:341`, `:385`, plus template sites `:346`, `:385`, `:452`, `:491` and `destination` at `:290`.
- `/api/health` must stay unauthenticated: `wm-cli/src/main.rs:867` uses it as a readiness gate and `wm_cli_web_test.rs` asserts 200 on it at `:166`, `:267`, `:271`, `:324`.

## Open Questions

- [ ] Is ARCHITECTURE accurate that the HTTP daemon is the "primary deployment target" supporting "remote access"? It contradicts the stated assumption that `wm web` is local-only, and determines whether FR-11/FR-12 are sufficient or need TLS and real auth.
- [ ] Does the findings-first rule require six separate spec pages, or is one umbrella spec plus six tasks acceptable given the shared root cause?
- [ ] Should previously published npm versions be deprecated? Depends on install counts not visible here.
- [ ] Should `/api/index/rebuild` remain reachable under D1? It mutates no user data but triggers heavy work, so it is a denial-of-service lever.