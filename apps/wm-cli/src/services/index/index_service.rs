use crate::shared::models::IndexAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;
use wm_core::config;

#[cfg(feature = "code-intel")]
use wm_constants::{STATE_DIR, WM_DIR};
#[cfg(feature = "code-intel")]
use wm_core::code_intel::services::CodeIndexDb;
#[cfg(feature = "code-intel")]
use wm_core::code_intel::CodeIndexStats;

const INDEX_REBUILD_TOOL: &str = "wm_index_rebuild";
const INDEX_EMBED_TOOL: &str = "wm_index_embed";

impl Command for IndexAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_index_action(self).await
    }
}

async fn run_index_action(action: IndexAction) -> Result<(), anyhow::Error> {
    match action {
        IndexAction::Rebuild {
            skip_embed,
            batch_size,
            since,
        } => rebuild_index(skip_embed, batch_size, since).await,
        IndexAction::Code { skip_hash_check } => refresh_code_index(skip_hash_check),
        IndexAction::Embed { batch_size, force } => embed_index(batch_size, force).await,
    }
}

async fn rebuild_index(
    skip_embed: bool,
    batch_size: usize,
    since: Option<String>,
) -> Result<(), anyhow::Error> {
    warn_unsupported_since(&since);
    let args = serde_json::json!({
        "skip_embed": skip_embed,
        "embed_batch_size": batch_size,
    });
    match call_tool(INDEX_REBUILD_TOOL, args).await {
        Ok(resp) => print_rebuild_stats(&resp),
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn warn_unsupported_since(since: &Option<String>) {
    if since.is_some() {
        eprintln!(
            "Warning: --since (cursor scanning) is not yet supported; performing a full rebuild."
        );
    }
}

fn print_rebuild_stats(resp: &serde_json::Value) {
    println!(
        "  Graph: {} nodes",
        resp["graph_nodes"].as_u64().unwrap_or(0)
    );
    println!("  Sections: {}", resp["sections"].as_u64().unwrap_or(0));
    println!("  Rebuild complete.");
}

fn refresh_code_index(skip_hash_check: bool) -> Result<(), anyhow::Error> {
    let root =
        config::detect_project_root().ok_or_else(|| anyhow::anyhow!("No project root found"))?;
    #[cfg(feature = "code-intel")]
    {
        ingest_code_index(&root, skip_hash_check)
    }
    #[cfg(not(feature = "code-intel"))]
    {
        let _ = root;
        anyhow::bail!("code-intel feature not enabled. Rebuild with --features code-intel.")
    }
}

#[cfg(feature = "code-intel")]
fn ingest_code_index(root: &std::path::Path, skip_hash_check: bool) -> Result<(), anyhow::Error> {
    use wm_core::code_intel::services::ingest_service::rebuild_code_index;

    let db = open_code_index_db(root)?;
    println!("Rebuilding code index...");
    match rebuild_code_index(&db, root, skip_hash_check) {
        Ok(stats) => print_code_index(&stats, root),
        Err(error) => anyhow::bail!("Code index rebuild failed: {}", error),
    }
    Ok(())
}

#[cfg(feature = "code-intel")]
fn open_code_index_db(root: &std::path::Path) -> Result<CodeIndexDb, anyhow::Error> {
    let db_dir = root.join(WM_DIR).join(STATE_DIR);
    let db_path = db_dir.join("code.db");
    std::fs::create_dir_all(&db_dir).ok();
    CodeIndexDb::open(db_path).map_err(|e| anyhow::anyhow!("Failed to open code db: {}", e))
}

#[cfg(feature = "code-intel")]
fn print_code_index(stats: &CodeIndexStats, root: &std::path::Path) {
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
    print_code_index_lag(root);
}

#[cfg(feature = "code-intel")]
fn print_code_index_lag(root: &std::path::Path) {
    match wm_core::engine::code_index_refresh_service::index_lag_seconds(root) {
        Ok(Some(0)) => println!("  index age: current"),
        Ok(Some(secs)) => println!("  index age: {}s behind", secs),
        Ok(None) => {}
        Err(_) => {}
    }
}

async fn embed_index(batch_size: usize, force: bool) -> Result<(), anyhow::Error> {
    let args = serde_json::json!({ "batch_size": batch_size, "force": force });
    match call_tool(INDEX_EMBED_TOOL, args).await {
        Ok(_) => println!("Embedding complete."),
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}
