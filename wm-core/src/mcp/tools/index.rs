use std::sync::Arc;
use tracing;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register index tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "wm_index.rebuild",
        "Full rebuild (graph + BM25 + embeddings)",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let skip_embed = args.optional_bool("skip_embed");
            let embed_batch_size = args.optional_int("embed_batch_size").unwrap_or(32);
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
            let mem_count = e.rebuild_memory_index(&memory_dir);

            let embed_count = if e.embedder.is_loaded() && !skip_embed {
                let old_hashes = e.vector_store.hashes.load_full();
                let old_entries = e.vector_store.entries.load_full();
                match crate::embed::build_embeddings(
                    &*e.embedder,
                    &sections,
                    &old_hashes,
                    Some(&old_entries),
                    embed_batch_size,
                ) {
                    Ok((new_entries, new_hashes)) => {
                        e.vector_store.swap(new_entries.clone(), new_hashes);
                        let root = std::env::current_dir().unwrap_or_default();
                        let vectors_path = root.join(".wm").join("state").join("vectors.bin");
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
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_index.embed",
        "Build embedding vectors only",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let batch_size = args.optional_int("batch_size").unwrap_or(32);
            let _force = args.optional_bool("force");

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

            match crate::embed::build_embeddings(
                &*e.embedder,
                &sections,
                &old_hashes,
                Some(&old_entries),
                batch_size,
            ) {
                Ok((new_entries, new_hashes)) => {
                    e.vector_store.swap(new_entries.clone(), new_hashes);
                    let root = std::env::current_dir().unwrap_or_default();
                    let vectors_path = root.join(".wm").join("state").join("vectors.bin");
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
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_index.status",
        "Show index state (sections, vectors, stale)",
        Arc::new(move |_params| {
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
        }),
    );
}
