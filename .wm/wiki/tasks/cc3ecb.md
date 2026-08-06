---
title: "WT: Add wm-cli page update subcommand"
id: cc3ecb
type: task
status: done
priority: high
tags: [spec:wiki-tool-reliability, cli]
relates_to:
  - {type: implements, target: wiki:specs:wiki-tool-reliability}
acceptance_criteria:
  - text: "PageAction::Update accepts stdin JSON with optional fields (title, content, status, tags, type) and mirrors MCP wm_page.update"
  - text: "Regression tests test_regression_create_no_doubled_wiki_dir and test_regression_content_flag_rejected pass, covering the doubled .wm/wiki path and flag-rejection fixes"
---

Add PageAction::Update variant to wm-cli main.rs. Accepts stdin JSON with optional fields: title, content, status, tags, type. Mirrors MCP wm_page.update tool.

## Implementation Notes

PageAction::Update (stdin JSON, mirrors MCP wm_page.update) already present at HEAD (5056592/373e86a5). Genuine fixes: page create/delete handlers were re-joining .wm/wiki onto the wiki dir from create_engine() → doubled path breaking index.md regeneration; fixed to bind (engine, wiki_dir) directly. Added test_regression_create_no_doubled_wiki_dir (RED first) + test_regression_content_flag_rejected.
