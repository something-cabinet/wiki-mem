---
title: "Wiki Tool Reliability: wm-cli page — no update command, --content flag breaks on multiline"
type: task
status: todo
tags: [bug, cli, tool-reliability]
relates_to:
  - {type: references, target: wiki:rules:tool-reliability-bug-tracking}
---

**Severity:** Medium

**Bug Description:** Two issues with wm-cli page operations:

1. **No `page update` command** — CLI has `get`, `list`, `create`, `delete`, `link`, `unlink` but no `update`. MCP has `wm_page.update` but there's no CLI equivalent.

2. **`--content` flag breaks on multiline input** — When content starts with `---` (YAML frontmatter), clap interprets the `---` as argument separators. This makes `wm-cli page create --content` unusable for wiki pages with frontmatter.

**Tool Name:** wm-cli page create / page update

**Full Input Parameters:**
```
wm-cli page create --page-type task "path" "Title" --content "---\ntitle: Test\n---\n\nBody"
```

**Full Error Output:**
```
error: unexpected argument '---' found
```

**Workaround:** Write `.md` files directly in `.wm/wiki/` using standard file tools, then rebuild index.

**Reproduction Steps:**
1. Run: `wm-cli page create --page-type task "test/foo" "Test" --content "---\ntitle: Test\ntype: task\n---\n\nBody"`
2. Observe: CLI fails with "unexpected argument '---'"

**Suggested Fix:** Either (a) support reading content from stdin, (b) support `--content-file` flag that reads from a file path, or (c) add `page update` subcommand. Fix the `--content` flag to not break on strings starting with `---`.
