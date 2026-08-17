use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};
use petgraph::visit::EdgeRef;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use constants::fs::*;
use constants::health::*;
use constants::wiki::*;
use wm_constants::*;
use wm_core::config::{self, GitTracking, ProjectConfig};

use wm_core::engine::MainEngine;

mod mcp_server;

mod tui;

mod constants;
mod models;

#[derive(Parser)]
#[command(name = "wm", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, global = true)]
    tui: bool,
}

/// MCP transport selection for `wm mcp`.
#[derive(Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
enum McpTransport {
    /// In-process stdio transport (unchanged default).
    #[default]
    Stdio,
    /// HTTP transport: spawn the wm-server daemon and serve the tool surface
    /// at POST /mcp on the given (or default) port.
    Http,
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
    },

    Web {
        #[arg(long)]
        port: Option<u16>,
    },

    Mcp {
        #[arg(long)]
        project: Option<PathBuf>,

        /// MCP transport: stdio (in-process, default) or http (spawns the
        /// wm-server daemon, which serves the same tool surface at POST /mcp).
        #[arg(long, value_enum, default_value_t = McpTransport::Stdio)]
        transport: McpTransport,

        /// Port for `--transport http` (defaults to the wm-server default).
        #[arg(long)]
        port: Option<u16>,
    },

    /// Generate MCP config for an agent platform.
    ///
    /// Platforms: opencode, kiro, claude, codex, cursor, antigravity.
    /// Use `all` to generate every platform's config.
    ///
    /// Every setup also emits the query-before-grep agent hook: an
    /// instruction to query `wm_graph`/`wm_search` before falling back to
    /// raw file greps. `--strict` additionally gates raw file reads behind a
    /// permission rule on platforms that support one (opencode, claude);
    /// on the other platforms strict is instruction-only.
    Setup {
        platform: String,

        /// Emit an enforced permission gate: raw file reads (read/grep/bash)
        /// require explicit approval until a wm_graph/wm_search query has
        /// been issued (per-platform capability permitting).
        #[arg(long)]
        strict: bool,
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

    /// Page operations.
    ///
    /// Page content is piped via stdin: `page create` and `page update` read
    /// stdin only (there is no --content flag), e.g.
    /// `echo '# Body' | wm-cli page create concepts/hello "Hello"`.
    Page {
        #[command(subcommand)]
        action: PageAction,
    },

    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },

    Health {
        #[command(subcommand)]
        action: HealthAction,
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

    Status {
        #[arg(long)]
        json: bool,
    },

    Config {
        #[command(subcommand)]
        action: ConfigAction,
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

    /// Create a wiki page.
    ///
    /// Content is read from STDIN only — there is no --content flag. Pipe the
    /// body in:
    ///
    ///   echo '# Hello' | wm-cli page create concepts/hello "Hello"
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

    /// Update a wiki page.
    ///
    /// The JSON update payload is read from STDIN only — there is no --content
    /// flag. Pipe the payload in:
    ///
    ///   echo '{"title": "New Title"}' | wm-cli page update wiki:concepts:hello
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

    Affected {
        node: String,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        json: bool,
    },

    Stats {
        #[arg(long)]
        json: bool,
    },

    /// Export a snapshot of the wiki graph. Exports are snapshots only —
    /// never a storage format; markdown pages stay canonical.
    ///
    /// Formats:
    ///   json     — full graph dump in the `wm_graph.full` wire shape
    ///   graphml  — directed GraphML (Gephi / yEd)
    ///   obsidian — a vault dir with one page per wiki page and
    ///              `[[wikilink]]` lines matching outbound edges
    ///
    /// `json` and `graphml` print to stdout unless `--out <file>` is given.
    /// `obsidian` requires `--out <vault-dir>`.
    Export {
        /// Export format: json | graphml | obsidian
        format: ExportFormat,
        /// Output file (json/graphml) or vault directory (obsidian).
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

/// Snapshot export formats.
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum ExportFormat {
    Json,
    Graphml,
    Obsidian,
}

#[derive(Subcommand)]
enum HealthAction {
    Audit {
        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        fix: bool,

        #[arg(long, default_value = "text")]
        format: String,
    },
}

fn health_status_label(empty_count: usize, broken_count: usize) -> &'static str {
    if empty_count == 0 && broken_count == 0 {
        return HEALTH_STATUS_CLEAN;
    }
    HEALTH_STATUS_ISSUES
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

    Remove {
        id: String,
        #[arg(long)]
        json: bool,
    },

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
    Start {
        id: String,
        #[arg(long)]
        json: bool,
    },

    Stop {
        id: String,
        #[arg(long)]
        json: bool,
    },

    Add {
        id: String,
        duration: String,
        #[arg(long)]
        json: bool,
    },

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
        /// Only process sections modified after this date (ISO 8601, e.g. 2026-07-01)
        #[arg(long)]
        since: Option<String>,
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
    Download {
        name: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
    Remove {
        name: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Get {
        key: String,
        #[arg(long)]
        json: bool,
    },
    Set {
        key: String,
        value: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
}

fn setup_logging() {
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(false);

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let log_dir = std::path::PathBuf::from(home).join(WM_DIR).join("logs");
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
    let engine = build_engine(&root);
    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
    (engine, wiki_dir)
}

fn build_engine(root: &Path) -> Arc<MainEngine> {
    let config = wm_core::config::load_config(root).unwrap_or_default();
    let engine = Arc::new(MainEngine::with_root(config, root.to_path_buf()));

    let old_memory_dir = root.join(WM_DIR).join("memory");
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

    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
    if wiki_dir.exists() {
        rebuild_from_engine(&engine, &wiki_dir);
        engine
            .state
            .stale_flag
            .store(false, std::sync::atomic::Ordering::Release);
    }
    engine
}

struct EngineHandle {
    registry: Arc<wm_core::ToolRegistry>,
}

fn engine_handle() -> anyhow::Result<Arc<EngineHandle>> {
    static HANDLE: std::sync::OnceLock<anyhow::Result<Arc<EngineHandle>>> =
        std::sync::OnceLock::new();
    let handle = HANDLE.get_or_init(|| {
        let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
        let _ = std::env::set_current_dir(&root);
        let (engine, _) = create_engine();
        let mut registry = wm_core::ToolRegistry::new();
        wm_core::mcp::tools::register_all_tools(&mut registry, engine.state.clone());
        Ok(Arc::new(EngineHandle {
            registry: Arc::new(registry),
        }))
    });
    match handle {
        Ok(arc) => Ok(Arc::clone(arc)),
        Err(e) => Err(anyhow::anyhow!("{e:#}")),
    }
}

async fn call_tool(name: &str, arguments: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let handle = engine_handle().map_err(|e| anyhow::anyhow!("{e:#}"))?;
    match handle.registry.dispatch_async(name, arguments).await {
        Ok(data) => Ok(data),
        Err(e) => Err(anyhow::anyhow!("[{}] {}", e.code, e.message)),
    }
}

fn rebuild_from_engine(engine: &Arc<MainEngine>, wiki_dir: &Path) -> usize {
    let ct = engine
        .state
        .config
        .read()
        .unwrap()
        .custom_edge_types
        .clone();
    let count = wm_core::graph::rebuild_graph_snapshot(&engine.state.graph, wiki_dir, &ct);

    let _ = wm_core::graph::auto_generate_index(wiki_dir, &engine.state.graph.load().0);

    let sections = wm_core::graph::build_sections_from_wiki(wiki_dir);
    engine
        .state
        .section_corpus
        .store(Arc::new(sections.clone()));
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

fn sync_agent_files(
    root: &std::path::Path,
    platforms: &[String],
    _force: bool,
) -> Result<(), anyhow::Error> {
    use std::collections::HashSet;
    let targets: Vec<&str> = if platforms.is_empty() {
        vec![
            "claude", "opencode", "kiro", "gemini", "copilot", "agents", "reasonix",
        ]
    } else {
        platforms.iter().map(|s| s.as_str()).collect()
    };

    let template_map: [(&str, &str); 6] = [
        ("CLAUDE.md", "shims/CLAUDE.md"),
        ("AGENTS.md", "shims/AGENTS.md"),
        ("GEMINI.md", "shims/GEMINI.md"),
        (
            ".github/copilot-instructions.md",
            "shims/copilot-instructions.md",
        ),
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
                eprintln!(
                    "Unknown platform: {}. Use `wm init --platform <name>` for MCP config.",
                    plat
                );
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
            println!(
                "  {} — also handled by {} platform (same file)",
                output_filename, plat
            );
        }
    }

    if targets.contains(&"opencode") {
        if let Some(file) = wm_core::embed_files::EmbeddedFiles::get("shims/OPENCODE.md") {
            if let Ok(content) = std::str::from_utf8(file.data.as_ref()) {
                if std::fs::write(root.join("OPENCODE.md"), content).is_ok()
                    && !written.contains("OPENCODE.md")
                {
                    written.insert("OPENCODE.md".to_string());
                    println!("  OPENCODE.md — agent instruction file generated");
                }
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

fn setup_platform_mcp(root: &Path, platform: &str) -> Result<(), anyhow::Error> {
    let resolved = resolve_mcp_binary();

    match platform {
        "opencode" => {
            let cfg = root.join("opencode.json");
            let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/opencode.json")
                .ok_or_else(|| {
                    anyhow::anyhow!("Embedded config not found: configs/opencode.json")
                })?;
            let mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
            let mcp = patch_mcp_command(mcp);
            wm_core::platform_service::write_merged_json(&cfg, mcp)?;

            if let Some(file) = wm_core::embed_files::EmbeddedFiles::get("shims/OPENCODE.md") {
                if let Ok(content) = std::str::from_utf8(file.data.as_ref()) {
                    std::fs::write(root.join("OPENCODE.md"), content)?;
                }
            }

            let skills_dir = root.join(".opencode").join("skills");
            sync_skills_to(&skills_dir)?;
            println!(
                "  {} — OpenCode MCP config (+ skills synced to .opencode/skills/)",
                cfg.display()
            );
        }
        "kiro" => {
            let cfg_dir = root.join(".kiro").join("settings");
            std::fs::create_dir_all(&cfg_dir).ok();
            let cfg = cfg_dir.join("mcp.json");
            let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/kiro_mcp.json")
                .ok_or_else(|| {
                    anyhow::anyhow!("Embedded config not found: configs/kiro_mcp.json")
                })?;
            let mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
            let mcp = patch_mcp_command(mcp);
            wm_core::platform_service::write_merged_json(&cfg, mcp)?;

            let kiro_skills = root.join(".kiro").join("skills");
            sync_skills_to(&kiro_skills)?;

            let steering_dir = root.join(".kiro").join("steering");
            std::fs::create_dir_all(&steering_dir).ok();
            if let Some(template) = wm_core::embed_files::EmbeddedFiles::get("steering/wiki-mem.md")
            {
                std::fs::write(steering_dir.join("wiki-mem.md"), &template.data).ok();
            }

            println!("  {} — Kiro MCP config (+ skills, steering)", cfg.display());
        }
        "claude" => {
            let cfg_file = root.join(".mcp.json");
            let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/dot_mcp.json")
                .ok_or_else(|| {
                    anyhow::anyhow!("Embedded config not found: configs/dot_mcp.json")
                })?;
            let mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
            let mcp = patch_mcp_command(mcp);
            wm_core::platform_service::write_merged_json(&cfg_file, mcp)?;

            let claude_skills = root.join(".claude").join("skills");
            sync_skills_to(&claude_skills)?;
            println!(
                "  {} — Claude MCP config (+ skills synced to .claude/skills/)",
                cfg_file.display()
            );
        }
        "codex" => {
            let d = root.join(".codex");
            std::fs::create_dir_all(&d).ok();
            let cfg_file = d.join("config.toml");
            wm_core::platform_service::write_toml_config(&cfg_file, &resolved)?;
            let skills_dir = root.join(".codex").join("skills");
            sync_skills_to(&skills_dir)?;
            println!(
                "  {} — Codex MCP config (TOML) (+ skills synced to .codex/skills/)",
                cfg_file.display()
            );
        }
        "cursor" => {
            let cfg_dir = root.join(".cursor");
            std::fs::create_dir_all(&cfg_dir).ok();
            let cfg = cfg_dir.join("mcp.json");
            let embedded = wm_core::embed_files::EmbeddedFiles::get("configs/cursor_mcp.json")
                .ok_or_else(|| {
                    anyhow::anyhow!("Embedded config not found: configs/cursor_mcp.json")
                })?;
            let mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
            let mcp = patch_mcp_command(mcp);
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
                .ok_or_else(|| {
                    anyhow::anyhow!("Embedded config not found: configs/antigravity_mcp.json")
                })?;
            let mcp: serde_json::Value = serde_json::from_slice(&embedded.data)?;
            let mcp = patch_mcp_command(mcp);
            wm_core::platform_service::write_merged_json(&cfg, mcp)?;

            let skills_dir = root.join(".agents").join("skills");
            sync_skills_to(&skills_dir)?;
            println!(
                "  {} — Antigravity MCP config (+ skills synced to .agents/skills/)",
                cfg.display()
            );
        }
        _ => {}
    }
    Ok(())
}

/// Emit the query-before-grep agent hook for a platform:
/// a shared instruction file telling the agent to query `wm_graph`/`wm_search`
/// before raw file greps, wired into each platform's instruction/permission
/// surface. In `--strict` mode platforms with a permission-rule mechanism
/// (opencode, claude) additionally gate raw file reads with an approval rule;
/// the others get the guidance as instructions only (documented on stdout).
fn setup_query_before_grep(
    root: &std::path::Path,
    platform: &str,
    strict: bool,
) -> Result<(), anyhow::Error> {
    use wm_core::agent_hooks::{
        claude_strict_settings, instruction_import_line, opencode_with_hook,
        query_before_grep_content, QUERY_BEFORE_GREP_REL_PATH,
    };

    let hook_path = root.join(QUERY_BEFORE_GREP_REL_PATH);
    if let Some(parent) = hook_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = query_before_grep_content(strict);
    std::fs::write(&hook_path, &content)?;

    match platform {
        "opencode" => {
            let cfg_path = root.join("opencode.json");
            if let Some(existing) = std::fs::read_to_string(&cfg_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            {
                let patched = opencode_with_hook(existing, strict);
                wm_core::platform_service::write_merged_json(&cfg_path, patched)?;
            }
            println!(
                "  {} — query-before-grep instruction hooked into opencode.json",
                QUERY_BEFORE_GREP_REL_PATH
            );
            if strict {
                println!(
                    "    strict: read/grep/bash permission-gated (ask) — raw reads require approval"
                );
            }
        }
        "claude" | "codex" => {
            let import = instruction_import_line();
            let target = root.join("CLAUDE.md");
            if let Ok(existing) = std::fs::read_to_string(&target) {
                if !existing.contains(&import) {
                    std::fs::write(&target, format!("{}\n\n{}", existing.trim_end(), import))?;
                }
            }
            if strict && platform == "claude" {
                let settings_dir = root.join(".claude");
                std::fs::create_dir_all(&settings_dir)?;
                wm_core::platform_service::write_merged_json(
                    &settings_dir.join("settings.json"),
                    claude_strict_settings(),
                )?;
                println!(
                    "  .claude/settings.json — strict: Read(//**)/Bash(*) permission-gated (ask)"
                );
            }
            println!(
                "  {} — query-before-grep instruction imported into CLAUDE.md",
                QUERY_BEFORE_GREP_REL_PATH
            );
            if strict && platform == "codex" {
                println!("    strict: instruction-only (codex has no per-tool permission rules)");
            }
        }
        "kiro" => {
            let import = instruction_import_line();
            let target = root.join("AGENTS.md");
            if let Ok(existing) = std::fs::read_to_string(&target) {
                if !existing.contains(&import) {
                    std::fs::write(&target, format!("{}\n\n{}", existing.trim_end(), import))?;
                }
            }
            println!(
                "  {} — query-before-grep instruction imported into AGENTS.md",
                QUERY_BEFORE_GREP_REL_PATH
            );
            if strict {
                println!("    strict: instruction-only (kiro has no permission rules)");
            }
        }
        "cursor" => {
            let rules_dir = root.join(".cursor").join("rules");
            std::fs::create_dir_all(&rules_dir)?;
            let rule = format!(
                "---\ndescription: Query wm_graph/wm_search before raw file greps\n---\n\n{}",
                content
            );
            std::fs::write(rules_dir.join("query-before-grep.mdc"), rule)?;
            println!("  .cursor/rules/query-before-grep.mdc — query-before-grep cursor rule");
            if strict {
                println!("    strict: instruction-only (cursor has no permission rules)");
            }
        }
        "antigravity" => {
            let skill_dir = root
                .join(".agents")
                .join("skills")
                .join("query-before-grep");
            std::fs::create_dir_all(&skill_dir)?;
            let skill = format!(
                "---\nname: query-before-grep\ndescription: Query wm_graph/wm_search before falling back to raw file greps\n---\n\n{}",
                content
            );
            std::fs::write(skill_dir.join("SKILL.md"), skill)?;
            println!(
                "  .agents/skills/query-before-grep/SKILL.md — query-before-grep skill"
            );
            if strict {
                println!("    strict: instruction-only (antigravity has no permission rules)");
            }
        }
        _ => {}
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

fn resolve_mcp_binary() -> String {
    "wm-cli".into()
}

fn patch_mcp_command(mut cfg: serde_json::Value) -> serde_json::Value {
    let resolved = resolve_mcp_binary();

    if let Some(cmd_arr) = cfg
        .pointer_mut("/mcp/wm/command")
        .and_then(|v| v.as_array_mut())
    {
        if let Some(first) = cmd_arr.first_mut() {
            *first = serde_json::Value::String(resolved);
        }
        return cfg;
    }

    if let Some(cmd) = cfg.pointer_mut("/mcpServers/wm/command") {
        if cmd.is_string() {
            *cmd = serde_json::Value::String(resolved);
        }
    }

    cfg
}

fn run_web(requested_port: u16, server_binary: &Path, project_root: &Path) -> anyhow::Result<()> {
    info!("Starting wm-server on port {requested_port}");
    let mut child = std::process::Command::new(server_binary)
        .arg("--port")
        .arg(requested_port.to_string())
        .current_dir(project_root)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start wm-server: {e}"))?;
    println!(
        "wm-server launched on port {requested_port} (project {})",
        project_root.display()
    );

    match child.wait() {
        Ok(status) if !status.success() => {
            let code = status.code().unwrap_or(1);
            eprintln!(
                "wm-server exited with code: {code} (project: {})",
                project_root.display()
            );
            std::process::exit(code);
        }
        Err(e) => eprintln!("Server process error: {e}"),
        _ => {}
    }
    Ok(())
}

/// HTTP MCP transport: launch the wm-server daemon (the same axum runtime
/// that serves the web API) and print where the MCP endpoint lives. The
/// daemon serves `POST /mcp` on the given port, guarded by the shared
/// `x-wm-token` credential persisted under `.wm/state/web-token`.
fn run_mcp_http(requested_port: u16, project_root: &Path) -> anyhow::Result<()> {
    let server_binary = resolve_server_binary();
    if !server_binary.exists() {
        eprintln!(
            "wm-server not found at {}. Build with: cargo build -p wm-server",
            server_binary.display()
        );
        return Ok(());
    }

    info!("Starting wm-server (MCP HTTP transport) on port {requested_port}");
    let mut child = std::process::Command::new(&server_binary)
        .arg("--port")
        .arg(requested_port.to_string())
        .current_dir(project_root)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start wm-server: {e}"))?;

    let token_file = project_root
        .join(WM_DIR)
        .join(STATE_DIR)
        .join("web-token");
    println!(
        "MCP endpoint: http://{LOCALHOST_ADDR}:{requested_port}/mcp (project {})",
        project_root.display()
    );
    println!("Token (x-wm-token header): read from {}", token_file.display());
    println!("Example: curl -X POST http://{LOCALHOST_ADDR}:{requested_port}/mcp \\");
    println!("  -H \"x-wm-token: $(cat {})\" -H \"content-type: application/json\" \\", token_file.display());
    println!("  -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"curl\",\"version\":\"0\"}}}}}}'");

    match child.wait() {
        Ok(status) if !status.success() => {
            let code = status.code().unwrap_or(1);
            eprintln!(
                "wm-server exited with code: {code} (project: {})",
                project_root.display()
            );
            std::process::exit(code);
        }
        Err(e) => eprintln!("Server process error: {e}"),
        _ => {}
    }
    Ok(())
}

pub(crate) fn resolve_server_binary() -> PathBuf {
    let server_name = if cfg!(windows) {
        "wm-server.exe"
    } else {
        "wm-server"
    };

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(server_name);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    if let Ok(path) = std::env::var("WM_SERVER_PATH") {
        let candidate = PathBuf::from(&path);
        if candidate.exists() {
            return candidate;
        }
    }

    let npm_scope = "@something-cabinet";
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = if let Some(p) = exe.parent() {
            p.to_path_buf()
        } else {
            PathBuf::from(".")
        };

        for _ in 0..8 {
            let check = dir.join("node_modules").join(npm_scope);
            if check.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&check) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with("wm-server-") && entry.path().is_dir() {
                            let candidate = entry.path().join(server_name);
                            if candidate.exists() {
                                return candidate;
                            }
                        }
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(server_name);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from(server_name)
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
        Commands::Init {
            project,
            platform,
            no_wizard,
        } => {
            let root = project.unwrap_or_else(|| std::env::current_dir().unwrap());

            let wm_dir = root.join(WM_DIR);
            std::fs::create_dir_all(wm_dir.join(WIKI_DIR)).ok();
            std::fs::create_dir_all(wm_dir.join(SOURCES_DIR)).ok();
            std::fs::create_dir_all(wm_dir.join(STATE_DIR)).ok();
            let agents_dir = root.join(".agent");
            std::fs::create_dir_all(agents_dir.join("skills")).ok();

            let config = ProjectConfig::default();
            let config_path = wm_dir.join(CONFIG_FILE);
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
                std::fs::create_dir_all(wm_dir.join(WIKI_DIR).join(dir)).ok();
            }

            let agents_md = r#"# AGENTS.md — Wiki Memory Engine Agent Handbook

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
            std::fs::write(wm_dir.join("AGENTS.md"), agents_md).ok();

            if let Some(shim) = wm_core::embed_files::EmbeddedFiles::get("shims/WIKI-MEM.md") {
                std::fs::write(root.join("WIKI-MEM.md"), &shim.data).ok();
            }

            info!("Initialized project at {}", root.display());
            println!("Wiki Memory Engine initialized at {}", root.display());

            let theme = ColorfulTheme::default();
            if !no_wizard && is_terminal::is_terminal(std::io::stdin()) {
                let enable_semantic = Confirm::with_theme(&theme)
                    .with_prompt("Enable semantic search (ONNX embeddings)? This requires downloading a ~134MB model")
                    .default(false)
                    .interact()
                    .unwrap_or(false);

                if enable_semantic {
                    let model_names = [
                        "bge-small-en-v1.5 — 384 dim, ~134MB, recommended",
                        "all-MiniLM-L6-v2 — 384 dim, ~90MB, faster",
                        "bge-base-en-v1.5 — 768 dim, ~438MB, highest accuracy",
                    ];
                    let model_choice = Select::with_theme(&theme)
                        .with_prompt("Select embedding model")
                        .items(&model_names)
                        .default(0)
                        .interact()
                        .unwrap_or(0);
                    let model_name = match model_choice {
                        1 => "all-MiniLM-L6-v2",
                        2 => "bge-base-en-v1.5",
                        _ => "bge-small-en-v1.5",
                    };

                    let config_path = root.join(WM_DIR).join(CONFIG_FILE);
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

                let git_options = [
                    "git-tracked — track everything (config, wiki pages, memory)",
                    "git-ignored — track config + wiki pages; ignore memory, generated files",
                    "none — no .gitignore changes (manage manually)",
                ];
                let git_mode = Select::with_theme(&theme)
                    .with_prompt("Git tracking mode for .wm/ directory")
                    .items(&git_options)
                    .default(0)
                    .interact()
                    .unwrap_or(0)
                    + 1;

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
                        2 => {
                            println!(
                                "  .gitignore: .wm/state/, .wm/memory/, .wm/versions/ ignored"
                            );
                            for dir in &[STATE_DIR, "memory", "versions"] {
                                let tracked = std::process::Command::new("git")
                                    .args(["ls-files", "--cached"])
                                    .arg(format!(".wm/{}", dir))
                                    .current_dir(&root)
                                    .output();
                                if let Ok(output) = tracked {
                                    if !output.stdout.is_empty() {
                                        std::process::Command::new("git")
                                            .args(["rm", "--cached", "-r"])
                                            .arg(format!(".wm/{}", dir))
                                            .current_dir(&root)
                                            .output()
                                            .ok();
                                    }
                                }
                            }
                        }
                        1 => println!("  .gitignore: .wm/ fully tracked"),
                        _ => println!("  .gitignore: unchanged (manage manually)"),
                    }
                }

                let config_path = root.join(WM_DIR).join(CONFIG_FILE);
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
                let platform_items = &[
                    "CLAUDE.md — Claude Code",
                    "OPENCODE.md + AGENTS.md — OpenCode",
                    "AGENTS.md — Kiro",
                    "GEMINI.md — Gemini",
                    ".github/copilot-instructions.md — GitHub Copilot",
                    "AGENTS.md — Generic agents",
                    "REASONIX.md — Reasonix",
                ];
                let platform_names = [
                    "claude", "opencode", "kiro", "gemini", "copilot", "agents", "reasonix",
                ];
                let selections = MultiSelect::with_theme(&theme)
                    .with_prompt("Generate platform agent instruction files? (Space to toggle, Enter to confirm)")
                    .items(platform_items)
                    .interact()
                    .unwrap_or_default();
                selections
                    .iter()
                    .map(|&i| platform_names[i].to_string())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            if !platforms.is_empty() {
                sync_agent_files(&root, &platforms, false)?;

                for platform in &platforms {
                    if matches!(
                        platform.as_str(),
                        "opencode" | "kiro" | "claude" | "codex" | "cursor" | "antigravity"
                    ) {
                        setup_platform_mcp(&root, platform)?;
                    }
                }
            }
        }
        Commands::Web { port } => {
            let port = port.unwrap_or(DEFAULT_PORT);

            let server_binary = resolve_server_binary();

            if !server_binary.exists() {
                eprintln!(
                    "wm-server not found at {}. Build with: cargo build -p wm-server",
                    server_binary.display()
                );
                return Ok(());
            }

            let project_root = match wm_core::config::detect_project_root() {
                Some(p) => p,
                None => {
                    eprintln!(
                        "No wiki-mem project found. Run 'wm init' in your project directory first."
                    );
                    return Ok(());
                }
            };

            run_web(port, &server_binary, &project_root)?;
        }
        Commands::Mcp {
            project,
            transport,
            port,
        } => {
            let project_root = determine_project_root(&project)?;

            match transport {
                McpTransport::Http => run_mcp_http(port.unwrap_or(DEFAULT_PORT), &project_root)?,
                McpTransport::Stdio => {
                    std::env::set_current_dir(&project_root).map_err(|e| {
                        anyhow::anyhow!(
                            "failed to change to project dir {}: {e}",
                            project_root.display()
                        )
                    })?;

                    info!("MCP server starting (project {})", project_root.display());

                    let engine = build_engine(&project_root);
                    let mut registry = wm_core::ToolRegistry::new();
                    wm_core::mcp::tools::register_all_tools(&mut registry, engine.state.clone());
                    let registry = Arc::new(registry);

                    mcp_server::serve(registry).await?;
                }
            }
        }
        Commands::Setup { platform, strict } => {
            let root =
                config::detect_project_root().unwrap_or_else(|| std::env::current_dir().unwrap());

            let all_platforms = [
                "opencode",
                "kiro",
                "claude",
                "codex",
                "cursor",
                "antigravity",
            ];
            let platforms: Vec<String> = if platform == "all" {
                all_platforms.iter().map(|p| p.to_string()).collect()
            } else {
                vec![platform.clone()]
            };

            let syncable: Vec<String> = platforms
                .iter()
                .filter(|p| {
                    matches!(
                        p.as_str(),
                        "opencode"
                            | "kiro"
                            | "claude"
                            | "codex"
                            | "gemini"
                            | "copilot"
                            | "agents"
                            | "reasonix"
                    )
                })
                .cloned()
                .collect();
            sync_agent_files(&root, &syncable, false)?;
            for p in &platforms {
                setup_platform_mcp(&root, p)?;
                setup_query_before_grep(&root, p, strict)?;
            }
        }
        Commands::Agents {
            sync: _sync,
            global,
        } => {
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
                let mut args = serde_json::Map::new();
                args.insert("q".into(), serde_json::json!(query));
                if let Some(t) = r#type {
                    args.insert("type".into(), serde_json::json!(t));
                }
                if let Some(m) = mode {
                    args.insert("mode".into(), serde_json::json!(m));
                }
                args.insert("limit".into(), serde_json::json!(limit));

                match call_tool("wm_search.query", serde_json::Value::Object(args)).await {
                    Ok(resp) => {
                        let mode_used = resp["mode"].as_str().unwrap_or("auto").to_string();
                        let results: Vec<serde_json::Value> = resp["results"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .map(|r| {
                                        serde_json::json!({
                                            "score": r["score"],
                                            "id": r["id"],
                                            "type": r["type"],
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

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
                                let type_tag = if r["type"].as_str() == Some("memory") {
                                    " [memory]"
                                } else {
                                    ""
                                };
                                println!(
                                    "  {:.2}  {}{}",
                                    r["score"].as_f64().unwrap_or(0.0),
                                    r["id"].as_str().unwrap_or("?"),
                                    type_tag
                                );
                            }
                            println!("{} results", results.len());
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            SearchAction::Retrieve {
                query,
                token_budget,
                json,
            } => {
                match call_tool(
                    "wm_search.retrieve",
                    serde_json::json!({ "q": query, "token_budget": token_budget }),
                )
                .await
                {
                    Ok(resp) => {
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "query": query,
                                    "token_budget": token_budget,
                                    "tokens_used": resp["tokens_used"],
                                    "result_count": resp["result_count"],
                                    "context": resp["context"],
                                }))?
                            );
                        } else {
                            if let Some(results) = resp["results"].as_array() {
                                for r in results {
                                    println!(
                                        "  {:.2}  {}",
                                        r["score"].as_f64().unwrap_or(0.0),
                                        r["id"].as_str().unwrap_or("?")
                                    );
                                }
                            }
                            if let Some(context) = resp["context"].as_str() {
                                if !context.is_empty() {
                                    println!("{}", context);
                                }
                            }
                            println!(
                                "{} context items",
                                resp["results"].as_array().map(|a| a.len()).unwrap_or(0)
                            );
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            SearchAction::Resolve { query, json } => {
                match call_tool("wm_search.resolve", serde_json::json!({ "q": query })).await {
                    Ok(result) => {
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
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Commands::Page { action } => match action {
            PageAction::Get { id, json } => {
                let page_id = id.split('#').next().unwrap_or(&id).to_string();
                match call_tool(
                    "wm_page",
                    serde_json::json!({ "action": "get", "id": page_id }),
                )
                .await
                {
                    Ok(resp) => {
                        let content = resp["content"].as_str().unwrap_or("").to_string();
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "content": content
                                }))?
                            );
                        } else {
                            println!("--- {} ---", id);
                            let display = if content.len() > 500 {
                                &content[..500]
                            } else {
                                &content
                            };
                            println!("{}", display);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            PageAction::List { json } => {
                match call_tool("wm_page", serde_json::json!({ "action": "list" })).await {
                    Ok(resp) => {
                        let pages = resp["pages"].as_array().cloned().unwrap_or_default();
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
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            PageAction::Create {
                path,
                title,
                page_type,
                json,
            } => {
                let pt = page_type.clone().unwrap_or_else(|| {
                    let first_segment = path
                        .trim_start_matches("wiki/")
                        .split('/')
                        .next()
                        .unwrap_or("concept");
                    wm_core::engine::PageType::from_dir_name(first_segment)
                        .map(|pt| pt.as_str().to_string())
                        .unwrap_or_else(|| "concept".to_string())
                });
                let mut content = String::new();
                std::io::stdin()
                    .read_to_string(&mut content)
                    .map_err(|e| anyhow::anyhow!("Failed to read stdin: {}", e))?;
                let mut args = serde_json::json!({
                    "action": "create",
                    "path": path.clone(),
                    "title": title,
                    "content": content,
                });
                if let Some(t) = page_type {
                    args["type"] = serde_json::json!(t);
                }
                match call_tool("wm_page", args).await {
                    Ok(resp) => {
                        let id = resp["id"].as_str().unwrap_or("").to_string();
                        let resp_type = resp["type"].as_str().unwrap_or(&pt).to_string();
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "path": path, "type": resp_type
                                }))?
                            );
                        } else {
                            println!("Created page: {} ({})", id, resp_type);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            PageAction::Delete { id, json } => {
                match call_tool(
                    "wm_page",
                    serde_json::json!({ "action": "delete", "id": id.clone() }),
                )
                .await
                {
                    Ok(resp) => {
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "status": resp["status"].as_str().unwrap_or("deleted")
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
                let mut args = updates;
                if let Some(obj) = args.as_object_mut() {
                    obj.insert("action".into(), serde_json::json!("update"));
                    obj.insert("id".into(), serde_json::json!(id));
                }

                match call_tool("wm_page", args).await {
                    Ok(_) => {
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "id": id, "status": "updated"
                                }))
                                .unwrap_or_default()
                            );
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
                let args = serde_json::json!({
                    "action": "link",
                    "id": id.clone(),
                    "target": target.clone(),
                    "edge_type": et.clone(),
                });
                match call_tool("wm_page", args).await {
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
                let args = serde_json::json!({
                    "action": "unlink",
                    "id": id.clone(),
                    "target": target.clone(),
                });
                match call_tool("wm_page", args).await {
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
                match call_tool("wm_graph.stats", serde_json::json!({})).await {
                    Ok(resp) => {
                        let nodes = resp["nodes"].as_u64().unwrap_or(0);
                        let edges = resp["edges"].as_u64().unwrap_or(0);
                        let mut type_counts = std::collections::BTreeMap::new();
                        if let Some(types) = resp["types"].as_object() {
                            for (t, c) in types {
                                type_counts.insert(t.clone(), c.as_u64().unwrap_or(0));
                            }
                        }
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
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            GraphAction::Path {
                start,
                end,
                max_depth,
                json,
            } => {
                let mut args = serde_json::Map::new();
                args.insert("start".into(), serde_json::json!(start.clone()));
                args.insert("end".into(), serde_json::json!(end.clone()));
                if let Some(d) = max_depth {
                    args.insert("max_depth".into(), serde_json::json!(d));
                }
                match call_tool("wm_graph.path", serde_json::Value::Object(args)).await {
                    Ok(resp) => {
                        let json_path = resp["path"].as_array().cloned().unwrap_or_default();
                        if json_path.is_empty() {
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
                            if json {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&serde_json::json!({
                                        "path": json_path, "length": json_path.len()
                                    }))?
                                );
                            } else {
                                println!("Path ({} hops):", json_path.len().saturating_sub(1));
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
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            GraphAction::Subgraph {
                center,
                depth,
                json,
            } => {
                let depth = depth.unwrap_or(1).min(5);
                let args = serde_json::json!({ "center": center.clone(), "depth": depth });
                match call_tool("wm_graph.subgraph", args).await {
                    Ok(resp) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(&resp)?);
                        } else {
                            println!("Subgraph around {} (depth {}):", center, depth);
                            if let Some(nodes) = resp["nodes"].as_array() {
                                for n in nodes {
                                    println!("  {}  {}", n["id"], n["title"]);
                                }
                            }
                            let edge_count = resp["edges"].as_array().map(|a| a.len()).unwrap_or(0);
                            println!(
                                "{} nodes, {} edges",
                                resp["node_count"].as_u64().unwrap_or(0),
                                edge_count
                            );
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            GraphAction::Affected {
                node,
                max_depth,
                json,
            } => {
                let mut args = serde_json::Map::new();
                args.insert("node".into(), serde_json::json!(node.clone()));
                if let Some(d) = max_depth {
                    args.insert("max_depth".into(), serde_json::json!(d));
                }
                match call_tool("wm_graph.affected", serde_json::Value::Object(args)).await {
                    Ok(resp) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(&resp)?);
                        } else {
                            let affected = resp["affected"].as_array().cloned().unwrap_or_default();
                            println!(
                                "Affected by removing {} ({}):",
                                node,
                                resp["kind"].as_str().unwrap_or("page")
                            );
                            for a in &affected {
                                let hops = a["hops"].as_array().map(|h| h.len()).unwrap_or(0);
                                println!("  {}  (depth {})", a["id"].as_str().unwrap_or(""), hops);
                                if let Some(hs) = a["hops"].as_array() {
                                    for h in hs {
                                        println!(
                                            "      {} {} -> {} (line {}, {})",
                                            h["edge_type"].as_str().unwrap_or(""),
                                            h["from"].as_str().unwrap_or(""),
                                            h["to"].as_str().unwrap_or(""),
                                            h["line"]
                                                .as_u64()
                                                .map(|l| l.to_string())
                                                .unwrap_or_else(|| "-".to_string()),
                                            h["provenance"].as_str().unwrap_or(""),
                                        );
                                    }
                                }
                            }
                            println!("{} affected", affected.len());
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            GraphAction::Neighbors { id, query, json } => {
                let mut args = serde_json::Map::new();
                args.insert("id".into(), serde_json::json!(id.clone()));
                if let Some(q) = query {
                    args.insert("query".into(), serde_json::json!(q));
                }
                match call_tool("wm_graph.neighbors", serde_json::Value::Object(args)).await {
                    Ok(resp) => {
                        let neighbors = resp["neighbors"].as_array().cloned().unwrap_or_default();
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "neighbors": neighbors, "total": neighbors.len()
                                }))?
                            );
                        } else {
                            for n in &neighbors {
                                println!("  {}  {}", n["id"], n["title"]);
                            }
                            println!("{} neighbors", neighbors.len());
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            GraphAction::Export { format, out } => {
                let (engine, wiki_dir) = create_engine();
                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                match format {
                    ExportFormat::Json => {
                        let text = serde_json::to_string_pretty(
                            &wm_core::graph::export::graph_to_json(graph),
                        )?;
                        match out {
                            Some(path) => {
                                std::fs::write(&path, text)?;
                                println!("Graph snapshot written to {}", path.display());
                            }
                            None => println!("{text}"),
                        }
                    }
                    ExportFormat::Graphml => {
                        let xml = wm_core::graph::export::graph_to_graphml(graph);
                        match out {
                            Some(path) => {
                                std::fs::write(&path, xml)?;
                                println!("GraphML written to {}", path.display());
                            }
                            None => print!("{xml}"),
                        }
                    }
                    ExportFormat::Obsidian => {
                        let Some(vault) = out else {
                            eprintln!("obsidian export requires --out <vault-dir>");
                            return Ok(());
                        };
                        let result =
                            wm_core::graph::export::export_obsidian(graph, &wiki_dir, &vault)?;
                        println!(
                            "Exported {} pages with {} wikilinks to {}",
                            result.pages,
                            result.wikilinks,
                            vault.display()
                        );
                    }
                }
            }
        },
        Commands::Health { action } => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
            let engine = Arc::new(MainEngine::new());
            if wiki_dir.exists() {
                rebuild_from_engine(&engine, &wiki_dir);
            }
            match action {
                HealthAction::Audit {
                    dry_run,
                    fix,
                    format,
                } => {
                    let is_dry_run = dry_run || !fix;

                    let sections = engine.state.section_corpus.load();
                    let snapshot = engine.state.graph.load();
                    let graph = &snapshot.0;
                    let index = &snapshot.1;

                    let mut section_page_ids: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    for sec in sections.iter() {
                        let page_id = sec
                            .section_id
                            .split('#')
                            .next()
                            .unwrap_or(&sec.section_id)
                            .to_string();
                        section_page_ids.insert(page_id);
                    }

                    let mut empty_pages: Vec<serde_json::Value> = Vec::new();
                    if wiki_dir.exists() {
                        let wd = wiki_dir.clone();
                        let sp_ids = section_page_ids.clone();
                        let g = graph.clone();
                        let mut dirs_to_visit: Vec<std::path::PathBuf> = vec![wd.clone()];
                        while let Some(dir) = dirs_to_visit.pop() {
                            if let Ok(rd) = std::fs::read_dir(&dir) {
                                for entry in rd.filter_map(|e| e.ok()) {
                                    let path = entry.path();
                                    if path.is_dir() {
                                        dirs_to_visit.push(path);
                                    } else if path
                                        .extension()
                                        .map(|ext| ext == WIKI_FILE_EXT)
                                        .unwrap_or(false)
                                    {
                                        let rel_path = path
                                            .strip_prefix(&wd)
                                            .unwrap_or(&path)
                                            .to_string_lossy()
                                            .replace(WIKI_FILE_EXT_DOT, "")
                                            .replace(PATH_SEPARATOR, &ID_SEPARATOR.to_string());
                                        let page_id = format!("{}{}", PAGE_ID_PREFIX, rel_path);
                                        if page_id == WIKI_INDEX_PAGE || page_id == WIKI_README_PAGE
                                        {
                                            continue;
                                        }
                                        if !sp_ids.contains(&page_id) {
                                            let is_task = rel_path.starts_with(TASK_PREFIX);
                                            let inbound_refs = g
                                                .node_indices()
                                                .filter(|&idx| {
                                                    g.edges_directed(
                                                        idx,
                                                        petgraph::Direction::Outgoing,
                                                    )
                                                    .any(|e| g[e.target()].id == page_id)
                                                })
                                                .count();
                                            empty_pages.push(serde_json::json!({
                                                "id": page_id,
                                                "is_task": is_task,
                                                "inbound_refs": inbound_refs,
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mut broken_refs: Vec<serde_json::Value> = Vec::new();
                    for idx in graph.node_indices() {
                        let meta = &graph[idx];
                        for (edge_type, target) in &meta.relates_to {
                            if !index.contains_key(target.as_str()) {
                                broken_refs.push(serde_json::json!({
                                    "source": meta.id,
                                    "target": target,
                                    "edge_type": edge_type.as_yaml_str(),
                                }));
                            }
                        }
                    }

                    let mut deleted_pages: Vec<String> = Vec::new();
                    let mut stubbed_pages: Vec<String> = Vec::new();
                    let mut fixed_refs: u32 = 0;
                    let mut removed_refs: u32 = 0;

                    if !is_dry_run {
                        let mut ci_index: std::collections::HashMap<String, String> =
                            std::collections::HashMap::new();
                        for idx in graph.node_indices() {
                            let meta = &graph[idx];
                            ci_index.insert(meta.id.to_lowercase(), meta.id.clone());
                        }

                        for p in &empty_pages {
                            let page_id = p["id"].as_str().unwrap_or("");
                            let is_task = p["is_task"].as_bool().unwrap_or(false);
                            let inbound = p["inbound_refs"].as_u64().unwrap_or(0);
                            if is_task && inbound == 0 {
                                let page_path = format!(
                                    "{}/{}.md",
                                    wiki_dir.display(),
                                    page_id
                                        .strip_prefix(PAGE_ID_PREFIX)
                                        .unwrap_or(page_id)
                                        .replace(ID_SEPARATOR, &PATH_SEPARATOR.to_string())
                                );
                                let path = std::path::Path::new(&page_path);
                                if path.exists() {
                                    if let Err(e) = std::fs::remove_file(path) {
                                        eprintln!("  Failed to delete {}: {}", page_id, e);
                                        continue;
                                    }
                                    deleted_pages.push(page_id.to_string());
                                }
                            }
                        }

                        for p in &empty_pages {
                            let page_id = p["id"].as_str().unwrap_or("");
                            let is_task = p["is_task"].as_bool().unwrap_or(false);
                            let inbound = p["inbound_refs"].as_u64().unwrap_or(0);
                            if !is_task || inbound == 0 {
                                continue;
                            }
                            let page_path = format!(
                                "{}/{}.md",
                                wiki_dir.display(),
                                page_id
                                    .strip_prefix(PAGE_ID_PREFIX)
                                    .unwrap_or(page_id)
                                    .replace(ID_SEPARATOR, &PATH_SEPARATOR.to_string())
                            );
                            let path = std::path::Path::new(&page_path);
                            if !path.exists() {
                                continue;
                            }
                            let Ok(content) = std::fs::read_to_string(path) else {
                                continue;
                            };
                            if content.trim().is_empty() {
                                continue;
                            }
                            let (fm, body) = wm_core::parser::extract_frontmatter(&content);
                            if !body.trim().is_empty() {
                                continue;
                            }
                            let is_active = fm
                                .as_ref()
                                .and_then(|f| f.status.as_deref())
                                .map(|s| {
                                    matches!(
                                        s.to_lowercase().as_str(),
                                        "todo" | "in-progress" | "draft"
                                    )
                                })
                                .unwrap_or(true);
                            if !is_active {
                                continue;
                            }
                            let title = fm
                                .as_ref()
                                .and_then(|f| f.title.as_deref())
                                .unwrap_or(page_id);
                            let stub_body = format!(
                                "## Overview\n\n(Task stub — no description yet for \"{}\".)\n\nAcceptance criteria and implementation details live in the wiki task page.",
                                title
                            );
                            let params = wm_core::page::PageUpdateParams {
                                content: Some(stub_body),
                                ..Default::default()
                            };
                            if let Err(e) =
                                wm_core::page::update_page(&engine.state, page_id, &params)
                            {
                                eprintln!("  Failed to stub {}: {}", page_id, e);
                                continue;
                            }
                            stubbed_pages.push(page_id.to_string());
                        }

                        let mut refs_by_source: std::collections::BTreeMap<
                            String,
                            Vec<(String, String, bool)>,
                        > = std::collections::BTreeMap::new();
                        for r in &broken_refs {
                            let source = r["source"].as_str().unwrap_or("").to_string();
                            let target = r["target"].as_str().unwrap_or("").to_string();
                            let et = r["edge_type"]
                                .as_str()
                                .unwrap_or(DEFAULT_EDGE_TYPE)
                                .to_string();
                            let is_case_fix = ci_index.contains_key(&target.to_lowercase());
                            refs_by_source.entry(source).or_default().push((
                                target,
                                et,
                                is_case_fix,
                            ));
                        }

                        for (source, refs) in &refs_by_source {
                            if let Some(&src_idx) = index.get(source) {
                                let meta = &graph[src_idx];
                                let mut new_list: Vec<serde_json::Value> = Vec::new();
                                for (et, target) in &meta.relates_to {
                                    let broken = refs.iter().find(|(t, _, _)| t == target);
                                    match broken {
                                        Some((_, _, true)) => {
                                            if let Some(correct) =
                                                ci_index.get(&target.to_lowercase())
                                            {
                                                new_list.push(serde_json::json!({
                                                    "type": et.as_yaml_str(),
                                                    "target": correct,
                                                }));
                                                fixed_refs = fixed_refs.saturating_add(1);
                                            }
                                        }
                                        Some(_) => {
                                            removed_refs = removed_refs.saturating_add(1);
                                        }
                                        None => {
                                            new_list.push(serde_json::json!({
                                                "type": et.as_yaml_str(),
                                                "target": target,
                                            }));
                                        }
                                    }
                                }
                                let params = wm_core::page::PageUpdateParams {
                                    relates_to: Some(new_list),
                                    ..Default::default()
                                };
                                if let Err(e) =
                                    wm_core::page::update_page(&engine.state, source, &params)
                                {
                                    eprintln!("  Failed to update refs for {}: {}", source, e);
                                }
                            }
                        }
                    }

                    let empty_count = empty_pages.len();
                    let broken_count = broken_refs.len();
                    let is_task_count = empty_pages
                        .iter()
                        .filter(|p| p["is_task"].as_bool().unwrap_or(false))
                        .count();
                    let delete_count = deleted_pages.len();
                    let status_label = health_status_label(empty_count, broken_count);
                    let output_fmt = models::output_format::OutputFormat::from_str(&format);

                    if matches!(output_fmt, models::output_format::OutputFormat::Json) {
                        let result = serde_json::json!({
                            "empty_pages": empty_pages,
                            "empty_count": empty_count,
                            "broken_refs": broken_refs,
                            "broken_ref_count": broken_count,
                            "dry_run": is_dry_run,
                            "status": status_label,
                            "fix_applied": !is_dry_run,
                            "deleted_pages": deleted_pages,
                            "stubbed_pages": stubbed_pages,
                            "case_fixed_refs": fixed_refs,
                            "removed_refs": removed_refs,
                        });
                        println!("{}", serde_json::to_string_pretty(&result)?);
                        return Ok(());
                    }

                    println!("Health Audit");
                    println!("============");
                    println!("  Dry-run: {}", is_dry_run);
                    println!(
                        "  Empty pages: {} ({} task pages)",
                        empty_count, is_task_count
                    );
                    for p in &empty_pages {
                        println!("    {}  [inbound: {}]", p["id"], p["inbound_refs"]);
                    }
                    println!("  Broken refs: {}", broken_count);
                    for r in &broken_refs {
                        println!(
                            "    {} -> {} ({})",
                            r["source"], r["target"], r["edge_type"]
                        );
                    }
                    if !is_dry_run {
                        println!("  Fix actions: {} pages deleted, {} pages stubbed, {} refs removed, {} refs case-corrected", delete_count, stubbed_pages.len(), removed_refs, fixed_refs);
                    }
                    println!("  Status: {}", status_label);
                }
            }
        }
        Commands::Source { action } => match action {
            SourceAction::List { state, json } => {
                let mut args = serde_json::Map::new();
                args.insert("action".into(), serde_json::json!("list"));
                if let Some(s) = state {
                    args.insert("state".into(), serde_json::json!(s));
                }
                match call_tool("wm_source", serde_json::Value::Object(args)).await {
                    Ok(resp) => {
                        let sources = resp["sources"].as_array().cloned().unwrap_or_default();
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
                match call_tool(
                    "wm_source",
                    serde_json::json!({ "action": "status", "id": id }),
                )
                .await
                {
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
                match call_tool(
                    "wm_source",
                    serde_json::json!({ "action": "remove", "id": id }),
                )
                .await
                {
                    Ok(_) => println!("Removed source: {}", id),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            SourceAction::Discover { json } => {
                match call_tool("wm_source", serde_json::json!({ "action": "discover" })).await {
                    Ok(resp) => {
                        let discovered = resp["discovered"].as_array().cloned().unwrap_or_default();
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
        },
        Commands::Task { action } => match action {
            TaskAction::Board { json } => {
                match call_tool("wm_task", serde_json::json!({ "action": "board" })).await {
                    Ok(board_json) => {
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "columns": board_json["columns"],
                                    "counts": board_json["counts"],
                                }))?
                            );
                        } else {
                            let columns = board_json["columns"]
                                .as_object()
                                .cloned()
                                .unwrap_or_default();
                            let column_order = [
                                "draft",
                                "todo",
                                "in-progress",
                                "in-review",
                                "blocked",
                                "done",
                                "reviewed",
                                "approved",
                                "superseded",
                                "cancelled",
                            ];
                            let terminal_statuses =
                                ["done", "reviewed", "approved", "superseded", "cancelled"];
                            let mut any_active = false;
                            for col_name in &column_order {
                                let items = columns.get(*col_name).and_then(|v| v.as_array());
                                let count = items.map(|v| v.len()).unwrap_or(0);
                                if count == 0 {
                                    continue;
                                }
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
                                        let p = t["priority"]
                                            .as_str()
                                            .unwrap_or(" ")
                                            .chars()
                                            .next()
                                            .unwrap_or(' ');
                                        println!("  {}  {}", p, t["title"].as_str().unwrap_or(""));
                                    }
                                }
                            }
                            if !any_active {
                                println!("(no active tasks — use --all to see terminal columns)");
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Commands::Log { action } => match action {
            LogAction::Recent { count, json } => {
                let log_path = std::path::Path::new(WM_DIR).join(LOG_FILE);
                let content = std::fs::read_to_string(&log_path).unwrap_or_default();
                let all_lines: Vec<&str> =
                    content.lines().filter(|l| !l.trim().is_empty()).collect();
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
                let log_path = std::path::Path::new(WM_DIR).join(LOG_FILE);
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
                let log_path = std::path::Path::new(WM_DIR).join(LOG_FILE);
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
        Commands::Lint { action } => match action {
            LintAction::Check { json } => {
                let stats = call_tool("wm_graph.stats", serde_json::json!({})).await;
                let lint = call_tool("wm_lint.check", serde_json::json!({})).await;
                match (stats, lint) {
                    (Ok(stats), Ok(lint)) => {
                        let nodes = stats["nodes"].as_u64().unwrap_or(0);
                        let edges = stats["edges"].as_u64().unwrap_or(0);
                        let orphans = lint["issues"]
                            .as_array()
                            .map(|issues| {
                                issues
                                    .iter()
                                    .filter(|i| i["type"].as_str() == Some("orphan"))
                                    .count()
                            })
                            .unwrap_or(0);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "nodes": nodes, "edges": edges, "orphans": orphans
                                }))?
                            );
                        } else {
                            println!("Lint check complete:");
                            println!("  Nodes: {}", nodes);
                            println!("  Edges: {}", edges);
                            println!("  Orphans: {}", orphans);
                        }
                    }
                    (Err(e), _) => eprintln!("Error: {}", e),
                    (_, Err(e)) => eprintln!("Error: {}", e),
                }
            }
            LintAction::Fix { json } => {
                match call_tool("wm_lint.fix", serde_json::json!({})).await {
                    Ok(resp) => {
                        let fixed = resp["fixed"].as_u64().unwrap_or(0);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "fixed": fixed,
                                }))?
                            );
                        } else {
                            println!("Fixed {} issue(s)", fixed);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Commands::Validate { json } => {
            match call_tool("wm_validate.check", serde_json::json!({})).await {
                Ok(resp) => {
                    let nodes = resp["nodes"].as_u64().unwrap_or(0);
                    let edges = resp["edges"].as_u64().unwrap_or(0);
                    let status = if nodes > 0 { "pass" } else { "empty" };
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "nodes": nodes,
                                "edges": edges,
                                "status": status,
                            }))?
                        );
                    } else {
                        println!("Validation complete: {} nodes, {} edges", nodes, edges);
                        if nodes > 0 {
                            println!("Status: pass");
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Index { action } => match action {
            IndexAction::Rebuild {
                skip_embed,
                batch_size,
                since,
            } => {
                if since.is_some() {
                    eprintln!(
                        "Warning: --since (cursor scanning) is not yet supported; performing a full rebuild."
                    );
                }
                let args = serde_json::json!({
                    "skip_embed": skip_embed,
                    "embed_batch_size": batch_size,
                });
                match call_tool("wm_index_rebuild", args).await {
                    Ok(resp) => {
                        println!(
                            "  Graph: {} nodes",
                            resp["graph_nodes"].as_u64().unwrap_or(0)
                        );
                        println!("  Sections: {}", resp["sections"].as_u64().unwrap_or(0));
                        println!("  Rebuild complete.");
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            IndexAction::Code { skip_hash_check } => {
                let root = config::detect_project_root()
                    .ok_or_else(|| anyhow::anyhow!("No project root found"))?;
                #[cfg(feature = "code-intel")]
                {
                    use wm_core::code_intel::services::ingest_service::rebuild_code_index;
                    use wm_core::code_intel::services::CodeIndexDb;

                    let db_dir = root.join(WM_DIR).join(STATE_DIR);
                    let db_path = db_dir.join("code.db");
                    std::fs::create_dir_all(&db_dir).ok();
                    let db = CodeIndexDb::open(db_path)
                        .map_err(|e| anyhow::anyhow!("Failed to open code db: {}", e))?;

                    println!("Rebuilding code index...");
                    match rebuild_code_index(&db, &root, skip_hash_check) {
                        Ok(stats) => {
                            println!("  {} files scanned", stats.files_scanned);
                            println!("  {} files changed", stats.files_changed);
                            println!(
                                "  {} symbols in index (+{} new)",
                                stats.total_symbols, stats.symbols_indexed
                            );
                            println!(
                                "  {} dependencies in index (+{} new)",
                                stats.total_deps, stats.deps_indexed
                            );
                            if !stats.errors.is_empty() {
                                println!("  {} errors (see logs)", stats.errors.len());
                            }
                            match wm_core::engine::code_index_refresh_service::index_lag_seconds(
                                &root,
                            ) {
                                Ok(Some(0)) => println!("  index age: current"),
                                Ok(Some(secs)) => println!("  index age: {}s behind", secs),
                                Ok(None) => {}
                                Err(_) => {}
                            }
                        }
                        Err(e) => anyhow::bail!("Code index rebuild failed: {}", e),
                    }
                }
                #[cfg(not(feature = "code-intel"))]
                {
                    let _ = root;
                    anyhow::bail!(
                        "code-intel feature not enabled. Rebuild with --features code-intel."
                    );
                }
            }
            IndexAction::Embed { batch_size, force } => {
                let args = serde_json::json!({ "batch_size": batch_size, "force": force });
                match call_tool("wm_index_embed", args).await {
                    Ok(_) => println!("Embedding complete."),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Commands::Time { action } => match action {
            TimeAction::Start { id, .. } => {
                match call_tool(
                    "wm_time",
                    serde_json::json!({ "action": "start", "id": id }),
                )
                .await
                {
                    Ok(resp) => {
                        let now = resp["time_started"].as_str().unwrap_or("").to_string();
                        println!("Time tracking started for {} at {}", id, now);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            TimeAction::Stop { id, .. } => {
                match call_tool("wm_time", serde_json::json!({ "action": "stop", "id": id })).await
                {
                    Ok(resp) => {
                        let elapsed = resp["time_spent"].as_str().unwrap_or("0h 0m");
                        println!("Time stopped for {}: {}", id, elapsed);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            TimeAction::Add { id, duration, .. } => {
                match call_tool(
                    "wm_time",
                    serde_json::json!({ "action": "add", "id": id, "duration": duration.clone() }),
                )
                .await
                {
                    Ok(_) => println!("Time added for {}: {}", id, duration),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            TimeAction::Report { json } => {
                match call_tool("wm_time", serde_json::json!({ "action": "report" })).await {
                    Ok(resp) => {
                        let tasks = resp["tasks"].as_array().cloned().unwrap_or_default();
                        let total_hours = resp["total_hours"].as_f64().unwrap_or(0.0);
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
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Commands::Model { action } => {
            match action {
                ModelAction::Download { name, .. } => {
                    #[cfg(feature = "onnx")]
                    {
                        let spinner = indicatif::ProgressBar::new_spinner();
                        spinner.set_style(
                            indicatif::ProgressStyle::default_spinner()
                                .template("{spinner:.green} {msg}")
                                .unwrap()
                                .tick_strings(&[
                                    "▹▹▹▹▹",
                                    "▸▹▹▹▹",
                                    "▹▸▹▹▹",
                                    "▹▹▸▹▹",
                                    "▹▹▹▸▹",
                                    "▹▹▹▹▸",
                                    "▪▪▪▪▪",
                                ]),
                        );
                        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
                        spinner.set_message(format!("Downloading {}...", name));
                        let home = std::env::var("HOME")
                            .or_else(|_| std::env::var("USERPROFILE"))
                            .unwrap_or_else(|_| ".".into());
                        let models_dir = std::path::PathBuf::from(home).join(WM_DIR).join("models");
                        let result = wm_core::embed::download_model(&name, &models_dir);
                        spinner.finish_and_clear();
                        match result {
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
                    let models_dir = std::path::PathBuf::from(home).join(WM_DIR).join("models");
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
                        .join(WM_DIR)
                        .join("models")
                        .join(&name);
                    if model_dir.exists() {
                        std::fs::remove_dir_all(&model_dir)?;
                        println!("Removed model: {}", name);
                    } else {
                        println!("Model not found: {}", name);
                    }
                }
            }
        }
        Commands::Status { json } => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
            let mut status = serde_json::json!({
                "project_root": root,
                "wiki_dir": wiki_dir,
            });
            if wiki_dir.exists() {
                let engine = Arc::new(MainEngine::new());
                if wiki_dir.exists() {
                    rebuild_from_engine(&engine, &wiki_dir);
                    let snapshot = engine.state.graph.load();
                    let node_count = snapshot.0.node_count();
                    let edge_count = snapshot.0.edge_count();

                    let mut task_counts = std::collections::BTreeMap::new();
                    for idx in snapshot.0.node_indices() {
                        if snapshot.0[idx].page_type == wm_core::engine::PageType::Task {
                            let st = format!("{:?}", snapshot.0[idx].status);
                            *task_counts.entry(st).or_insert(0) += 1;
                        }
                    }

                    let sections = engine.state.section_corpus.load().len();
                    let bm25_docs = engine.state.bm25_index.load().total_docs;
                    let embed_loaded = engine.state.embedder.is_loaded();
                    let model_name = engine.state.embedder.model_name().to_string();

                    status["graph_nodes"] = serde_json::json!(node_count);
                    status["graph_edges"] = serde_json::json!(edge_count);
                    status["sections"] = serde_json::json!(sections);
                    status["bm25_docs"] = serde_json::json!(bm25_docs);
                    status["embedding_loaded"] = serde_json::json!(embed_loaded);
                    status["embedding_model"] = serde_json::json!(model_name);
                    status["tasks_by_status"] = serde_json::json!(task_counts);
                }
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!("Wiki Memory Engine — Project Status");
                println!("  Project root:  {}", root.display());
                println!("  Wiki:          {}", wiki_dir.display());
                if let Some(n) = status.get("graph_nodes").and_then(|v| v.as_u64()) {
                    println!("  Graph nodes:   {}", n);
                }
                if let Some(e) = status.get("graph_edges").and_then(|v| v.as_u64()) {
                    println!("  Graph edges:   {}", e);
                }
                if let Some(s) = status.get("sections").and_then(|v| v.as_u64()) {
                    println!("  Sections:      {}", s);
                }
                if let Some(d) = status.get("bm25_docs").and_then(|v| v.as_u64()) {
                    println!("  BM25 docs:     {}", d);
                }
                if let Some(emb) = status.get("embedding_loaded").and_then(|v| v.as_bool()) {
                    let model = status
                        .get("embedding_model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("none");
                    if emb {
                        println!("  Embeddings:    {} (loaded)", model);
                    } else {
                        println!("  Embeddings:    {} (not loaded)", model);
                    }
                }
                if let Some(tasks) = status.get("tasks_by_status").and_then(|v| v.as_object()) {
                    println!("  Tasks:");
                    for (st, count) in tasks {
                        println!("    {}: {}", st, count);
                    }
                }
            }
        }
        Commands::Config { action } => match action {
            ConfigAction::Get { key, json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let cfg_path = root.join(WM_DIR).join(CONFIG_FILE);
                if !cfg_path.exists() {
                    anyhow::bail!("Config file not found: {}", cfg_path.display());
                }
                let content = std::fs::read_to_string(&cfg_path)?;
                let cfg: serde_json::Value = serde_json::from_str(&content)?;
                let value = cfg.pointer(&format!("/{}", key.replace('.', "/")));
                match value {
                    Some(v) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(v)?);
                        } else {
                            println!("{}: {}", key, v);
                        }
                    }
                    None => anyhow::bail!("Config key not found: {}", key),
                }
            }
            ConfigAction::Set { key, value, json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let cfg_path = root.join(WM_DIR).join(CONFIG_FILE);
                if !cfg_path.exists() {
                    anyhow::bail!("Config file not found: {}", cfg_path.display());
                }
                let content = std::fs::read_to_string(&cfg_path)?;
                let mut cfg: serde_json::Value = serde_json::from_str(&content)?;
                let parsed: serde_json::Value = serde_json::from_str(&value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
                let pointer = format!("/{}", key.replace('.', "/"));
                if let Some(target) = cfg.pointer_mut(&pointer) {
                    *target = parsed.clone();
                } else {
                    anyhow::bail!("Cannot set key: {}", key);
                }
                std::fs::write(&cfg_path, serde_json::to_string_pretty(&cfg)?)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({"ok": true, "key": key, "value": parsed})
                        )?
                    );
                } else {
                    println!("  {} = {}", key, parsed);
                }
            }
            ConfigAction::List { json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let cfg_path = root.join(WM_DIR).join(CONFIG_FILE);
                if !cfg_path.exists() {
                    anyhow::bail!("Config file not found: {}", cfg_path.display());
                }
                let content = std::fs::read_to_string(&cfg_path)?;
                let cfg: serde_json::Value = serde_json::from_str(&content)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&cfg)?);
                } else {
                    fn print_config(v: &serde_json::Value, prefix: &str) {
                        match v {
                            serde_json::Value::Object(map) => {
                                for (k, val) in map {
                                    let path = if prefix.is_empty() {
                                        k.clone()
                                    } else {
                                        format!("{}.{}", prefix, k)
                                    };
                                    match val {
                                        serde_json::Value::Object(_) => print_config(val, &path),
                                        _ => println!("  {}: {}", path, val),
                                    }
                                }
                            }
                            _ => println!("  {}: {}", prefix, v),
                        }
                    }
                    print_config(&cfg, "");
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
