use std::path::{Path, PathBuf};

use petgraph::stable_graph::StableGraph;
use wm_core::engine::{GraphEdge, WikiPageMeta};

use crate::graph::ExportFormat;
use crate::shared::services::create_engine;

const OBSIDIAN_OUT_REQUIRED: &str = "obsidian export requires --out <vault-dir>";

pub fn graph_export(format: ExportFormat, out: Option<PathBuf>) -> Result<(), anyhow::Error> {
    let (engine, wiki_dir) = create_engine();
    let snapshot = engine.state.graph.load();
    let graph = &snapshot.0;
    match format {
        ExportFormat::Json => export_json(graph, out),
        ExportFormat::Graphml => export_graphml(graph, out),
        ExportFormat::Obsidian => export_obsidian_vault(graph, &wiki_dir, out),
    }
}

fn export_json(
    graph: &StableGraph<WikiPageMeta, GraphEdge>,
    out: Option<PathBuf>,
) -> Result<(), anyhow::Error> {
    let text = serde_json::to_string_pretty(&wm_core::graph::export::graph_to_json(graph))?;
    if let Some(path) = out {
        std::fs::write(&path, text)?;
        println!("Graph snapshot written to {}", path.display());
        return Ok(());
    }
    println!("{text}");
    Ok(())
}

fn export_graphml(
    graph: &StableGraph<WikiPageMeta, GraphEdge>,
    out: Option<PathBuf>,
) -> Result<(), anyhow::Error> {
    let xml = wm_core::graph::export::graph_to_graphml(graph);
    if let Some(path) = out {
        std::fs::write(&path, xml)?;
        println!("GraphML written to {}", path.display());
        return Ok(());
    }
    print!("{xml}");
    Ok(())
}

fn export_obsidian_vault(
    graph: &StableGraph<WikiPageMeta, GraphEdge>,
    wiki_dir: &Path,
    out: Option<PathBuf>,
) -> Result<(), anyhow::Error> {
    let Some(vault) = out else {
        eprintln!("{}", OBSIDIAN_OUT_REQUIRED);
        return Ok(());
    };
    let result = wm_core::graph::export::export_obsidian(graph, wiki_dir, &vault)?;
    println!(
        "Exported {} pages with {} wikilinks to {}",
        result.pages,
        result.wikilinks,
        vault.display()
    );
    Ok(())
}
