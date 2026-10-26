---
title: Windows release build fails on a unix-only Command import
type: task
id: "wiki:tasks:windows-release-build-fails-on-a-unix-only-command-import"
status: done
priority: high
tags: [ci, windows, release, build]
acceptance_criteria:
  - text: "The Command import is cfg(unix)-gated"
  - text: "Linux/macOS clippy and lock_error_helper tests stay green"
  - text: "Master CI is green"
  - text: "The v0.6.1 release build passes on all four targets"
relates_to:
  - {type: implements, target: wiki:specs:windows-release-import-fix}
implementation_notes: 'Fixed in 091b3af: import gated with #[cfg(unix)]. v0.6.1 tag moved to 091b3af (no GitHub Release or published artifact existed); tag run 40270884474 is fully green including publish (x86_64-pc-windows-msvc), and master run 40270875849 is green.'
---

The v0.6.1 release run (39890622851) failed only on x86_64-pc-windows-msvc: `unused import: std::process::Command` in packages/wm-code-intel/src/helpers/lock_error_helper.rs. The import is used only by the #[cfg(unix)] lsof/ps helpers; Windows compiles with -D warnings, so unused_imports fails the build.