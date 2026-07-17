// ─── Shared types ───────────────────────────────────────────────────────────

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt;

/// A normalized embedding vector.
#[derive(Clone, Debug, PartialEq)]
pub struct EmbedVector(pub Vec<f32>);

impl EmbedVector {
    pub fn dim(&self) -> usize {
        self.0.len()
    }

    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.0.iter().map(|x| x * x).sum();
        if norm_sq > 1e-12 {
            let norm = norm_sq.sqrt();
            for x in &mut self.0 {
                *x /= norm;
            }
        }
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

/// Errors produced by embedder implementations.
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
                write!(f, "dimension mismatch: expected {}, got {}", expected, actual)
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

/// Trait implemented by embedder backends.
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbedVector>, EmbedError> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    fn is_loaded(&self) -> bool;
    fn model_name(&self) -> &str;
    fn output_dim(&self) -> usize;
}

/// A deterministic mock embedder for tests.
pub struct MockEmbedder {
    dim: usize,
}

impl MockEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn hash_vec(&self, text: &str) -> EmbedVector {
        let mut hasher = DefaultHasher::new();
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

/// A section of a wiki page.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SectionDoc {
    pub section_id: String,
    pub page_id: String,
    pub header: String,
    pub body: String,
}

// ─── Vector database ────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};

/// Database connection + dimension.
struct InnerDb {
    conn: turso::Connection,
    #[allow(dead_code)]
    dim: u32,
}

/// Vector database backed by turso (SQLite with vector extension).
///
/// All async database operations are wrapped in `tokio::task::block_in_place`
/// + `Handle::current().block_on()`, making this safe to call from inside
/// any existing tokio multi-thread runtime (e.g. `#[tokio::main]`).
pub struct VectorDb {
    db: Arc<Mutex<InnerDb>>,
}

// ─── Async implementations ────────────────────────────────────────────────────

async fn open_db(path: &str) -> Result<turso::Connection, String> {
    let db = turso::Builder::new_local(path)
        .build()
        .await
        .map_err(|e| e.to_string())?;
    let conn = db.connect().map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL DEFAULT '',
            embedding BLOB,
            token_count INTEGER DEFAULT 0
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS content_hashes (
            source_id TEXT PRIMARY KEY,
            hash TEXT NOT NULL
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(conn)
}

async fn store_vectors_impl(
    conn: &turso::Connection,
    entries: &HashMap<String, Vec<f32>>,
    hashes: &HashMap<String, String>,
) -> Result<(), String> {
    for (id, vec) in entries {
        let blob: Vec<u8> = vec.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT INTO chunks (id, content, embedding, token_count)
             VALUES (?1, '', ?2, 0)
             ON CONFLICT(id) DO UPDATE SET embedding = excluded.embedding",
            (id.as_str(), blob.as_slice()),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    for (source_id, hash) in hashes {
        conn.execute(
            "INSERT INTO content_hashes (source_id, hash)
             VALUES (?1, ?2)
             ON CONFLICT(source_id) DO UPDATE SET hash = excluded.hash",
            (source_id.as_str(), hash.as_str()),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn load_all_impl(
    conn: &turso::Connection,
) -> Result<(HashMap<String, Vec<f32>>, HashMap<String, String>), String> {
    let mut vectors: HashMap<String, Vec<f32>> = HashMap::new();
    let mut rows = conn
        .query(
            "SELECT id, embedding FROM chunks WHERE embedding IS NOT NULL",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let id: String = row.get(0).map_err(|e| e.to_string())?;
        let blob: Vec<u8> = row.get(1).map_err(|e| e.to_string())?;
        let vec: Vec<f32> = blob
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        vectors.insert(id, vec);
    }

    let mut hashes: HashMap<String, String> = HashMap::new();
    let mut hash_rows = conn
        .query("SELECT source_id, hash FROM content_hashes", ())
        .await
        .map_err(|e| e.to_string())?;
    while let Some(row) = hash_rows.next().await.map_err(|e| e.to_string())? {
        let source_id: String = row.get(0).map_err(|e| e.to_string())?;
        let hash: String = row.get(1).map_err(|e| e.to_string())?;
        hashes.insert(source_id, hash);
    }

    Ok((vectors, hashes))
}

async fn load_hashes_impl(
    conn: &turso::Connection,
) -> Result<HashMap<String, String>, String> {
    let mut hashes: HashMap<String, String> = HashMap::new();
    let mut rows = conn
        .query("SELECT source_id, hash FROM content_hashes", ())
        .await
        .map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let source_id: String = row.get(0).map_err(|e| e.to_string())?;
        let hash: String = row.get(1).map_err(|e| e.to_string())?;
        hashes.insert(source_id, hash);
    }
    Ok(hashes)
}

async fn rebuild_write_impl(
    conn: &turso::Connection,
    upserts: &[(String, String, Vec<f32>, String)],
    current_ids: &[String],
) -> Result<(), String> {
    for (id, body, vec, hash) in upserts {
        let blob: Vec<u8> = vec.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT INTO chunks (id, content, embedding, token_count)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 content = excluded.content,
                 embedding = excluded.embedding,
                 token_count = excluded.token_count",
            (id.as_str(), body.as_str(), blob.as_slice(), body.len() as i64),
        )
        .await
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO content_hashes (source_id, hash)
             VALUES (?1, ?2)
             ON CONFLICT(source_id) DO UPDATE SET hash = excluded.hash",
            (id.as_str(), hash.as_str()),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    // Remove stale entries
    let mut known: Vec<String> = Vec::new();
    {
        let mut rows = conn
            .query("SELECT source_id FROM content_hashes", ())
            .await
            .map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let id: String = row.get(0).map_err(|e| e.to_string())?;
            known.push(id);
        }
    }

    for id in &known {
        if !current_ids.contains(id) {
            conn.execute("DELETE FROM chunks WHERE id = ?1", [id.as_str()])
                .await
                .map_err(|e| e.to_string())?;
            conn.execute(
                "DELETE FROM content_hashes WHERE source_id = ?1",
                [id.as_str()],
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

async fn search_impl(
    conn: &turso::Connection,
    query: &[f32],
    limit: usize,
) -> Result<Vec<(String, f32)>, String> {
    let query_bytes: Vec<u8> = query.iter().flat_map(|f| f.to_le_bytes()).collect();

    let sql = "SELECT id, vector_distance_cos(embedding, vector32(?1)) AS score
               FROM chunks
               WHERE embedding IS NOT NULL
               ORDER BY score
               LIMIT ?2";

    let mut rows = conn
        .query(sql, (query_bytes.as_slice(), limit as i64))
        .await
        .map_err(|e| e.to_string())?;

    let mut results: Vec<(String, f32)> = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let id: String = row.get(0).map_err(|e| e.to_string())?;
        let score: f64 = row.get(1).map_err(|e| e.to_string())?;
        results.push((id, score as f32));
    }

    Ok(results)
}

// ─── Public API ────────────────────────────────────────────────────────────────

/// Run an async operation, bridging sync↔async.
/// Works both inside and outside a tokio runtime.
fn run_async<F, T>(f: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    match tokio::runtime::Handle::try_current() {
        Ok(_) => tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(f)),
        Err(_) => {
            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            rt.block_on(f)
        }
    }
}

impl VectorDb {
    /// Open or create the vector database at the given path.
    ///
    /// Must be called from within a tokio multi-thread runtime context
    /// (e.g. inside `#[tokio::main]` or a `#[tokio::test]`).
    pub fn open(path: PathBuf, dim: u32) -> Result<Self, String> {
        let path_str = path.to_str().ok_or("invalid path")?.to_string();
        let conn = run_async(open_db(&path_str))?;
        Ok(Self {
            db: Arc::new(Mutex::new(InnerDb { conn, dim })),
        })
    }

    /// Store vectors from an in-memory cache into the database.
    /// `entries`: section_id → raw float vector
    /// `hashes`: section_id → hex-encoded SHA-256 hash
    pub fn store_vectors_raw(
        &self,
        entries: &HashMap<String, Vec<f32>>,
        hashes: &HashMap<String, String>,
    ) -> Result<(), String> {
        let entries = entries.clone();
        let hashes = hashes.clone();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(store_vectors_impl(&db.conn, &entries, &hashes))
    }

    /// Load all vectors from the database into memory.
    /// Returns (section_id → float vector, section_id → hex-encoded hash).
    pub fn load_all_raw(
        &self,
    ) -> Result<(HashMap<String, Vec<f32>>, HashMap<String, String>), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(load_all_impl(&db.conn))
    }

    /// Rebuild the index from sections. Only re-embeds changed sections.
    ///
    /// Embedding happens in the calling thread (synchronous); database writes
    /// use `block_in_place` to run async turso operations.
    pub fn rebuild(
        &self,
        sections: &[SectionDoc],
        embedder: &dyn Embedder,
    ) -> Result<(), String> {
        let sections = sections.to_vec();
        let current_ids: Vec<String> = sections.iter().map(|s| s.section_id.clone()).collect();

        // Step 1: Load existing hashes from DB
        let existing_hashes: HashMap<String, String> = {
            let db = self.db.lock().map_err(|e| e.to_string())?;
            run_async(load_hashes_impl(&db.conn))?
        };

        // Step 2: Compute hashes, identify changed sections, embed locally (sync)
        let mut upserts: Vec<(String, String, Vec<f32>, String)> = Vec::new();
        for sec in &sections {
            let hash = hex::encode(Sha256::digest(sec.body.as_bytes()));
            if existing_hashes.get(&sec.section_id) != Some(&hash) {
                let vec = embedder.embed(&sec.body).map_err(|e| e.to_string())?;
                upserts.push((sec.section_id.clone(), sec.body.clone(), vec.0, hash));
            }
        }

        // Step 3: Write changes to DB
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(rebuild_write_impl(&db.conn, &upserts, &current_ids))
    }

    /// Search for nearest vectors by cosine similarity using vector_distance_cos.
    /// Returns (section_id, score) pairs sorted by similarity (descending).
    pub fn search(
        &self,
        query_vec: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>, String> {
        let query = query_vec.to_vec();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(search_impl(&db.conn, &query, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_open_in_memory() {
        let path = PathBuf::from(":memory:");
        let vdb = VectorDb::open(path, 4).expect("should open :memory: db");
        // Smoke test: the db is open and tables exist
        drop(vdb);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_and_search() {
        let path = PathBuf::from(":memory:");
        let vdb = VectorDb::open(path.clone(), 4).expect("open");

        let sections = vec![
            SectionDoc {
                section_id: "s1".into(),
                page_id: "p1".into(),
                header: "Header 1".into(),
                body: "hello world".into(),
            },
            SectionDoc {
                section_id: "s2".into(),
                page_id: "p1".into(),
                header: "Header 2".into(),
                body: "foo bar baz".into(),
            },
        ];

        let embedder = MockEmbedder::new(4);
        vdb.rebuild(&sections, &embedder).expect("rebuild");

        // Search with a query vector
        let query = embedder.embed("hello world").unwrap();
        let results = vdb.search(&query.0, 5).expect("search");

        assert!(!results.is_empty(), "should return at least one result");
        // s1 (identical content) should be ranked first
        assert_eq!(results[0].0, "s1", "most similar should be s1");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rebuild_incremental() {
        let path = PathBuf::from(":memory:");
        let vdb = VectorDb::open(path.clone(), 4).expect("open");
        let embedder = MockEmbedder::new(4);

        let mut sections = vec![SectionDoc {
            section_id: "s1".into(),
            page_id: "p1".into(),
            header: "H1".into(),
            body: "original".into(),
        }];

        vdb.rebuild(&sections, &embedder).expect("initial rebuild");

        // Same content → no change
        vdb.rebuild(&sections, &embedder).expect("second rebuild no-op");

        // Changed content
        sections[0].body = "modified".into();
        vdb.rebuild(&sections, &embedder).expect("rebuild changed");

        let query = embedder.embed("modified").unwrap();
        let results = vdb.search(&query.0, 5).expect("search");
        assert_eq!(results[0].0, "s1");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search_empty_db() {
        let path = PathBuf::from(":memory:");
        let vdb = VectorDb::open(path, 4).expect("open");
        let query = vec![0.1f32; 4];
        let results = vdb.search(&query, 5).expect("search empty");
        assert!(results.is_empty(), "empty db should return no results");
    }
}
