---
title: One home variable per platform, verified, no chain
type: memory
tags: [windows, paths, cross-platform]
status: active
---

HOME is ignored on Windows even when set; USERPROFILE is authoritative there, probing HOMEDRIVE+HOMEPATH only if it exists and is a directory. A preference chain makes the resolved location shell-dependent (Git Bash sets HOME to an MSYS-style relative-looking path). The "." fallback is banned outright — it would put the state dir in the CWD. Full reference: @wiki/decisions/one-authoritative-home-variable-per-platform