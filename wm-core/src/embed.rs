use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use sha2::{Digest, Sha256};

// ─── EmbedError ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum EmbedError {
    ModelNotLoaded(String),
    Inference(String),
    Tokenization(String),
    DimensionMismatch { expected: usize, actual: usize },
    BatchTooLarge { size: usize, max: usize },
    ModelNotFound(String),
    Download(String),
}

impl fmt::Display for EmbedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbedError::ModelNotLoaded(msg) => write!(f, "model not loaded: {}", msg),
            EmbedError::Inference(msg) => write!(f, "inference error: {}", msg),
            EmbedError::Tokenization(msg) => write!(f, "tokenization error: {}", msg),
            EmbedError::DimensionMismatch { expected, actual } => {
                write!(
                    f,
                    "dimension mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            EmbedError::BatchTooLarge { size, max } => {
                write!(f, "batch size {} exceeds limit {}", size, max)
            }
            EmbedError::ModelNotFound(msg) => write!(f, "model file not found: {}", msg),
            EmbedError::Download(msg) => write!(f, "download error: {}", msg),
        }
    }
}

impl std::error::Error for EmbedError {}

// ─── EmbedVector ─────────────────────────────────────────────

/// A normalized embedding vector
#[derive(Clone, Debug, PartialEq)]
pub struct EmbedVector(pub Vec<f32>);

impl EmbedVector {
    pub fn dim(&self) -> usize {
        self.0.len()
    }

    /// L2-normalize in-place
    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.0.iter().map(|x| x * x).sum();
        if norm_sq > 1e-12 {
            let norm = norm_sq.sqrt();
            for x in &mut self.0 {
                *x /= norm;
            }
        }
    }

    /// Consume and return normalized
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

// ─── Cosine Similarity ───────────────────────────────────────

/// Cosine similarity between two L2-normalized vectors.
/// Returns 0.0 if dimensions don't match.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f32 = a[..len].iter().zip(&b[..len]).map(|(x, y)| x * y).sum();
    dot.clamp(0.0, 1.0) as f64
}

// ─── Top-K Cosine (Linear Scan) ──────────────────────────────

/// Linear scan over all stored vectors to find top-k by cosine similarity.
pub fn top_k_cosine(
    query: &[f32],
    vectors: &HashMap<String, EmbedVector>,
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

// ─── Search Mode ─────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode {
    Keyword,
    Semantic,
    Hybrid,
}

impl SearchMode {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "semantic" => SearchMode::Semantic,
            "hybrid" => SearchMode::Hybrid,
            _ => SearchMode::Keyword,
        }
    }

    /// Auto-detect: code identifiers → keyword, natural language → hybrid
    pub fn auto_detect(query: &str) -> Self {
        let has_code_pattern = query.contains('_')
            || query.contains('-')
            || query.chars().filter(|c| c.is_uppercase()).count() > 2;
        if has_code_pattern {
            SearchMode::Keyword
        } else {
            SearchMode::Hybrid
        }
    }
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchMode::Keyword => write!(f, "keyword"),
            SearchMode::Semantic => write!(f, "semantic"),
            SearchMode::Hybrid => write!(f, "hybrid"),
        }
    }
}

// ─── RRF Fusion ──────────────────────────────────────────────

/// Reciprocal Rank Fusion combining BM25 and semantic rankings.
pub fn rrf_fusion(
    bm25_results: &[(String, f64)],
    semantic_results: &[(String, f64)],
    rrf_k: f64,
) -> Vec<(String, f64)> {
    use std::collections::HashMap as Hm;

    let mut scores: Hm<&str, f64> = Hm::new();

    // BM25 contribution: 1/(k + rank)
    for (rank, (id, _)) in bm25_results.iter().enumerate() {
        *scores.entry(id.as_str()).or_insert(0.0) +=         1.0 / (rrf_k + rank as f64);
    }

    // Semantic contribution: 1/(k + rank)
    for (rank, (id, _)) in semantic_results.iter().enumerate() {
        *scores.entry(id.as_str()).or_insert(0.0) +=         1.0 / (rrf_k + rank as f64);
    }

    // Sort descending by RRF score, tie-breaker: doc in BOTH lists
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

// ─── Embedder Trait ─────────────────────────────────────────

pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbedVector>, EmbedError> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    fn is_loaded(&self) -> bool;
    fn model_name(&self) -> &str;
    fn output_dim(&self) -> usize;
}

// ─── NoopEmbedder ────────────────────────────────────────────

/// Always reports "not loaded". Used when embed feature is off or no model downloaded.
pub struct NoopEmbedder {
    dim: usize,
}

impl Default for NoopEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl NoopEmbedder {
    pub fn new() -> Self {
        Self { dim: 0 }
    }
}

impl Embedder for NoopEmbedder {
    fn embed(&self, _text: &str) -> Result<EmbedVector, EmbedError> {
        Err(EmbedError::ModelNotLoaded("no embedder configured".into()))
    }
    fn is_loaded(&self) -> bool {
        false
    }
    fn model_name(&self) -> &str {
        "none"
    }
    fn output_dim(&self) -> usize {
        self.dim
    }
}

// ─── MockEmbedder ────────────────────────────────────────────

/// Deterministic mock using DefaultHasher seeded by text.
/// Same text always produces same vector. Used in tests.
pub struct MockEmbedder {
    dim: usize,
}

impl MockEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn hash_vec(&self, text: &str) -> EmbedVector {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        let mut rng = oorandom::Rand64::new(seed.into());
        let mut vec = Vec::with_capacity(self.dim);
        for _ in 0..self.dim {
            vec.push(rng.rand_float() as f32);
        }
        EmbedVector(vec).normalized()
    }
}

impl Embedder for MockEmbedder {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError> {
        Ok(self.hash_vec(text))
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn model_name(&self) -> &str {
        "mock"
    }
    fn output_dim(&self) -> usize {
        self.dim
    }
}

// ─── VectorStore ─────────────────────────────────────────────

/// Thread-safe vector store with ArcSwap for lock-free reads.
pub struct VectorStore {
    /// section_id → L2-normalized EmbedVector
    pub entries: ArcSwap<HashMap<String, EmbedVector>>,
    /// Which model produced these vectors
    pub model_name: String,
    /// Section content hash → skip re-embedding if unchanged
    pub hashes: ArcSwap<HashMap<String, [u8; 32]>>,
}

impl VectorStore {
    pub fn new(model_name: &str) -> Self {
        Self {
            entries: ArcSwap::from_pointee(HashMap::new()),
            model_name: model_name.to_string(),
            hashes: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    /// Atomically swap in new maps (lock-free reads).
    pub fn replace_entries_and_hashes(
        &self,
        new_entries: HashMap<String, EmbedVector>,
        new_hashes: HashMap<String, [u8; 32]>,
    ) {
        self.entries.store(Arc::new(new_entries));
        self.hashes.store(Arc::new(new_hashes));
    }

    /// Get a consistent snapshot for queries (Arc clone, no lock).
    pub fn snapshot(&self) -> Arc<HashMap<String, EmbedVector>> {
        self.entries.load_full()
    }

    /// Load from vectors.bin on disk
    pub fn load_from_disk(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("read error: {}", e))?;
        let (header, entries, hashes) = VectorsBin::read(&data)?;
        let store = Self {
            entries: ArcSwap::from_pointee(entries),
            model_name: header.model_name.clone(),
            hashes: ArcSwap::from_pointee(hashes),
        };
        Ok(store)
    }

    /// Write to vectors.bin on disk
    pub fn save_to_disk(&self, path: &Path) -> Result<(), String> {
        // Single atomic snapshot to prevent TOCTOU between entries and hashes
        let entries_arc = self.entries.load_full();
        let hashes_arc = self.hashes.load_full();
        let data = VectorsBin::write(&self.model_name, &entries_arc, &hashes_arc)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir error: {}", e))?;
        }
        // Atomic write: write to temp file, then rename to prevent partial file on crash
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &data).map_err(|e| format!("write error: {}", e))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("rename error: {}", e))?;
        Ok(())
    }
}

// ─── vectors.bin Binary Format ───────────────────────────────

/// Header (32-byte aligned)
#[allow(dead_code)]
struct VectorsHeader {
    magic: [u8; 4], // b"WMV\0"
    version: u32,   // 1 (LE)
    dim: u32,       // 384
    count: u64,     // N
    model_name_len: u32,
    model_name: String,
}

const VECTORS_MAGIC: [u8; 4] = [b'W', b'M', b'V', 0];
const VECTORS_VERSION: u32 = 1;

struct VectorsBin;

impl VectorsBin {
    fn write(
        model_name: &str,
        entries: &HashMap<String, EmbedVector>,
        hashes: &HashMap<String, [u8; 32]>,
    ) -> Result<Vec<u8>, String> {
        let dim = entries.values().next().map(|v| v.dim()).unwrap_or(0) as u32;
        let count = entries.len() as u64;
        let model_name_bytes = model_name.as_bytes();
        let model_name_len = model_name_bytes.len() as u32;

        // Calculate size: header + padding + entries
        let header_size = 4 + 4 + 4 + 8 + 4; // 24 bytes
        let model_name_padded = (model_name_len as usize).div_ceil(32) * 32;
        let data_start = header_size + model_name_padded;

        let mut buf =
            Vec::with_capacity(data_start + count as usize * (4 + 256 + 32 + dim as usize * 4 + 8));

        // Header
        buf.extend_from_slice(&VECTORS_MAGIC);
        buf.extend_from_slice(&VECTORS_VERSION.to_le_bytes());
        buf.extend_from_slice(&dim.to_le_bytes());
        buf.extend_from_slice(&count.to_le_bytes());
        buf.extend_from_slice(&model_name_len.to_le_bytes());
        buf.extend_from_slice(model_name_bytes);
        // Pad to 32-byte boundary
        let pad_len = model_name_padded - model_name_len as usize;
        buf.extend(std::iter::repeat_n(0u8, pad_len));

        // Entries
        for (section_id, vec) in entries.iter() {
            let id_bytes = section_id.as_bytes();
            let id_len = id_bytes.len() as u32;
            let content_hash = hashes.get(section_id).copied().unwrap_or([0u8; 32]);

            buf.extend_from_slice(&id_len.to_le_bytes());
            buf.extend_from_slice(id_bytes);
            // Pad id to 256 bytes max
            let id_padded = (id_len as usize).div_ceil(8) * 8;
            let id_pad = id_padded.saturating_sub(id_bytes.len());
            buf.extend(std::iter::repeat_n(0u8, id_pad));

            buf.extend_from_slice(&content_hash);

            // Vector as f32 LE bytes
            for &val in &vec.0 {
                buf.extend_from_slice(&val.to_le_bytes());
            }
        }

        Ok(buf)
    }

    #[allow(clippy::type_complexity)]
    fn read(
        data: &[u8],
    ) -> Result<
        (
            VectorsHeader,
            HashMap<String, EmbedVector>,
            HashMap<String, [u8; 32]>,
        ),
        String,
    > {
        if data.len() < 24 {
            return Err("file too short".into());
        }

        let mut offset = 0;
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&data[offset..offset + 4]);
        offset += 4;
        if magic != VECTORS_MAGIC {
            return Err("invalid magic bytes".into());
        }

        let version = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        if version != VECTORS_VERSION {
            return Err(format!("unsupported version: {}", version));
        }

        let dim = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        let count = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;
        let model_name_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Validate model_name_len bounds
        let name_len = model_name_len as usize;
        if offset + name_len > data.len() {
            return Err("truncated file: model_name".into());
        }
        let model_name_bytes = &data[offset..offset + name_len];
        let model_name = String::from_utf8_lossy(model_name_bytes).to_string();

        // Pad to 32-byte boundary, cap at remaining data
        let model_name_padded = name_len.div_ceil(32) * 32;
        let next_offset = 24usize.checked_add(model_name_padded).unwrap_or(data.len());
        offset = next_offset.min(data.len()); // header size + padded name

        let dim_usize = dim as usize;
        let mut entries = HashMap::with_capacity(count as usize);
        let mut hashes = HashMap::with_capacity(count as usize);

        for _ in 0..count {
            if offset + 4 > data.len() {
                return Err("truncated file: id_len".into());
            }
            let id_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + id_len > data.len() {
                return Err("truncated file: id".into());
            }
            let id = String::from_utf8_lossy(&data[offset..offset + id_len]).to_string();
            offset += id_len.div_ceil(8) * 8; // pad to 8-byte alignment

            if offset + 32 > data.len() {
                return Err("truncated file: hash".into());
            }
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&data[offset..offset + 32]);
            offset += 32;

            if offset + dim_usize * 4 > data.len() {
                return Err("truncated file: vector".into());
            }
            let mut vec = Vec::with_capacity(dim_usize);
            for _ in 0..dim_usize {
                let val = f32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                vec.push(val);
                offset += 4;
            }

            entries.insert(id.clone(), EmbedVector(vec));
            hashes.insert(id, hash);
        }

        Ok((
            VectorsHeader {
                magic,
                version,
                dim,
                count,
                model_name_len,
                model_name,
            },
            entries,
            hashes,
        ))
    }
}

// ─── build_embeddings (hash-aware incremental) ───────────────

/// Build embedding vectors for all sections, skipping unchanged ones.
/// Returns (new_entries, new_hashes).
#[allow(clippy::type_complexity)]
pub fn rebuild_embeddings_skip_unchanged(
    embedder: &dyn Embedder,
    sections: &[crate::engine::SectionDoc],
    old_hashes: &HashMap<String, [u8; 32]>,
    old_entries_snap: Option<&HashMap<String, EmbedVector>>,
    batch_size: usize,
) -> Result<(HashMap<String, EmbedVector>, HashMap<String, [u8; 32]>), EmbedError> {
    let mut new_entries = HashMap::new();

    // Phase 1: Identify changed sections (parallel compute + hash, sequential merge)
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
    let mut to_embed: Vec<&crate::engine::SectionDoc> = Vec::new();
    for (i, (section_id, hash_bytes, changed)) in phase1.into_iter().enumerate() {
        new_hashes.insert(section_id, hash_bytes);
        if changed {
            to_embed.push(&sections[i]);
        }
    }

    // Phase 2: Embed changed sections in batches
    for chunk in to_embed.chunks(batch_size) {
        let texts: Vec<&str> = chunk.iter().map(|s| s.body.as_str()).collect();
        let vectors = embedder.embed_batch(&texts)?;
        for (sec, vec) in chunk.iter().zip(vectors) {
            new_entries.insert(sec.section_id.clone(), vec.normalized());
        }
    }

    // Phase 3: Carry forward unchanged vectors (parallel compute, sequential merge)
    if let Some(old) = old_entries_snap {
        let carry: Vec<(String, EmbedVector)> = sections
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

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_vectors_bin_roundtrip() {
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

        let data = VectorsBin::write("bge-small-en-v1.5", &entries, &hashes).unwrap();
        let (header, loaded_entries, loaded_hashes) = VectorsBin::read(&data).unwrap();

        assert_eq!(header.dim, 3);
        assert_eq!(header.count, 2);
        assert_eq!(header.model_name, "bge-small-en-v1.5");
        assert_eq!(entries.len(), loaded_entries.len());
        assert!(entries.contains_key("test:id:1"));
        assert!(entries.contains_key("test:id:2"));
        assert_eq!(loaded_hashes["test:id:1"], [1u8; 32]);
    }

    #[test]
    fn test_vector_store_swap_and_snapshot() {
        let store = VectorStore::new("test");
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
        let sections = vec![crate::engine::SectionDoc {
            section_id: "s1".into(),
            page_id: "p1".into(),
            header: "H1".into(),
            body: "hello world".into(),
        }];

        // First build
        let (entries, hashes) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &HashMap::new(), None, 32).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(hashes.contains_key("s1"));

        // Second build with same content → no new embeddings needed
        let (entries2, _) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &hashes, Some(&entries), 32).unwrap();
        assert_eq!(entries2.len(), 1);
        // Vector should be carried forward (same reference)
        assert_eq!(entries["s1"].0, entries2["s1"].0);
    }

    #[test]
    fn test_build_embeddings_detects_change() {
        let embedder = MockEmbedder::new(4);
        let mut sections = vec![crate::engine::SectionDoc {
            section_id: "s1".into(),
            page_id: "p1".into(),
            header: "H1".into(),
            body: "original content".into(),
        }];

        // First build
        let (old_entries, old_hashes) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &HashMap::new(), None, 32).unwrap();
        let old_vec = old_entries["s1"].0.clone();

        // Change content
        sections[0].body = "modified content".into();
        let (new_entries, _) =
            rebuild_embeddings_skip_unchanged(&embedder, &sections, &old_hashes, Some(&old_entries), 32).unwrap();

        // Vector should have changed
        assert_ne!(old_vec, new_entries["s1"].0);
    }
}
