---
title: Establish ownership of a broken build before fixing or reverting
type: memory
tags: [workflow, orchestration, build]
status: active
---

When `cargo check`/`cargo test` fails mid-session in a repo with more than one active workstream, prove the breakage is yours before acting: `git ls-files --error-unmatch <file>` to see if the failing file is untracked/new, check whether the errors point at files you never touched, and confirm the suite passed earlier on the same tree. In this session the crate was broken by a concurrent main.rs rewrite (`apps/wm-cli/src/services/` and `shared/` were new and untracked); the right move was to revert our own two Rust touchpoints rather than fix another workstream's file or share its build target.