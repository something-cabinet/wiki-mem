# Specialist Skills Reference

Subagent skills managed by `reasonix-orchestrate`. These are available as
convention-based subagent prompts within the WM workflow defined in `WIKI-MEM.md`.

| Skill | Lane | Read-only? |
|---|---|---|
| fixer | Bounded implementation | No |
| designer | UI/UX design and polish | No |
| architect | System design, trade-offs, ADRs | Yes (convention) |
| code-reviewer | General code review | Yes (convention) |
| rust-reviewer | Rust-specific review | Yes (convention) |
| database-reviewer | Schema, query, migration review | Yes (convention) |
| rust-build-resolver | Dependency and build issues | Yes (convention) |

Invoke via `run_skill(name="<skill>", arguments="<task>")`.

**Note:** These are convention-based. Reasonix does not enforce tool permissions
per skill — read-only is advisory. The built-in subagents (explore, research,
review, security-review) have actual read-only enforcement.
