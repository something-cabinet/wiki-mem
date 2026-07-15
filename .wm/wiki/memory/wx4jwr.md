---
title: Code intelligence via regex for Rust projects
type: memory
tags: [code, rust, regex, code-intelligence]
created_at: "2026-07-09T07:54:43.995Z"
updated_at: "2026-07-09T07:54:43.995Z"
---

For Rust projects without full AST tooling, regex-based code search works well. wm_code.search uses walkdir+regex for pattern search, wm_code.symbols parses pub fn/struct/enum/trait declarations with regex, wm_code.deps parses use statements. No external dependencies needed. Adequate for most code navigation needs without bringing in rust-analyzer or tree-sitter.