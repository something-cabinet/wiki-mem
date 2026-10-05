---
title: NC-01 Ban all comments + strip rustdoc
type: task
id: "wiki:tasks:nc-01-ban-all-comments--strip-rustdoc"
status: done
priority: high
tags: [from-spec, spec:ban-all-comments, policy, p1]
spec: specs/ban-all-comments
acceptance_criteria:
  - text: "Rule no-comments-in-code bans rustdoc; doc-comment-convention void"
  - text: "CONVENTIONS updated (no rustdoc exemption)"
  - text: "No comment lines remain in production .rs code"
  - text: "cargo check/clippy/tests clean"
---

Ban all comments including rustdoc: update rules + CONVENTIONS, then strip existing doc comments workspace-wide. Per specs/ban-all-comments. Depends on RW-01.