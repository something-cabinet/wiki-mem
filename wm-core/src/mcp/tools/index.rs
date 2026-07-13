use std::sync::Arc;
use tracing;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;

// ─── Input types ───────────────────────────────────────────────

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

/// Register index tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_admin(
        "wm_index.rebuild",
        "Full rebuild (graph + BM25 + embeddings)",
        move |input: WmIndexRebuildInput| {
            let skip_embed = input.skip_embed.unwrap_or(false);
            let embed_batch_size = input.embed_batch_size.unwrap_or(32);
            let root = std::env::current_dir().map_err(|e| ToolError::internal(e.to_string()))?;
            let wiki_dir = root.join(".wm").join("wiki");

            if !wiki_dir.exists() {
                return Err(ToolError::internal(
                    "No wiki directory found. Run 'wm init' first.",
                ));
            }

            let count = e.rebuild_graph(&wiki_dir);

            let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
            e.section_corpus.store(Arc::new(sections.clone()));

            let docs: Vec<crate::search::IndexedDoc> = sections
                .iter()
                .map(|s| crate::search::IndexedDoc {
                    id: s.section_id.clone(),
                    fields: vec![
                        crate::search::Field::new("header", &s.header, 4.0),
                        crate::search::Field::new("body", &s.body, 1.0),
                    ],
                })
                .collect();
            let bm25 = crate::search::Bm25Index::build(docs);
            e.bm25_index.store(Arc::new(bm25));

            let memory_dir = root.join(".wm").join("memory");
            let mem_count = e.rebuild_memory_index_from_disk(&memory_dir);

            let embed_count = if e.embedder.is_loaded() && !skip_embed {
                let old_hashes = e.vector_store.hashes.load_full();
                let old_entries = e.vector_store.entries.load_full();
                match crate::embed::rebuild_embeddings_skip_unchanged(
                    &*e.embedder,
                    &sections,
                    &old_hashes,
                    Some(&old_entries),
                    embed_batch_size,
                ) {
                    Ok((new_entries, new_hashes)) => {
                        e.vector_store.replace_entries_and_hashes(new_entries.clone(), new_hashes);
                        let root = std::env::current_dir().unwrap_or_default();
                        let vectors_path = root.join(".wm").join("state").join("wm_vectors.bin");
                        if let Err(err) = e.vector_store.save_to_disk(&vectors_path) {
                            tracing::warn!("Failed to persist vectors.bin: {}", err);
                        }
                        new_entries.len()
                    }
                    Err(err) => {
                        tracing::warn!("Embedding rebuild failed: {}", err);
                        0
                    }
                }
            } else if !e.embedder.is_loaded() && !skip_embed {
                tracing::info!("Skipping embeddings — no model loaded. Run 'wm model download'.");
                0
            } else {
                0
            };

            let _ = crate::graph::auto_generate_index(&wiki_dir, &e.graph.load().0);

            e.stale_flag
                .store(false, std::sync::atomic::Ordering::Release);

            Ok(serde_json::json!({
                "status": "ok",
                "graph_nodes": count,
                "sections": sections.len(),
                "sections_embedded": embed_count,
                "memory_indexed": mem_count,
                "message": "Full rebuild complete"
            }))
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_index.embed",
        "Build embedding vectors only",
        move |input: WmIndexEmbedInput| {
            let batch_size = input.batch_size.unwrap_or(32);

            if !e.embedder.is_loaded() {
                return Err(ToolError::internal(
                    "No embedding model loaded. Run 'wm model download' first.",
                ));
            }

            let sections = e.section_corpus.load();
            if sections.is_empty() {
                return Err(ToolError::internal(
                    "No sections found. Run 'wm index.rebuild' first.",
                ));
            }

            let old_hashes = e.vector_store.hashes.load_full();
            let old_entries = e.vector_store.entries.load_full();

            match crate::embed::rebuild_embeddings_skip_unchanged(
                &*e.embedder,
                &sections,
                &old_hashes,
                Some(&old_entries),
                batch_size,
            ) {
                Ok((new_entries, new_hashes)) => {
                    e.vector_store.replace_entries_and_hashes(new_entries.clone(), new_hashes);
                    let root = std::env::current_dir().unwrap_or_default();
                    let vectors_path = root.join(".wm").join("state").join("wm_vectors.bin");
                    if let Err(err) = e.vector_store.save_to_disk(&vectors_path) {
                        tracing::warn!("Failed to persist vectors.bin: {}", err);
                    }
                    Ok(serde_json::json!({
                        "status": "ok",
                        "sections_embedded": new_entries.len(),
                        "message": "Embedding complete"
                    }))
                }
                Err(err) => Err(ToolError::internal(format!("Embedding failed: {}", err))),
            }
        },
    );

    let e = engine.clone();
    registry.register_read(
        "wm_index.status",
        "Show index state (sections, vectors, stale)",
        move |_input: WmIndexStatusInput| {
            let (graph_nodes, graph_edges) = {
                let snap = e.graph.load();
                (snap.0.node_count(), snap.0.edge_count())
            };
            let sections = e.section_corpus.load().len();
            let bm25_docs = e.bm25_index.load().total_docs;
            let memory_docs = e.memory_index.load().total_docs;
            let vectors = e.vector_store.snapshot().len();
            let model = e.embedder.model_name().to_string();
            let embedder_loaded = e.embedder.is_loaded();
            let stale = e.stale_flag.load(std::sync::atomic::Ordering::Acquire);

            Ok(serde_json::json!({
                "graph_nodes": graph_nodes,
                "graph_edges": graph_edges,
                "sections": sections,
                "bm25_indexed": bm25_docs,
                "memory_indexed": memory_docs,
                "vectors_persisted": vectors,
                "model": model,
                "embedder_loaded": embedder_loaded,
                "stale": stale,
            }))
        },
    );
}
