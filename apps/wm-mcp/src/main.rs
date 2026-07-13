// ─── wm-mcp: Thin MCP Proxy ───────────────────────────
//
// Following the blog pattern (https://rup12.net/posts/write-your-mcps-in-rust/):
// Each MCP tool handler proxies to the wm-server HTTP API.
// No embedded engine, no state — pure protocol adapter.
//
// Architecture:
//   OpenCode ──stdio──► wm-mcp ──HTTP──► wm-server (localhost:PORT)

use std::sync::Arc;

use clap::Parser;
use reqwest::blocking::Client as HttpClient;
use serde_json::Value;
use tracing::info;

use wm_core::error::ToolError;
use wm_core::mcp::transport::{serve_rmcp, ToolRegistry};

#[derive(Parser)]
#[command(name = "wm-mcp", about = "Thin MCP proxy that delegates to wm-server HTTP API")]
struct Cli {
    /// Base URL of the wm-server HTTP API
    #[arg(long, default_value = "http://127.0.0.1:3000")]
    server_url: String,
}

/// Create a proxy handler that forwards params as JSON body to wm-server
fn make_handler(
    client: HttpClient,
    base_url: String,
    path: &'static str,
) -> Arc<dyn Fn(Value) -> Result<Value, ToolError> + Send + Sync> {
    let url = format!("{}/api{}", base_url, path);
    Arc::new(move |params: Value| -> Result<Value, ToolError> {
        let resp = client
            .post(&url)
            .json(&params)
            .send()
            .map_err(|e| ToolError::internal(format!("HTTP request failed: {}", e)))?;
        let body: Value = resp
            .json()
            .map_err(|e| ToolError::internal(format!("HTTP response parse failed: {}", e)))?;
        Ok(body)
    })
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();
    let base_url = cli.server_url.trim_end_matches('/').to_string();
    info!("Starting wm-mcp proxy -> {}/api", base_url);

    let client = HttpClient::new();

    // ─── Register proxy tools ─────────────────────────
    // Each tool handler proxies to wm-server via HTTP POST.
    // Tool input params are forwarded as JSON request body.

    let mut registry = ToolRegistry::new();

    // wm_search.*
    registry.register_with_desc(
        "wm_search.query",
        "Search the wiki and/or memory (keyword/semantic/hybrid)",
        make_handler(client.clone(), base_url.clone(), "/search"),
    );
    registry.register_with_desc(
        "wm_search.retrieve",
        "Context assembly with token budget",
        make_handler(client.clone(), base_url.clone(), "/search/retrieve"),
    );
    registry.register_with_desc(
        "wm_search.resolve",
        "Resolve a query to a page ID",
        make_handler(client.clone(), base_url.clone(), "/search/resolve"),
    );

    // wm_page.*
    registry.register_with_desc(
        "wm_page.get",
        "Get page content by ID",
        make_handler(client.clone(), base_url.clone(), "/pages/get"),
    );
    registry.register_with_desc(
        "wm_page.list",
        "List all wiki pages",
        make_handler(client.clone(), base_url.clone(), "/pages/list"),
    );
    registry.register_with_desc(
        "wm_page.create",
        "Create a new wiki page",
        make_handler(client.clone(), base_url.clone(), "/pages"),
    );
    registry.register_with_desc(
        "wm_page.update",
        "Update page frontmatter fields",
        make_handler(client.clone(), base_url.clone(), "/pages/update"),
    );
    registry.register_with_desc(
        "wm_page.delete",
        "Delete a page and its file",
        make_handler(client.clone(), base_url.clone(), "/pages/delete"),
    );
    registry.register_with_desc(
        "wm_page.link",
        "Add a typed edge between pages",
        make_handler(client.clone(), base_url.clone(), "/pages/link"),
    );
    registry.register_with_desc(
        "wm_page.unlink",
        "Remove a typed edge between pages",
        make_handler(client.clone(), base_url.clone(), "/pages/unlink"),
    );

    // wm_task.*
    registry.register_with_desc(
        "wm_task.board",
        "Task board grouped by status",
        make_handler(client.clone(), base_url.clone(), "/tasks/board"),
    );
    registry.register_with_desc(
        "wm_task.list",
        "List tasks with optional filters",
        make_handler(client.clone(), base_url.clone(), "/tasks/list"),
    );
    registry.register_with_desc(
        "wm_task.create",
        "Create a task wiki page",
        make_handler(client.clone(), base_url.clone(), "/tasks"),
    );
    registry.register_with_desc(
        "wm_task.get",
        "Get a task by ID",
        make_handler(client.clone(), base_url.clone(), "/tasks/get"),
    );
    registry.register_with_desc(
        "wm_task.update",
        "Update a task",
        make_handler(client.clone(), base_url.clone(), "/tasks/update"),
    );
    registry.register_with_desc(
        "wm_task.delete",
        "Delete a task by ID",
        make_handler(client.clone(), base_url.clone(), "/tasks/delete"),
    );

    // wm_graph.*
    registry.register_with_desc(
        "wm_graph.stats",
        "Graph statistics (node/edge counts by type)",
        make_handler(client.clone(), base_url.clone(), "/graph/stats"),
    );
    registry.register_with_desc(
        "wm_graph.neighbors",
        "Get typed edges from a page",
        make_handler(client.clone(), base_url.clone(), "/graph/neighbors"),
    );
    registry.register_with_desc(
        "wm_graph.path",
        "Find shortest path between two pages",
        make_handler(client.clone(), base_url.clone(), "/graph/path"),
    );
    registry.register_with_desc(
        "wm_graph.subgraph",
        "Get neighborhood around a page node",
        make_handler(client.clone(), base_url.clone(), "/graph/subgraph"),
    );

    // wm_memory.*
    registry.register_with_desc(
        "wm_memory.list",
        "List memory entries",
        make_handler(client.clone(), base_url.clone(), "/memory/list"),
    );
    registry.register_with_desc(
        "wm_memory.get",
        "Get a single memory entry by ID",
        make_handler(client.clone(), base_url.clone(), "/memory/get"),
    );
    registry.register_with_desc(
        "wm_memory.add",
        "Create a new memory entry",
        make_handler(client.clone(), base_url.clone(), "/memory"),
    );
    registry.register_with_desc(
        "wm_memory.update",
        "Update an existing memory entry",
        make_handler(client.clone(), base_url.clone(), "/memory/update"),
    );
    registry.register_with_desc(
        "wm_memory.delete",
        "Delete a memory entry by ID",
        make_handler(client.clone(), base_url.clone(), "/memory/delete"),
    );

    // wm_time.*
    registry.register_with_desc(
        "wm_time.start",
        "Start time tracking on a task",
        make_handler(client.clone(), base_url.clone(), "/time/start"),
    );
    registry.register_with_desc(
        "wm_time.stop",
        "Stop time tracking, record elapsed",
        make_handler(client.clone(), base_url.clone(), "/time/stop"),
    );
    registry.register_with_desc(
        "wm_time.add",
        "Manually add time to a task",
        make_handler(client.clone(), base_url.clone(), "/time"),
    );
    registry.register_with_desc(
        "wm_time.report",
        "Time report across all tasks",
        make_handler(client.clone(), base_url.clone(), "/time/report"),
    );

    // wm_initial
    registry.register_with_desc(
        "wm_initial",
        "Get project state, graph stats, and model status",
        make_handler(client.clone(), base_url.clone(), "/initial"),
    );

    // wm_help
    registry.register_with_desc(
        "wm_help",
        "Search tool documentation (optional: q=pattern)",
        make_handler(client.clone(), base_url.clone(), "/help"),
    );

    // wm_project.*
    registry.register_with_desc(
        "wm_project.status",
        "Project status information",
        make_handler(client.clone(), base_url.clone(), "/status"),
    );
    registry.register_with_desc(
        "wm_project.detect",
        "Detect project root from current directory",
        make_handler(client.clone(), base_url.clone(), "/project/detect"),
    );
    registry.register_with_desc(
        "wm_project.set",
        "Set the current project root",
        make_handler(client.clone(), base_url.clone(), "/project/set"),
    );

    // wm_lint.*
    registry.register_with_desc(
        "wm_lint.check",
        "Check wiki for common issues",
        make_handler(client.clone(), base_url.clone(), "/lint"),
    );
    registry.register_with_desc(
        "wm_lint.fix",
        "Auto-fix common issues",
        make_handler(client.clone(), base_url.clone(), "/lint/fix"),
    );

    // wm_validate.check
    registry.register_with_desc(
        "wm_validate.check",
        "Validate wiki health",
        make_handler(client.clone(), base_url.clone(), "/validate"),
    );

    // wm_doc.*
    registry.register_with_desc(
        "wm_doc.list",
        "List documents in the wiki",
        make_handler(client.clone(), base_url.clone(), "/doc/list"),
    );
    registry.register_with_desc(
        "wm_doc.get",
        "Read a doc by path",
        make_handler(client.clone(), base_url.clone(), "/doc/get"),
    );
    registry.register_with_desc(
        "wm_doc.create",
        "Create a new doc",
        make_handler(client.clone(), base_url.clone(), "/doc"),
    );
    registry.register_with_desc(
        "wm_doc.update",
        "Update an existing doc",
        make_handler(client.clone(), base_url.clone(), "/doc/update"),
    );
    registry.register_with_desc(
        "wm_doc.delete",
        "Delete a doc",
        make_handler(client.clone(), base_url.clone(), "/doc/delete"),
    );

    // wm_template.*
    registry.register_with_desc(
        "wm_template.list",
        "List all templates",
        make_handler(client.clone(), base_url.clone(), "/template/list"),
    );
    registry.register_with_desc(
        "wm_template.get",
        "Get a single template by name",
        make_handler(client.clone(), base_url.clone(), "/template/get"),
    );
    registry.register_with_desc(
        "wm_template.create",
        "Create a new template",
        make_handler(client.clone(), base_url.clone(), "/template"),
    );
    registry.register_with_desc(
        "wm_template.run",
        "Render a template with variable substitution",
        make_handler(client.clone(), base_url.clone(), "/template/run"),
    );

    // wm_code.*
    registry.register_with_desc(
        "wm_code.search",
        "Search source code files by text pattern",
        make_handler(client.clone(), base_url.clone(), "/code/search"),
    );
    registry.register_with_desc(
        "wm_code.symbols",
        "Find symbol definitions",
        make_handler(client.clone(), base_url.clone(), "/code/symbols"),
    );
    registry.register_with_desc(
        "wm_code.deps",
        "Show import dependencies between files",
        make_handler(client.clone(), base_url.clone(), "/code/deps"),
    );

    // wm_index.*
    registry.register_with_desc(
        "wm_index.rebuild",
        "Full rebuild (graph + BM25 + embeddings)",
        make_handler(client.clone(), base_url.clone(), "/index/rebuild"),
    );
    registry.register_with_desc(
        "wm_index.embed",
        "Build embedding vectors only",
        make_handler(client.clone(), base_url.clone(), "/index/embed"),
    );
    registry.register_with_desc(
        "wm_index.status",
        "Show index state",
        make_handler(client.clone(), base_url.clone(), "/index/status"),
    );

    // wm_decision.*
    registry.register_with_desc(
        "wm_decision.create",
        "Create a new architectural decision record",
        make_handler(client.clone(), base_url.clone(), "/decision"),
    );
    registry.register_with_desc(
        "wm_decision.get",
        "Get a decision record by ID",
        make_handler(client.clone(), base_url.clone(), "/decision/get"),
    );

    // wm_ref.*
    registry.register_with_desc(
        "wm_ref.extract",
        "Extract all @doc/, @task/, @memory/ references from markdown",
        make_handler(client.clone(), base_url.clone(), "/ref/extract"),
    );
    registry.register_with_desc(
        "wm_ref.resolve",
        "Resolve a single @reference string",
        make_handler(client.clone(), base_url.clone(), "/ref/resolve"),
    );
    registry.register_with_desc(
        "wm_ref.resolve_all",
        "Extract and resolve all @references in markdown",
        make_handler(client.clone(), base_url.clone(), "/ref/resolve-all"),
    );

    // wm_source.*
    registry.register_with_desc(
        "wm_source.list",
        "List sources with optional state filter",
        make_handler(client.clone(), base_url.clone(), "/source/list"),
    );
    registry.register_with_desc(
        "wm_source.status",
        "Get detailed source status",
        make_handler(client.clone(), base_url.clone(), "/source/status"),
    );

    // wm_log.*
    registry.register_with_desc(
        "wm_log.recent",
        "Recent log entries",
        make_handler(client.clone(), base_url.clone(), "/log/recent"),
    );
    registry.register_with_desc(
        "wm_log.since",
        "Log entries since a marker",
        make_handler(client.clone(), base_url.clone(), "/log/since"),
    );
    registry.register_with_desc(
        "wm_log.filter",
        "Filter log entries by text",
        make_handler(client.clone(), base_url.clone(), "/log/filter"),
    );

    // wm_model.*
    registry.register_with_desc(
        "wm_model.list",
        "List cached and available models",
        make_handler(client.clone(), base_url.clone(), "/model/list"),
    );
    registry.register_with_desc(
        "wm_model.status",
        "Show current model state",
        make_handler(client.clone(), base_url.clone(), "/model/status"),
    );
    registry.register_with_desc(
        "wm_model.download",
        "Download an embedding model",
        make_handler(client.clone(), base_url.clone(), "/model/download"),
    );
    registry.register_with_desc(
        "wm_model.remove",
        "Remove a cached model",
        make_handler(client.clone(), base_url.clone(), "/model/remove"),
    );

    // Start the MCP server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        info!("wm-mcp proxy ready — {} tools registered (estimated)",
            registry.list_tools().len());
        serve_rmcp(registry).await
    })
}
