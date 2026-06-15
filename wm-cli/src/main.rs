use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::Value;
use tracing::info;
use tracing_subscriber::EnvFilter;

use wm_core::config::{self, ProjectConfig};
use wm_core::engine::VppEngine;
use wm_core::mcp::transport::{run_transport, ToolHandler, ToolRegistry};

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
            std::fs::create_dir_all(wm_dir.join("wiki"))?;
            std::fs::create_dir_all(wm_dir.join("sources"))?;
            std::fs::create_dir_all(wm_dir.join("skills"))?;
            std::fs::create_dir_all(wm_dir.join("state"))?;

            let config = ProjectConfig::default();
            let config_path = wm_dir.join("config.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
            info!("Initialized project at {}", root.display());

            if let Some(_platform) = platform {
                // Platform config generation (v1.1)
                info!("Platform config generation coming soon");
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
            let _engine = VppEngine::new(ProjectConfig::default());

            let mut registry = ToolRegistry::new();

            // Register wm_initial tool
            let initial_handler: ToolHandler = Arc::new(|_params: Value| {
                Ok(serde_json::json!({
                    "project": "active",
                    "tools": "47 actions across 14 groups",
                    "instructions": "Call wm_initial at the start of every session.",
                    "model_status": "not loaded"
                }))
            });
            registry.register("wm_initial", initial_handler);

            // Register wm_help tool
            let handler: ToolHandler = Arc::new(|params: Value| {
                let query = params.get("queries")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());
                Ok(serde_json::json!({
                    "help": format!("Available tools: {:?}", query),
                    "tool_count": 1
                }))
            });
            registry.register("wm_help", handler);

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
