---
title: ONNX Embedding Integration (v2.0)
type: spec
tags:
  - spec
  - rust
  - onnx
  - embeddings
  - semantic-search
  - wiki-mem
---
id: wiki:specs:onnx-embedding-integration

## Overview

Adds ONNX-powered semantic embeddings to the wiki-mem engine (CLI: `wm`, config dir: `.wm/`). Reuses the existing `ArcSwap` lock-free read pattern, incremental hash tracking (SHA-256 per section), and BM25 infrastructure. Introduces an `Embedder` trait, `SearchMode` enum with RRF hybrid fusion (k=60), flat-binary vector storage, and a model lifecycle subcommand tree.

This is a non-breaking, additive spec. When the model is absent, the engine falls back to BM25-only with zero startup delay.


---
id: wiki:specs:onnx-embedding-integration

## 1. Crate Additions

### 1.1 New Dependencies

Add to Cargo.toml:

```toml
[dependencies]
ort = { version = "2", features = ["download-binaries"] }
tokenizers = { version = "0.21", default-features = false, features = ["http"] }
memmap2 = "0.9"
ndarray = "0.16"
oorandom = "11"
reqwest = { version = "0.12", features = ["rustls-tls", "stream"], default-features = false }
indicatif = "0.17"
```

### 1.2 Crate Rationale

| Crate | Why | Size |
|-------|-----|------|
| `ort` | ONNX Runtime safe Rust bindings. `download-binaries` links native lib at build time. | ~25-28 MB |
| `tokenizers` | HuggingFace tokenizer for BGE. Reads `tokenizer.json` alongside `model.onnx`. | ~2 MB |
| `memmap2` | Zero-copy mmap for `vectors.bin`. 10k vectors = ~15 MB, OS pages in on demand. | Negligible |
| `ndarray` | Tensor representation. `ort` returns `ndarray::ArrayView`. | Transitively pulled |
| `oorandom` | Fast deterministic PRNG for MockEmbedder. No `getrandom`/`rand` needed. | ~50 KB |
| `reqwest` | HTTPS download of model files from HuggingFace Hub. | ~3 MB |
| `indicatif` | Progress bars during `wm model download`. | ~200 KB |

### 1.3 Binary Size Budget

- Release binary target: **<40 MB** (revised from <30 MB to accommodate ONNX Runtime).
- `ort` + linked ONNX Runtime: ~25-28 MB
- `tokenizers`: ~2 MB | `reqwest`: ~3 MB | Existing code: ~5 MB
- **Total:** ~35-38 MB typical.

### 1.4 Model File Size

- `bge-small-en-v1.5` ONNX model: ~134 MB
- `tokenizer.json`: ~2 MB
- **Total model cache:** ~136 MB at `~/.wm/models/bge-small-en-v1.5/`
- Model is explicit download only - never bundled, never auto-fetched.

### 1.5 Feature Flag

All embedding code lives behind a Cargo feature flag:

```toml
[features]
default = []
embed = ["ort", "tokenizers", "memmap2", "ndarray", "reqwest", "indicatif"]
```

The `embed` feature is **off by default**. When disabled, the `Embedder` type compiles as `NoopEmbedder` that always reports `is_loaded() == false`. This keeps the non-embedding binary small.


---
id: wiki:specs:onnx-embedding-integration

## 2. Embedder Trait + Implementation

### 2.1 Module Layout

```
src/embed/
  mod.rs        - Embedder trait, EmbedVector, EmbedError, cosine_similarity
  onnx.rs       - OnnxEmbedder (feature-gated behind `embed`)
  mock.rs       - MockEmbedder for tests (deterministic hash-based)
  noop.rs       - NoopEmbedder (compile-time fallback when feature disabled)
```

### 2.2 Core Types

```rust
// file: src/embed/mod.rs

#[derive(Error, Debug)]
pub enum EmbedError {
    #[error("model not loaded: {0}")]
    ModelNotLoaded(String),
    #[error("inference error: {0}")]
    Inference(String),
    #[error("tokenization error: {0}")]
    Tokenization(String),
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("batch size {size} exceeds limit {max}")]
    BatchTooLarge { size: usize, max: usize },
    #[error("model file not found: {0}")]
    ModelNotFound(String),
    #[error("download error: {0}")]
    Download(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmbedVector(pub Vec<f32>);

impl EmbedVector {
    pub fn dim(&self) -> usize { self.0.len() }

    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.0.iter().map(|x| x * x).sum();
        if norm_sq > 1e-12 {
            let norm = norm_sq.sqrt();
            for x in &mut self.0 { *x /= norm; }
        }
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

/// Cosine similarity between two L2-normalized vectors.
/// PRECONDITION: both MUST already be normalized.
/// cos(a,b) = dot(a,b), clamped to [0.0, 1.0].
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    dot.clamp(0.0, 1.0)
}

pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbedVector>, EmbedError> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    fn is_loaded(&self) -> bool;
    fn model_name(&self) -> &str;
    fn output_dim(&self) -> usize;
}
```

### 2.3 OnnxEmbedder - Design (not full implementation)

```rust
// file: src/embed/onnx.rs  (feature-gated behind `embed`)

pub struct OnnxEmbedder {
    session: Arc<ort::Session>,
    tokenizer: tokenizers::Tokenizer,
    model_name: String,
    dim: usize,               // 384
    max_batch_size: usize,    // 64
    loaded: bool,
}

impl OnnxEmbedder {
    /// Returns Ok(None) if model directory is missing - caller falls back to BM25-only.
    pub fn load(model_dir: &Path, model_name: &str) -> Result<Option<Self>, EmbedError> {
        let model_path = model_dir.join(model_name).join("model.onnx");
        let tok_path   = model_dir.join(model_name).join("tokenizer.json");

        if !model_path.exists() { return Ok(None); }
        if !tok_path.exists()   { return Ok(None); }

        // ONNX environment is a global singleton (OnceLock<Environment>)
        // Session: GraphOptimizationLevel::Level3, intra_threads=4
        // Tokenizer loaded from file
        // Returns Ok(Some(Self { ... }))
        todo!("ONNX init - ~30 lines")
    }
}

impl Embedder for OnnxEmbedder {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError> {
        // delegates to embed_batch(&[text])
        todo!()
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbedVector>, EmbedError> {
        // 1. Tokenize batch → input_ids + attention_mask tensors
        // 2. session.run([input_ids, attention_mask])
        // 3. Extract hidden state output → [batch, seq_len, 384]
        // 4. CLS-pool: take position 0 for each batch item
        // 5. L2-normalize each 384-dim vector
        // 6. Return Vec<EmbedVector>
        todo!("ONNX inference - ~25 lines")
    }

    fn is_loaded(&self) -> bool { self.loaded }
    fn model_name(&self) -> &str { &self.model_name }
    fn output_dim(&self) -> usize { self.dim }
}
```

### 2.4 NoopEmbedder

```rust
// Always reports "not loaded". Used when embed feature is off or no model downloaded.
pub struct NoopEmbedder { dim: usize }

impl Embedder for NoopEmbedder {
    fn embed(&self, _text: &str) -> Result<EmbedVector, EmbedError> {
        Err(EmbedError::ModelNotLoaded("no embedder configured".into()))
    }
    fn is_loaded(&self) -> bool { false }
    fn model_name(&self) -> &str { "none" }
    fn output_dim(&self) -> usize { self.dim }
}
```

### 2.5 MockEmbedder

```rust
// Deterministic mock using DefaultHasher seeded by text.
// Same text always produces same vector. Used in tests.
pub struct MockEmbedder { dim: usize }

impl Embedder for MockEmbedder {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError> {
        // hash(text) -> seed -> oorandom RNG -> dim random floats -> normalize
        Ok(self.hash_vec(text))
    }
    fn is_loaded(&self) -> bool { true }
    fn model_name(&self) -> &str { "mock" }
    fn output_dim(&self) -> usize { self.dim }
}
```


---
id: wiki:specs:onnx-embedding-integration

## 3. Search Mode Enum + RRF

### 3.1 SearchMode Enum

```rust
// file: src/search/mode.rs

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SearchMode {
    /// BM25 keyword search only (default, always available).
    Keyword,
    /// Cosine similarity over embedding vectors.
    /// Fails gracefully if embedder not loaded.
    Semantic,
    /// Reciprocal Rank Fusion (k=60) merging BM25 and semantic rankings.
    /// Falls back to BM25-only if embedder not loaded.
    Hybrid,
}
```

### 3.2 RRF Algorithm

RRF score for document d:
  score(d) = 1/(k + rank_bm25(d)) + 1/(k + rank_semantic(d))
  where k = 60 (Cormack et al. 2009)

Why k=60: Higher k dampens contribution of top-1 results, preventing a single #1 rank from dominating fusion. Values <20 make fusion too sensitive to ranking position.

### 3.3 RRF Implementation

```rust
// file: src/search/rrf.rs

const RRF_K: f32 = 60.0;

pub fn rrf_fuse<T: Clone + Eq + Hash>(
    bm25_ranked: &[(T, f32)],      // (doc_id, bm25_score), sorted desc
    semantic_ranked: &[(T, f32)],  // (doc_id, cosine_sim), sorted desc
    limit: usize,
) -> Vec<(T, f32)> {
    let mut scores: HashMap<&T, f32> = HashMap::new();

    // BM25 contribution: 1/(k + rank)
    for (rank, (doc_id, _)) in bm25_ranked.iter().enumerate() {
        *scores.entry(doc_id).or_insert(0.0) += 1.0 / (RRF_K + rank as f32);
    }

    // Semantic contribution: 1/(k + rank)
    for (rank, (doc_id, _)) in semantic_ranked.iter().enumerate() {
        *scores.entry(doc_id).or_insert(0.0) += 1.0 / (RRF_K + rank as f32);
    }

    // Sort descending by RRF score
    // Tie-breaker: doc appearing in BOTH lists ranks higher
    let mut fused: Vec<_> = scores.into_iter().collect();
    fused.sort_by(|(a_id, a_score), (b_id, b_score)| {
        b_score.partial_cmp(a_score).unwrap_or(Equal)
            .then_with(|| {
                let a_both = semantic_ranked.iter().any(|(id,_)| id == *a_id);
                let b_both = semantic_ranked.iter().any(|(id,_)| id == *b_id);
                b_both.cmp(&a_both)
            })
    });

    fused.truncate(limit);
    fused.into_iter().map(|(id, s)| (id.clone(), s)).collect()
}
```

### 3.4 Query Flow

```
search.query(q, mode, limit=10)
  |
  +-- mode == Keyword ----------------------------------------
  |   bm25.search(q, limit) -> results
  |
  +-- mode == Semantic ---------------------------------------
  |   if embedder.is_loaded():
  |     query_vec = embedder.embed(q)      (~10ms ONNX)
  |     results = top_k_cosine(query_vec, vectors, limit)  (~5ms scan)
  |   else:
  |     return Err("semantic search unavailable: no model loaded")
  |
  +-- mode == Hybrid -----------------------------------------
      if embedder.is_loaded():
        bm25_results = bm25.search(q, limit * 2)   // wider recall
        query_vec = embedder.embed(q)
        semantic_results = top_k_cosine(query_vec, vectors, limit * 2)
        results = rrf_fuse(bm25_results, semantic_results, limit)  (<1ms)
      else:
        warn!("hybrid falling back to BM25-only (no embedder)")
        results = bm25.search(q, limit)   // graceful fallback
```

### 3.5 Cosine Top-K

```rust
/// Linear scan over all stored vectors. For <100k entries at 384 dims,
/// this is faster than approximate nearest-neighbor (no HNSW overhead).
/// Benchmarked: 50k vectors x 384 dims = ~8ms on modern x86 CPU.
pub fn top_k_cosine(
    query: &[f32],
    vectors: &HashMap<String, EmbedVector>,
    k: usize,
) -> Vec<(String, f32)> {
    use std::collections::BinaryHeap;
    use std::cmp::Reverse;

    let mut heap = BinaryHeap::with_capacity(k + 1);
    for (doc_id, ev) in vectors {
        let sim = cosine_similarity(query, &ev.0);
        heap.push(Reverse((OrderedFloat(sim), doc_id.clone())));
        if heap.len() > k { heap.pop(); }
    }
    let mut results: Vec<_> = heap.into_iter()
        .map(|r| (r.0.1, r.0.0.0))
        .collect();
    results.reverse(); // highest first
    results
}
```

**Why linear scan over ANN (HNSW):** For 10k-50k sections, linear scan runs in 3-10ms. HNSW would save ~5ms but adds 20+ MB memory and index-build complexity. Linear wins on YAGNI until corpus exceeds 200k sections.


---
id: wiki:specs:onnx-embedding-integration

## 4. Vector Storage

### 4.1 On-Disk Format (vectors.bin)

Flat binary at `.wm/state/vectors.bin`:

```
+------------------------------------------------------+
| Header (32-byte aligned)                              |
|  magic:        [u8; 4]    = b"WMV\0"                 |
|  version:      u32        = 1 (LE)                   |
|  dim:          u32        = 384                      |
|  count:        u64        = N                        |
|  model_name_len: u32                                 |
|  model_name:   [u8; model_name_len] (UTF-8)          |
|  padding to 32-byte boundary                         |
+------------------------------------------------------+
| Entry array (N entries)                               |
|  For each entry:                                     |
|    section_id_len: u32 (LE)                          |
|    section_id:     [u8; section_id_len] (UTF-8)      |
|    content_hash:   [u8; 32]  (SHA-256 raw bytes)     |
|    vector:         [f32; 384] (LE, 1536 bytes)       |
|    padding to 8-byte alignment                       |
+------------------------------------------------------+
```

No serde/bincode - raw bytes are faster and produce smallest files.

### 4.2 In-Memory Structure

```rust
// file: src/state.rs - addition to EngineState

pub struct VectorStore {
    /// section_id -> L2-normalized EmbedVector
    pub entries: ArcSwap<HashMap<String, EmbedVector>>,
    /// Which model produced these vectors (validated on load)
    pub model_name: String,
    /// Section content hash -> skip re-embedding if unchanged
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
    pub fn swap(&self, new_entries: HashMap<String, EmbedVector>,
                new_hashes: HashMap<String, [u8; 32]>) {
        self.entries.store(Arc::new(new_entries));
        self.hashes.store(Arc::new(new_hashes));
    }

    /// Get a consistent snapshot for queries (Arc clone, no lock).
    pub fn snapshot(&self) -> Arc<HashMap<String, EmbedVector>> {
        self.entries.load_full()
    }
}
```

### 4.3 Why ArcSwap over DashMap

- `ArcSwap<HashMap<>>`: lock-free reads via `load_full()`. Queries see a consistent snapshot. Write frequency is extremely low (rebuild only).
- `DashMap`: provides per-shard locking for writes but does NOT provide consistent snapshots - a query iterating entries could see partial updates.
- ArcSwap's atomic pointer swap is cheaper than DashMap's shard-level locks for this read-heavy, write-rare pattern.

### 4.4 Snapshot Safety

```rust
// CORRECT: snapshot() returns Arc, keeping HashMap alive during iteration
let snap = vector_store.snapshot();  // Arc<HashMap<...>>
for (id, vec) in snap.iter() { ... }  // Safe

// INCORRECT: load() returns a raw reference that may dangle after swap
let guard = vector_store.entries.load();  // Guard<Arc<...>>
// another thread calls swap() -> guard may dangle
for (id, vec) in guard.iter() { ... }  // Potentially unsafe!
```

### 4.5 Embedding Rebuild (hash-aware)

```rust
pub async fn build_embeddings(
    embedder: &dyn Embedder,
    sections: &[SectionDoc],
    old_hashes: &HashMap<String, [u8; 32]>,
    old_entries_snap: Option<&HashMap<String, EmbedVector>>,
    batch_size: usize,
) -> Result<(HashMap<String, EmbedVector>, HashMap<String, [u8; 32]>), EmbedError> {
    let mut new_entries = HashMap::new();
    let mut new_hashes   = HashMap::new();

    // Phase 1: Identify changed sections
    let mut to_embed: Vec<&SectionDoc> = Vec::new();
    for sec in sections {
        let h = sha256(sec.body.as_bytes());
        new_hashes.insert(sec.section_id.clone(), h);
        if old_hashes.get(&sec.section_id) != Some(&h) {
            to_embed.push(sec);
        }
    }

    // Phase 2: Embed changed sections in batches
    for chunk in to_embed.chunks(batch_size) {
        let texts: Vec<&str> = chunk.iter().map(|s| s.body.as_str()).collect();
        let vectors = embedder.embed_batch(&texts)?;
        for (sec, vec) in chunk.iter().zip(vectors) {
            new_entries.insert(sec.section_id.clone(), vec);
        }
    }

    // Phase 3: Carry forward unchanged vectors
    if let Some(old) = old_entries_snap {
        for sec in sections {
            if !new_entries.contains_key(&sec.section_id) {
                if let Some(vec) = old.get(&sec.section_id) {
                    new_entries.insert(sec.section_id.clone(), vec.clone());
                }
            }
        }
    }

    Ok((new_entries, new_hashes))
}
```

### 4.6 File System Update

```
.wm/
  state/
    hashes.json         # Content hashes for pages + sections (existing)
    vector_hashes.json  # Section -> SHA256 for embedding skip (NEW)
    bm25.idx            # Serialized BM25 index (existing)
    vectors.bin         # Flat binary embedding store (NEW)
```


---
id: wiki:specs:onnx-embedding-integration

## 5. Model Lifecycle

### 5.1 Cache Directory

```
~/.wm/
  models/
    bge-small-en-v1.5/
      model.onnx           # ONNX model file (~134 MB)
      tokenizer.json        # HuggingFace tokenizer (~2 MB)
      manifest.json         # Download metadata
    <other-model>/
  logs/
    wm.log
```

### 5.2 CLI Commands

```bash
# List available models (cached + remote catalog)
wm model list
#   Model                  Status   Size      Dim
#   bge-small-en-v1.5      cached   134 MB    384
#   bge-base-en-v1.5       remote   438 MB    768
#   all-MiniLM-L6-v2       remote   90 MB     384

# Download a model
wm model download bge-small-en-v1.5
#   Downloading bge-small-en-v1.5...
#   [==========] 134 MB / 134 MB (12.3 MB/s)
#   Verifying SHA256... OK
#   Cached at ~/.wm/models/bge-small-en-v1.5/

# Show model status
wm model status
#   Model: bge-small-en-v1.5
#   Status: loaded
#   Dimension: 384
#   Sections indexed: 1,247
#   Cache: ~/.wm/models/bge-small-en-v1.5/

# Remove a model
wm model remove bge-small-en-v1.5
```

### 5.3 Download Flow

```
wm model download <name>
  |
  +-- Check if already cached (manifest.json exists + SHA matches) -> skip
  |
  +-- Fetch model registry (embedded in binary or remote)
  |   {
  |     "bge-small-en-v1.5": {
  |       "model_url": "https://huggingface.co/BAAI/bge-small-en-v1.5/...",
  |       "tokenizer_url": "https://huggingface.co/.../tokenizer.json",
  |       "sha256_model": "abc123...",
  |       "sha256_tokenizer": "def456...",
  |       "dim": 384,
  |       "size_bytes": 140640256
  |     }
  |   }
  |
  +-- Create ~/.wm/models/<name>/ directory
  +-- Download model.onnx (stream + progress bar + SHA256 verify)
  +-- Download tokenizer.json (stream + SHA256 verify)
  +-- Write manifest.json
  +-- Done. Embedder will load on next engine init.
```

### 5.4 Startup Behavior

```rust
// file: src/engine.rs - init sequence

pub async fn init_engine(config: &ProjectConfig) -> EngineState {
    // ... existing graph + BM25 init (unchanged) ...

    // Model loading - NON-BLOCKING
    let model_dir = get_model_cache_dir();  // ~/.wm/models/
    let model_name = config.model.as_deref().unwrap_or("bge-small-en-v1.5");

    let embedder: Box<dyn Embedder> = match OnnxEmbedder::load(&model_dir, model_name) {
        Ok(Some(e)) => {
            info!("ONNX embedder loaded: {} ({} dims)", e.model_name(), e.output_dim());
            Box::new(e)
        }
        Ok(None) => {
            info!("No model found. Run `wm model download {}` for semantic search.", model_name);
            Box::new(NoopEmbedder::new())
        }
        Err(e) => {
            warn!("ONNX load failed: {} - falling back to BM25-only", e);
            Box::new(NoopEmbedder::new())
        }
    };

    // Load existing vectors.bin (also non-blocking)
    let vector_store = VectorStore::load_from_disk(project_root)
        .unwrap_or_else(|e| {
            warn!("vectors.bin load failed: {} - will rebuild on demand", e);
            VectorStore::new(model_name)
        });

    EngineState {
        // ... existing fields ...
        embedder,      // NEW
        vector_store,  // NEW
    }
}
```

**Key guarantee:** If no model cached, `OnnxEmbedder::load()` returns `Ok(None)`. Engine proceeds with `NoopEmbedder`. Zero startup delay. No network calls.

### 5.5 Manifest Format

```json
{
    "model": "bge-small-en-v1.5",
    "dim": 384,
    "downloaded_at": "2026-06-15T14:32:11Z",
    "version": "1.0",
    "model_sha256": "abc123def456...",
    "tokenizer_sha256": "789ghi012jkl..."
}
```


---
id: wiki:specs:onnx-embedding-integration

## 6. Tool Changes

### 6.1 EngineState Additions

```rust
pub struct EngineState {
    // ... existing fields (unchanged) ...
    pub embedder:      Box<dyn Embedder>,    // NEW - NoopEmbedder if no model
    pub vector_store:  VectorStore,          // NEW - empty if no vectors.bin
}
```

### 6.2 search.query - New "mode" Parameter

```json
{
    "name": "search.query",
    "inputSchema": {
        "type": "object",
        "properties": {
            "q": { "type": "string" },
            "mode": {
                "type": "string",
                "enum": ["keyword", "semantic", "hybrid"],
                "default": "keyword"
            },
            "type": { "type": "string" },
            "status": { "type": "string" },
            "tag": { "type": "array", "items": {"type": "string"} },
            "limit": { "type": "integer", "default": 10, "minimum": 1, "maximum": 100 }
        },
        "required": ["q"]
    }
}
```

Response additions:
```json
{
    "results": [...],
    "mode_used": "hybrid",
    "embedder_loaded": true,
    "search_time_ms": 12
}
```

`mode_used` reflects what actually ran. If `mode=hybrid` but no embedder, `mode_used="keyword"` and a warning is logged.

### 6.3 index.rebuild - New Parameters

```json
{
    "name": "index.rebuild",
    "inputSchema": {
        "type": "object",
        "properties": {
            "skip_embed": {
                "type": "boolean",
                "default": false,
                "description": "Rebuild BM25 index only; skip embedding phase."
            },
            "embed_batch_size": {
                "type": "integer",
                "default": 32,
                "description": "Batch size for ONNX inference."
            }
        }
    }
}
```

Rebuild flow:
```
index.rebuild (skip_embed=false)
  |
  +-- Phase 1-3: Scan wiki, build graph, build BM25 (unchanged)
  +-- Phase 4: Build embeddings (NEW)
  |   if embedder.is_loaded() && !skip_embed:
  |     build_embeddings(embedder, sections, old_hashes, batch_size)
  |     vector_store.swap(new_entries, new_hashes)
  |     persist vectors.bin (async background write)
  |   if !embedder.is_loaded():
  |     log: "Skipping embeddings - no model loaded"
  +-- Phase 5: ArcSwap atomic swap of BM25 + corpus + vectors
```

### 6.4 New Tool: index.embed

```json
{
    "name": "index.embed",
    "description": "Build embedding vectors for all wiki sections. Requires a downloaded model.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "batch_size": {
                "type": "integer",
                "default": 32,
                "minimum": 1,
                "maximum": 64
            },
            "force": {
                "type": "boolean",
                "default": false,
                "description": "Re-embed all sections, ignoring content hashes."
            }
        }
    }
}
```

### 6.5 New MCP Tool Group: model

| Tool | Action | Description |
|------|--------|-------------|
| `model.list` | `list` | List cached + available remote models |
| `model.download` | `download` | Download model by name (long-running, streaming) |
| `model.status` | `status` | Current model state, cache size, index count |
| `model.remove` | `remove` | Delete cached model |

### 6.6 CLI Command Additions

```bash
# Model management (new)
wm model list                                    # List available models
wm model download <name>                         # Download model
wm model status                                  # Show current model status
wm model remove <name>                           # Remove cached model

# Index changes
wm index rebuild [--skip-embed]                  # Rebuild BM25 + embeddings
wm index embed [--batch-size N] [--force]        # Build embeddings only

# Search changes
wm search <query> [--mode keyword|semantic|hybrid] # Search with mode
```

### 6.7 initial Tool - Updated Response

When model loaded:
```json
{
    "model": {
        "loaded": true,
        "name": "bge-small-en-v1.5",
        "dim": 384,
        "sections_indexed": 1247
    },
    "search_modes_available": ["keyword", "semantic", "hybrid"]
}
```

When model not loaded:
```json
{
    "model": {
        "loaded": false,
        "hint": "Run `wm model download bge-small-en-v1.5` to enable semantic search."
    },
    "search_modes_available": ["keyword"]
}
```


---
id: wiki:specs:onnx-embedding-integration

## 7. Constraints & Edge Cases

### 7.1 Explicit Download Only (Opt-In)

- `wm model download <name>` is the ONLY path to obtain a model.
- Engine NEVER auto-downloads at startup or query time.
- `ort` `download-binaries` fetches ONNX Runtime native lib at **build time** (not runtime) - this is a build dependency, acceptable.
- `tokenizers` `http` feature used only during `wm model download`. No network access otherwise.

### 7.2 Offline After Download

- Once `model.onnx` + `tokenizer.json` exist in `~/.wm/models/<name>/`, zero network access required.
- Model registry (URLs, hashes) embedded in binary via `include_str!("models_registry.json")`.
- `wm model list` works offline (shows cached models). `wm model download` requires internet.
- SHA-256 verification post-download against embedded hash prevents corrupted models from loading.

### 7.3 Latency Budget (<50ms per query)

| Component | Time | Notes |
|-----------|------|-------|
| Query tokenization | <1ms | BGE tokenizer, <512 tokens |
| ONNX inference (single) | <10ms | 384-dim, CPU, 4 threads |
| Cosine linear scan (10k) | <5ms | 384 x 10k f32 ops, SIMD |
| BM25 search (existing) | <5ms | Already benchmarked |
| RRF fusion (top 20 x2) | <1ms | Two sorted lists merge |
| JSON formatting | <2ms | Result serialization |
| **Total hybrid query** | **<25ms** | Well within 50ms |
| Keyword-only query | <10ms | Unchanged |

### 7.4 Memory Budget

| Component | Memory |
|-----------|--------|
| ONNX Runtime resident | ~30 MB |
| vectors.bin mmap | ~0 MB (virtual, paged on demand) |
| In-memory HashMap (10k) | ~15.4 MB (10000 x (1536 + String key)) |
| Tokenizer | ~5 MB |
| **Total overhead** | **~50 MB** |

### 7.5 Graceful Degradation Matrix

| State | semantic mode | hybrid mode | index.rebuild | index.embed |
|-------|--------------|-------------|---------------|-------------|
| No model downloaded | Error | Falls back BM25 + warn | Skips embed + hint | Error |
| Model present, no vectors | Works (on-the-fly) | Works (on-the-fly) | Builds vectors | Builds vectors |
| Model removed, vectors exist | Works (pre-computed) | Works (pre-computed) | Error: incompatible | Error: no model |
| ONNX init fails | Falls back BM25 | Falls back BM25 | Skips embed | Error |
| vectors.bin corrupted | Rebuilds next query | Rebuilds next query | Overwrites | Overwrites |

### 7.6 Model Validation on Vector Load

When loading `vectors.bin`, header's `model_name` is compared against configured model. Mismatch (e.g., switching models) invalidates stored vectors because dimensions differ:

```rust
fn validate_vectors_bin(header: &Header, current_model: &str) -> Result<(), String> {
    if header.model_name != current_model {
        return Err(format!(
            "vectors.bin built with '{}' but current model is '{}'. Run `wm index embed --force`.",
            header.model_name, current_model
        ));
    }
    if header.dim != 384 {
        return Err(format!("expected dim 384, got {}", header.dim));
    }
    Ok(())
}
```

### 7.7 Concurrency

Follows existing ArcSwap pattern:
- **Reads:** `vector_store.snapshot()` -> `Arc<HashMap<>>` - lock-free, consistent.
- **Writes:** Only during rebuild. Build new HashMap in isolation, atomically swap.
- **File writes:** `vectors.bin` written asynchronously (spawn_blocking) after in-memory swap. Never blocks queries.

### 7.8 Cross-Platform

Same `model.onnx` works on Linux, macOS (Intel + Apple Silicon), Windows. Only platform-specific component is ONNX Runtime native lib, handled by `ort` `download-binaries` at build time. For Apple Silicon, `ort` should auto-select ARM64 native library.

---
id: wiki:specs:onnx-embedding-integration

## Acceptance Criteria (Embedding-Specific)

- [ ] **AC-E1:** `wm model download bge-small-en-v1.5` downloads model + tokenizer + verifies SHA-256
- [ ] **AC-E2:** Engine starts with no model present - no network requests, no errors, no delay
- [ ] **AC-E3:** `search.query(mode="hybrid")` fuses BM25 + cosine rankings with k=60 RRF
- [ ] **AC-E4:** `search.query(mode="semantic")` errors cleanly when embedder not loaded
- [ ] **AC-E5:** `search.query(mode="hybrid")` falls back to BM25 with warning when embedder not loaded
- [ ] **AC-E6:** `index.rebuild --skip-embed` rebuilds BM25 only, leaves vectors unchanged
- [ ] **AC-E7:** `index.rebuild` (no flags) rebuilds both BM25 and embeddings when model loaded
- [ ] **AC-E8:** Changed sections re-embedded; unchanged sections carry forward (hash-skip)
- [ ] **AC-E9:** `vectors.bin` binary format roundtrips: write -> read-back -> identical HashMap
- [ ] **AC-E10:** Hybrid query <50ms end-to-end (ONNX + cosine + RRF + serialization)
- [ ] **AC-E11:** Cosine similarity returns 1.0 for identical texts, <0.3 for unrelated texts
- [ ] **AC-E12:** `wm model remove` cleans up ~/.wm/models/<name>/
- [ ] **AC-E13:** `initial` tool reports model loaded/unloaded + available search modes
- [ ] **AC-E14:** NoopEmbedder (feature disabled) compiles without ort/tokenizers/reqwest
- [ ] **AC-E15:** `vectors.bin` validation rejects mismatched model with clear error
- [ ] **AC-E16:** Concurrent queries see consistent vector snapshots during rebuild (ArcSwap)
- [ ] **AC-E17:** Batch embedding (32 texts) completes in <300ms for 384-dim output
- [ ] **AC-E18:** MockEmbedder produces deterministic, reproducible vectors for same input

---
id: wiki:specs:onnx-embedding-integration

## Implementation Milestone (v2.0 - Week N)

1. Add `ort`, `tokenizers`, `memmap2` dependencies behind `embed` feature flag
2. Implement `Embedder` trait + `NoopEmbedder` + `MockEmbedder`
3. Implement `OnnxEmbedder::load()` + `embed_batch()` using ONNX Runtime
4. Implement `vectors.bin` binary format (write + read + validate header)
5. Implement `VectorStore` with `ArcSwap<HashMap<>>` + `snapshot()`
6. Implement `cosine_similarity()` + `top_k_cosine()` linear scan
7. Implement `SearchMode` enum + `rrf_fuse()` with k=60
8. Wire `search.query` mode parameter + query flow dispatch
9. Implement `build_embeddings()` with hash-aware incremental skip
10. Wire into `index.rebuild` (Phase 4) + `index.embed` standalone tool
11. Implement model registry + `wm model list/download/status/remove`
12. Model download: HTTPS streaming + progress bar + SHA-256 verification
13. Startup: non-blocking model load, graceful NoopEmbedder fallback
14. `initial` tool: report model state + available search modes
15. Integration tests: RRF correctness, hash-skip, binary roundtrip, graceful degradation
16. Benchmark: verify <50ms hybrid query, <300ms batch-32 embedding
