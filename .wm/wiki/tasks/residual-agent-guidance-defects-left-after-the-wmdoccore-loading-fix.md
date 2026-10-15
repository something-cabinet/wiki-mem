---
title: Residual agent-guidance defects left after the wm_doc/core-loading fix
type: task
id: "wiki:tasks:residual-agent-guidance-defects-left-after-the-wmdoccore-loading-fix"
status: todo
priority: medium
tags: [tool-reliability, docs, guidance]
acceptance_criteria:
  - text: "wm-template embedded and .opencode copies reconciled against the actual wm_template surface (run action removed or reinstated)"
  - text: "Skill tool-call syntax either matches the real MCP tool+action shape or an explicit note explains the shorthand"
  - text: "wm init scaffolds .wm/wiki/core/ and .wm/wiki/rules/ (or WIKI-MEM.md stops requiring them unconditionally)"
  - text: "typed-decision-record-schema reference body matches is_record_bearing and the migrated wiki (exclusions section corrected, status promoted or marked superseded)"
---

Findings from the wm_doc/core-doc bootstrap fix (task `wiki:tasks:wiki-tool-reliability-wmdocget-core-doc-bootstrap-guidance--id-vs-path-plus-core-session-loading`). All are guidance/metadata drift, not code defects. None blocked that fix.

1. **`wm-template` skill copies have diverged.** `.opencode/skills/wm-template/SKILL.md` documents `wm_template.run` + `dryRun`, but task `wiki:tasks:d41ec7` removed `wm_template.run` and the template engine. The embedded copy still lists only list/get/create. Two files, two different stale stories; the `.opencode` copy instructs a call that no longer exists.

2. **Dotted pseudo-syntax in skills does not match the real tool surface.** Skills write `wm_search.query({...})`, `wm_page.get({...})`, `wm_task.board()`, but the MCP surface is `wm_search_query` / `wm_page` / `wm_task` taking an `action` field (see `wm_help`). Agents copy the pseudo-calls literally and get schema errors.

3. **`wm init` scaffolds neither `.wm/wiki/core/` nor `.wm/wiki/rules/`.** A fresh `wm init --no-wizard` creates only concepts/decisions/howto/memory/patterns/reference/specs/tasks. WIKI-MEM.md and `.wm/AGENTS.md` now instruct agents to read every file under `core/` and `rules/`, so every fresh project starts with a missing-doc fallback. Either scaffold both dirs (with a seed `core/README.md`) or soften the instruction.

4. **`@wiki/reference/typed-decision-record-schema` is stale and self-contradicting.** It is `status: draft` and its TD-02 exclusion list says `rule`/`core`/`memory`/`task`/`spec` must NOT get record bodies, but `wm_engine::decision::canonical_questions::is_record_bearing` returns `true` for **every** page type and the live wiki was migrated accordingly (commit d0e8347). The spec also names it the normative contract.

5. **`wm_doc.delete` still does not exist as such** — the `wm_doc` adapter maps delete to `wm_page` delete by `path`; skills previously advertised `id`. Now fixed, but the alias/path-vs-id split remains an easy trap; consider documenting the split in `wm_help` docs output rather than only in markdown.