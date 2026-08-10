use crate::mcp::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing;
use wm_constants::*;

#[derive(Deserialize, JsonSchema)]
struct WmIndexRebuildInput {
    #[schemars(description = "Skip embedding rebuild")]
    skip_embed: Option<bool>,
    #[schemars(description = "Batch size for embedding")]
    embed_batch_size: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
struct WmIndexEmbedInput {
    #[schemars(description = "Batch size for embedding")]
    batch_size: Option<usize>,
    #[schemars(description = "Force re-embedding of all sections")]
    force: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct WmIndexStatusInput {}

/// Resolve the active ONNX model file path so the incremental-rebuild
/// version-tracking triggers (#89/#74) can fingerprint it. Returns `None`
/// when no embedder is loaded or the model file cannot be resolved.
fn active_model_path(engine: &EngineState) -> Option<PathBuf> {
    if !engine.embedder.is_loaded() {
        return None;
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    Some(
        PathBuf::from(home)
            .join(WM_DIR)
            .join("models")
            .join(engine.embedder.model_name())
            .join("model.onnx"),
    )
}

fn rebuild_embeddings(
    engine: &EngineState,
    sections: &[crate::vector_db::SectionDoc],
    embed_batch_size: usize,
) -> usize {
    if !engine.embedder.is_loaded() {
        tracing::info!("Skipping embeddings — no model loaded. Run 'wm model download'.");
        return 0;
    }
    let old_hashes = engine.vector_store.hashes.load_full();
    let old_entries = engine.vector_store.entries.load_full();
    let model_path = active_model_path(engine);
    let old_meta = engine.vector_store.embedding_metadata();
    let current_meta = wm_embed::current_embedding_metadata(model_path.as_deref());
    match wm_embed::rebuild_embeddings_skip_unchanged(
        &*engine.embedder,
        sections,
        &old_hashes,
        Some(&old_entries),
        embed_batch_size,
        model_path.as_deref(),
        &old_meta,
    ) {
        Ok((new_entries, new_hashes)) => {
            let embed_count = new_entries.len();
            engine
                .vector_store
                .replace_entries_and_hashes(new_entries, new_hashes);
            engine.vector_store.set_embedding_metadata(current_meta);
            if let Err(err) = engine.vector_store.save_to_disk() {
                tracing::warn!("Failed to persist vectors to turso: {}", err);
            }
            embed_count
        }
        Err(err) => {
            tracing::warn!("Embedding rebuild failed: {}", err);
            0
        }
    }
}

fn embed_sections(
    engine: &EngineState,
    sections: &[crate::vector_db::SectionDoc],
    batch_size: usize,
    force: bool,
) -> Result<usize, ToolError> {
    if !engine.embedder.is_loaded() {
        return Err(ToolError::internal(
            "No embedding model loaded. Run 'wm model download' first.",
        ));
    }
    if sections.is_empty() {
        return Err(ToolError::internal(
            "No sections found. Run 'wm_index_rebuild' first.",
        ));
    }
    let old_hashes = match force {
        true => Arc::new(HashMap::new()),
        false => engine.vector_store.hashes.load_full(),
    };
    let old_entries = match force {
        true => None,
        false => Some(engine.vector_store.entries.load_full()),
    };
    let model_path = active_model_path(engine);
    let old_meta = engine.vector_store.embedding_metadata();
    let current_meta = wm_embed::current_embedding_metadata(model_path.as_deref());
    let (new_entries, new_hashes) = match wm_embed::rebuild_embeddings_skip_unchanged(
        &*engine.embedder,
        sections,
        &old_hashes,
        old_entries.as_deref(),
        batch_size,
        model_path.as_deref(),
        &old_meta,
    ) {
        Ok(result) => result,
        Err(err) => {
            return Err(ToolError::internal(format!("Embedding failed: {}", err)));
        }
    };
    let embed_count = new_entries.len();
    engine
        .vector_store
        .replace_entries_and_hashes(new_entries, new_hashes);
    engine.vector_store.set_embedding_metadata(current_meta);
    if let Err(err) = engine.vector_store.save_to_disk() {
        tracing::warn!("Failed to persist vectors to turso: {}", err);
    }
    Ok(embed_count)
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_index_rebuild",
        "Full rebuild — graph, BM25 index, and embeddings",
        {
            let engine = engine.clone();
            move |input: WmIndexRebuildInput| {
                let skip_embed = input.skip_embed.unwrap_or(false);
                let embed_batch_size = input.embed_batch_size.unwrap_or(EMBED_BATCH_SIZE);
                let root =
                    std::env::current_dir().map_err(|e| ToolError::internal(e.to_string()))?;
                let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);

                if !wiki_dir.exists() {
                    return Err(ToolError::internal(
                        "No wiki directory found. Run 'wm init' first.",
                    ));
                }

                let count = engine.rebuild_graph(&wiki_dir);

                let sections = Arc::new(crate::graph::build_sections_from_wiki(&wiki_dir));
                engine.section_corpus.store(sections.clone());

                let docs: Vec<crate::search::IndexedDoc> = sections
                    .iter()
                    .map(crate::search::indexed_doc_from_section)
                    .collect();
                let bm25 = crate::search::Bm25Index::build(docs);
                engine.bm25_index.store(Arc::new(bm25));

                let embed_count = match skip_embed {
                    true => 0,
                    false => rebuild_embeddings(&engine, &sections, embed_batch_size),
                };

                let _ = crate::graph::auto_generate_index(&wiki_dir, &engine.graph.load().0);

                engine
                    .stale_flag
                    .store(false, std::sync::atomic::Ordering::Release);

                Ok(serde_json::json!({
                    "status": "ok",
                    "graph_nodes": count,
                    "sections": sections.len(),
                    "sections_embedded": embed_count,
                    "message": "Full rebuild complete"
                }))
            }
        },
    );

    registry.register_typed(
        "wm_index_status",
        "Show index state (nodes, sections, vectors, stale)",
        {
            let engine = engine.clone();
            move |_input: WmIndexStatusInput| {
                let (graph_nodes, graph_edges) = {
                    let snap = engine.graph.load();
                    (snap.0.node_count(), snap.0.edge_count())
                };
                let sections = engine.section_corpus.load().len();
                let bm25_docs = engine.bm25_index.load().total_docs;
                let vectors = engine.vector_store.snapshot().len();
                let model = engine.embedder.model_name().to_string();
                let embedder_loaded = engine.embedder.is_loaded();
                let stale = engine.stale_flag.load(std::sync::atomic::Ordering::Acquire);

                Ok(serde_json::json!({
                    "graph_nodes": graph_nodes,
                    "graph_edges": graph_edges,
                    "sections": sections,
                    "bm25_indexed": bm25_docs,
                    "vectors_persisted": vectors,
                    "model": model,
                    "embedder_loaded": embedder_loaded,
                    "stale": stale,
                }))
            }
        },
    );

    registry.register_typed("wm_index_embed", "Build embedding vectors only", {
        let engine = engine.clone();
        move |input: WmIndexEmbedInput| {
            let batch_size = input.batch_size.unwrap_or(EMBED_BATCH_SIZE);
            let force = input.force.unwrap_or(false);
            let sections = engine.section_corpus.load();
            let embed_count = embed_sections(&engine, &sections, batch_size, force)?;
            Ok(serde_json::json!({
                "status": "ok",
                "sections_embedded": embed_count,
                "message": "Embedding complete"
            }))
        }
    });
}
