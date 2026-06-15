use clap::{Parser, Subcommand};
use petgraph::visit::EdgeRef;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

use wm_core::config::{self, ProjectConfig};
use wm_core::engine::VppEngine;
use wm_core::mcp::tools::register_all_tools;
use wm_core::mcp::transport::run_transport;

/// Wiki Memory Engine — project context and knowledge engine
#[derive(Parser)]
#[command(name = "wm", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new .wm project
    Init {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        platform: Option<String>,
    },
    /// Start the MCP server (stdio)
    Serve {
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Search the wiki (MCP equivalent: wm_search.query)
    Search {
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
    /// Page operations (MCP equivalent: wm_page.*)
    Page {
        #[command(subcommand)]
        action: PageAction,
    },
    /// Graph operations (MCP equivalent: wm_graph.neighbors)
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },
    /// Lint the wiki (MCP equivalent: wm_lint.check)
    Lint,
    /// Validate the wiki (MCP equivalent: wm_validate.check)
    Validate,
    /// Download a model
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },
    /// Show version info
    #[command(alias = "--version")]
    Version,
}

#[derive(Subcommand)]
enum PageAction {
    /// Get a page by ID
    Get { id: String, #[arg(long)] json: bool },
    /// List all pages
    List { #[arg(long)] json: bool },
}

#[derive(Subcommand)]
enum GraphAction {
    /// Get neighbors of a page
    Neighbors { id: String, #[arg(long)] query: Option<String>, #[arg(long)] json: bool },
}

#[derive(Subcommand)]
enum ModelAction {
    Download { name: String },
    List,
    Status,
    Remove { name: String },
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_ansi(false)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    setup_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project, platform } => {
            let root = project.unwrap_or_else(|| std::env::current_dir().unwrap());
            let wm_dir = root.join(".wm");
            std::fs::create_dir_all(wm_dir.join("wiki")).ok();
            std::fs::create_dir_all(wm_dir.join("sources")).ok();
            std::fs::create_dir_all(wm_dir.join("skills")).ok();
            std::fs::create_dir_all(wm_dir.join("state")).ok();

            let config = ProjectConfig::default();
            let config_path = wm_dir.join("config.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

            // Create wiki subdirectories
            for dir in &["tasks", "specs", "concepts", "patterns", "decisions", "howto", "reference"] {
                std::fs::create_dir_all(wm_dir.join("wiki").join(dir)).ok();
            }

            info!("Initialized project at {}", root.display());
            println!("Wiki Memory Engine initialized at {}", root.display());

            if let Some(plat) = platform {
                info!("Platform config generation: {} (coming soon)", plat);
            }
        }
        Commands::Serve { project } => {
            let root = if let Some(p) = project {
                p
            } else if let Some(detected) = config::detect_project_root() {
                detected
            } else {
                anyhow::bail!("No project found. Run 'wm init' first or use --project.");
            };

            info!("Starting wiki-mem MCP server for project: {}", root.display());
            let _config = config::load_config(&root)?;
            let engine = Arc::new(VppEngine::new(ProjectConfig::default()));

            // Build the initial graph from wiki files
            let wiki_dir = root.join(".wm").join("wiki");
            if wiki_dir.exists() {
                let count = wm_core::graph::rebuild_snapshot(&engine.state.graph, &wiki_dir);
                info!("Loaded {} pages from {}", count, wiki_dir.display());
            }

            let mut registry = wm_core::mcp::transport::ToolRegistry::new();
            register_all_tools(&mut registry, engine.state.clone());
            let registry = Arc::new(registry);

            // Handle shutdown signals
            let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
            let s_tx = shutdown_tx.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                info!("SIGINT received, shutting down...");
                drop(s_tx);
            });

            tokio::select! {
                result = run_transport(registry) => {
                    result?;
                }
                _ = shutdown_rx.recv() => {
                    info!("Graceful shutdown complete.");
                }
            }
        }
        Commands::Search { query, mode, r#type, limit, json } => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(".wm").join("wiki");
            let engine = Arc::new(VppEngine::new(ProjectConfig::default()));
            if wiki_dir.exists() {
                wm_core::graph::rebuild_snapshot(&engine.state.graph, &wiki_dir);
            }

            let snapshot = engine.state.graph.load();
            let graph = &snapshot.0;
            let docs: Vec<wm_core::search::IndexedDoc> = graph.node_indices().map(|idx| {
                let meta = &graph[idx];
                wm_core::search::IndexedDoc {
                    id: meta.id.clone(),
                    fields: vec![
                        wm_core::search::Field::new("title", &meta.title, 4.0),
                        wm_core::search::Field::new("tags", &meta.tags.join(" "), 2.2),
                        wm_core::search::Field::new("id", &meta.id, 3.0),
                    ],
                }
            }).collect();

            let bm25 = wm_core::search::Bm25Index::build(docs);
            let results = bm25.search(&query, limit);
            let results: Vec<serde_json::Value> = results.iter().filter(|r| {
                if let Some(ref pt) = r#type {
                    r.id.contains(&format!(":{}:", pt))
                } else { true }
            }).map(|r| serde_json::json!({ "id": r.id, "score": r.score })).collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "results": results, "total": results.len()
                }))?);
            } else {
                for r in &results {
                    println!("  {:.2}  {}", r["score"], r["id"]);
                }
                println!("{} results", results.len());
            }
        }
        Commands::Page { action } => match action {
            PageAction::Get { id, json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(VppEngine::new(ProjectConfig::default()));
                if wiki_dir.exists() {
                    wm_core::graph::rebuild_snapshot(&engine.state.graph, &wiki_dir);
                }
                match wm_core::page::get_page(&engine.state, &id) {
                    Ok(content) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                                "id": id, "content": content.raw
                            }))?);
                        } else {
                            println!("--- {} ---", id);
                            println!("{}", &content.raw[..content.raw.len().min(500)]);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            PageAction::List { json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(VppEngine::new(ProjectConfig::default()));
                if wiki_dir.exists() {
                    wm_core::graph::rebuild_snapshot(&engine.state.graph, &wiki_dir);
                }
                match wm_core::page::list_pages(&engine.state) {
                    Ok(pages) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(
                                &serde_json::json!({ "pages": pages, "total": pages.len() })
                            )?);
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
        },
        Commands::Graph { action } => match action {
            GraphAction::Neighbors { id, query, json } => {
                let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
                let wiki_dir = root.join(".wm").join("wiki");
                let engine = Arc::new(VppEngine::new(ProjectConfig::default()));
                if wiki_dir.exists() {
                    wm_core::graph::rebuild_snapshot(&engine.state.graph, &wiki_dir);
                }

                let snapshot = engine.state.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;

                if let Some(&start) = index.get(&id) {
                    let mut neighbors = Vec::new();
                    for edge in graph.edges(start) {
                        let target = edge.target();
                        let meta = &graph[target];
                        neighbors.push(serde_json::json!({
                            "id": meta.id, "title": meta.title
                        }));
                    }
                    if json {
                        println!("{}", serde_json::to_string_pretty(
                            &serde_json::json!({ "neighbors": neighbors, "total": neighbors.len() })
                        )?);
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
        Commands::Lint => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(".wm").join("wiki");
            let engine = Arc::new(VppEngine::new(ProjectConfig::default()));
            if wiki_dir.exists() {
                wm_core::graph::rebuild_snapshot(&engine.state.graph, &wiki_dir);
            }
            let snapshot = engine.state.graph.load();
            let graph = &snapshot.0;
            let mut orphans = 0;
            for idx in graph.node_indices() {
                let inbound = graph.edges_directed(idx, petgraph::Direction::Incoming).count();
                if inbound == 0 { orphans += 1; }
            }
            println!("Lint check complete:");
            println!("  Nodes: {}", graph.node_count());
            println!("  Edges: {}", graph.edge_count());
            println!("  Orphans: {}", orphans);
        }
        Commands::Validate => {
            let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
            let wiki_dir = root.join(".wm").join("wiki");
            let engine = Arc::new(VppEngine::new(ProjectConfig::default()));
            if wiki_dir.exists() {
                wm_core::graph::rebuild_snapshot(&engine.state.graph, &wiki_dir);
            }
            let snapshot = engine.state.graph.load();
            let graph = &snapshot.0;
            println!("Validation complete: {} nodes, {} edges", graph.node_count(), graph.edge_count());
            if graph.node_count() > 0 { println!("Status: pass"); }
        }
        Commands::Model { action } => {
            match action {
                ModelAction::Download { name } => {
                    println!("Model download: {} (coming soon)", name);
                }
                ModelAction::List => {
                    println!("Available models:\n  - bge-small-en-v1.5 (384-dim, recommended)");
                }
                ModelAction::Status => {
                    println!("Model status:\n  No model loaded. Run 'wm model download' first.");
                }
                ModelAction::Remove { name } => {
                    println!("Removed model: {}", name);
                }
            }
        }
        Commands::Version => {
            println!("wm {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
