---
title: X-02 Windows CI fixture + unverified-claim honesty clause
type: task
id: "wiki:tasks:x-02-windows-ci-fixture-and-honesty-clause"
status: todo
tags: [from-spec, spec:configurable-wm-root, windows, ci, testing]
acceptance_criteria:
  - text: A Windows CI job runs the fixture covering USERPROFILE, HOME-ignored, HOMEDRIVE+HOMEPATH, all-unset, ~/ expansion, drive-relative rejection, UNC canonicalisation, mixed separators and case-differing prefixes
  - text: Until that job exists, the spec Limitations and the wm_root documentation state plainly that all Windows path claims are unverified and the feature is macOS/Linux-verified only
priority: medium
---

schema_version: 1
state: |-
  Implements AC-12 and AC-13 of @wiki/specs/configurable-wm-root

  - Today `ci.yml:210` builds `x86_64-pc-windows-msvc` on `windows-latest` in the release matrix, but every check/test job (`ci.yml:24,66,93,118,143,169`) runs on `ubuntu-latest`. A Windows binary ships whose path-resolution code has never run under test. The only two `#[cfg(windows)]` sites are test helpers.
  - Add a Windows fixture exercising, against the real APIs: `USERPROFILE` normal; `HOME` set but ignored on Windows (the cross-shell determinism test); `USERPROFILE` unset with a valid `HOMEDRIVE`+`HOMEPATH` probe; all-unset producing the hard-error payload shape; `~/` expansion; drive-relative `C:foo` rejection; UNC canonicalisation with `\\?\`-stripped confine comparison; mixed-separator config round-trip; and a case-differing prefix confine test.
  - Until this exists, the spec's *Limitations* section and the `wm_root` documentation must state that all Windows path claims are unverified and the feature is macOS/Linux-verified only. An unverified claim shipped as fact is the documentation-level form of the `durable-write-or-error` violation.
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