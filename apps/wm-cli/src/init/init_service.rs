use std::path::Path;

use crate::init::InitArgs;
use crate::shared::services::{setup_platform_mcp, sync_agent_files};
use crate::shared::traits::Command;
use tracing::info;
use wm_constants::{CONFIG_FILE, SOURCES_DIR, STATE_DIR, WIKI_DIR, WM_DIR};
use wm_core::config::ProjectConfig;

use super::init_wizard_helper::{resolve_platforms, run_wizard};

const AGENTS_MD_TEMPLATE: &str = r#"# AGENTS.md — Wiki Memory Engine Agent Handbook

## Wiki Conventions

### 10 Page Types
| Type | Directory | Purpose |
|------|-----------|---------|
| task | `wiki/tasks/` | Actionable units of work with acceptance criteria |
| spec | `wiki/specs/` | Functional/non-functional requirements, goals |
| concept | `wiki/concepts/` | Domain concepts, terminology, architecture |
| pattern | `wiki/patterns/` | Reusable solutions, when-to-use, examples |
| decision | `wiki/decisions/` | ADRs: context, options, rationale, outcome |
| howto | `wiki/howto/` | Step-by-step guides, tutorials |
| reference | `wiki/reference/` | API docs, error codes, configuration tables |
| core | `wiki/core/` | Meta-project docs defining how the project works |
| rule | `wiki/rules/` | Enforceable project rules and invariants |
| memory | `wiki/memory/` | Durable knowledge entries (short summaries with links to full docs) |

### Frontmatter Schema
Every wiki page starts with YAML frontmatter:
```yaml
---
title: Page Title
type: task|spec|concept|pattern|decision|howto|reference|core|rule
status: todo|in-progress|done|draft|reviewed|approved|active
tags: [tag1, tag2]
---
```

### Record Body
Every page type is record-bearing: the body after the frontmatter is a typed-decision record, not free prose.
```yaml
schema_version: 1
state: |-
  (the page prose, headings preserved, indented two spaces)
questions:
  - id: <canonical-question-id>
    type: choice
    instructions: <one-line prompt>
    options: [<label>, <label>]
answers: {}
```
`state` holds the prose; `questions` must equal the canonical set for the page type; `answers` keys are a subset (may be empty). Full contract: `@wiki/reference/typed-decision-record-schema`. Convert a prose page with `wm page migrate-records --only <path>`.

## Session Start

Read these before doing any work:
1. **Core docs** — every file under `.wm/wiki/core/` (`core/README`, `core/ARCHITECTURE`, `core/CONVENTIONS`, `core/critical-patterns`)
2. **Rules** — every active page under `.wm/wiki/rules/` is binding
3. **Memory** — `wm_memory.list({"layer": "project"})` and `wm_memory.list({"layer": "global"})`

`wm_page.get` takes `id` (`wiki:core:README`). `wm_doc.get` takes a wiki-relative `path` (`core/README`) and has no `id` parameter.

## Workflow Instructions

Always follow this sequence for every request:
1. **Search** — Gather relevant context using `wm_search.query`, `wm_search.retrieve`, or `wm_graph.neighbors`
2. **Gather context** — Read full pages with `wm_page.get`; retrieve context packs with `wm_search.retrieve`
3. **Plan** — Create or update task pages with `wm_page.create` / `wm_page.update`; define acceptance criteria
4. **Implement** — Execute the plan; update pages as needed; link related pages with `wm_page.link`

## Tool Usage Rules

1. **Prefix**: All tools use the `wm_` prefix (e.g., `wm_search.query`, `wm_page.get`)
2. **Initial call**: Always call `wm_initial` first to get project state, graph stats, and available search modes
3. **Search before act**: Search the wiki before creating or modifying pages to avoid duplication
4. **Use JSON output**: Prefer JSON mode (`json=true`) for structured responses in automated workflows
5. **Reads by id**: `wm_page.get` takes `id`; `wm_doc.get` takes a wiki-relative `path` only (never `id`)

## Canonical Workflows

### 1. wm-init — Session Initialization
- Trigger: Start of new session
- Steps: Call `wm_initial` → List docs → Read every `core/*` doc → Load all `rules/*` → Check tasks/board → Load memory → Summarize

### 2. wm-research — Project Research
- Trigger: Need to understand context
- Steps: Search (`wm_search.query`) → Read pages (`wm_page.get`) → Graph traversal (`wm_graph.neighbors`)
- Cross-entity search across pages + memory

### 3. wm-plan — Task Planning
- Trigger: Task assigned
- Steps: Search wiki for related specs → Plan with ACs → Validate → Wait for approval
- Supports `--from @wiki/specs/<name>` for spec-wide task generation

### 4. wm-implement — Code & Documentation
- Trigger: Plan approved
- Steps: Follow plan → Check ACs → Validate → Run SDD verification if spec-linked
- Tracks progress with `wm_time.start/stop`

### 5. wm-review — Code Review
- Trigger: Implementation complete
- Steps: Multi-perspective review → Severity findings (P0/P1/P2/P3) → Fix P1
- Reviews real diff for correctness, clarity, and consistency

### 6. wm-commit — Verification & Commit
- Trigger: Review passed
- Steps: Validate (`wm_validate.check`) → Lint (`wm_lint.check`) → Commit with conventional format
- Asks user before committing

### 7. wm-extract — Knowledge Extraction
- Trigger: Pattern discovered
- Steps: Review source → Check for duplicates → Save memory/learning → Promote to critical
- Saves what cost time to learn

### 8. wm-flow — Spec/Task Wave Orchestrator
- Trigger: Approved spec with multiple tasks
- Steps: Task discovery → Parallel gate → Implementation loop → Review → Verify
- Spawns sub-agents for parallel-safe work
"#;

const WIKI_SUBDIRS: [&str; 8] = [
    "tasks",
    "specs",
    "concepts",
    "patterns",
    "decisions",
    "howto",
    "reference",
    "memory",
];
const AGENT_DIR: &str = ".agent";
const SKILLS_DIR: &str = "skills";
const AGENTS_MD_FILE: &str = "AGENTS.md";
const WIKI_MEM_SHIM: &str = "shims/WIKI-MEM.md";
const WIKI_MEM_FILE: &str = "WIKI-MEM.md";
const MCP_PLATFORMS: [&str; 6] = [
    "opencode", "kiro", "claude", "codex", "cursor", "antigravity",
];

impl Command for InitArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_init(self)
    }
}

fn run_init(args: InitArgs) -> Result<(), anyhow::Error> {
    let root = args.project.unwrap_or_else(|| std::env::current_dir().unwrap());
    scaffold_project(&root)?;
    info!("Initialized project at {}", root.display());
    println!("Wiki Memory Engine initialized at {}", root.display());
    if needs_wizard(args.no_wizard) {
        run_wizard(&root);
    }
    let platforms = resolve_platforms(args.platform, args.no_wizard);
    configure_platforms(&root, &platforms)
}

fn needs_wizard(no_wizard: bool) -> bool {
    !no_wizard && is_terminal::is_terminal(std::io::stdin())
}

fn scaffold_project(root: &Path) -> Result<(), anyhow::Error> {
    let wm_dir = root.join(WM_DIR);
    create_project_dirs(root, &wm_dir);
    write_default_config(&wm_dir)?;
    create_wiki_dirs(&wm_dir);
    write_agents_md(&wm_dir);
    write_wiki_mem_shim(root);
    Ok(())
}

fn create_project_dirs(root: &Path, wm_dir: &Path) {
    std::fs::create_dir_all(wm_dir.join(WIKI_DIR)).ok();
    std::fs::create_dir_all(wm_dir.join(SOURCES_DIR)).ok();
    std::fs::create_dir_all(wm_dir.join(STATE_DIR)).ok();
    std::fs::create_dir_all(root.join(AGENT_DIR).join(SKILLS_DIR)).ok();
}

fn write_default_config(wm_dir: &Path) -> Result<(), anyhow::Error> {
    let config = ProjectConfig::default();
    let config_path = wm_dir.join(CONFIG_FILE);
    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn create_wiki_dirs(wm_dir: &Path) {
    for dir in &WIKI_SUBDIRS {
        std::fs::create_dir_all(wm_dir.join(WIKI_DIR).join(dir)).ok();
    }
}

fn write_agents_md(wm_dir: &Path) {
    std::fs::write(wm_dir.join(AGENTS_MD_FILE), AGENTS_MD_TEMPLATE).ok();
}

fn write_wiki_mem_shim(root: &Path) {
    if let Some(shim) = wm_core::embed_files::EmbeddedFiles::get(WIKI_MEM_SHIM) {
        std::fs::write(root.join(WIKI_MEM_FILE), &shim.data).ok();
    }
}

fn configure_platforms(root: &Path, platforms: &[String]) -> Result<(), anyhow::Error> {
    if platforms.is_empty() {
        return Ok(());
    }
    sync_agent_files(root, platforms, false)?;
    for platform in platforms {
        if MCP_PLATFORMS.contains(&platform.as_str()) {
            setup_platform_mcp(root, platform)?;
        }
    }
    Ok(())
}
