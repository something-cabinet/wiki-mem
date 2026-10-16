---
title: 'Decision: One authoritative home variable per platform, verified'
type: decision
id: "wiki:decisions:one-authoritative-home-variable-per-platform"
status: approved
tags: [decision, windows, paths, cross-platform]
relates_to:
  - {type: implements, target: wiki:specs:configurable-wm-root}
---

schema_version: 1
state: |-
  ## Context

  `~/.wm` resolution was copy-pasted in ten places as `HOME` → `USERPROFILE` → `"."`. Three separate problems compound. A **chain makes the location shell-dependent**: Git Bash sets `HOME` while PowerShell does not, and on Windows `HOME` may hold an MSYS-style `/c/Users/...` value that `PathBuf` parses as *relative*. A **`.` fallback** quietly resolves to the current working directory. And the ten copies mean ten places a trust gate could be bypassed by accident.

  The `"."` fallback was survivable while it only meant "the wiki"; once a configured root can point *outside* the project, a silent `.` resolution means "the wiki is wherever the agent's CWD happened to be".

  ## Decision

  One authoritative variable per platform, and verification as part of resolution:

  | Platform | Authority | Behaviour |
  |---|---|---|
  | Unix / macOS | `HOME` only | unset, empty or unverifiable to hard error |
  | Windows | `USERPROFILE` only | if unset, probe `HOMEDRIVE`+`HOMEPATH` only when the concatenation exists and is a directory; else hard error |

  `HOME` is ignored on Windows **even when set**, and the resolved path must exist and be a directory. No resolution path falls back past the platform authority.

  ## Rationale

  Disagreement becomes unreachable rather than resolved, so determinism beats deference to the launching shell. For a store that must be found again tomorrow by a different process, an unrepeatable location is worse than any single wrong answer. `LOCALAPPDATA` is rejected as a home substitute — it is for app-local caches, not user knowledge, and the global scope already standardised on the profile directory.

  ## Consequences

  Ten duplicated resolutions collapse into one `home_dir() -> Result<PathBuf, ConfigError>`, making the most security- and correctness-critical path in the binary a single audit point with one test suite. **Windows currently ships in the release matrix (`ci.yml:210`) while every check and test job runs on `ubuntu-latest`, so these claims are unverified until a Windows fixture exists** — and until then the documentation must say so.

  ## Related

  - @wiki/specs/configurable-wm-root
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}