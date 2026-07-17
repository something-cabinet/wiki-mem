# Agent Orchestration -- Specialist Lanes

## Implementation

### fixer
- Lane: Bounded implementation and execution
- Delegate when: Well-defined multi-file implementation, parallelizable work

### designer
- Lane: UI/UX design, polish, and implementation
- Delegate when: User-facing interfaces needing polish, visual consistency

## Architecture & Design

### architect
- Lane: System design, trade-off analysis, ADRs, scalability
- Delegate when: Architecture decisions, design reviews, scalability planning

## Code Review

### code-reviewer
- Lane: General code review across all languages
- Delegate when: Code quality, security, best practices review

### rust-reviewer
- Lane: Rust-specific review with clippy, safety, concurrency checks
- Delegate when: Rust code review with idiomatic patterns and performance

## Database

### database-reviewer
- Lane: Database schema, query, and migration review
- Delegate when: Reviewing SQL, schema changes, migrations

## Build & Tooling

### rust-build-resolver
- Lane: Rust dependency and build issue resolution
- Delegate when: Cargo build failures, dep conflicts, feature flag issues

## Built-in Subagents (no setup needed)
- explore: general codebase exploration (built-in Reasonix)
- research: web + code research (built-in Reasonix)
- review: code review (built-in Reasonix)
- security-review: security review (built-in Reasonix)

## Delegation Rules
1. Implementation work: fixer
2. UI/UX work: designer
3. Architecture decisions: architect
4. Code review: code-reviewer or rust-reviewer
5. Database review: database-reviewer
6. Build issues: rust-build-resolver
7. General exploration: use built-in explore/research

Note: Reasonix skills are convention-based, not enforced. The subagent receives the same tool set as the orchestrator. Read-only rules are advisory.
