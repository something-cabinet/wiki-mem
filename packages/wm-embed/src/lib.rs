use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use wm_constants::*;

pub mod vector_db;

pub mod models;
pub mod services;

pub use models::*;
pub use services::*;

#[cfg(feature = "onnx")]
pub use services::onnx::{download_model, EmbeddingModel};

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f32 = a[..len].iter().zip(&b[..len]).map(|(x, y)| x * y).sum();
    f64::from(dot.clamp(0.0, 1.0))
}

pub fn top_k_cosine(
    query: &[f32],
    vectors: &HashMap<String, crate::vector_db::EmbedVector>,
    k: usize,
) -> Vec<(String, f64)> {
    let mut results: Vec<(String, f64)> = vectors
        .par_iter()
        .map(|(doc_id, ev)| {
            let sim = cosine_similarity(query, &ev.0);
            (doc_id.clone(), sim)
        })
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    results.truncate(k);
    results
}

pub fn rrf_fusion(
    bm25_results: &[(String, f64)],
    semantic_results: &[(String, f64)],
    rrf_k: f64,
) -> Vec<(String, f64)> {
    use std::collections::HashMap as Hm;

    let mut scores: Hm<&str, f64> = Hm::new();

    for (rank, (id, _)) in bm25_results.iter().enumerate() {
        *scores.entry(id.as_str()).or_insert(0.0) += 1.0 / (rrf_k + rank as f64);
    }

    for (rank, (id, _)) in semantic_results.iter().enumerate() {
        *scores.entry(id.as_str()).or_insert(0.0) += 1.0 / (rrf_k + rank as f64);
    }

    let mut fused: Vec<(String, f64)> = scores
        .into_iter()
        .map(|(id, s)| (id.to_string(), s))
        .collect();
    fused.sort_by(|(a_id, a_score), (b_id, b_score)| {
        b_score
            .partial_cmp(a_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                let a_both = semantic_results.iter().any(|(id, _)| id == a_id);
                let b_both = semantic_results.iter().any(|(id, _)| id == b_id);
                b_both.cmp(&a_both)
            })
    });

    fused
}

fn read_vectors_bin(data: &[u8]) -> Result<VectorsBinData, String> {
    const MAGIC: [u8; 4] = [b'W', b'M', b'V', 0];
    const VERSION: u32 = 1;

    if data.len() < 24 {
        return Err("file too short".into());
    }

    let mut offset = 0usize;

    let mut magic = [0u8; 4];
    let end = offset.checked_add(4).ok_or("overflow: magic")?;
    magic.copy_from_slice(&data[offset..end]);
    offset = end;
    if magic != MAGIC {
        return Err("invalid magic bytes".into());
    }

    let end = offset.checked_add(4).ok_or("overflow: version")?;
    let version = u32::from_le_bytes(data[offset..end].try_into().unwrap());
    offset = end;
    if version != VERSION {
        return Err(format!("unsupported version: {}", version));
    }

    let end = offset.checked_add(4).ok_or("overflow: dim")?;
    let dim = u32::from_le_bytes(data[offset..end].try_into().unwrap());
    offset = end;

    let end = offset.checked_add(8).ok_or("overflow: count")?;
    let count = u64::from_le_bytes(data[offset..end].try_into().unwrap());
    offset = end;

    let end = offset.checked_add(4).ok_or("overflow: model_name_len")?;
    let model_name_len: usize =
        usize::try_from(u32::from_le_bytes(data[offset..end].try_into().unwrap())).unwrap_or(0);
    offset = end;

    let end = offset
        .checked_add(model_name_len)
        .ok_or("overflow: model_name slice")?;
    if end > data.len() {
        return Err("truncated file: model_name".into());
    }
    let model_name = String::from_utf8_lossy(&data[offset..end]).to_string();

    let model_name_padded = model_name_len
        .div_ceil(32)
        .checked_mul(32)
        .ok_or("overflow: model_name padding")?;
    offset = 24usize.checked_add(model_name_padded).unwrap_or(data.len());

    let dim_usize: usize = usize::try_from(dim).unwrap_or(0);
    let count_usize: usize = usize::try_from(count).unwrap_or(0);
    let mut entries = HashMap::with_capacity(count_usize);
    let mut hashes = HashMap::with_capacity(count_usize);

    for _ in 0..count_usize {
        let end = offset.checked_add(4).ok_or("overflow: id_len")?;
        if end > data.len() {
            return Err("truncated file: id_len".into());
        }
        let id_len: usize =
            usize::try_from(u32::from_le_bytes(data[offset..end].try_into().unwrap())).unwrap_or(0);
        offset = end;

        let end = offset.checked_add(id_len).ok_or("overflow: id")?;
        if end > data.len() {
            return Err("truncated file: id".into());
        }
        let id = String::from_utf8_lossy(&data[offset..end]).to_string();
        let id_padded = id_len
            .div_ceil(8)
            .checked_mul(8)
            .ok_or("overflow: id padding")?;
        offset = offset.checked_add(id_padded).ok_or("overflow: id offset")?;

        let end = offset.checked_add(32).ok_or("overflow: content_hash")?;
        if end > data.len() {
            return Err("truncated file: content_hash".into());
        }
        let mut content_hash = [0u8; 32];
        content_hash.copy_from_slice(&data[offset..end]);
        offset = end;

        let vec_len = dim_usize.checked_mul(4).ok_or("overflow: vec_len")?;
        let end = offset.checked_add(vec_len).ok_or("overflow: vector data")?;
        if end > data.len() {
            return Err("truncated file: vector data".into());
        }
        let mut vec = Vec::with_capacity(dim_usize);
        for i in 0..dim_usize {
            let elem_start = offset
                .checked_add(i.checked_mul(4).ok_or("overflow: elem offset")?)
                .ok_or("overflow: elem start")?;
            let elem_end = elem_start.checked_add(4).ok_or("overflow: elem end")?;
            let val = f32::from_le_bytes(data[elem_start..elem_end].try_into().unwrap());
            vec.push(val);
        }
        offset = end;

        entries.insert(id.clone(), vec);
        hashes.insert(id, content_hash);
    }

    Ok((model_name, entries, hashes))
}

/// Migrate old `vectors.bin` format to turso (SQLite) vector database.
///
pub fn migrate_vectors_bin_to_turso(project_root: &Path) -> Result<usize, String> {
    let bin_path = project_root
        .join(WM_DIR)
        .join(STATE_DIR)
        .join(VECTOR_BIN_FILE);
    if !bin_path.exists() {
        return Ok(0);
    }

    let data = std::fs::read(&bin_path).map_err(|e| format!("read vectors.bin error: {}", e))?;
    let (_model_name, raw_entries, raw_hash_map) =
        read_vectors_bin(&data).map_err(|e| format!("parse vectors.bin error: {}", e))?;

    let dim = raw_entries
        .values()
        .next()
        .map(|v| u32::try_from(v.len()).unwrap_or(0))
        .unwrap_or(0);

    let db_path = project_root
        .join(WM_DIR)
        .join(STATE_DIR)
        .join(VECTOR_DB_FILE);
    let db = crate::vector_db::VectorDb::open(db_path, dim)
        .map_err(|e| format!("turso open error: {}", e))?;
    let db_arc = Arc::new(db);

    let raw_hashes: HashMap<String, String> = raw_hash_map
        .iter()
        .map(|(k, v)| (k.clone(), hex::encode(v)))
        .collect();

    db_arc
        .store_vectors_raw(&raw_entries, &raw_hashes)
        .map_err(|e| format!("turso write error: {}", e))?;

    std::fs::remove_file(&bin_path).map_err(|e| format!("delete vectors.bin error: {}", e))?;

    Ok(raw_entries.len())
}

/// Parsed binary vectors: (model_name, entries, content_hashes).
type VectorsBinData = (String, HashMap<String, Vec<f32>>, HashMap<String, [u8; 32]>);

/// Map of section ID to embedding vector.
pub type EmbeddingMap = HashMap<String, crate::vector_db::EmbedVector>;
/// Map of section ID to content hash.
pub type HashCache = HashMap<String, [u8; 32]>;

/// Metadata stored alongside the hash cache for change-detection logic.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingMetadata {
    /// File modification timestamp of the ONNX model at last embed time.
    pub model_modified_at: String,
    /// Version string for the chunking/section-splitting logic.
    pub chunking_version: String,
}

/// Compute the embedding metadata for the current environment.
///
/// `model_modified_at` is derived from the model file's mtime (when the file
/// exists); `chunking_version` is the crate version, which covers the
/// section-splitting logic in `wm-embed`.
///
pub fn current_embedding_metadata(model_path: Option<&std::path::Path>) -> EmbeddingMetadata {
    let model_modified_at = model_path
        .and_then(|p| std::fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string()
        })
        .unwrap_or_default();
    EmbeddingMetadata {
        model_modified_at,
        chunking_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Rebuild embeddings, skipping sections whose content hasn't changed.
///
pub fn rebuild_embeddings_skip_unchanged(
    embedder: &dyn services::Embedder,
    sections: &[crate::vector_db::SectionDoc],
    old_hashes: &HashCache,
    old_entries_snap: Option<&EmbeddingMap>,
    batch_size: usize,
    model_path: Option<&std::path::Path>,
    old_meta: &EmbeddingMetadata,
) -> Result<(EmbeddingMap, HashCache), crate::vector_db::EmbedError> {
    let mut new_entries = HashMap::new();

    // Current metadata derived from the actual model file (if provided) + crate version.
    let current_meta = current_embedding_metadata(model_path);

    // Check model version — if model file changed, force full re-embed.
    // Skip check if old_meta has no baseline (default/backward-compatible).
    let model_changed = if old_meta.model_modified_at.is_empty() {
        false
    } else {
        !current_meta.model_modified_at.is_empty()
            && current_meta.model_modified_at != old_meta.model_modified_at
    };

    // Check chunking version — if chunking logic changed, force full re-embed.
    // Skip check if old_meta has no baseline (default/backward-compatible).
    let chunking_changed = if old_meta.chunking_version.is_empty() {
        false
    } else {
        old_meta.chunking_version != env!("CARGO_PKG_VERSION")
    };

    // Establish a baseline on the first indexed run: when no metadata was ever
    // persisted (pre-version-tracking stores) but the caller can provide a
    // model path, force a full re-embed so the persisted baseline reflects the
    // current model + chunking version. Callers that pass no model path (e.g.
    // legacy CLI callers) keep the old incremental behavior.
    let baseline_established =
        !old_meta.model_modified_at.is_empty() || !old_meta.chunking_version.is_empty();
    let establish_baseline = model_path.is_some() && !baseline_established;

    let force_reembed = model_changed || chunking_changed || establish_baseline;

    let phase1: Vec<(String, [u8; 32], bool)> = sections
        .par_iter()
        .map(|sec| {
            let h = Sha256::digest(sec.body.as_bytes());
            let hash_bytes: [u8; 32] = h.into();
            let changed = force_reembed || old_hashes.get(&sec.section_id) != Some(&hash_bytes);
            (sec.section_id.clone(), hash_bytes, changed)
        })
        .collect();

    let mut new_hashes = HashMap::with_capacity(phase1.len());
    let mut to_embed: Vec<&crate::vector_db::SectionDoc> = Vec::new();
    for (i, (section_id, hash_bytes, changed)) in phase1.into_iter().enumerate() {
        new_hashes.insert(section_id, hash_bytes);
        if changed {
            to_embed.push(&sections[i]);
        }
    }

    // Adaptive batch sizing: group sections by token count.
    // Short texts (<100 tokens) use the configured batch_size.
    // Longer texts use proportionally smaller batches, capped at 32,768 total tokens.
    let max_tokens_per_batch: usize = 32768;
    let mut adaptive_batches: Vec<Vec<&crate::vector_db::SectionDoc>> = Vec::new();
    let mut current_batch: Vec<&crate::vector_db::SectionDoc> = Vec::new();
    let mut current_tokens: usize = 0;
    for sec in &to_embed {
        let token_count = sec.body.split_whitespace().count().max(1);
        let would_be_tokens = current_tokens.wrapping_add(token_count);
        if !current_batch.is_empty() && would_be_tokens > max_tokens_per_batch {
            adaptive_batches.push(std::mem::take(&mut current_batch));
            current_tokens = 0;
        }
        current_batch.push(sec);
        current_tokens = current_tokens.wrapping_add(token_count);
        if current_batch.len() >= batch_size {
            adaptive_batches.push(std::mem::take(&mut current_batch));
            current_tokens = 0;
        }
    }
    if !current_batch.is_empty() {
        adaptive_batches.push(current_batch);
    }

    // Position-change reuse: check if unchanged content exists under a different ID
    if let Some(old) = old_entries_snap {
        let old_by_hash: HashMap<&[u8; 32], &String> =
            old_hashes.iter().map(|(id, h)| (h, id)).collect();
        for sec in sections.iter() {
            if new_entries.contains_key(&sec.section_id) {
                continue;
            }
            if let Some(hash) = new_hashes.get(&sec.section_id) {
                // If this section's hash exists elsewhere, reuse that vector
                if let Some(old_id) = old_by_hash.get(hash) {
                    if ***old_id != sec.section_id {
                        if let Some(vec) = old.get(*old_id) {
                            new_entries.insert(sec.section_id.clone(), vec.clone());
                        }
                    }
                }
            }
        }
    }

    for chunk in &adaptive_batches {
        let texts: Vec<&str> = chunk.iter().map(|s| s.body.as_str()).collect();
        let vectors = embedder.embed_batch(&texts)?;
        for (sec, vec) in chunk.iter().zip(vectors) {
            new_entries.insert(sec.section_id.clone(), vec.normalized());
        }
    }

    if let Some(old) = old_entries_snap {
        let carry: Vec<(String, crate::vector_db::EmbedVector)> = sections
            .par_iter()
            .filter_map(|sec| {
                if !new_entries.contains_key(&sec.section_id) {
                    old.get(&sec.section_id)
                        .map(|vec| (sec.section_id.clone(), vec.clone()))
                } else {
                    None
                }
            })
            .collect();
        for (id, vec) in carry {
            new_entries.insert(id, vec);
        }
    }

    Ok((new_entries, new_hashes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_db::EmbedVector;

    #[test]
    fn test_search_mode_auto_detect() {
        assert_eq!(SearchMode::auto_detect("ERR_AUTH_401"), SearchMode::Keyword);
        assert_eq!(SearchMode::auto_detect("auth-service"), SearchMode::Keyword);
        assert_eq!(
            SearchMode::auto_detect("how does authentication work"),
            SearchMode::Hybrid
        );
    }

    #[test]
    fn test_cosine_similarity() {
        let a = EmbedVector(vec![1.0, 0.0, 0.0]).normalized();
        let b = EmbedVector(vec![1.0, 0.0, 0.0]).normalized();
        assert!((cosine_similarity(&a.0, &b.0) - 1.0).abs() < 0.001);

        let c = EmbedVector(vec![0.0, 1.0, 0.0]).normalized();
        assert!((cosine_similarity(&a.0, &c.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_rrf_fusion() {
        let bm25 = vec![("b".into(), 0.9), ("a".into(), 0.5), ("c".into(), 0.3)];
        let sem = vec![("b".into(), 0.8), ("c".into(), 0.6), ("a".into(), 0.3)];
        let fused = rrf_fusion(&bm25, &sem, 60.0);
        assert_eq!(fused.len(), 3);
        assert_eq!(fused[0].0, "b", "b is rank 1 in both lists");
    }

    #[test]
    fn test_noop_embedder() {
        let e = NoopEmbedder::new();
        assert!(!e.is_loaded());
        assert_eq!(e.output_dim(), 0);
        assert!(e.embed("test").is_err());
    }

    #[test]
    fn test_embed_vector_normalize() {
        let v = EmbedVector(vec![3.0, 0.0, 4.0]).normalized();
        let norm: f32 = v.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
        assert_eq!(v.dim(), 3);
    }

    #[test]
    fn test_mock_embedder_deterministic() {
        let m = MockEmbedder::new(4);
        let v1 = m.embed("hello").unwrap();
        let v2 = m.embed("hello").unwrap();
        assert_eq!(v1.0, v2.0, "mock should be deterministic");

        let v3 = m.embed("world").unwrap();
        assert_ne!(v1.0, v3.0, "different inputs should differ");
    }

    #[test]
    fn test_top_k_cosine_basic() {
        let mut vectors = HashMap::new();
        vectors.insert("a".into(), EmbedVector(vec![1.0, 0.0]).normalized());
        vectors.insert("b".into(), EmbedVector(vec![0.0, 1.0]).normalized());
        vectors.insert("c".into(), EmbedVector(vec![0.9, 0.1]).normalized());

        let query = EmbedVector(vec![1.0, 0.0]).normalized();
        let results = top_k_cosine(&query.0, &vectors, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "a", "most similar to query");
    }

    #[test]
    fn test_turso_roundtrip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = VectorStore::new("test", tmp.path());

        let mut entries = HashMap::new();
        entries.insert(
            "test:id:1".into(),
            EmbedVector(vec![1.0, 0.0, 0.0]).normalized(),
        );
        entries.insert(
            "test:id:2".into(),
            EmbedVector(vec![0.0, 1.0, 0.0]).normalized(),
        );

        let mut hashes = HashMap::new();
        hashes.insert("test:id:1".into(), [1u8; 32]);
        hashes.insert("test:id:2".into(), [2u8; 32]);

        store.replace_entries_and_hashes(entries, hashes);
        store.save_to_disk().unwrap();

        let loaded = VectorStore::load_from_disk(tmp.path()).unwrap();
        assert_eq!(loaded.snapshot().len(), 2);
        assert!(loaded.snapshot().contains_key("test:id:1"));
        assert!(loaded.snapshot().contains_key("test:id:2"));
    }

    #[test]
    fn test_vector_store_swap_and_snapshot() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = VectorStore::new("test", tmp.path());
        assert!(store.snapshot().is_empty());

        let mut entries = HashMap::new();
        entries.insert("a".into(), EmbedVector(vec![1.0]).normalized());
        let mut hashes = HashMap::new();
        hashes.insert("a".into(), [0u8; 32]);
        store.replace_entries_and_hashes(entries, hashes);

        assert_eq!(store.snapshot().len(), 1);
        assert!(store.snapshot().contains_key("a"));
    }

    #[test]
    fn test_build_embeddings_no_changes() {
        let embedder = MockEmbedder::new(4);
        let sections = vec![crate::vector_db::SectionDoc {
            section_id: "s1".into(),
            page_id: "p1".into(),
            header: "H1".into(),
            body: "hello world".into(),
            title: "Page 1".into(),
            tags: vec![],
        }];

        let meta = EmbeddingMetadata::default();
        let (entries, hashes) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &HashMap::new(),
            None,
            32,
            None,
            &meta,
        )
        .unwrap();
        assert_eq!(entries.len(), 1);
        assert!(hashes.contains_key("s1"));

        let (entries2, _) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &hashes,
            Some(&entries),
            32,
            None,
            &meta,
        )
        .unwrap();
        assert_eq!(entries2.len(), 1);
        assert_eq!(entries["s1"].0, entries2["s1"].0);
    }

    #[test]
    fn test_build_embeddings_detects_change() {
        let embedder = MockEmbedder::new(4);
        let mut sections = vec![crate::vector_db::SectionDoc {
            section_id: "s1".into(),
            page_id: "p1".into(),
            header: "H1".into(),
            body: "original content".into(),
            title: "Page 1".into(),
            tags: vec![],
        }];

        let meta = EmbeddingMetadata::default();
        let (old_entries, old_hashes) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &HashMap::new(),
            None,
            32,
            None,
            &meta,
        )
        .unwrap();
        let old_vec = old_entries["s1"].0.clone();

        sections[0].body = "modified content".into();
        let (new_entries, _) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &old_hashes,
            Some(&old_entries),
            32,
            None,
            &meta,
        )
        .unwrap();

        assert_ne!(old_vec, new_entries["s1"].0);
    }

    /// An embedder that counts how many texts it embeds, so tests can prove a
    /// full re-embed happened (vs. an incremental no-op).
    struct CountingEmbedder {
        inner: MockEmbedder,
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl CountingEmbedder {
        fn new(dim: usize) -> Self {
            Self {
                inner: MockEmbedder::new(dim),
                calls: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            }
        }

        fn count(&self) -> usize {
            self.calls.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    impl crate::vector_db::Embedder for CountingEmbedder {
        fn embed(&self, text: &str) -> Result<EmbedVector, crate::vector_db::EmbedError> {
            self.calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.inner.embed(text)
        }
        fn is_loaded(&self) -> bool {
            true
        }
        fn model_name(&self) -> &str {
            "counting"
        }
        fn output_dim(&self) -> usize {
            self.inner.output_dim()
        }
    }

    fn two_section_docs() -> Vec<crate::vector_db::SectionDoc> {
        vec![
            crate::vector_db::SectionDoc {
                section_id: "wiki:p1#alpha".into(),
                page_id: "wiki:p1".into(),
                header: "Alpha".into(),
                body: "hello world".into(),
                title: "Page 1".into(),
                tags: vec![],
            },
            crate::vector_db::SectionDoc {
                section_id: "wiki:p1#beta".into(),
                page_id: "wiki:p1".into(),
                header: "Beta".into(),
                body: "foo bar baz".into(),
                title: "Page 1".into(),
                tags: vec![],
            },
        ]
    }

    /// #89 — a changed model file (different mtime) forces a full re-embed.
    #[test]
    fn test_model_change_triggers_full_reembed() {
        let dir = tempfile::TempDir::new().unwrap();
        let model_path = dir.path().join("model.onnx");
        std::fs::write(&model_path, b"model-v1").unwrap();

        let embedder = CountingEmbedder::new(4);
        let sections = two_section_docs();

        // First run: no persisted baseline + a model path → establish baseline.
        let (entries, hashes) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &HashMap::new(),
            None,
            32,
            Some(&model_path),
            &EmbeddingMetadata::default(),
        )
        .unwrap();
        assert_eq!(entries.len(), 2);
        let baseline_calls = embedder.count();
        assert_eq!(baseline_calls, 2, "baseline establishment embeds every section");

        // Same model file + matching baseline → incremental, nothing re-embedded.
        let current_meta = current_embedding_metadata(Some(&model_path));
        let (entries2, _) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &hashes,
            Some(&entries),
            32,
            Some(&model_path),
            &current_meta,
        )
        .unwrap();
        assert_eq!(
            embedder.count(),
            baseline_calls,
            "no re-embed when model is unchanged"
        );
        assert_eq!(entries2["wiki:p1#alpha"].0, entries["wiki:p1#alpha"].0);

        // Model file replaced (stale baseline mtime) → full re-embed.
        let stale_meta = EmbeddingMetadata {
            model_modified_at: "2000-01-01T00:00:00.000Z".into(),
            chunking_version: env!("CARGO_PKG_VERSION").into(),
        };
        let (entries3, _) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &hashes,
            Some(&entries2),
            32,
            Some(&model_path),
            &stale_meta,
        )
        .unwrap();
        assert_eq!(
            embedder.count(),
            baseline_calls + 2,
            "model change must trigger a full re-embed"
        );
        assert_eq!(entries3.len(), 2);
    }

    /// #74 — a changed chunking version forces a full re-embed.
    #[test]
    fn test_chunking_version_change_triggers_full_reembed() {
        let dir = tempfile::TempDir::new().unwrap();
        let model_path = dir.path().join("model.onnx");
        std::fs::write(&model_path, b"model-v1").unwrap();

        let embedder = CountingEmbedder::new(4);
        let sections = two_section_docs();

        let (entries, hashes) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &HashMap::new(),
            None,
            32,
            Some(&model_path),
            &EmbeddingMetadata::default(),
        )
        .unwrap();
        let baseline_calls = embedder.count();

        // Baseline carries an older chunking_version → mismatch forces re-embed.
        let stale_meta = EmbeddingMetadata {
            model_modified_at: current_embedding_metadata(Some(&model_path)).model_modified_at,
            chunking_version: "0.0.0".into(),
        };
        let (entries2, _) = rebuild_embeddings_skip_unchanged(
            &embedder,
            &sections,
            &hashes,
            Some(&entries),
            32,
            Some(&model_path),
            &stale_meta,
        )
        .unwrap();
        assert_eq!(
            embedder.count(),
            baseline_calls + 2,
            "chunking version change must trigger a full re-embed"
        );
        assert_eq!(entries2.len(), 2);
    }

    /// #14 — VectorStore upsert (page CRUD) adds vectors to both memory + turso.
    #[test]
    fn test_upsert_sections_adds_to_store() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = VectorStore::new("test", tmp.path());

        let mut entries = HashMap::new();
        entries.insert(
            "wiki:p1#alpha".into(),
            EmbedVector(vec![1.0, 0.0]).normalized(),
        );
        let mut hashes = HashMap::new();
        hashes.insert("wiki:p1#alpha".into(), [1u8; 32]);
        store.upsert_sections(entries, hashes);

        assert!(store.snapshot().contains_key("wiki:p1#alpha"));
        let results = store.search_turso(&[1.0, 0.0], 10);
        assert!(
            results.iter().any(|(id, _)| id == "wiki:p1#alpha"),
            "upserted vector must be searchable from turso"
        );
    }

    /// #14 — removing a page's sections clears its vectors from memory + turso.
    #[test]
    fn test_remove_sections_for_page_clears_store() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = VectorStore::new("test", tmp.path());

        let mut entries = HashMap::new();
        entries.insert(
            "wiki:p1#alpha".into(),
            EmbedVector(vec![1.0, 0.0]).normalized(),
        );
        entries.insert(
            "wiki:p2#beta".into(),
            EmbedVector(vec![0.0, 1.0]).normalized(),
        );
        let mut hashes = HashMap::new();
        hashes.insert("wiki:p1#alpha".into(), [1u8; 32]);
        hashes.insert("wiki:p2#beta".into(), [2u8; 32]);
        store.replace_entries_and_hashes(entries, hashes);
        store.save_to_disk().unwrap();

        store.remove_sections_for_page("wiki:p1");

        let snapshot = store.snapshot();
        assert!(!snapshot.contains_key("wiki:p1#alpha"));
        assert!(snapshot.contains_key("wiki:p2#beta"));
        let results = store.search_turso(&[1.0, 0.0], 10);
        assert!(
            !results.iter().any(|(id, _)| id == "wiki:p1#alpha"),
            "removed page's vector must not be searchable from turso"
        );
    }

    /// #89/#74 — embedding metadata persists across store reloads.
    #[test]
    fn test_metadata_persists_across_store_reload() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = VectorStore::new("test", tmp.path());
        let meta = EmbeddingMetadata {
            model_modified_at: "2026-01-01T00:00:00.000Z".into(),
            chunking_version: env!("CARGO_PKG_VERSION").into(),
        };
        store.set_embedding_metadata(meta.clone());
        store.save_to_disk().unwrap();

        let loaded = VectorStore::load_from_disk(tmp.path()).unwrap();
        assert_eq!(
            loaded.embedding_metadata().model_modified_at,
            meta.model_modified_at
        );
        assert_eq!(
            loaded.embedding_metadata().chunking_version,
            meta.chunking_version
        );
    }
}
