---
title: CWR-02 home_dir()/expand_user_path() — one helper, per-platform authority
type: task
id: "wiki:tasks:cwr-02-home-resolution-single-helper"
status: todo
tags: [from-spec, spec:configurable-wm-root, paths, windows, cross-platform]
acceptance_criteria:
  - text: home_dir() is the only home resolution in the binary; the ten duplicated HOME/USERPROFILE copies are deleted, compiler-verified
  - text: 'Windows: USERPROFILE is authoritative and HOME is ignored even when set; HOMEDRIVE+HOMEPATH is probed only when the concatenation exists and is a directory'
  - text: All-unset returns a hard error naming platform, variable consulted and remediation; no resolution path falls back to "."
priority: high
---

schema_version: 1
state: |-
  Implements D12-D14 and AC-12/AC-13 of @wiki/specs/configurable-wm-root

  - Add `home_dir() -> Result<PathBuf, ConfigError>` and `expand_user_path()` beside `WM_DIR`/`WIKI_DIR`.
  - One authoritative variable per platform: Unix/macOS `HOME` only; Windows `USERPROFILE` only, probing `HOMEDRIVE`+`HOMEPATH` only when the concatenation exists and is a directory. `HOME` is **ignored on Windows even when set** — a preference chain makes the location shell-dependent (Git Bash sets `HOME`, PowerShell does not; an MSYS-style `/c/Users/...` value parses as relative).
  - Verification is part of resolution: the result must exist and be a directory.
  - `LOCALAPPDATA` is rejected as a home substitute.
  - Delete the ten duplicated `HOME`/`USERPROFILE` resolutions — `model_service.rs`, `agents_service.rs`, `logging_helper.rs`, `detect.rs`, `model.rs`, `main_engine_factory.rs`, `index.rs`, `manifest_store.rs`, plus one test. Delete, do not suppress (`no-dead-code-clone-scanning`).
  - The `""` fallback chain dies everywhere: no resolution path may fall back past the platform authority.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}