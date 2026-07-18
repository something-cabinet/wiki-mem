---
title: Add WM wiki support for Reasonix orchestrator
type: task
status: todo
---

**Severity:** Medium

**Observed:** The `reasonix-orchestrate` binary and `.reasonix/skills/` are installed, but there's no WM integration — no wiki pages document the orchestrator, no spec exists for the Reasonix integration, and the skills aren't registered as WM artefacts.

**What's needed:**
- Add `reasonix-orchestrate` to the wiki: create a concept page or ADR documenting the orchestrator, its skills, and installation flow
- Add a wiki task for installing/running `reasonix-orchestrate --init` as part of project setup
- Consider registering the 7 skills as WM skill entries (via `wm_skill.*` or the skill engine) so they appear in the wiki's skill index
- Update `WIKI-MEM.md` or `AGENTS.md` with a reference to the orchestrator if appropriate
- The ORCHESTRATOR.md and skill definitions should be referenced from WM docs so agents using WM tools can discover them

**Key question:** Should the WM MCP server expose the orchestrator skills in its tool registry, or keep them purely as Reasonix subagent skills?

**Acceptance Criteria:**
- [ ] Wiki page (concept or ADR) documents the orchestrator system and its 7 skills
- [ ] A wiki task exists for `reasonix-orchestrate --init` as a setup step
- [ ] WIKI-MEM.md or AGENTS.md references the orchestrator if appropriate
- [ ] Clear boundary between WM-managed tools and Reasonix subagent skills