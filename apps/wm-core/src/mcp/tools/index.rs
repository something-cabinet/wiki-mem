use std::collections::HashMap;
use std::sync::Arc;
use tracing;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;

// ─── Input structs ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct RebuildInput {
    #[schemars(description = "Skip embedding rebuild")]
    skip_embed: Option<bool>,
    #[schemars(description = "Batch size for embedding")]
    embed_batch_size: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
struct EmbedInput {
    #[schemars(description = "Batch size for embedding")]
    batch_size: Option<usize>,
    #[schemars(description = "Force re-embedding of all sections")]
    force: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct StatusInput {}

/// Register index tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_index.rebuild ────────────────────────────────────
    registry.register_typed(
        "wm_index.rebuild",
        "Full rebuild — graph, BM25 index, and embeddings",
        {
            let engine = engine.clone();
            move |input: RebuildInput| {
                let skip_embed = input.skip_embed.unwrap_or(false);
                let embed_batch_size = input.embed_batch_size.unwrap_or(32);
                let root =
                    std::env::current_dir().map_err(|e| ToolError::internal(e.to_string()))?;
                let wiki_dir = root.join(".wm").join("wiki");

                if !wiki_dir.exists() {
                    return Err(ToolError::internal(
                        "No wiki directory found. Run 'wm init' first.",
                    ));
                }

                let count = engine.rebuild_graph(&wiki_dir);

                let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
                engine.section_corpus.store(Arc::new(sections.clone()));

                let docs: Vec<crate::search::IndexedDoc> = sections
                    .iter()
                    .map(|s| crate::search::IndexedDoc {
                        id: s.section_id.clone(),
                        fields: vec![
                            crate::search::Field::new("header", &s.header, 4.0),
                            crate::search::Field::new("body", &s.body, 1.0),
                            crate::search::Field::new("id", &s.section_id, 0.0),
                            crate::search::Field::new("title", &s.title, 0.0),
                            crate::search::Field::new("tags", &s.tags.join(" "), 0.0),
                        ],
                    })
                    .collect();
                let bm25 = crate::search::Bm25Index::build(docs);
                engine.bm25_index.store(Arc::new(bm25));

                let embed_count = if engine.embedder.is_loaded() && !skip_embed {
                    let old_hashes = engine.vector_store.hashes.load_full();
                    let old_entries = engine.vector_store.entries.load_full();
                    match wm_embed::rebuild_embeddings_skip_unchanged(
                        &*engine.embedder,
                        &sections,
                        &old_hashes,
                        Some(&old_entries),
                        embed_batch_size,
                    ) {
                        Ok((new_entries, new_hashes)) => {
                            let embed_count = new_entries.len();
                            let entries: HashMap<String, crate::vector_db::EmbedVector> = new_entries;
                            engine
                                .vector_store
                                .replace_entries_and_hashes(entries, new_hashes);
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
                } else if !engine.embedder.is_loaded() && !skip_embed {
                    tracing::info!("Skipping embeddings — no model loaded. Run 'wm model download'.");
                    0
                } else {
                    0
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

    // ─── wm_index.embed ──────────────────────────────────────
    registry.register_typed(
        "wm_index.embed",
        "Build embedding vectors only",
        {
            let engine = engine.clone();
            move |input: EmbedInput| {
                let batch_size = input.batch_size.unwrap_or(32);
                let force = input.force.unwrap_or(false);

                if !engine.embedder.is_loaded() {
                    return Err(ToolError::internal(
                        "No embedding model loaded. Run 'wm model download' first.",
                    ));
                }

                let sections = engine.section_corpus.load();
                if sections.is_empty() {
                    return Err(ToolError::internal(
                        "No sections found. Run 'wm index.rebuild' first.",
                    ));
                }

                let old_hashes: HashMap<String, [u8; 32]> = if force {
                    // Force re-embedding: pass empty old data so everything is re-embedded
                    HashMap::new()
                } else {
                    engine.vector_store.hashes.load_full().as_ref().clone()
                };
                let old_entries: Option<HashMap<String, crate::vector_db::EmbedVector>> = if force {
                    None
                } else {
                    Some(engine.vector_store.entries.load_full().as_ref().clone())
                };

                let (new_entries, new_hashes) = match wm_embed::rebuild_embeddings_skip_unchanged(
                    &*engine.embedder,
                    &sections,
                    &old_hashes,
                    old_entries.as_ref(),
                    batch_size,
                ) {
                    Ok(result) => result,
                    Err(err) => {
                        return Err(ToolError::internal(format!("Embedding failed: {}", err)));
                    }
                };

                let embed_count = new_entries.len();
                let entries: HashMap<String, crate::vector_db::EmbedVector> = new_entries;
                engine
                    .vector_store
                    .replace_entries_and_hashes(entries, new_hashes);
                if let Err(err) = engine.vector_store.save_to_disk() {
                    tracing::warn!("Failed to persist vectors to turso: {}", err);
                }
                Ok(serde_json::json!({
                    "status": "ok",
                    "sections_embedded": embed_count,
                    "message": "Embedding complete"
                }))
            }
        },
    );

    // ─── wm_index.status ─────────────────────────────────────
    registry.register_typed(
        "wm_index.status",
        "Show index state (nodes, sections, vectors, stale)",
        move |_input: StatusInput| {
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
        },
    );
}
