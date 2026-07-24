use clap::{Parser, Subcommand};
use petgraph::visit::EdgeRef;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use wm_core::config::{self, GitTracking, ProjectConfig};


use wm_core::engine::{EngineState, MainEngine};
use wm_core::ToolRegistry;
mod mcp_transport;
use mcp_transport::serve_rmcp;

mod tui;


#[derive(Parser)]
#[command(name = "wm", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    
    #[arg(long, global = true)]
    tui: bool,
}

#[derive(Subcommand)]
enum Commands {
    
    Init {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        platform: Option<String>,
        
        #[arg(long)]
        no_wizard: bool,
        
        #[arg(long)]
        full: bool,
    },
    
    Web {
        #[arg(long)]
        port: Option<u16>,
    },
    
    Mcp {
        #[arg(long)]
        project: Option<PathBuf>,
    },
    
    Setup {
        
        platform: String,
        
        #[arg(long)]
        global: bool,
    },
    
    Upgrade {
        
        #[arg(long)]
        no_path: bool,
    },
    
    Agents {
        
        #[arg(long)]
        sync: bool,
        
        #[arg(long)]
        global: bool,
    },
    
    Tui,
    
    Search {
        #[command(subcommand)]
        action: SearchAction,
    },
    
    Index {
        #[command(subcommand)]
        action: IndexAction,
    },
    
    Page {
        #[command(subcommand)]
        action: PageAction,
    },
    
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },
    
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },
    
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    
    Log {
        #[command(subcommand)]
        action: LogAction,
    },
    
    Lint {
        #[command(subcommand)]
        action: LintAction,
    },
    
    Validate {
        #[arg(long)]
        json: bool,
    },
    
    Time {
        #[command(subcommand)]
        action: TimeAction,
    },
    
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },
    
    #[command(alias = "--version")]
    Version,
    
    MigrateMemory,
}

#[derive(Subcommand)]
enum SearchAction {
    
    Query {
        query: String,
        #[arg(long)]
        mode: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    
    Retrieve {
        query: String,
        #[arg(long, default_value = "8192")]
        token_budget: usize,
        #[arg(long)]
        json: bool,
    },
    
    Resolve {
        query: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum PageAction {
    
    Get {
        id: String,
        #[arg(long)]
        json: bool,
    },
    
    List {
        #[arg(long)]
        json: bool,
    },
    
    Create {
        path: String,
        title: String,
        #[arg(long)]
        page_type: Option<String>,
        #[arg(long)]
        json: bool,
    },
    
    Delete {
        id: String,
        #[arg(long)]
        json: bool,
    },
    
    Update {
        
        id: String,
        
        #[arg(long)]
        json: bool,
    },
    
    Link {
        id: String,
        target: String,
        #[arg(long)]
        edge_type: Option<String>,
        #[arg(long)]
        json: bool,
    },
    
    Unlink {
        id: String,
        target: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum GraphAction {
    
    Neighbors {
        id: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        json: bool,
    },
    
    Path {
        start: String,
        end: String,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        json: bool,
    },
    
    Subgraph {
        center: String,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        json: bool,
    },
    
    Stats {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum SourceAction {
    
    List {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        json: bool,
    },
    
    Status {
        id: String,
        #[arg(long)]
        json: bool,
    },
    
    Remove { id: String, #[arg(long)] json: bool },
    
    Discover {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum TaskAction {
    
    Board {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum LintAction {
    
    Check {
        #[arg(long)]
        json: bool,
    },
    
    Fix {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum TimeAction {
    
    Start { id: String, #[arg(long)] json: bool },
    
    Stop { id: String, #[arg(long)] json: bool },
    
    Add { id: String, duration: String, #[arg(long)] json: bool },
    
    Report {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum LogAction {
    
    Recent {
        #[arg(long, default_value = "20")]
        count: usize,
        #[arg(long)]
        json: bool,
    },
    
    Since {
        marker: String,
        #[arg(long)]
        json: bool,
    },
    
    Filter {
        text: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum IndexAction {
    
    Rebuild {
        #[arg(long)]
        skip_embed: bool,
        #[arg(long, default_value = "32")]
        batch_size: usize,
    },
    
    Embed {
        #[arg(long, default_value = "32")]
        batch_size: usize,
        #[arg(long)]
        force: bool,
    },
    
    Code {
        #[arg(long)]
        skip_hash_check: bool,
    },
}

#[derive(Subcommand)]
enum ModelAction {
    Download { name: String, #[arg(long)] json: bool },
    List { #[arg(long)] json: bool },
    Status { #[arg(long)] json: bool },
    Remove { name: String, #[arg(long)] json: bool },
}

fn setup_logging() {
    
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(false);

    
    
    
    
    
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let log_dir = std::path::PathBuf::from(home).join(".wm").join("logs");
    std::fs::create_dir_all(&log_dir).ok();
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::DAILY,
        log_dir,
        "wm.log",
    );
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();
}




fn create_engine() -> (Arc<MainEngine>, PathBuf) {
    let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
    let wiki_dir = root.join(".wm").join("wiki");
    let engine = Arc::new(MainEngine::new());

    
    let old_memory_dir = root.join(".wm").join("memory");
    if old_memory_dir.exists() {
        match wm_core::page::migrate_old_memory_json(&engine.state) {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Auto-migrated {} memory entries to wiki pages", count);
                }
            }
            Err(e) => tracing::warn!("Memory migration failed: {}", e),
        }
    }

    if wiki_dir.exists() {
        rebuild_from_engine(&engine, &wiki_dir);
        engine.state.stale_flag.store(false, std::sync::atomic::Ordering::Release);
    }
    (engine, wiki_dir)
}


fn rebuild_from_engine(engine: &Arc<MainEngine>, wiki_dir: &Path) -> usize {
    let ct = engine.state.config.read().unwrap().custom_edge_types.clone();
    let count = wm_core::graph::rebuild_graph_snapshot(&engine.state.graph, wiki_dir, &ct);

    
    let sections = wm_core::graph::build_sections_from_wiki(wiki_dir);
    engine.state.section_corpus.store(Arc::new(sections.clone()));
    let docs: Vec<wm_core::search::IndexedDoc> = sections
        .iter()
        .map(|s| wm_core::search::IndexedDoc {
            id: s.section_id.clone(),
            fields: vec![
                wm_core::search::Field::new("header", &s.header, 4.0),
                wm_core::search::Field::new("body", &s.body, 1.0),
                wm_core::search::Field::new("id", &s.section_id, 0.0),
                wm_core::search::Field::new("title", &s.title, 0.0),
                wm_core::search::Field::new("tags", &s.tags.join(" "), 0.0),
            ],
        })
        .collect();
    let bm25 = wm_core::search::Bm25Index::build(docs);
    engine.state.bm25_index.store(Arc::new(bm25));

    count
}



fn sync_agent_files(root: &std::path::Path, platforms: &[String], _force: bool) -> Result<(), anyhow::Error> {
    use std::collections::HashSet;
    let targets: Vec<&str> = if platforms.is_empty() {
        vec!["claude", "opencode", "kiro", "gemini", "copilot", "agents", "reasonix"]
    } else {
        platforms.iter().map(|s| s.as_str()).collect()
    };

    
    let template_map: [(&str, &str); 6] = [
        ("CLAUDE.md", "shims/CLAUDE.md"),
        ("AGENTS.md", "shims/AGENTS.md"),
        ("GEMINI.md", "shims/GEMINI.md"),
        (".github/copilot-instructions.md", "shims/copilot-instructions.md"),
        ("REASONIX.md", "shims/REASONIX.md"),
        ("OPENCODE.md", "shims/OPENCODE.md"),
    ];

    
    let compat_map: [(&str, &str); 8] = [
        ("claude", "CLAUDE.md"),
        ("codex", "CLAUDE.md"),
        ("opencode", "AGENTS.md"),
        ("kiro", "AGENTS.md"),
        ("agents", "AGENTS.md"),
        ("gemini", "GEMINI.md"),
        ("copilot", ".github/copilot-instructions.md"),
        ("reasonix", "REASONIX.md"),
    ];

    let mut written: HashSet<String> = HashSet::new();

    for plat in &targets {
        let output_filename = match compat_map.iter().find(|(p, _)| p == plat) {
            Some((_, fname)) => fname,
            None => {
                eprintln!("Unknown platform: {}. Use `wm setup <platform>` for MCP config.", plat);
                continue;
            }
        };

        
        let output_str = *output_filename;
        let template_key = match template_map.iter().find(|(fname, _)| *fname == output_str) {
            Some((_, key)) => key,
            None => continue,
        };

        
        let content = wm_core::embed_files::EmbeddedFiles::get(template_key)
            .and_then(|f| std::str::from_utf8(f.data.as_ref()).ok().map(String::from))
            .ok_or_else(|| anyhow::anyhow!("Embedded shim template not found: {}", template_key))?;

        let path = if output_filename.starts_with(".github") {
            let d = root.join(".github");
            std::fs::create_dir_all(&d).ok();
            d.join("copilot-instructions.md")
        } else {
            root.join(output_filename)
        };
        if written.insert(output_filename.to_string()) {
            std::fs::write(&path, content)?;
            println!("  {} — agent instruction file generated", output_filename);
        } else {
            println!("  {} — also handled by {} platform (same file)", output_filename, plat);
        }
    }
    
    if targets.contains(&"opencode") {
        if let Some(file) = wm_core::embed_files::EmbeddedFiles::get("shims/OPENCODE.md") {
            if let Ok(content) = std::str::from_utf8(file.data.as_ref()) {
                std::fs::write(root.join("OPENCODE.md"), content).ok();
            }
        }
    }
    Ok(())
}



fn sync_skills_to(platform_skills_dir: &std::path::Path) -> Result<(), anyhow::Error> {
    let skills = wm_core::skill::load_embedded_skills();
    if skills.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(platform_skills_dir)?;
    for skill in &skills {
        let skill_subdir = platform_skills_dir.join(&skill.name);
        std::fs::create_dir_all(&skill_subdir)?;
        
        let embed_path = format!("skills/{}/SKILL.md", skill.name);
        if let Some(file) = wm_core::embed_files::EmbeddedFiles::get(&embed_path) {
            std::fs::write(skill_subdir.join("SKILL.md"), &file.data)?;
        }
    }
    Ok(())
}



fn determine_project_root(project: &Option<PathBuf>) -> Result<PathBuf, anyhow::Error> {
    if let Some(path) = project {
        Ok(path.clone())
    } else {
        config::detect_project_root()
            .ok_or_else(|| anyhow::anyhow!("No project root found. Run 'wm init' first."))
    }
}



fn json_to_page_updates(json: &serde_json::Value) -> wm_core::page::PageUpdateParams {
    let mut params = wm_core::page::PageUpdateParams::default();
    if let Some(obj) = json.as_object() {
        for (k, v) in obj {
            match k.as_str() {
                "title" => params.title = v.as_str().map(String::from),
                "content" => params.content = v.as_str().map(String::from),
                "status" => params.status = v.as_str().map(String::from),
                "priority" => params.priority = v.as_str().map(String::from),
                "assignee" => params.assignee = v.as_str().map(String::from),
                "tags" => {
                    params.tags = v.as_array().map(|a| {
                        a.iter().filter_map(|x| x.as_str().map(String::from)).collect()
                    });
                }
                "relates_to" => {
                    params.relates_to = v.as_array().map(|a| a.clone());
                }
                "remove_relates_to" => {
                    params.remove_relates_to = v.as_str().map(String::from);
                }
                "acceptance_criteria" => {
                    params.acceptance_criteria = v.as_array().map(|a| {
                        a.iter()
                            .filter_map(|item| {
                                Some(wm_core::engine::AcceptanceCriterion {
                                    text: item.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                                    checked: item.get("checked").and_then(|c| c.as_bool()).unwrap_or(false),
                                })
                            })
                            .collect()
                    });
                }
                "implementation_plan" => params.implementation_plan = v.as_str().map(String::from),
                "implementation_notes" => params.implementation_notes = v.as_str().map(String::from),
                "append_notes" => params.append_notes = v.as_str().map(String::from),
                "type" => params.r#type = v.as_str().map(String::from),
                "checked_ac" => {
                    params.checked_ac = v.as_array().map(|a| a.iter().filter_map(|x| x.as_u64()).collect());
                }
                "unchecked_ac" => {
                    params.unchecked_ac = v.as_array().map(|a| a.iter().filter_map(|x| x.as_u64()).collect());
                }
                _ => {}
            }
        }
    }
    params
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    setup_logging();
    let cli = Cli::parse();

    
    if cli.tui || (cli.command.is_none() && is_terminal::is_terminal(std::io::stdout())) {
        let (engine, _) = create_engine();
        return crate::tui::run_tui(engine);
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            
            eprintln!("No command given. Use --help for usage, or call interactively for TUI.");
            return Ok(());
        }
    };

    match command {
        Commands::Init { project, platform, no_wizard, full } => {
            let root = project.unwrap_or_else(|| std::env::current_dir().unwrap());

            
            if full {
                if let Ok(dst) = wm_core::install::install_binary() {
                    println!("  Installed WM to {}", dst.display());
                    wm_core::install::ensure_on_path().ok();
                }
            }
            let wm_dir = root.join(".wm");
            std::fs::create_dir_all(wm_dir.join("wiki")).ok();
            std::fs::create_dir_all(wm_dir.join("sources")).ok();
            std::fs::create_dir_all(wm_dir.join("state")).ok();
            let agents_dir = root.join(".agent");
            std::fs::create_dir_all(agents_dir.join("skills")).ok();

            let config = ProjectConfig::default();
            let config_path = wm_dir.join("config.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

            
            for dir in &[
                "tasks",
                "specs",
                "concepts",
                "patterns",
                "decisions",
                "howto",
                "reference",
                "memory",
            ] {
                std::fs::create_dir_all(wm_dir.join("wiki").join(dir)).ok();
            }

            

            
            let agents_md = r#"# AGENTS.md — Wiki Memory Engine Agent Handbook

## Wiki Conventions

### 7 Page Types
| Type | Directory | Purpose |
|------|-----------|---------|
| task | `wiki/tasks/` | Actionable units of work with acceptance criteria |
| spec | `wiki/specs/` | Functional/non-functional requirements, goals |
| concept | `wiki/concepts/` | Domain concepts, terminology, architecture |
| pattern | `wiki/patterns/` | Reusable solutions, when-to-use, examples |
| decision | `wiki/decisions/` | ADRs: context, options, rationale, outcome |
| howto | `wiki/howto/` | Step-by-step guides, tutorials |
| reference | `wiki/reference/` | API docs, error codes, configuration tables |

### Frontmatter Schema
Every wiki page starts with YAML frontmatter:
```yaml
---
title: Page Title
type: task|spec|concept|pattern|decision|howto|reference
status: todo|in-progress|done|draft|reviewed|approved
tags: [tag1, tag2]
priority: low|medium|high|urgent
assignee: name
confidence: high|medium|low
---
```
Per-type fields (spec): `functional_requirements`, `non_functional_requirements`, `general_goals`
Per-type fields (decision): `decision.context`, `decision.options`, `decision.rationale`, `decision.outcome`
Per-type fields (task): `acceptance_criteria`, `estimate`, `prerequisites`

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

## Canonical Workflows

### 1. wm-init — Session Initialization
- Trigger: Start of new session
- Steps: Call `wm_initial` → List docs → Check tasks/board → Load memory → Summarize

### 2. wm-research — Project Research
- Trigger: Need to understand context
- Steps: Search (`wm_search.query`) → Read pages (`wm_page.get`) → Graph traversal (`wm_graph.neighbors`)
- Cross-entity search across pages + memory

### 3. wm-plan — Task Planning
- Trigger: Task assigned
- Steps: Search wiki for related specs → Plan with ACs → Validate → Wait for approval
- Supports `--from @doc/<spec>` for spec-wide task generation

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
            std::fs::write(wm_dir.join("AGENTS.md"), agents_md).ok();

            info!("Initialized project at {}", root.display());
            println!("Wiki Memory Engine initialized at {}", root.display());

            
            if !no_wizard && is_terminal::is_terminal(std::io::stdin()) {
                println!();
                print!("Enable semantic search (ONNX embeddings)? This requires downloading a ~134MB model. [y/N]: ");
                std::io::stdout().flush().ok();
                let mut sem_input = String::new();
                std::io::stdin().read_line(&mut sem_input).ok();
                let enable_semantic = sem_input.trim().eq_ignore_ascii_case("y");

                if enable_semantic {
                    println!();
                    println!("Select embedding model:");
                    let models = [
                        ("1", "bge-small-en-v1.5", "384 dim, ~134MB — recommended for most projects"),
                        ("2", "all-MiniLM-L6-v2", "384 dim, ~90MB — faster, slightly less accurate"),
                        ("3", "bge-base-en-v1.5", "768 dim, ~438MB — highest accuracy"),
                    ];
                    for (key, name, desc) in &models {
                        println!("  {}. {} ({})", key, name, desc);
                    }
                    print!("Enter selection [1]: ");
                    std::io::stdout().flush().ok();
                    let mut model_input = String::new();
                    std::io::stdin().read_line(&mut model_input).ok();
                    let model_choice = model_input.trim().parse::<usize>().unwrap_or(1);
                    let model_name = models
                        .get(model_choice.checked_sub(1).unwrap_or(0))
                        .map(|(_, n, _)| *n)
                        .unwrap_or("bge-small-en-v1.5");

                    
                    let config_path = root.join(".wm").join("config.json");
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        if let Ok(mut cfg) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(embed) = cfg.get_mut("embedding") {
                                if let Some(obj) = embed.as_object_mut() {
                                    obj.insert("model_name".into(), serde_json::json!(model_name));
                                }
                            }
                            if let Ok(updated) = serde_json::to_string_pretty(&cfg) {
                                let _ = std::fs::write(&config_path, &updated);
                            }
                        }
                    }
                    println!("  Semantic search enabled (model: {})", model_name);
                } else {
                    println!("  Semantic search disabled (keyword-only mode)");
                }

                
                println!();
                println!("Git tracking mode for .wm/ directory:");
                println!("  1. git-tracked — track everything (config, wiki pages, memory)");
                println!("  2. git-ignored — track config + wiki pages; ignore memory, generated files");
                println!("  3. none — no .gitignore changes (manage manually)");
                print!("Enter selection [1]: ");
                std::io::stdout().flush().ok();
                let mut git_input = String::new();
                std::io::stdin().read_line(&mut git_input).ok();
                let git_mode = git_input.trim().parse::<usize>().unwrap_or(1);

                
                let git_tracking = match git_mode {
                    2 => GitTracking {
                        memory: Some(true),
                        versions: Some(true),
                        state: Some(true),
                    },
                    1 => GitTracking {
                        memory: Some(false),
                        versions: Some(false),
                        state: Some(false),
                    },
                    _ => GitTracking {
                        memory: None,
                        versions: None,
                        state: None,
                    },
                };
                if let Err(e) = config::apply_git_tracking(&root, &git_tracking) {
                    eprintln!("  Warning: failed to update .gitignore: {}", e);
                } else {
                    match git_mode {
                        2 => println!("  .gitignore: .wm/state/, .wm/memory/, .wm/versions/ ignored"),
                        1 => println!("  .gitignore: .wm/ fully tracked"),
                        _ => println!("  .gitignore: unchanged (manage manually)"),
                    }
                }

                
                let config_path = root.join(".wm").join("config.json");
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(mut cfg) = serde_json::from_str::<serde_json::Value>(&content) {
                        cfg["git_tracking"] = serde_json::json!(git_tracking);
                        if let Ok(updated) = serde_json::to_string_pretty(&cfg) {
                            let _ = std::fs::write(&config_path, &updated);
                        }
                    }
                }
            }

            
            let platforms: Vec<String> = if let Some(plat) = platform {
                vec![plat.to_lowercase()]
            } else if no_wizard {
                Vec::new()
            } else if is_terminal::is_terminal(std::io::stdin()) {
                
                println!();
                println!("Generate platform agent instruction files?");
                println!("Select platforms (comma-separated numbers, or 0 to skip):");
                let platform_list: [(&str, &str, &str); 7] = [
                    ("1", "claude", "CLAUDE.md — Claude Code"),
                    ("2", "opencode", "AGENTS.md + opencode.json — OpenCode"),
                    ("3", "kiro", "AGENTS.md + .kiro/settings/mcp.json — Kiro"),
                    ("4", "gemini", "GEMINI.md — Gemini"),
                    ("5", "copilot", ".github/copilot-instructions.md — GitHub Copilot"),
                    ("6", "agents", "AGENTS.md — Generic agents"),
                    ("7", "reasonix", "REASONIX.md — Reasonix"),
                ];
                for (key, _name, desc) in &platform_list {
                    println!("  {}. {}", key, desc);
                }
                print!("Enter selection [0]: ");
use std::io::Write;
                std::io::stdout().flush().ok();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
                let input = input.trim();
                if input.is_empty() || input == "0" {
                    Vec::new()
                } else {
                    input.split(',')
                        .filter_map(|s| s.trim().parse::<usize>().ok())
                        .filter_map(|i| platform_list.iter().find(|(k, _, _)| k.parse::<usize>().ok() == Some(i)))
                        .map(|(_, name, _)| name.to_string())
                        .collect::<Vec<_>>()
                }
            } else {
                Vec::new()
            };

            sync_agent_files(&root, &platforms, false)?;
        }
        Commands::Web { port } => {
            let port = port.unwrap_or(4090);
            
            
            let server_binary = match std::env::current_exe() {
                Ok(p) => {
                    let mut path = p.parent().unwrap_or(Path::new(".")).to_path_buf();
                    path.push(if cfg!(windows) { "wm-server.exe" } else { "wm-server" });
                    path
                }
                Err(_) => PathBuf::from("wm-server"),
            };

            if !server_binary.exists() {
                eprintln!("wm-server not found at {}. Build with: cargo build -p wm-server", server_binary.display());
                return Ok(());
            }

            info!("Starting wm-server on port {}...", port);
            match std::process::Command::new(&server_binary)
                .arg("--port")
                .arg(port.to_string())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .spawn()
            {
                Ok(mut child) => {
                    match child.wait() {
                        Ok(status) if !status.success() => {
                            eprintln!("wm-server exited with code: {:?}", status.code());
                        }
                        Err(e) => eprintln!("Server process error: {e}"),
                        _ => {}
                    }
                }
                Err(e) => eprintln!("Failed to start wm-server: {e}"),
            }
        }
        Commands::Mcp { project } => {
            let project_root = determine_project_root(&project)?;

            
            let config = config::load_config(&project_root).unwrap_or_default();
            let (engine_state, audit_rx) = EngineState::new(config, project_root.clone());
            let engine = Arc::new(engine_state);

            
            let mut registry = ToolRegistry::new();
            wm_core::mcp::tools::register_all_tools(&mut registry, engine.clone());

            
            tokio::spawn(async move {
                let mut rx = audit_rx;
                while rx.recv().await.is_some() {}
            });

            
            let wiki_dir = project_root.join(".wm").join("wiki");
            if wiki_dir.exists() {
                engine.rebuild_graph(&wiki_dir);
            }

            info!(
                "MCP server ready (direct mode, {} tools registered)",
                registry.list_tools().len()
            );

            
            let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
            let s_tx = shutdown_tx.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                info!("SIGINT received, shutting down...");
                drop(s_tx);
            });

            tokio::select! {
                result = serve_rmcp(registry) => {
                    result?;
                }
                _ = shutdown_rx.recv() => {
                    info!("Graceful shutdown complete.");
                }
            }
        }
        Commands::Upgrade { no_path } => {
            let dst = wm_core::install::install_binary().map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("  Installed WM to {}", dst.display());
            if !no_path {
                wm_core::install::ensure_on_path().map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("  Registered ~\\.wm\\bin on user PATH");
            }
        }
        Commands::Setup { platform, global } => {
            let root = if global {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                PathBuf::from(home)
            } else {
                config::detect_project_root()
                    .unwrap_or_else(|| std::env::current_dir().unwrap())
            };

            let bin_path = std::env::current_exe()
                .unwrap_or_else(|_| PathBuf::from("wm-cli"))
                .to_string_lossy().to_string();

            
            let opencode_cmd = if wm_core::install::is_installed() {
                "wm-cli".into()
            } else {
                bin_path.clone()
            };

            match platform.to_lowercase().as_str() {
                "opencode" => {
                    let cfg = if global {
                        let d = root.join(".config").join("opencode");
                        std::fs::create_dir_all(&d).ok();
                        d.join("opencode.json")
                    } else {
                        root.join("opencode.json")
                    };
                    let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/opencode.json")
                        .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/opencode.json"))?;
                    let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                    if let Some(cmd_arr) = mcp.pointer_mut("/mcp/wm/command").and_then(|v| v.as_array_mut()) {
                        if cmd_arr.len() == 2 && cmd_arr[0] == "wm-cli" {
                            cmd_arr[0] = serde_json::Value::String(opencode_cmd.clone());
                        }
                    }
                    wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                    
                    if let Some(file) = wm_core::embed_files::EmbeddedFiles::get("shims/OPENCODE.md") {
                        if let Ok(content) = std::str::from_utf8(file.data.as_ref()) {
                            std::fs::write(root.join("OPENCODE.md"), content)?;
                        }
                    }
                    
                    let skills_dir = root.join(".opencode").join("skills");
                    sync_skills_to(&skills_dir)?;
                    println!("  {} — OpenCode MCP config (+ skills synced to .opencode/skills/)", cfg.display());
                }
                "kiro" => {
                    let cfg_dir = if global {
                        root.join(".kiro").join("settings")
                    } else {
                        root.join(".kiro").join("settings")
                    };
                    std::fs::create_dir_all(&cfg_dir).ok();
                    let cfg = cfg_dir.join("mcp.json");
                    let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/kiro_mcp.json")
                        .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/kiro_mcp.json"))?;
                    let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                    if let Some(cmd_val) = mcp.pointer_mut("/mcpServers/wm/command") {
                        if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                            *cmd_val = serde_json::Value::String(bin_path.clone());
                        }
                    }
                    wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                    
                    let kiro_skills = if global {
                        root.join(".kiro").join("skills")
                    } else {
                        root.join(".kiro").join("skills")
                    };
                    sync_skills_to(&kiro_skills)?;
                    println!("  {} — Kiro MCP config (+ skills synced to {})", cfg.display(), kiro_skills.display());
                }
                "claude" => {
                    
                    let cfg_file = root.join(".mcp.json");
                    let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/dot_mcp.json")
                        .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/dot_mcp.json"))?;
                    let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                    if let Some(cmd_val) = mcp.pointer_mut("/mcpServers/wm/command") {
                        if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                            *cmd_val = serde_json::Value::String(bin_path.clone());
                        }
                    }
                    wm_core::platform_service::write_merged_json(&cfg_file, mcp)?;

                    if global {
                        
                        let app_data = std::env::var("APPDATA")
                            .or_else(|_| std::env::var("HOME"))
                            .unwrap_or_else(|_| ".".into());
                        let desktop_dir = PathBuf::from(app_data).join("Claude");
                        std::fs::create_dir_all(&desktop_dir).ok();
                        let desktop_cfg = desktop_dir.join("claude_desktop_config.json");
                        let embedded_desktop = wm_core::embed_files::EmbeddedFiles::get("configs/dot_mcp.json")
                            .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/dot_mcp.json"))?;
                        let mut desktop_mcp: serde_json::Value = serde_json::from_slice(&embedded_desktop.data)?;
                        if let Some(cmd_val) = desktop_mcp.pointer_mut("/mcpServers/wm/command") {
                            if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                                *cmd_val = serde_json::Value::String(bin_path.clone());
                            }
                        }
                        wm_core::platform_service::write_merged_json(&desktop_cfg, desktop_mcp)?;
                        println!("  {} — Claude project MCP config", cfg_file.display());
                        println!("  {} — Claude Desktop global config", desktop_cfg.display());
                    } else {
                        
                        let claude_skills = root.join(".claude").join("skills");
                        sync_skills_to(&claude_skills)?;
                        println!("  {} — Claude MCP config (+ skills synced to {})", cfg_file.display(), claude_skills.display());
                    }
                }
                "codex" => {
                    
                    let cfg_file = if global {
                        let d = root.join(".codex");
                        std::fs::create_dir_all(&d).ok();
                        d.join("config.toml")
                    } else {
                        let d = root.join(".codex");
                        std::fs::create_dir_all(&d).ok();
                        d.join("config.toml")
                    };
                    wm_core::platform_service::write_toml_config(&cfg_file, &bin_path)?;
                    
                    let skills_dir = root.join(".codex").join("skills");
                    sync_skills_to(&skills_dir)?;
                    println!("  {} — Codex MCP config (TOML) (+ skills synced to .codex/skills/)", cfg_file.display());
                }
                "cursor" => {
                    let cfg_dir = if global {
                        root.join(".cursor")
                    } else {
                        root.join(".cursor")
                    };
                    std::fs::create_dir_all(&cfg_dir).ok();
                    let cfg = cfg_dir.join("mcp.json");
                    let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/cursor_mcp.json")
                        .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/cursor_mcp.json"))?;
                    let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                    if let Some(cmd_val) = mcp.pointer_mut("/mcpServers/wm/command") {
                        if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                            *cmd_val = serde_json::Value::String(bin_path.clone());
                        }
                    }
                    wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                    
                    let skills_dir = root.join(".agent").join("skills");
                    sync_skills_to(&skills_dir)?;
                    println!("  {} — Cursor MCP config (+ skills synced)", cfg.display());
                }
                "antigravity" => {
                    
                    let gemini_dir = root.join(".gemini").join("antigravity");
                    std::fs::create_dir_all(&gemini_dir).ok();
                    let cfg = gemini_dir.join("mcp_config.json");
                    let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/antigravity_mcp.json")
                        .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/antigravity_mcp.json"))?;
                    let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                    if let Some(cmd_val) = mcp.pointer_mut("/mcpServers/wm/command") {
                        if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                            *cmd_val = serde_json::Value::String(bin_path.clone());
                        }
                    }
                    wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                    
                    let skills_dir = root.join(".agents").join("skills");
                    sync_skills_to(&skills_dir)?;
                    println!("  {} — Antigravity MCP config (+ skills synced to .agents/skills/)", cfg.display());
                }
                "gemini" => {
                    
                    let skills_dir = root.join(".agent").join("skills");
                    sync_skills_to(&skills_dir)?;
                    println!("  Skills synced to {} (Gemini CLI uses platform-managed config)", skills_dir.display());
                }
                "agents" => {
                    
                    let skills_dir = root.join(".agent").join("skills");
                    sync_skills_to(&skills_dir)?;
                    println!("  Skills synced to {}", skills_dir.display());
                }
                "reasonix" => {
                    
                    let plats = vec!["reasonix".into()];
                    sync_agent_files(&root, &plats, false)?;
                }
                "all" => {
                    
                    for dir in &[".claude/skills", ".agent/skills", ".opencode/skills", ".kiro/skills", ".codex/skills", ".agents/skills"] {
                        let skills_dir = root.join(dir);
                        sync_skills_to(&skills_dir)?;
                        println!("  Skills synced to {}", skills_dir.display());
                    }
                    
                    for plat in &["opencode", "claude", "kiro", "codex", "cursor"] {
                        match *plat {
                            "opencode" => {
                                let cfg = root.join("opencode.json");
                                let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/opencode.json")
                                    .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/opencode.json"))?;
                                let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                                if let Some(cmd_arr) = mcp.pointer_mut("/mcp/wm/command").and_then(|v| v.as_array_mut()) {
                                    if cmd_arr.len() == 2 && cmd_arr[0] == "wm-cli" {
                                        cmd_arr[0] = serde_json::Value::String(bin_path.clone());
                                    }
                                }
                                wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                                println!("  {} — OpenCode MCP config", cfg.display());
                            }
                            "claude" => {
                                let cfg = root.join(".mcp.json");
                                let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/dot_mcp.json")
                                    .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/dot_mcp.json"))?;
                                let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                                if let Some(cmd_val) = mcp.pointer_mut("/mcpServers/wm/command") {
                                    if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                                        *cmd_val = serde_json::Value::String(bin_path.clone());
                                    }
                                }
                                wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                                println!("  {} — Claude MCP config", cfg.display());
                            }
                            "kiro" => {
                                let cfg = root.join(".kiro").join("settings").join("mcp.json");
                                std::fs::create_dir_all(cfg.parent().unwrap()).ok();
                                let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/kiro_mcp.json")
                                    .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/kiro_mcp.json"))?;
                                let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                                if let Some(cmd_val) = mcp.pointer_mut("/mcpServers/wm/command") {
                                    if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                                        *cmd_val = serde_json::Value::String(bin_path.clone());
                                    }
                                }
                                wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                                println!("  {} — Kiro MCP config", cfg.display());
                            }
                            "codex" => {
                                let cfg = root.join(".codex").join("config.toml");
                                std::fs::create_dir_all(cfg.parent().unwrap()).ok();
                                wm_core::platform_service::write_toml_config(&cfg, &bin_path)?;
                                println!("  {} — Codex MCP config", cfg.display());
                            }
                            "cursor" => {
                                let cfg = root.join(".cursor").join("mcp.json");
                                std::fs::create_dir_all(cfg.parent().unwrap()).ok();
                                let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/cursor_mcp.json")
                                    .ok_or_else(|| anyhow::anyhow!("Embedded config not found: configs/cursor_mcp.json"))?;
                                let mut mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
                                if let Some(cmd_val) = mcp.pointer_mut("/mcpServers/wm/command") {
                                    if cmd_val.as_str() == Some("wm-cli") && bin_path != "wm-cli" {
                                        *cmd_val = serde_json::Value::String(bin_path.clone());
                                    }
                                }
                                wm_core::platform_service::write_merged_json(&cfg, mcp)?;
                                println!("  {} — Cursor MCP config", cfg.display());
                            }
                            _ => {}
                        }
                    }
                }
                other => {
                    eprintln!("Unknown platform: {}. Supported: claude, codex, opencode, kiro, cursor, antigravity, gemini, agents, all", other);
                }
            }
        }
        Commands::Agents { sync: _sync, global } => {
            let root = if global {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                PathBuf::from(home)
            } else {
                PathBuf::from(".")
            };
            
            let platforms: Vec<String> = Vec::new(); 
            sync_agent_files(&root, &platforms, false)?;
            println!("Agent instruction files synced.");
        }
        Commands::Tui => {
            let (engine, _) = create_engine();
            if let Err(e) = crate::tui::run_tui(engine) {
                eprintln!("TUI error: {e}");
            }
        }
        Commands::Search { action } => match action {
            SearchAction::Query {
                query,
                mode,
                r#type,
                limit,
                json,
            } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                }

                let mode_val = mode.clone().unwrap_or_else(|| "auto".into());
                let qp = wm_core::search::QueryParams {
                    query: query.clone(),
                    r#type: r#type.unwrap_or_else(|| "all".into()),
                    mode: mode_val.clone(),
                    limit,
                    offset: 0,
                    recency: true,
                };

                let resp = wm_core::search::query::run_unified_search(&engine.state, &qp).unwrap_or_default();
                let mode_used = mode_val;
                let results: Vec<serde_json::Value> = resp.results.iter().map(|r| {
                    serde_json::json!({
                        "score": r.score,
                        "id": r.id,
                        "type": r.r#type,
                    })
                }).collect();

                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "mode": mode_used,
                            "results": results,
                            "total": results.len()
                        }))?
                    );
                } else {
                    println!("Mode: {}", mode_used);
                    for r in &results {
                        let type_tag = if r["type"].as_str() == Some("memory") { " [memory]" } else { "" };
                        println!("  {:.2}  {}{}", r["score"].as_f64().unwrap_or(0.0), r["id"].as_str().unwrap_or("?"), type_tag);
                    }
                    println!("{} results", results.len());
                }
            }
            SearchAction::Retrieve {
                query,
                token_budget,
                json,
            } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                }

                
                let qp = wm_core::search::QueryParams {
                    query: query.clone(),
                    r#type: "page".into(),
                    mode: "auto".into(),
                    limit: 1,
                    offset: 0,
                    recency: false,
                };
                let resp = wm_core::search::query::run_unified_search(&engine.state, &qp).unwrap_or_default();
                let bfs_seed = resp.results.first().map(|r| r.id.clone()).unwrap_or_else(|| query.clone());

                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;
                let context = wm_core::search::retrieve_context(graph, index, &bfs_seed, token_budget, None);
                if json {
                    let context_text: String = context
                        .iter()
                        .map(|(_, _, text)| text.as_str())
                        .fold(
                            String::new(),
                            |mut acc, s| {
                                if !acc.is_empty() { acc.push('\n'); }
                                acc.push_str(s);
                                acc
                            },
                        );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "query": query,
                            "token_budget": token_budget,
                            "tokens_used": context_text.len() / 4,
                            "result_count": context.len(),
                            "context": context_text,
                        }))?
                    );
                } else {
                    for (id, score, text) in &context {
                        println!("  {:.2}  {}  {}", score, id, text);
                    }
                    println!("{} context items", context.len());
                }
            }
            SearchAction::Resolve { query, json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                }
                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;

                let result = if let Some(&idx) = index.get(&query) {
                    let meta = &graph[idx];
                    serde_json::json!({
                        "resolved": true,
                        "id": meta.id,
                        "title": meta.title,
                        "page_type": format!("{:?}", meta.page_type).to_lowercase(),
                    })
                } else {
                    
                    let mut matched = None;
                    for idx in graph.node_indices() {
                        let meta = &graph[idx];
                        if meta.title.eq_ignore_ascii_case(&query)
                            || meta.id.eq_ignore_ascii_case(&query)
                        {
                            matched = Some(serde_json::json!({
                                "resolved": true,
                                "id": meta.id,
                                "title": meta.title,
                                "page_type": format!("{:?}", meta.page_type).to_lowercase(),
                            }));
                            break;
                        }
                    }
                    match matched {
                        Some(v) => v,
                        None => {
                            let bm25 = engine.state.bm25_index.load();
                            let results = bm25.search(&query, 5);
                            if !results.is_empty() {
                                let candidates: Vec<serde_json::Value> = results.iter().map(|r| {
                                    serde_json::json!({ "id": r.id, "score": r.score, "snippet": r.snippet })
                                }).collect();
                                serde_json::json!({ "resolved": false, "candidates": candidates, "total": candidates.len() })
                            } else {
                                serde_json::json!({ "resolved": false, "candidates": [], "total": 0 })
                            }
                        }
                    }
                };

                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    if result["resolved"].as_bool().unwrap_or(false) {
                        println!("Resolved: {} ({})", result["id"], result["title"]);
                    } else {
                        println!("Not resolved");
                        if let Some(candidates) = result["candidates"].as_array() {
                            for c in candidates {
                                println!("  {:.2}  {}", c["score"], c["id"]);
                            }
                        }
                    }
                }
            }
        },
        Commands::Page { action } => match action {
            PageAction::Get { id, json } => {
                let (engine, _root) = create_engine();
                let content = wm_core::page::get_page_raw(&engine.state, &id)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "id": id, "content": content
                        }))?
                    );
                } else {
                    println!("--- {} ---", id);
                    let display = if content.len() > 500 { &content[..500] } else { &content };
                    println!("{}", display);
                }
            }
            PageAction::List { json } => {
                let (engine, _root) = create_engine();
                let pages = wm_core::page::list_pages(&engine.state, None)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({ "pages": pages, "total": pages.len() })
                        )?
                    );
                } else {
                    for p in &pages {
                        println!("  {}  [{}]", p["id"], p["type"].as_str().unwrap_or(""));
                    }
                    println!("{} pages", pages.len());
                }
            }
            PageAction::Create {
                path,
                title,
                page_type,
                json,
            } => {
                let pt = page_type.unwrap_or_else(|| {
                    let first_segment = path
                        .trim_start_matches("wiki/")
                        .split('/')
                        .next()
                        .unwrap_or("concept");
                    match first_segment {
                        "tasks" => "task",
                        "specs" => "spec",
                        "concepts" => "concept",
                        "patterns" => "pattern",
                        "decisions" => "decision",
                        "howto" => "howto",
                        "reference" => "reference",
                        _ => "concept",
                    }
                    .to_string()
                });
                let default_status = if pt == "task" { "todo" } else { "draft" };
                let frontmatter = format!("title: {}\ntype: {}\nstatus: {}\n", title, pt, default_status);
                let mut content = String::new();
                std::io::stdin().read_to_string(&mut content)
                    .map_err(|e| anyhow::anyhow!("Failed to read stdin: {}", e))?;
                let (engine, root) = create_engine();
                let wiki_dir = root.join(".wm").join("wiki");
                match wm_core::page::create_page(&engine.state, &path, &frontmatter, &content) {
                    Ok(id) => {
                        
                        let path_clean = path.trim_start_matches("wiki/");
                        let file_path = wiki_dir.join(format!("{}.md", path_clean));
                        wm_core::graph::handle_file_change(&wiki_dir, &file_path, &engine.state);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "path": path, "type": pt
                                }))?
                            );
                        } else {
                            println!("Created page: {} ({})", id, pt);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            PageAction::Delete { id, json } => {
                let (engine, root) = create_engine();
                let wiki_dir = root.join(".wm").join("wiki");
                let path = wm_core::page::helpers::resolve_id_to_path(&root, &id)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                match wm_core::page::delete_page(&engine.state, &id) {
                    Ok(_) => {
                        if wiki_dir.exists() {
                            wm_core::graph::handle_file_delete(&wiki_dir, &path, &engine.state);
                        }
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "status": "deleted"
                                }))?
                            );
                        } else {
                            println!("Deleted page: {}", id);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            PageAction::Update { id, json } => {
                let mut input = String::new();
                std::io::stdin()
                    .read_to_string(&mut input)
                    .map_err(|e| anyhow::anyhow!("Failed to read stdin: {e}"))?;
                let updates: serde_json::Value = serde_json::from_str(&input)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON on stdin: {e}"))?;

                let (engine, _root) = create_engine();
                let params = json_to_page_updates(&updates);
                match wm_core::page::update_page(&engine.state, &id, &params) {
                    Ok(_) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                                "id": id, "status": "updated"
                            })).unwrap_or_default());
                        } else {
                            println!("Updated: {}", id);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            PageAction::Link {
                id,
                target,
                edge_type,
                json,
            } => {
                let et = edge_type.unwrap_or_else(|| "relates_to".into());
                let (engine, _root) = create_engine();
                let params = wm_core::page::PageUpdateParams {
                    relates_to: Some(vec![serde_json::json!({"type": et, "target": target})]),
                    ..Default::default()
                };
                match wm_core::page::update_page(&engine.state, &id, &params) {
                    Ok(_) => {
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "target": target, "type": et, "status": "linked"
                                }))?
                            );
                        } else {
                            println!("Linked {} --[{}]--> {}", id, et, target);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            PageAction::Unlink { id, target, json } => {
                let (engine, _root) = create_engine();
                let params = wm_core::page::PageUpdateParams {
                    remove_relates_to: Some(target.clone()),
                    ..Default::default()
                };
                match wm_core::page::update_page(&engine.state, &id, &params) {
                    Ok(_) => {
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "target": target, "status": "unlinked"
                                }))?
                            );
                        } else {
                            println!("Unlinked {} from {}", id, target);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        },
        Commands::Graph { action } => match action {
            GraphAction::Stats { json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                }
                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                let mut type_counts = std::collections::BTreeMap::new();
                for idx in graph.node_indices() {
                    let meta = &graph[idx];
                    let type_str = meta.page_type.as_str().to_string();
                    *type_counts.entry(type_str).or_insert(0) += 1;
                }
                let nodes = graph.node_count();
                let edges = graph.edge_count();
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "nodes": nodes,
                            "edges": edges,
                            "types": type_counts,
                        }))?
                    );
                } else {
                    println!("Graph stats:");
                    println!("  Nodes: {}", nodes);
                    println!("  Edges: {}", edges);
                    if !type_counts.is_empty() {
                        println!("  Types:");
                        for (t, c) in &type_counts {
                            println!("    {}: {}", t, c);
                        }
                    }
                }
            }
            GraphAction::Path {
                start,
                end,
                max_depth,
                json,
            } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                }
                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;
                let start_node = match index.get(&start) {
                    Some(&n) => n,
                    None => {
                        eprintln!("Page not found: {}", start);
                        return Ok(());
                    }
                };
                let end_node = match index.get(&end) {
                    Some(&n) => n,
                    None => {
                        eprintln!("Page not found: {}", end);
                        return Ok(());
                    }
                };
                let max_d = max_depth.unwrap_or(10) as usize;
                let path = wm_core::graph::find_path(graph, index, start_node, end_node, max_d);
                if path.is_empty() {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "path": [], "length": 0, "note": "No path found"
                            }))?
                        );
                    } else {
                        println!("No path found between {} and {}", start, end);
                    }
                } else {
                    let json_path: Vec<serde_json::Value> = path
                        .iter()
                        .map(|(id, title, edge_type)| {
                            serde_json::json!({
                                "id": id,
                                "title": title,
                                "edge_from_parent": edge_type,
                            })
                        })
                        .collect();
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "path": json_path, "length": json_path.len()
                            }))?
                        );
                    } else {
                        println!("Path ({} hops):", json_path.len() - 1);
                        for p in &json_path {
                            println!(
                                "  {}  [{}]",
                                p["id"],
                                p["edge_from_parent"].as_str().unwrap_or("")
                            );
                        }
                    }
                }
            }
            GraphAction::Subgraph {
                center,
                depth,
                json,
            } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                }
                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;
                let depth = depth.unwrap_or(1).min(5);
                let start = match index.get(&center) {
                    Some(&s) => s,
                    None => {
                        eprintln!("Page not found: {}", center);
                        return Ok(());
                    }
                };
                use std::collections::VecDeque;
                let mut visited = std::collections::HashSet::new();
                let mut queue = VecDeque::new();
                let mut nodes = Vec::new();
                let mut edges = Vec::new();
                visited.insert(start);
                queue.push_back((start, 0usize));
                while let Some((current, d)) = queue.pop_front() {
                    if d > depth {
                        continue;
                    }
                    let meta = &graph[current];
                    nodes.push(serde_json::json!({
                        "id": meta.id, "title": meta.title,
                        "type": format!("{:?}", meta.page_type).to_lowercase(),
                        "depth": d,
                    }));
                    for edge in graph.edges(current) {
                        let target = edge.target();
                        edges.push(serde_json::json!({
                            "source": graph[current].id,
                            "target": graph[target].id,
                            "type": format!("{:?}", edge.weight()).to_lowercase(),
                        }));
                        if visited.insert(target) {
                            queue.push_back((target, d + 1));
                        }
                    }
                }
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "center": center, "depth": depth,
                            "nodes": nodes, "edges": edges,
                            "node_count": nodes.len(),
                        }))?
                    );
                } else {
                    println!("Subgraph around {} (depth {}):", center, depth);
                    for n in &nodes {
                        println!("  {}  {}", n["id"], n["title"]);
                    }
                    println!("{} nodes, {} edges", nodes.len(), edges.len());
                }
            }
            GraphAction::Neighbors {
                id,
                query,
                json,
            } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                }

                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;

                if let Some(&start) = index.get(&id) {
                    let mut neighbors = Vec::new();
                    for edge in graph.edges(start) {
                        let target = edge.target();
                        let meta = &graph[target];
                        let priority = edge.weight().priority() as f64;
                        let score = if let Some(ref q) = query {
                            let q_lower = q.to_lowercase();
                            let title = meta.title.to_lowercase();
                            let relevance = if title == q_lower { 8.0 }
                                else if title.contains(&q_lower) { 4.0 }
                                else if meta.tags.iter().any(|t| t.to_lowercase().contains(&q_lower)) { 2.2 }
                                else { 0.0 };
                            priority * (1.0 + relevance)
                        } else {
                            priority
                        };
                        neighbors.push(serde_json::json!({
                            "id": meta.id, "title": meta.title, "score": score
                        }));
                    }
                    neighbors.sort_by(|a, b| {
                        let sa = a["score"].as_f64().unwrap_or(0.0);
                        let sb = b["score"].as_f64().unwrap_or(0.0);
                        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(
                                &serde_json::json!({ "neighbors": neighbors, "total": neighbors.len() })
                            )?
                        );
                    } else {
                        for n in &neighbors {
                            println!("  {}  {}", n["id"], n["title"]);
                        }
                        println!("{} neighbors", neighbors.len());
                    }
                } else {
                    eprintln!("Page not found: {}", id);
                }
            }
        },
        Commands::Source { action } => {
            let engine = Arc::new(MainEngine::new());
            match action {
                SourceAction::List { state, json } => {
                    let state = state.as_deref();
                    match wm_core::source::list_sources(&engine.state, state) {
                        Ok(sources) => {
                            if json {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(
                                        &serde_json::json!({ "sources": sources, "total": sources.len() })
                                    )?
                                );
                            } else {
                                for s in &sources {
                                    println!("  {}  [{:?}]", s["id"], s["state"]);
                                }
                                println!("{} sources", sources.len());
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                SourceAction::Status { id, json } => {
                    match wm_core::source::source_status(&engine.state, &id) {
                        Ok(status) => {
                            if json {
                                println!("{}", serde_json::to_string_pretty(&status)?);
                            } else {
                                println!("ID:       {}", status["id"]);
                                println!("State:    {}", status["state"]);
                                println!("Pages:    {}", status["page_count"]);
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                SourceAction::Remove { id, .. } => {
                    match wm_core::source::remove_source(&engine.state, &id) {
                        Ok(_) => println!("Removed source: {}", id),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                SourceAction::Discover { json } => {
                    let (dirs, exts) = {
                        let config = engine.state.config.read().map_err(|_| anyhow::anyhow!("config lock poisoned"))?;
                        (config.source_dirs.clone(), config.source_extensions.clone())
                    };
                    match wm_core::source::discover_sources(&engine.state, &dirs, Some(&exts)) {
                        Ok(discovered) => {
                            if json {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(
                                        &serde_json::json!({ "discovered": discovered, "total": discovered.len() })
                                    )?
                                );
                            } else {
                                for id in &discovered {
                                    println!("  {}", id);
                                }
                                println!("Discovered {} sources", discovered.len());
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
        }
        Commands::Task { action } => {
            match action {
                TaskAction::Board { json } => {
                    let (engine, _root) = create_engine();
                    let board = wm_core::task::build_task_board(&engine.state);
                    let board_json: serde_json::Value = board.into();
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "columns": board_json["columns"],
                                "counts": board_json["counts"],
                            }))?
                        );
                    } else {
                        let columns = board_json["columns"].as_object().cloned().unwrap_or_default();
                        let column_order = [
                            "draft", "todo", "in-progress", "in-review", "blocked",
                            "done", "reviewed", "approved", "superseded", "cancelled",
                        ];
                        let terminal_statuses = ["done", "reviewed", "approved", "superseded", "cancelled"];
                        let mut any_active = false;
                        for col_name in &column_order {
                            let items = columns.get(*col_name).and_then(|v| v.as_array());
                            let count = items.map(|v| v.len()).unwrap_or(0);
                            if count == 0 { continue; }
                            if terminal_statuses.contains(col_name) && count > 5 {
                                let label = col_name.to_uppercase().replace('-', " ");
                                println!("{} ({}) — use --all to list", label, count);
                                continue;
                            }
                            any_active = true;
                            let label = col_name.to_uppercase().replace('-', " ");
                            println!("{} ({})", label, count);
                            if let Some(items) = items {
                                for t in items {
                                    let p = t["priority"].as_str().unwrap_or(" ").chars().next().unwrap_or(' ');
                                    println!("  {}  {}", p, t["title"].as_str().unwrap_or(""));
                                }
                            }
                        }
                        if !any_active {
                            println!("(no active tasks — use --all to see terminal columns)");
                        }
                    }
                }
            }
        }
        Commands::Log { action } => match action {
            LogAction::Recent { count, json } => {
                let log_path = std::path::Path::new(".wm").join("log.jsonl");
                let content = std::fs::read_to_string(&log_path).unwrap_or_default();
                let all_lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
                let total = all_lines.len();
                let start = total.saturating_sub(count);
                let lines: Vec<&str> = all_lines[start..].to_vec();
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({ "entries": lines, "total": total })
                        )?
                    );
                } else {
                    for l in &lines {
                        println!("{}", l);
                    }
                }
            }
            LogAction::Since { marker, json } => {
                let log_path = std::path::Path::new(".wm").join("log.jsonl");
                let content = std::fs::read_to_string(&log_path).unwrap_or_default();
                let lines: Vec<&str> = content
                    .lines()
                    .skip_while(|line| !line.contains(&marker))
                    .skip(1)
                    .collect();
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({ "entries": lines, "total": lines.len() })
                        )?
                    );
                } else {
                    for l in &lines {
                        println!("{}", l);
                    }
                }
            }
            LogAction::Filter { text, json } => {
                let log_path = std::path::Path::new(".wm").join("log.jsonl");
                let content = std::fs::read_to_string(&log_path).unwrap_or_default();
                let lines: Vec<&str> = content
                    .lines()
                    .filter(|line| line.to_lowercase().contains(&text.to_lowercase()))
                    .collect();
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({ "entries": lines, "total": lines.len() })
                        )?
                    );
                } else {
                    for l in &lines {
                        println!("{}", l);
                    }
                }
            }
        },
        Commands::Lint { action } => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(".wm").join("wiki");
            let engine = Arc::new(MainEngine::new());
            if wiki_dir.exists() {
                rebuild_from_engine(&engine, &wiki_dir);
            }
            match action {
                LintAction::Check { json } => {
                    let snapshot = engine.state.graph.load();
                    let graph = &snapshot.0;
                    let mut orphans = 0;
                    for idx in graph.node_indices() {
                        let inbound = graph
                            .edges_directed(idx, petgraph::Direction::Incoming)
                            .count();
                        if inbound == 0 {
                            orphans += 1;
                        }
                    }
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "nodes": graph.node_count(), "edges": graph.edge_count(), "orphans": orphans
                            }))?
                        );
                    } else {
                        println!("Lint check complete:");
                        println!("  Nodes: {}", graph.node_count());
                        println!("  Edges: {}", graph.edge_count());
                        println!("  Orphans: {}", orphans);
                    }
                }
                LintAction::Fix { json } => {
                    let snapshot = engine.state.graph.load();
                    let fixed = wm_core::graph::auto_fix_missing_frontmatter(&snapshot.0, &engine.state.write_channel);
                    if json {
                        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                            "fixed": fixed,
                        }))?);
                    } else {
                        println!("Fixed {} issue(s)", fixed);
                    }
                }
            }
        }
        Commands::Validate { json } => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(".wm").join("wiki");
            let engine = Arc::new(MainEngine::new());
            if wiki_dir.exists() {
                rebuild_from_engine(&engine, &wiki_dir);
            }
            let snapshot = engine.state.graph.load();
            let graph = &snapshot.0;
            if json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "nodes": graph.node_count(),
                    "edges": graph.edge_count(),
                    "status": if graph.node_count() > 0 { "pass" } else { "empty" },
                }))?);
            } else {
                println!(
                    "Validation complete: {} nodes, {} edges",
                    graph.node_count(),
                    graph.edge_count()
                );
                if graph.node_count() > 0 {
                    println!("Status: pass");
                }
            }
        }
        Commands::Index { action } => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            match action {
                IndexAction::Rebuild {
                    skip_embed,
                    batch_size,
                } => {
                    let wiki_dir = root.join(".wm").join("wiki");
                    if !wiki_dir.exists() {
                        anyhow::bail!("No wiki directory found. Run 'wm init' first.");
                    }
                    println!("Rebuilding index...");
                    let engine = Arc::new(MainEngine::new());
                    let count = rebuild_from_engine(&engine, &wiki_dir);
                    println!("  Graph: {} nodes", count);

                    let sections = wm_core::graph::build_sections_from_wiki(&wiki_dir);
                    engine
                        .state
                        .section_corpus
                        .store(Arc::new(sections.clone()));
                    println!("  Sections: {}", sections.len());

                    let docs: Vec<wm_core::search::IndexedDoc> = sections
                        .iter()
                        .map(|s| wm_core::search::IndexedDoc {
                            id: s.section_id.clone(),
                            fields: vec![
                                wm_core::search::Field::new("header", &s.header, 4.0),
                                wm_core::search::Field::new("body", &s.body, 1.0),
                                wm_core::search::Field::new("id", &s.section_id, 0.0),
                                wm_core::search::Field::new("title", &s.title, 0.0),
                                wm_core::search::Field::new("tags", &s.tags.join(" "), 0.0),
                            ],
                        })
                        .collect();
                    let bm25 = wm_core::search::Bm25Index::build(docs);
                    engine.state.bm25_index.store(Arc::new(bm25));
                    println!("  BM25 index built");

                    if !skip_embed && engine.state.embedder.is_loaded() {
                        let old_hashes = engine.state.vector_store.hashes.load_full();
                        let old_entries = engine.state.vector_store.entries.load_full();
                        match wm_core::embed::rebuild_embeddings_skip_unchanged(
                            &*engine.state.embedder,
                            &sections,
                            &old_hashes,
                            Some(&old_entries),
                            batch_size,
                        ) {
                            Ok((new_entries, new_hashes)) => {
                                engine.state.vector_store.replace_entries_and_hashes(new_entries, new_hashes);
                                engine.state.vector_store.save_to_disk().ok();
                                println!("  Embeddings built");
                            }
                            Err(e) => println!("  Embedding failed: {}", e),
                        }
                    } else if !engine.state.embedder.is_loaded() && !skip_embed {
                        println!("  Skipping embeddings — no model loaded.");
                    }

                    #[cfg(feature = "code-intel")]
                    {
                        use wm_core::code_intel::services::ingest_service::rebuild_code_index;
                        use wm_core::code_intel::services::CodeIndexDb;

                        let db_dir = root.join(".wm").join("state");
                        let db_path = db_dir.join("code.db");
                        std::fs::create_dir_all(&db_dir).ok();

                        match CodeIndexDb::open(db_path) {
                            Ok(db) => {
                                match rebuild_code_index(&db, &root) {
                                    Ok((files, syms, deps, _errors)) => {
                                        tracing::info!("Code index rebuilt: {} files, {} symbols, {} deps", files, syms, deps);
                                        println!("  Code index: {} files, {} symbols, {} deps", files, syms, deps);
                                    }
                                    Err(e) => tracing::warn!("Code index rebuild failed: {}", e),
                                }
                            }
                            Err(e) => tracing::warn!("Failed to open code.db: {}", e),
                        }
                    }

                    println!("Rebuild complete.");
                }
                IndexAction::Code { skip_hash_check } => {
                    if skip_hash_check {
                        tracing::info!("skip_hash_check flag acknowledged — hash-check behavior is always active");
                    }
                    let root = config::detect_project_root()
                        .ok_or_else(|| anyhow::anyhow!("No project root found"))?;
                    #[cfg(feature = "code-intel")]
                    {
                        use wm_core::code_intel::services::ingest_service::rebuild_code_index;
                        use wm_core::code_intel::services::CodeIndexDb;

                        let db_dir = root.join(".wm").join("state");
                        let db_path = db_dir.join("code.db");
                        std::fs::create_dir_all(&db_dir).ok();
                        let db = CodeIndexDb::open(db_path)
                            .map_err(|e| anyhow::anyhow!("Failed to open code db: {}", e))?;

                        println!("Rebuilding code index...");
                        match rebuild_code_index(&db, &root) {
                            Ok((files, syms, deps, errors)) => {
                                println!("  {} files scanned", files);
                                println!("  {} symbols indexed", syms);
                                println!("  {} dependencies indexed", deps);
                                if !errors.is_empty() {
                                    println!("  {} errors (see logs)", errors.len());
                                }
                            }
                            Err(e) => anyhow::bail!("Code index rebuild failed: {}", e),
                        }
                    }
                    #[cfg(not(feature = "code-intel"))]
                    {
                        let _ = root;
                        anyhow::bail!("code-intel feature not enabled. Rebuild with --features code-intel.");
                    }
                }
                IndexAction::Embed {
                    batch_size,
                    force: _force,
                } => {
                    let engine = Arc::new(MainEngine::new());
                    if !engine.state.embedder.is_loaded() {
                        anyhow::bail!("No embedding model loaded. Run 'wm model download' first.");
                    }
                    let sections = engine.state.section_corpus.load();
                    if sections.is_empty() {
                        anyhow::bail!("No sections found. Run 'wm index rebuild' first.");
                    }
                    let old_hashes = engine.state.vector_store.hashes.load_full();
                    let old_entries = engine.state.vector_store.entries.load_full();
                    match wm_core::embed::rebuild_embeddings_skip_unchanged(
                        &*engine.state.embedder,
                        &sections,
                        &old_hashes,
                        Some(&old_entries),
                        batch_size,
                    ) {
                        Ok((new_entries, new_hashes)) => {
                            engine.state.vector_store.replace_entries_and_hashes(new_entries, new_hashes);
                            engine.state.vector_store.save_to_disk().ok();
                            println!("Embedding complete.");
                        }
                        Err(e) => println!("Embedding failed: {}", e),
                    }
                }
            }
        }
        Commands::Time { action } => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(".wm").join("wiki");
            let engine = Arc::new(MainEngine::new());
            if wiki_dir.exists() {
                let snapshot = engine.state.graph.load();
                if snapshot.0.node_count() == 0 {
                    rebuild_from_engine(&engine, &wiki_dir);
                }
            }
            match action {
                TimeAction::Start { id, .. } => {
                            let now = chrono::Utc::now().to_rfc3339();
                    let params = wm_core::page::PageUpdateParams {
                        time_started: Some(now.clone()),
                        ..Default::default()
                    };
                    match wm_core::page::update_page(&engine.state, &id, &params) {
                        Ok(_) => println!("Time tracking started for {} at {}", id, now),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                TimeAction::Stop { id, .. } => {
                    
                    let content = std::fs::read_to_string(
                        format!(".wm/wiki/{}.md", id.replace(':', "/")),
                    )
                    .unwrap_or_default();
                    let (fm, _) = wm_core::parser::extract_frontmatter(&content);
                    let time_started = fm
                        .as_ref()
                        .and_then(|f| f.time_started.as_deref())
                        .unwrap_or("");
                    let now = chrono::Utc::now();
                    let elapsed =
                        if let Ok(started) = chrono::DateTime::parse_from_rfc3339(time_started) {
                            let dur = now.signed_duration_since(started);
                            format!("{}h {}m", dur.num_hours(), dur.num_minutes() % 60)
                        } else {
                            "0h 0m".into()
                        };
                    let params = wm_core::page::PageUpdateParams {
                        time_spent: Some(elapsed.clone()),
                        ..Default::default()
                    };
                    match wm_core::page::update_page(&engine.state, &id, &params) {
                        Ok(_) => println!("Time stopped for {}: {}", id, elapsed),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                TimeAction::Add { id, duration, .. } => {
                    let params = wm_core::page::PageUpdateParams {
                        time_spent: Some(duration.clone()),
                        ..Default::default()
                    };
                    match wm_core::page::update_page(&engine.state, &id, &params) {
                        Ok(_) => println!("Time added for {}: {}", id, duration),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                TimeAction::Report { json } => {
                    let snapshot = engine.state.graph.load();
                    let graph = &snapshot.0;
                    let mut total_hours = 0f64;
                    let mut tasks = Vec::new();
                    for idx in graph.node_indices() {
                        let meta = &graph[idx];
                        if meta.page_type != wm_core::engine::PageType::Task {
                            continue;
                        }
                        let content = std::fs::read_to_string(&meta.path).unwrap_or_default();
                        let (fm, _) = wm_core::parser::extract_frontmatter(&content);
                        let time_spent = fm
                            .as_ref()
                            .and_then(|f| f.time_spent.as_deref())
                            .unwrap_or("");
                        if let Some(h) = time_spent
                            .split('h')
                            .next()
                            .and_then(|s| s.trim().parse::<f64>().ok())
                        {
                            total_hours += h;
                        }
                        tasks.push(serde_json::json!({ "id": meta.id, "title": meta.title, "time_spent": time_spent }));
                    }
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(
                                &serde_json::json!({ "tasks": tasks, "total_hours": total_hours })
                            )?
                        );
                    } else {
                        println!("Time report:");
                        for t in &tasks {
                            println!("  {}  {}", t["time_spent"], t["id"]);
                        }
                        println!("Total: {:.1}h", total_hours);
                    }
                }
            }
        }
        Commands::Model { action } => match action {
            ModelAction::Download { name, .. } => {
                #[cfg(feature = "onnx")]
                {
                    let home = std::env::var("HOME")
                        .or_else(|_| std::env::var("USERPROFILE"))
                        .unwrap_or_else(|_| ".".into());
                    let models_dir = std::path::PathBuf::from(home).join(".wm").join("models");
                    match wm_core::embed::download_model(&name, &models_dir) {
                        Ok(dir) => println!("Model downloaded to {}", dir.display()),
                        Err(e) => eprintln!("Download failed: {}", e),
                    }
                }
                #[cfg(not(feature = "onnx"))]
                {
                    let _ = name;
                    eprintln!("Model download requires the 'onnx' feature. Rebuild with --features onnx.");
                }
            }
            ModelAction::List { .. } => {
                let engine = Arc::new(MainEngine::new());
                let loaded = engine.state.embedder.is_loaded();
                let model_name = engine.state.embedder.model_name().to_string();
                let indexed = engine.state.vector_store.snapshot().len();
                println!(
                    "Active model: {} {}",
                    model_name,
                    if loaded { "(loaded)" } else { "(not loaded)" }
                );
                println!("Sections indexed: {}", indexed);
                println!();
                println!("Cached models:");
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                let models_dir = std::path::PathBuf::from(home).join(".wm").join("models");
                if models_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&models_dir) {
                        for e in entries.flatten() {
                            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                let n = e.file_name().to_string_lossy().to_string();
                                println!(
                                    "  - {} {}",
                                    n,
                                    if n == model_name { "(active)" } else { "" }
                                );
                            }
                        }
                    }
                }
                println!();
                println!("Available for download:");
                println!("  - bge-small-en-v1.5 (384-dim, 134 MB, recommended)");
                println!("  - bge-base-en-v1.5 (768-dim, 438 MB)");
                println!("  - all-MiniLM-L6-v2 (384-dim, 90 MB)");
            }
            ModelAction::Status { .. } => {
                let engine = Arc::new(MainEngine::new());
                println!("Model:            {}", engine.state.embedder.model_name());
                println!("Loaded:           {}", engine.state.embedder.is_loaded());
                println!("Dimensions:       {}", engine.state.embedder.output_dim());
                println!(
                    "Sections indexed: {}",
                    engine.state.vector_store.snapshot().len()
                );
            }
            ModelAction::Remove { name, .. } => {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                let model_dir = std::path::PathBuf::from(home)
                    .join(".wm")
                    .join("models")
                    .join(&name);
                if model_dir.exists() {
                    std::fs::remove_dir_all(&model_dir)?;
                    println!("Removed model: {}", name);
                } else {
                    println!("Model not found: {}", name);
                }
            }
        },
        Commands::Version => {
            println!("wm {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::MigrateMemory => {
            let (engine, _) = create_engine();
            match wm_core::page::migrate_old_memory_json(&engine.state) {
                Ok(count) => println!("Migrated {} memory entries to wiki pages.", count),
                Err(e) => eprintln!("Migration error: {}", e),
            }
        }
    }

    Ok(())
}
