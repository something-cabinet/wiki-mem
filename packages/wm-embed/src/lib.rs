use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use sha2::{Digest, Sha256};

pub mod vector_db;

pub mod models;
pub mod services;

pub use models::*;
pub use services::*;

#[cfg(feature = "onnx")]
pub use services::onnx::{OnnxEmbedder, download_model};

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f32 = a[..len].iter().zip(&b[..len]).map(|(x, y)| x * y).sum();
    dot.clamp(0.0, 1.0) as f64
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

fn read_vectors_bin(
    data: &[u8],
) -> Result<(String, HashMap<String, Vec<f32>>, HashMap<String, [u8; 32]>), String> {
    const MAGIC: [u8; 4] = [b'W', b'M', b'V', 0];
    const VERSION: u32 = 1;

    if data.len() < 24 {
        return Err("file too short".into());
    }

    let mut offset = 0usize;

    let mut magic = [0u8; 4];
    magic.copy_from_slice(&data[offset..offset + 4]);
    offset += 4;
    if magic != MAGIC {
        return Err("invalid magic bytes".into());
    }

    let version = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;
    if version != VERSION {
        return Err(format!("unsupported version: {}", version));
    }

    let dim = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;

    let count = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    offset += 8;

    let model_name_len =
        u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    if offset + model_name_len > data.len() {
        return Err("truncated file: model_name".into());
    }
    let model_name =
        String::from_utf8_lossy(&data[offset..offset + model_name_len]).to_string();

    let model_name_padded = model_name_len.div_ceil(32) * 32;
    offset = 24usize.checked_add(model_name_padded).unwrap_or(data.len());

    let dim_usize = dim as usize;
    let count_usize = count as usize;
    let mut entries = HashMap::with_capacity(count_usize);
    let mut hashes = HashMap::with_capacity(count_usize);

    for _ in 0..count {
        if offset + 4 > data.len() {
            return Err("truncated file: id_len".into());
        }
        let id_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + id_len > data.len() {
            return Err("truncated file: id".into());
        }
        let id = String::from_utf8_lossy(&data[offset..offset + id_len]).to_string();
        offset += id_len.div_ceil(8) * 8;

        if offset + 32 > data.len() {
            return Err("truncated file: content_hash".into());
        }
        let mut content_hash = [0u8; 32];
        content_hash.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        let vec_len = dim_usize * 4;
        if offset + vec_len > data.len() {
            return Err("truncated file: vector data".into());
        }
        let mut vec = Vec::with_capacity(dim_usize);
        for i in 0..dim_usize {
            let start = offset + i * 4;
            let val = f32::from_le_bytes(data[start..start + 4].try_into().unwrap());
            vec.push(val);
        }
        offset += vec_len;

        entries.insert(id.clone(), vec);
        hashes.insert(id, content_hash);
    }

    Ok((model_name, entries, hashes))
}

pub fn migrate_vectors_bin_to_turso(project_root: &Path) -> Result<usize, String> {
    let bin_path = project_root.join(".wm").join("state").join("vectors.bin");
    if !bin_path.exists() {
        return Ok(0);
    }

    let data = std::fs::read(&bin_path).map_err(|e| format!("read vectors.bin error: {}", e))?;
    let (_model_name, raw_entries, raw_hash_map) =
        read_vectors_bin(&data).map_err(|e| format!("parse vectors.bin error: {}", e))?;

    let dim = raw_entries
        .values()
        .next()
        .map(|v| v.len() as u32)
        .unwrap_or(0);

    let db_path = project_root.join(".wm").join("state").join("vectors.db");
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

/// This return type is complex because it bundles the full embedding state:
/// new embedding vectors + their content hashes. Extracting a type alias
/// would add indirection without improving readability at call sites.
#[allow(clippy::type_complexity)]
pub fn rebuild_embeddings_skip_unchanged(
    embedder: &dyn services::Embedder,
    sections: &[crate::vector_db::SectionDoc],
    old_hashes: &HashMap<String, [u8; 32]>,
    old_entries_snap: Option<&HashMap<String, crate::vector_db::EmbedVector>>,
    batch_size: usize,
) -> Result<(HashMap<String, crate::vector_db::EmbedVector>, HashMap<String, [u8; 32]>), crate::vector_db::EmbedError> {
    let mut new_entries = HashMap::new();

    let phase1: Vec<(String, [u8; 32], bool)> = sections
        .par_iter()
        .map(|sec| {
            let h = Sha256::digest(sec.body.as_bytes());
            let hash_bytes: [u8; 32] = h.into();
            let changed = old_hashes.get(&sec.section_id) != Some(&hash_bytes);
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

    for chunk in to_embed.chunks(batch_size) {
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

        let (entries, hashes) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &HashMap::new(), None, 32).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(hashes.contains_key("s1"));

        let (entries2, _) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &hashes, Some(&entries), 32).unwrap();
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

        let (old_entries, old_hashes) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &HashMap::new(), None, 32).unwrap();
        let old_vec = old_entries["s1"].0.clone();

        sections[0].body = "modified content".into();
        let (new_entries, _) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &old_hashes, Some(&old_entries), 32).unwrap();

        assert_ne!(old_vec, new_entries["s1"].0);
    }
}
