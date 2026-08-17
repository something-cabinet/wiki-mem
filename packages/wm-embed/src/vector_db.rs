use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

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
#[derive(Debug)]
pub enum EmbedError {
    ModelNotLoaded(String),
    Inference(String),
    Tokenization(String),
    DimensionMismatch {
        expected: usize,
        actual: usize,
    },
    BatchTooLarge {
        size: usize,
        max: usize,
    },
    ModelNotFound(String),
    Download(String),
    /// No embedder is configured (e.g., NoopEmbedder). Semantic search is unavailable.
    SemanticUnavailable(String),
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
            EmbedError::SemanticUnavailable(msg) => {
                write!(f, "semantic search unavailable: {}", msg)
            }
        }
    }
}

impl std::error::Error for EmbedError {}

/// Trait implemented by embedder backends.
pub trait Embedder: Send + Sync {
    /// Embed a single text string into a vector.
    ///
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError>;
    /// Embed a batch of text strings into vectors.
    ///
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

pub use wm_engine::models::page::section_model::SectionDoc;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};

/// Database connection + dimension.
struct InnerDb {
    conn: turso::Connection,
    _dim: u32,
}

/// Vector database backed by turso (SQLite with vector extension).
///
/// All async database operations are wrapped in `tokio::task::block_in_place`
/// + `Handle::current().block_on()`, making this safe to call from inside
///   any existing tokio multi-thread runtime (e.g. `#[tokio::main]`).
pub struct VectorDb {
    db: Arc<Mutex<InnerDb>>,
}

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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS embed_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
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

/// Collect IDs from a table column that are not present in the given set.
///
/// Rows are collected into a Vec first to avoid cursor invalidation
/// when deleting rows from the same table being iterated.
async fn collect_orphan_ids(
    conn: &turso::Connection,
    query: &str,
    live_ids: &HashMap<String, impl Sized>,
) -> Result<Vec<String>, String> {
    let mut orphans: Vec<String> = Vec::new();
    let mut rows = conn.query(query, ()).await.map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let id: String = row.get(0).map_err(|e| e.to_string())?;
        if !live_ids.contains_key(&id) {
            orphans.push(id);
        }
    }
    Ok(orphans)
}

/// Delete chunks and content_hashes rows for a list of IDs.
async fn delete_ids_from_chunks_and_hashes(
    conn: &turso::Connection,
    ids: &[String],
) -> Result<(), String> {
    for id in ids {
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
    Ok(())
}

/// Upsert all entries/hashes, then delete any stored vector whose id is no
/// longer present (orphan reconciliation). Used by the production rebuild
/// path, where `entries`/`hashes` are the complete authoritative set.
async fn store_vectors_sync_impl(
    conn: &turso::Connection,
    entries: &HashMap<String, Vec<f32>>,
    hashes: &HashMap<String, String>,
) -> Result<(), String> {
    store_vectors_impl(conn, entries, hashes).await?;

    let orphan_chunk_ids =
        collect_orphan_ids(conn, "SELECT id FROM chunks", entries).await?;
    delete_ids_from_chunks_and_hashes(conn, &orphan_chunk_ids).await?;

    let orphan_hash_ids =
        collect_orphan_ids(conn, "SELECT source_id FROM content_hashes", hashes).await?;
    delete_ids_from_chunks_and_hashes(conn, &orphan_hash_ids).await?;

    Ok(())
}

async fn delete_vectors_with_prefix_impl(
    conn: &turso::Connection,
    prefix: &str,
) -> Result<(), String> {
    let pattern = format!("{}%", prefix);
    conn.execute("DELETE FROM chunks WHERE id LIKE ?1", [pattern.as_str()])
        .await
        .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM content_hashes WHERE source_id LIKE ?1",
        [pattern.as_str()],
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

async fn store_metadata_impl(conn: &turso::Connection, json: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO embed_meta (key, value) VALUES ('embedding_metadata', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        (json,),
    )
    .await
    .map_err(|e| e.to_string())
    .map(|_| ())
}

async fn load_metadata_impl(conn: &turso::Connection) -> Result<crate::EmbeddingMetadata, String> {
    let mut rows = conn
        .query(
            "SELECT value FROM embed_meta WHERE key = 'embedding_metadata'",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let json: String = row.get(0).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    } else {
        Ok(crate::EmbeddingMetadata::default())
    }
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

async fn load_hashes_impl(conn: &turso::Connection) -> Result<HashMap<String, String>, String> {
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
            (
                id.as_str(),
                body.as_str(),
                blob.as_slice(),
                body.len() as i64,
            ),
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

    let mut stale_ids: Vec<String> = Vec::new();
    {
        let mut rows = conn
            .query("SELECT source_id FROM content_hashes", ())
            .await
            .map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let id: String = row.get(0).map_err(|e| e.to_string())?;
            stale_ids.push(id);
        }
    }

    for id in &stale_ids {
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

/// Loaded vector data: (section_id → float vector, section_id → hex-encoded hash).
pub type LoadedVectors = (HashMap<String, Vec<f32>>, HashMap<String, String>);

impl VectorDb {
    /// Open or create the vector database at the given path.
    ///
    /// Must be called from within a tokio multi-thread runtime context
    /// (e.g. inside `#[tokio::main]` or a `#[tokio::test]`).
    ///
    pub fn open(path: PathBuf, dim: u32) -> Result<Self, String> {
        let path_str = path.to_str().ok_or("invalid path")?.to_string();
        let conn = run_async(open_db(&path_str))?;
        Ok(Self {
            db: Arc::new(Mutex::new(InnerDb { conn, _dim: dim })),
        })
    }

    /// Store vectors from an in-memory cache into the database.
    /// `entries`: section_id → raw float vector
    /// `hashes`: section_id → hex-encoded SHA-256 hash
    ///
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

    /// Upsert all vectors and delete any stored vector whose id is not in the
    /// provided set. `entries`/`hashes` must be the complete authoritative set
    /// (used by the production rebuild/save path so deleted pages leave no
    /// orphan vectors behind).
    ///
    pub fn store_vectors_sync(
        &self,
        entries: &HashMap<String, Vec<f32>>,
        hashes: &HashMap<String, String>,
    ) -> Result<(), String> {
        let entries = entries.clone();
        let hashes = hashes.clone();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(store_vectors_sync_impl(&db.conn, &entries, &hashes))
    }

    /// Delete every vector whose id starts with `prefix` (e.g. `wiki:p1#`).
    ///
    pub fn delete_vectors_with_prefix(&self, prefix: &str) -> Result<(), String> {
        let prefix = prefix.to_string();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(delete_vectors_with_prefix_impl(&db.conn, &prefix))
    }

    /// Persist embedding metadata (model mtime + chunking version) into the DB.
    ///
    pub fn store_metadata(&self, meta: &crate::EmbeddingMetadata) -> Result<(), String> {
        let json = serde_json::to_string(meta).map_err(|e| e.to_string())?;
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(store_metadata_impl(&db.conn, &json))
    }

    /// Load the persisted embedding metadata (empty/default if never stored).
    ///
    pub fn load_metadata(&self) -> Result<crate::EmbeddingMetadata, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(load_metadata_impl(&db.conn))
    }

    /// Load all vectors from the database into memory.
    /// Returns (section_id → float vector, section_id → hex-encoded hash).
    ///
    pub fn load_all_raw(&self) -> Result<LoadedVectors, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(load_all_impl(&db.conn))
    }

    /// Rebuild the index from sections. Only re-embeds changed sections.
    ///
    /// Embedding happens in the calling thread (synchronous); database writes
    /// use `block_in_place` to run async turso operations.
    ///
    pub fn rebuild(&self, sections: &[SectionDoc], embedder: &dyn Embedder) -> Result<(), String> {
        let sections = sections.to_vec();
        let current_ids: Vec<String> = sections.iter().map(|s| s.section_id.clone()).collect();

        let existing_hashes: HashMap<String, String> = {
            let db = self.db.lock().map_err(|e| e.to_string())?;
            run_async(load_hashes_impl(&db.conn))?
        };

        let mut upserts: Vec<(String, String, Vec<f32>, String)> = Vec::new();
        for sec in &sections {
            let hash = hex::encode(Sha256::digest(sec.body.as_bytes()));
            if existing_hashes.get(&sec.section_id) != Some(&hash) {
                let vec = embedder.embed(&sec.body).map_err(|e| e.to_string())?;
                upserts.push((sec.section_id.clone(), sec.body.clone(), vec.0, hash));
            }
        }

        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(rebuild_write_impl(&db.conn, &upserts, &current_ids))
    }

    /// Search for nearest vectors by cosine similarity using vector_distance_cos.
    /// Returns (section_id, score) pairs sorted by similarity (descending).
    ///
    pub fn search(&self, query_vec: &[f32], limit: usize) -> Result<Vec<(String, f32)>, String> {
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
                title: "Page 1".into(),
                tags: vec![],
            },
            SectionDoc {
                section_id: "s2".into(),
                page_id: "p1".into(),
                header: "Header 2".into(),
                body: "foo bar baz".into(),
                title: "Page 1".into(),
                tags: vec![],
            },
        ];

        let embedder = MockEmbedder::new(4);
        vdb.rebuild(&sections, &embedder).expect("rebuild");

        let query = embedder.embed("hello world").unwrap();
        let results = vdb.search(&query.0, 5).expect("search");

        assert!(!results.is_empty(), "should return at least one result");
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
            title: "Page 1".into(),
            tags: vec![],
        }];

        vdb.rebuild(&sections, &embedder).expect("initial rebuild");

        vdb.rebuild(&sections, &embedder)
            .expect("second rebuild no-op");

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

    fn section_doc(id: &str, page_id: &str, body: &str) -> SectionDoc {
        SectionDoc {
            section_id: id.into(),
            page_id: page_id.into(),
            header: "H".into(),
            body: body.into(),
            title: "Page".into(),
            tags: vec![],
        }
    }

    /// #14 — the production rebuild path (store_vectors_sync) removes orphan
    /// vectors: embed a page, delete one of its sections, sync again — the
    /// deleted section's vector is gone from the store and from search.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sync_removes_orphan_vectors_on_rebuild() {
        let vdb = VectorDb::open(":memory:".into(), 4).expect("open");
        let embedder = MockEmbedder::new(4);

        let s1 = section_doc("wiki:p1#alpha", "wiki:p1", "hello world");
        let s2 = section_doc("wiki:p1#beta", "wiki:p1", "foo bar baz");
        let s3 = section_doc("wiki:p1#gamma", "wiki:p1", "qux quux");

        let mut entries = HashMap::new();
        let mut hashes = HashMap::new();
        for s in [&s1, &s2, &s3] {
            let v = embedder.embed(&s.body).unwrap().0;
            entries.insert(s.section_id.clone(), v);
            hashes.insert(
                s.section_id.clone(),
                hex::encode(Sha256::digest(s.body.as_bytes())),
            );
        }
        vdb.store_vectors_sync(&entries, &hashes).expect("sync 1");

        let mut entries2 = HashMap::new();
        let mut hashes2 = HashMap::new();
        let s = &s1;
        let v = embedder.embed(&s.body).unwrap().0;
        entries2.insert(s.section_id.clone(), v);
        hashes2.insert(
            s.section_id.clone(),
            hex::encode(Sha256::digest(s.body.as_bytes())),
        );
        vdb.store_vectors_sync(&entries2, &hashes2).expect("sync 2");

        let query = embedder.embed("hello world").unwrap();
        let results = vdb.search(&query.0, 10).expect("search");
        let ids: Vec<String> = results.iter().map(|(id, _)| id.clone()).collect();
        assert!(
            ids.contains(&"wiki:p1#alpha".to_string()),
            "surviving section should still be searchable"
        );
        assert!(
            !ids.contains(&"wiki:p1#beta".to_string())
                && !ids.contains(&"wiki:p1#gamma".to_string()),
            "orphan vectors for deleted sections must be gone from search"
        );
        let (loaded_entries, loaded_hashes) = vdb.load_all_raw().expect("load all");
        assert_eq!(loaded_entries.len(), 1, "only one vector should remain");
        assert!(loaded_entries.contains_key("wiki:p1#alpha"));
        assert!(!loaded_entries.contains_key("wiki:p1#beta"));
        assert!(!loaded_entries.contains_key("wiki:p1#gamma"));
        assert_eq!(loaded_hashes.len(), 1, "only one hash should remain");
    }
}
