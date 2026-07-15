use rayon::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use sha2::{Digest, Sha256};

use crate::vector_db;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    Auto,
    Keyword,
    Semantic,
    Hybrid,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Hybrid
    }
}

impl SearchMode {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "auto" => SearchMode::Auto,
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
            SearchMode::Auto => write!(f, "auto"),
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
    /// Optional turso-backed durable store
    pub db: Option<Arc<vector_db::VectorDb>>,
}

impl VectorStore {
    /// Create a new empty VectorStore, optionally opening turso at `project_root/.wm/state/vectors.db`.
    pub fn new(model_name: &str, project_root: &Path) -> Self {
        let db_dir = project_root.join(".wm").join("state");
        let db_path = db_dir.join("vectors.db");
        let _ = std::fs::create_dir_all(&db_dir);
        let db = vector_db::VectorDb::open(db_path, 0)
            .ok()
            .map(Arc::new);
        Self {
            entries: ArcSwap::from_pointee(HashMap::new()),
            model_name: model_name.to_string(),
            hashes: ArcSwap::from_pointee(HashMap::new()),
            db,
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

    /// Load from turso database at `project_root/.wm/state/vectors.db`.
    pub fn load_from_disk(project_root: &Path) -> Result<Self, String> {
        let db_dir = project_root.join(".wm").join("state");
        let db_path = db_dir.join("vectors.db");
        let _ = std::fs::create_dir_all(&db_dir);
        let db = vector_db::VectorDb::open(db_path, 0).map_err(|e| format!("turso open error: {}", e))?;
        let db_arc = Arc::new(db);
        let (raw_entries, raw_hashes) = db_arc
            .load_all_raw()
            .map_err(|e| format!("turso load error: {}", e))?;
        let mut entries_map = HashMap::with_capacity(raw_entries.len());
        let mut hashes_map = HashMap::with_capacity(raw_entries.len());
        for (id, vec) in raw_entries {
            entries_map.insert(id.clone(), EmbedVector(vec));
            if let Some(hash_hex) = raw_hashes.get(&id) {
                let hash_bytes: [u8; 32] = hex::decode(hash_hex)
                    .ok()
                    .and_then(|v| v.try_into().ok())
                    .unwrap_or([0u8; 32]);
                hashes_map.insert(id, hash_bytes);
            }
        }
        let store = Self {
            entries: ArcSwap::from_pointee(entries_map),
            model_name: String::new(),
            hashes: ArcSwap::from_pointee(hashes_map),
            db: Some(db_arc),
        };
        Ok(store)
    }

    /// Write to turso database (no-op if no db configured).
    pub fn save_to_disk(&self) -> Result<(), String> {
        let db = self.db.as_ref().ok_or_else(|| "no turso database configured".to_string())?;
        let entries_arc = self.entries.load_full();
        let hashes_arc = self.hashes.load_full();
        let raw_entries: HashMap<String, Vec<f32>> = entries_arc
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();
        let raw_hashes: HashMap<String, String> = hashes_arc
            .iter()
            .map(|(k, v)| (k.clone(), hex::encode(v)))
            .collect();
        db.store_vectors_raw(&raw_entries, &raw_hashes)
            .map_err(|e| format!("turso write error: {}", e))?;
        Ok(())
    }

    /// Search turso for nearest vectors (fallback from in-memory search).
    pub fn search_turso(&self, query_vec: &[f32], limit: usize) -> Vec<(String, f32)> {
        match &self.db {
            Some(db) => db.search(query_vec, limit).unwrap_or_default(),
            None => vec![],
        }
    }
}

/// Read vectors.bin binary format without external dependencies.
///
/// Returns `(model_name, id→vector map, id→sha256_hash map)`.
/// Format defined in the old `wm-vectors-bin` crate (zero-dependency std-only parser):
///
/// Header (32-byte aligned): magic `b"WMV\0"` (4), version u32 (4), dim u32 (4),
/// count u64 (8), model_name_len u32 (4), model_name bytes, pad to 32 bytes.
///
/// Entries: id_len u32, id bytes, padded to 8 bytes, content_hash [u8; 32],
/// vector f32[dim].
fn read_vectors_bin(
    data: &[u8],
) -> Result<(String, HashMap<String, Vec<f32>>, HashMap<String, [u8; 32]>), String> {
    const MAGIC: [u8; 4] = [b'W', b'M', b'V', 0];
    const VERSION: u32 = 1;

    if data.len() < 24 {
        return Err("file too short".into());
    }

    let mut offset = 0usize;

    // Magic
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&data[offset..offset + 4]);
    offset += 4;
    if magic != MAGIC {
        return Err("invalid magic bytes".into());
    }

    // Version
    let version = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;
    if version != VERSION {
        return Err(format!("unsupported version: {}", version));
    }

    // Dim
    let dim = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;

    // Count
    let count = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    offset += 8;

    // Model name length
    let model_name_len =
        u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    // Read model name
    if offset + model_name_len > data.len() {
        return Err("truncated file: model_name".into());
    }
    let model_name =
        String::from_utf8_lossy(&data[offset..offset + model_name_len]).to_string();

    // Skip padded model_name to 32-byte boundary
    let model_name_padded = model_name_len.div_ceil(32) * 32;
    offset = 24usize.checked_add(model_name_padded).unwrap_or(data.len());

    let dim_usize = dim as usize;
    let count_usize = count as usize;
    let mut entries = HashMap::with_capacity(count_usize);
    let mut hashes = HashMap::with_capacity(count_usize);

    for _ in 0..count {
        // Id length
        if offset + 4 > data.len() {
            return Err("truncated file: id_len".into());
        }
        let id_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // Section id
        if offset + id_len > data.len() {
            return Err("truncated file: id".into());
        }
        let id = String::from_utf8_lossy(&data[offset..offset + id_len]).to_string();
        offset += id_len.div_ceil(8) * 8; // pad to 8-byte alignment

        // Content hash
        if offset + 32 > data.len() {
            return Err("truncated file: content_hash".into());
        }
        let mut content_hash = [0u8; 32];
        content_hash.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        // Vector data
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

/// Migrate vectors from the old `vectors.bin` file to turso.
/// Reads `vectors.bin` (if it exists), writes all vectors to turso, then deletes `vectors.bin`.
pub fn migrate_vectors_bin_to_turso(project_root: &Path) -> Result<usize, String> {
    let bin_path = project_root.join(".wm").join("state").join("vectors.bin");
    if !bin_path.exists() {
        return Ok(0);
    }

    // Read old binary format (inline parser, no wm-vectors-bin dependency)
    let data = std::fs::read(&bin_path).map_err(|e| format!("read vectors.bin error: {}", e))?;
    let (_model_name, raw_entries, raw_hash_map) =
        read_vectors_bin(&data).map_err(|e| format!("parse vectors.bin error: {}", e))?;

    // Determine dimension from first entry
    let dim = raw_entries
        .values()
        .next()
        .map(|v| v.len() as u32)
        .unwrap_or(0);

    // Open turso
    let db_path = project_root.join(".wm").join("state").join("vectors.db");
    let db = vector_db::VectorDb::open(db_path, dim)
        .map_err(|e| format!("turso open error: {}", e))?;
    let db_arc = Arc::new(db);

    // Convert hashes to hex
    let raw_hashes: HashMap<String, String> = raw_hash_map
        .iter()
        .map(|(k, v)| (k.clone(), hex::encode(v)))
        .collect();

    db_arc
        .store_vectors_raw(&raw_entries, &raw_hashes)
        .map_err(|e| format!("turso write error: {}", e))?;

    // Remove old file
    std::fs::remove_file(&bin_path).map_err(|e| format!("delete vectors.bin error: {}", e))?;

    Ok(raw_entries.len())
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

        // Reload
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
