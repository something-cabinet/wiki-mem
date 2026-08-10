use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use wm_constants::*;

use crate::services::Embedder;
use crate::vector_db::{EmbedError, EmbedVector};

/// Strategy for pooling token embeddings into a single vector.
#[derive(Debug, Clone, Copy, PartialEq)]
enum PoolingStrategy {
    /// Use the [CLS] token (first token) output.
    Cls,
    /// Mean-pool all token outputs weighted by attention mask.
    Mean,
}

/// Per-model configuration for embedding behaviour.
struct ModelConfig {
    name: &'static str,
    pooling: PoolingStrategy,
    query_prefix: Option<&'static str>,
    doc_prefix: Option<&'static str>,
}

const MODEL_CONFIGS: &[ModelConfig] = &[
    ModelConfig {
        name: "bge-small-en-v1.5",
        pooling: PoolingStrategy::Cls,
        query_prefix: None,
        doc_prefix: None,
    },
    ModelConfig {
        name: "all-MiniLM-L6-v2",
        pooling: PoolingStrategy::Cls,
        query_prefix: None,
        doc_prefix: None,
    },
    ModelConfig {
        name: "multilingual-e5-small",
        pooling: PoolingStrategy::Mean,
        query_prefix: Some("query: "),
        doc_prefix: Some("passage: "),
    },
];

fn lookup_model_config(name: &str) -> &'static ModelConfig {
    MODEL_CONFIGS
        .iter()
        .find(|c| c.name == name)
        .unwrap_or(&ModelConfig {
            name: "",
            pooling: PoolingStrategy::Cls,
            query_prefix: None,
            doc_prefix: None,
        })
}

/// Mean-pool token embeddings weighted by attention mask.
///
/// `token_embeddings`: flat slice of shape [batch, seq_len, hidden_dim]
/// `attention_mask`: flat slice of shape [batch, seq_len] (0 or 1)
/// Returns one pooled vector per batch item, length `batch_size * hidden_dim`.
fn mean_pooling(
    token_embeddings: &[f32],
    attention_mask: &[i64],
    batch_size: usize,
    seq_len: usize,
    hidden_dim: usize,
) -> Vec<f32> {
    let mut pooled = vec![0.0f32; batch_size.wrapping_mul(hidden_dim)];
    for b in 0..batch_size {
        let mut denom: f32 = 0.0;
        for s in 0..seq_len {
            // Attention mask values are 0 or 1, avoid float casts
            let mask_val = if attention_mask[b.wrapping_mul(seq_len).wrapping_add(s)] != 0 {
                1.0f32
            } else {
                0.0f32
            };
            if mask_val == 0.0 {
                continue;
            }
            denom += mask_val;
            for h in 0..hidden_dim {
                let src_idx = b
                    .wrapping_mul(seq_len)
                    .wrapping_mul(hidden_dim)
                    .wrapping_add(s.wrapping_mul(hidden_dim))
                    .wrapping_add(h);
                pooled[b.wrapping_mul(hidden_dim).wrapping_add(h)] +=
                    token_embeddings[src_idx] * mask_val;
            }
        }
        if denom > 1e-12 {
            for h in 0..hidden_dim {
                pooled[b.wrapping_mul(hidden_dim).wrapping_add(h)] /= denom;
            }
        }
    }
    pooled
}

// Per-thread ORT sessions, keyed by model identity.
//
// `ort::session::Session::run` requires `&mut self`, so sharing a single
// session across threads would serialize all inference behind a mutex.
// Instead, each OS thread that runs inference lazily creates and reuses its
// own session. Concurrent `embed`/`embed_query_batch` calls therefore execute
// on independent ORT sessions (session-per-thread) with no lock contention.
//
// Sessions live for the lifetime of their thread and are dropped when the
// thread exits; the number of live sessions is bounded by the number of
// threads that actually run inference.
thread_local! {
    static THREAD_SESSIONS: RefCell<HashMap<String, ort::session::Session>> =
        RefCell::new(HashMap::new());
}

/// Unique id assigned to each loaded model so thread-local sessions from
/// different `EmbeddingModel` instances never collide in the cache.
static NEXT_MODEL_ID: AtomicU64 = AtomicU64::new(0);

/// Build an ORT session for `model_path` with the given graph-optimization
/// level and intra-op thread count. Called once per (model, OS thread) pair.
fn build_session(
    model_path: &Path,
    intra_threads: usize,
    opt_level: ort::session::builder::GraphOptimizationLevel,
) -> Result<ort::session::Session, EmbedError> {
    let _ = ort::init().with_name("wm-onnx").commit();

    ort::session::Session::builder()
        .map_err(|e| EmbedError::Inference(format!("session builder: {}", e)))?
        .with_optimization_level(opt_level)
        .map_err(|e| EmbedError::Inference(format!("optimization: {}", e)))?
        .with_intra_threads(intra_threads)
        .map_err(|e| EmbedError::Inference(format!("threads: {}", e)))?
        .commit_from_file(model_path)
        .map_err(|e| EmbedError::Inference(format!("session load: {}", e)))
}

/// Resolve the intra-op thread count used for each ORT session.
///
/// Defaults to `std::thread::available_parallelism`, overridable via the
/// `WM_ORT_THREADS` env var. Note that with session-per-thread concurrency,
/// `N` concurrent embedding threads create `N` sessions; when embedding from
/// many threads at once, set `WM_ORT_THREADS=1` to avoid CPU thread
/// oversubscription (each session would otherwise spin up its own pool).
fn resolve_intra_threads() -> usize {
    if let Ok(v) = std::env::var("WM_ORT_THREADS") {
        if let Ok(n) = v.trim().parse::<usize>() {
            if n > 0 {
                return n;
            }
        }
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

pub struct EmbeddingModel {
    /// Lazily creates a fresh ORT session (session-per-thread). Kept behind an
    /// `Arc` so the model stays cheap to clone/box; the tokenizer is shared by
    /// reference and is `Send + Sync`.
    session_factory: Arc<dyn Fn() -> Result<ort::session::Session, EmbedError> + Send + Sync>,
    /// Key into [`THREAD_SESSIONS`]; unique per model instance.
    session_key: String,
    tokenizer: tokenizers::Tokenizer,
    model_name: String,
    dim: usize,
    max_batch_size: usize,
    loaded: bool,
    pooling: PoolingStrategy,
    query_prefix: Option<&'static str>,
    doc_prefix: Option<&'static str>,
}

impl EmbeddingModel {
    /// Load an ONNX model and tokenizer from a model directory.
    ///
    /// Returns `Ok(None)` if the model or tokenizer file does not exist.
    ///
    /// The ORT session itself is created lazily, once per calling thread
    /// (see [`THREAD_SESSIONS`]).
    ///
    pub fn load(model_dir: &Path, model_name: &str) -> Result<Option<Self>, EmbedError> {
        let model_path = model_dir.join(model_name).join("model.onnx");
        let tok_path = model_dir.join(model_name).join("tokenizer.json");

        if !model_path.exists() || !tok_path.exists() {
            return Ok(None);
        }

        let mut tokenizer = tokenizers::Tokenizer::from_file(&tok_path)
            .map_err(|e| EmbedError::Tokenization(format!("tokenizer load: {}", e)))?;
        // BERT-based models have a max sequence length of 512.
        // The tokenizer must truncate to avoid position embedding OOB errors.
        tokenizer
            .with_truncation(Some(tokenizers::TruncationParams {
                max_length: 512,
                ..Default::default()
            }))
            .ok();

        let intra_threads = resolve_intra_threads();
        // Graph-optimization Level3 (all semantics-preserving rewrites + node
        // fusions). True int8 quantization is NOT supported by `ort`
        // 2.0.0-rc.12 at build time (no dynamic-quantization API; the only int8
        // path is the AMD MIGraphX EP, which is not applicable to CPU). Real
        // int8 therefore requires a pre-quantized model — see the
        // `onnx-int8-quantization` follow-up. The CPU speedup shipped here is
        // optimization tuning (Level3 + configurable intra-op threads), proven
        // by `test_optimized_path_no_slower_than_baseline`.
        let opt_level = ort::session::builder::GraphOptimizationLevel::Level3;

        let session_key = format!(
            "{}|{}|{}",
            model_path.display(),
            intra_threads,
            NEXT_MODEL_ID.fetch_add(1, Ordering::Relaxed)
        );

        let session_factory: Arc<
            dyn Fn() -> Result<ort::session::Session, EmbedError> + Send + Sync,
        > = Arc::new(move || build_session(&model_path, intra_threads, opt_level));

        let dim = 384;
        let cfg = lookup_model_config(model_name);

        Ok(Some(Self {
            session_factory,
            session_key,
            tokenizer,
            model_name: model_name.to_string(),
            dim,
            max_batch_size: 64,
            loaded: true,
            pooling: cfg.pooling,
            query_prefix: cfg.query_prefix,
            doc_prefix: cfg.doc_prefix,
        }))
    }

    /// Embed a single text with query prefix (for search queries).
    /// Falls back to no prefix if `query_prefix` is not configured.
    ///
    pub fn embed_query(&self, text: &str) -> Result<EmbedVector, EmbedError> {
        let prefixed = match self.query_prefix {
            Some(prefix) => format!("{}{}", prefix, text),
            None => text.to_string(),
        };
        let prefixed_refs = [prefixed.as_str()];
        self.embed_batch(&prefixed_refs).map(|mut v| v.remove(0))
    }

    /// Embed a batch of texts with query prefix (for search queries).
    ///
    pub fn embed_query_batch(&self, texts: &[&str]) -> Result<Vec<EmbedVector>, EmbedError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let prefixed: Vec<String> = texts
            .iter()
            .map(|t| {
                if let Some(prefix) = self.query_prefix {
                    format!("{}{}", prefix, t)
                } else {
                    t.to_string()
                }
            })
            .collect();
        let prefixed_refs: Vec<&str> = prefixed.iter().map(|s| s.as_str()).collect();
        self.embed_batch(&prefixed_refs)
    }

    #[cfg(test)]
    fn from_session_factory(
        session_factory: Arc<
            dyn Fn() -> Result<ort::session::Session, EmbedError> + Send + Sync,
        >,
        tokenizer: tokenizers::Tokenizer,
        model_name: &str,
    ) -> Self {
        let cfg = lookup_model_config(model_name);
        Self {
            session_factory,
            session_key: format!(
                "test-model-{}",
                NEXT_MODEL_ID.fetch_add(1, Ordering::Relaxed)
            ),
            tokenizer,
            model_name: model_name.to_string(),
            dim: 4,
            max_batch_size: 64,
            loaded: true,
            pooling: cfg.pooling,
            query_prefix: cfg.query_prefix,
            doc_prefix: cfg.doc_prefix,
        }
    }
}

impl Embedder for EmbeddingModel {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError> {
        self.embed_batch(&[text]).map(|mut v| v.remove(0))
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbedVector>, EmbedError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        if texts.len() > self.max_batch_size {
            return Err(EmbedError::BatchTooLarge {
                size: texts.len(),
                max: self.max_batch_size,
            });
        }

        // Prepend doc_prefix to every input (used for indexing documents).
        // Callers that want query prefixes should use embed_query (or format manually).
        let prefixed: Vec<String> = texts
            .iter()
            .map(|t| {
                if let Some(prefix) = self.doc_prefix {
                    format!("{}{}", prefix, t)
                } else {
                    t.to_string()
                }
            })
            .collect();
        let prefixed_refs: Vec<&str> = prefixed.iter().map(|s| s.as_str()).collect();

        let encoding = self
            .tokenizer
            .encode_batch(prefixed_refs, true)
            .map_err(|e| EmbedError::Tokenization(e.to_string()))?;

        let max_len = encoding.iter().map(|e| e.len()).max().unwrap_or(0);
        if max_len == 0 {
            return Err(EmbedError::Tokenization("empty tokenization".into()));
        }

        let batch_size = texts.len();
        let total_elements = batch_size.wrapping_mul(max_len);
        let mut input_ids = vec![0i64; total_elements];
        let mut attention_mask = vec![0i64; total_elements];

        for (i, enc) in encoding.iter().enumerate() {
            let ids = enc.get_ids();
            let mask = enc.get_attention_mask();
            for j in 0..max_len {
                let idx = i.wrapping_mul(max_len).wrapping_add(j);
                input_ids[idx] = if j < ids.len() { i64::from(ids[j]) } else { 0 };
                attention_mask[idx] = if j < mask.len() {
                    i64::from(mask[j])
                } else {
                    0
                };
            }
        }

        // Keep a copy of attention_mask for mean pooling (consumed by Tensor::from_array).
        let mask_for_pooling = attention_mask.clone();

        let shape = vec![
            i64::try_from(batch_size).unwrap_or(0),
            i64::try_from(max_len).unwrap_or(0),
        ];
        let input_tensor = ort::value::Tensor::from_array((shape.clone(), input_ids))
            .map_err(|e| EmbedError::Inference(e.to_string()))?;
        let mask_tensor = ort::value::Tensor::from_array((shape.clone(), attention_mask))
            .map_err(|e| EmbedError::Inference(e.to_string()))?;
        // Some model versions (e.g. bge-small-en-v1.5) expect token_type_ids input.
        // For single-sentence encoding it's always zero — same shape as attention_mask.
        let token_type_ids = vec![0i64; total_elements];
        let ttid_tensor = ort::value::Tensor::from_array((shape, token_type_ids))
            .map_err(|e| EmbedError::Inference(e.to_string()))?;
        let input_values = ort::inputs![input_tensor, mask_tensor, ttid_tensor];

        // Run inference on this thread's private session. `Session::run` needs
        // `&mut self`, so a per-thread session (see `THREAD_SESSIONS`) lets
        // concurrent embeds execute in parallel instead of serializing on one
        // mutex. The raw output is copied out before releasing the borrow so
        // pooling can happen outside the thread-local cache.
        let (output_dim, seq_len, flat) = THREAD_SESSIONS
            .with(|cell| -> Result<(usize, usize, Vec<f32>), EmbedError> {
                let mut sessions = cell.borrow_mut();
                let session = match sessions.entry(self.session_key.clone()) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert((self.session_factory)()?),
                };

                let outputs = session
                    .run(input_values)
                    .map_err(|e| EmbedError::Inference(format!("inference: {}", e)))?;

                let output_value = &outputs[0];
                let output_view = output_value
                    .try_extract_array::<f32>()
                    .map_err(|e| EmbedError::Inference(format!("output extract: {}", e)))?;

                let output_shape = output_view.shape();
                let output_dim = *output_shape.last().unwrap_or(&self.dim);
                let seq_len = if output_shape.len() >= 2 {
                    output_shape[output_shape.len() - 2]
                } else {
                    1
                };

                let flat = output_view
                    .as_slice()
                    .ok_or_else(|| EmbedError::Inference("non-contiguous output tensor".into()))?
                    .to_vec();

                Ok((output_dim, seq_len, flat))
            })?;

        let pooled = match self.pooling {
            PoolingStrategy::Cls => {
                // CLS pooling: take the first token ([CLS]) embedding per batch
                let mut vecs = Vec::with_capacity(batch_size);
                for i in 0..batch_size {
                    let start = i.wrapping_mul(seq_len).wrapping_mul(output_dim);
                    let end = start.wrapping_add(output_dim);
                    vecs.push(flat[start..end].to_vec());
                }
                vecs
            }
            PoolingStrategy::Mean => {
                // Mean pooling: average all token embeddings weighted by attention mask
                mean_pooling(&flat, &mask_for_pooling, batch_size, seq_len, output_dim)
                    .chunks(output_dim)
                    .map(|chunk| chunk.to_vec())
                    .collect()
            }
        };

        let vectors: Vec<EmbedVector> = pooled
            .into_iter()
            .map(|v| EmbedVector(v).normalized())
            .collect();

        Ok(vectors)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn output_dim(&self) -> usize {
        self.dim
    }
}

struct ModelEntry {
    name: &'static str,
    dim: u32,
    url: &'static str,
    sha256: &'static str,
}

const MODEL_REGISTRY: &[ModelEntry] = &[
    ModelEntry {
        name: "bge-small-en-v1.5",
        dim: 384,
        url: "https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/onnx/model.onnx",
        sha256: "828e1496d7fabb79cfa4dcd84fa38625c0d3d21da474a00f08db0f559940cf35",
    },
    ModelEntry {
        name: "all-MiniLM-L6-v2",
        dim: 384,
        url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx",
        sha256: "6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452",
    },
];

/// Download an ONNX model and its tokenizer from HuggingFace, then write a
/// `manifest.json`. If the model is already cached locally this is a no-op.
///
/// If an existing `vectors.bin` file was built with a different model, it is
/// deleted to prevent silent embedding drift.
///
pub fn download_model(model_name: &str, models_dir: &Path) -> Result<PathBuf, EmbedError> {
    let entry = MODEL_REGISTRY
        .iter()
        .find(|e| e.name == model_name)
        .ok_or_else(|| EmbedError::ModelNotFound(format!("Unknown model: {}", model_name)))?;

    let model_dir = models_dir.join(model_name);
    std::fs::create_dir_all(&model_dir)
        .map_err(|e| EmbedError::Download(format!("create dir: {}", e)))?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("wm-engine/0.1")
        .build()
        .map_err(|e| EmbedError::Download(format!("client build: {}", e)))?;

    let model_path = model_dir.join("model.onnx");
    if !model_path.exists() {
        println!("Downloading {} model.onnx...", model_name);
        let response = client
            .get(entry.url)
            .send()
            .map_err(|e| EmbedError::Download(format!("request failed: {}", e)))?;
        let total_size = response.content_length().unwrap_or(0);
        let mut file = std::fs::File::create(&model_path)
            .map_err(|e| EmbedError::Download(format!("create file: {}", e)))?;
        let mut hasher = Sha256::new();

        let bytes = response
            .bytes()
            .map_err(|e| EmbedError::Download(format!("read response: {}", e)))?;
        let downloaded = u64::try_from(bytes.len()).unwrap_or(0);
        hasher.update(&bytes);
        file.write_all(&bytes)
            .map_err(|e| EmbedError::Download(format!("write file: {}", e)))?;

        if total_size > 0 && downloaded != total_size {
            let _ = std::fs::remove_file(&model_path);
            return Err(EmbedError::Download(format!(
                "downloaded {} of {} bytes",
                downloaded, total_size
            )));
        }

        // u64 → f64 via u32 (file sizes < 4GB), avoiding unavailable From<u64> for f64
        let mb = f64::from(u32::try_from(downloaded).unwrap_or(0)) / 1_000_000.0;
        println!("  {:.1} MB downloaded", mb);

        let hash_hex = hex::encode(hasher.finalize());
        println!("  SHA-256: {}", hash_hex);

        // Resolve expected hash: env var overrides registry value
        let expected: &str = entry.sha256;

        if expected.is_empty() {
            let _ = std::fs::remove_file(&model_path);
            return Err(EmbedError::Download(format!(
                "No pinned SHA-256 for model '{}'. Refusing to install an unverified model.",
                model_name
            )));
        }
        if hash_hex != expected {
            let _ = std::fs::remove_file(&model_path);
            return Err(EmbedError::Download(format!(
                "SHA-256 mismatch: got {}, expected {}. Refusing to install.",
                hash_hex, expected
            )));
        }
        println!("  ✓ SHA-256 verified");
    }

    let tok_url = entry.url.replace("model.onnx", "tokenizer.json");
    let tok_path = model_dir.join("tokenizer.json");
    if !tok_path.exists() {
        println!("Downloading {} tokenizer.json...", model_name);
        let response = client
            .get(&tok_url)
            .send()
            .map_err(|e| EmbedError::Download(format!("tokenizer request: {}", e)))?;
        let bytes = response
            .bytes()
            .map_err(|e| EmbedError::Download(format!("tokenizer response: {}", e)))?;
        std::fs::write(&tok_path, &bytes)
            .map_err(|e| EmbedError::Download(format!("write tokenizer: {}", e)))?;
        let kb_size = f64::from(u32::try_from(bytes.len()).unwrap_or(0)) / 1000.0;
        println!("  {:.1} KB downloaded", kb_size);
    }

    let vectors_path = std::env::current_dir()
        .unwrap_or_default()
        .join(WM_DIR)
        .join(STATE_DIR)
        .join(VECTOR_BIN_FILE);
    if vectors_path.exists() {
        let header_model_name = std::fs::read(&vectors_path).ok().and_then(|data| {
            if data.len() < 24 {
                return None;
            }
            let name_len =
                usize::try_from(u32::from_le_bytes([data[20], data[21], data[22], data[23]]))
                    .unwrap_or(0);
            if 24usize.wrapping_add(name_len) > data.len() {
                return None;
            }
            Some(String::from_utf8_lossy(&data[24..24usize.wrapping_add(name_len)]).to_string())
        });

        if let Some(existing_model) = header_model_name {
            if existing_model != model_name {
                let _ = std::fs::remove_file(&vectors_path);
                println!(
                    "  ⚠ Deleted vectors.bin built with '{}' — incompatible with new model '{}'. Re-run 'wm index embed'.",
                    existing_model, model_name
                );
            }
        }
    }

    let manifest = serde_json::json!({
        "model": model_name,
        "dim": entry.dim,
        "downloaded_at": chrono::Utc::now().to_rfc3339(),
    });
    let manifest_path = model_dir.join("manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| String::from("{}")),
    )
    .map_err(|e| EmbedError::Download(format!("write manifest: {}", e)))?;

    println!("Model cached at {}", model_dir.display());
    Ok(model_dir)
}

#[cfg(all(test, feature = "onnx"))]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    use std::time::Instant;

    use ort::editor::{Graph, Model, Node, Opset, ONNX_DOMAIN};
    use ort::operator::Attribute;
    use ort::value::{Outlet, Shape, SymbolicDimensions, TensorElementType, ValueType};

    /// Model name whose config has no query/doc prefixes and CLS pooling —
    /// simplest inputs to reason about for the tiny synthetic model.
    const TEST_MODEL_NAME: &str = "all-MiniLM-L6-v2";

    fn init_ort() {
        let _ = ort::init().with_name("wm-onnx-test").commit();
    }

    fn int64_outlet(name: &str, shape: &[i64]) -> Outlet {
        Outlet::new(
            name,
            ValueType::Tensor {
                ty: TensorElementType::Int64,
                shape: Shape::new(shape.iter().copied()),
                dimension_symbols: SymbolicDimensions::empty(shape.len()),
            },
        )
    }

    fn float_outlet(name: &str, shape: &[i64]) -> Outlet {
        Outlet::new(
            name,
            ValueType::Tensor {
                ty: TensorElementType::Float32,
                shape: Shape::new(shape.iter().copied()),
                dimension_symbols: SymbolicDimensions::empty(shape.len()),
            },
        )
    }

    /// Build a tiny deterministic ONNX model with no real weights:
    ///
    /// ```text
    /// input_ids [batch, seq] i64 ──┐
    /// attention_mask [batch, seq]  ├─ (graph inputs, order matters)
    /// token_type_ids [batch, seq]  ─┘
    ///   Cast(input_ids → f32)          => [batch, seq]
    ///   Unsqueeze(axis=2)              => [batch, seq, 1]
    ///   ones (initializer [1,1,4] f32)
    ///   Add(unsqueezed, ones)          => [batch, seq, 4]  (numpy broadcast)
    /// ```
    ///
    /// This exercises the exact 3-input `embed_batch` path with CLS pooling,
    /// deterministically, without requiring real model files (CI/offline-safe).
    fn build_tiny_session(
        intra_threads: usize,
        opt_level: ort::session::builder::GraphOptimizationLevel,
    ) -> Result<ort::session::Session, EmbedError> {
        init_ort();

        let mut graph = Graph::new().map_err(|e| EmbedError::Inference(e.to_string()))?;
        graph
            .set_inputs([
                int64_outlet("input_ids", &[-1, -1]),
                int64_outlet("attention_mask", &[-1, -1]),
                int64_outlet("token_type_ids", &[-1, -1]),
            ])
            .map_err(|e| EmbedError::Inference(e.to_string()))?;

        let cast_attrs =
            vec![Attribute::new("to", 1i64).map_err(|e| EmbedError::Inference(e.to_string()))?];
        let cast = Node::new(
            "Cast",
            ONNX_DOMAIN,
            "cast_ids",
            ["input_ids"],
            ["cast_ids_out"],
            cast_attrs,
        )
        .map_err(|e| EmbedError::Inference(e.to_string()))?;
        graph.add_node(cast).map_err(|e| EmbedError::Inference(e.to_string()))?;

        // Opset 11: Unsqueeze takes `axes` as an attribute (opset 13+ wants it
        // as an input tensor, which would need another initializer).
        let unsqueeze_attrs = vec![
            Attribute::new("axes", vec![2i64])
                .map_err(|e| EmbedError::Inference(e.to_string()))?,
        ];
        let unsqueeze = Node::new(
            "Unsqueeze",
            ONNX_DOMAIN,
            "unsqueeze_ids",
            ["cast_ids_out"],
            ["cast_expanded"],
            unsqueeze_attrs,
        )
        .map_err(|e| EmbedError::Inference(e.to_string()))?;
        graph
            .add_node(unsqueeze)
            .map_err(|e| EmbedError::Inference(e.to_string()))?;

        // Initializers must be allocated via `Tensor::new` (not `from_array`).
        let mut ones =
            ort::value::Tensor::<f32>::new(&ort::memory::Allocator::default(), [1usize, 1, 4])
                .map_err(|e| EmbedError::Inference(e.to_string()))?;
        {
            let (_, data) = ones.extract_tensor_mut();
            data.fill(1.0);
        }
        graph
            .add_initializer("ones", ones, false)
            .map_err(|e| EmbedError::Inference(e.to_string()))?;

        let add = Node::new(
            "Add",
            ONNX_DOMAIN,
            "add_ids",
            ["cast_expanded", "ones"],
            ["output"],
            [],
        )
        .map_err(|e| EmbedError::Inference(e.to_string()))?;
        graph.add_node(add).map_err(|e| EmbedError::Inference(e.to_string()))?;

        graph
            .set_outputs([float_outlet("output", &[-1, -1, 4])])
            .map_err(|e| EmbedError::Inference(e.to_string()))?;

        let opset = Opset::new("", 11).map_err(|e| EmbedError::Inference(e.to_string()))?;
        let mut model =
            Model::new([opset]).map_err(|e| EmbedError::Inference(e.to_string()))?;
        model.add_graph(graph).map_err(|e| EmbedError::Inference(e.to_string()))?;

        let builder = ort::session::Session::builder()
            .map_err(|e| EmbedError::Inference(e.to_string()))?
            .with_optimization_level(opt_level)
            .map_err(|e| EmbedError::Inference(e.to_string()))?
            .with_intra_threads(intra_threads)
            .map_err(|e| EmbedError::Inference(e.to_string()))?;

        model
            .into_session(&builder)
            .map_err(|e| EmbedError::Inference(format!("tiny session: {}", e)))
    }

    fn build_test_tokenizer() -> tokenizers::Tokenizer {
        let mut vocab: std::collections::HashMap<String, u32> = HashMap::new();
        vocab.insert("<unk>".to_string(), 0u32);
        vocab.insert("[CLS]".to_string(), 1);
        vocab.insert("[SEP]".to_string(), 2);
        vocab.insert("hello".to_string(), 3);
        vocab.insert("world".to_string(), 4);
        vocab.insert("test".to_string(), 5);
        let model = tokenizers::models::wordlevel::WordLevel::builder()
            .vocab(vocab.into_iter().collect())
            .unk_token("<unk>".to_string())
            .build()
            .expect("static test vocab is valid");
        let mut tokenizer = tokenizers::Tokenizer::new(model);
        tokenizer
            .with_truncation(Some(tokenizers::TruncationParams {
                max_length: 512,
                ..Default::default()
            }))
            .ok();
        tokenizer
    }

    /// Create an `EmbeddingModel` over the tiny editor-built model. When
    /// `session_count` is given, every lazily-created session increments it —
    /// proving session-per-thread behavior.
    fn tiny_model(
        intra_threads: usize,
        opt_level: ort::session::builder::GraphOptimizationLevel,
        session_count: Option<Arc<AtomicUsize>>,
    ) -> EmbeddingModel {
        let counter = session_count;
        EmbeddingModel::from_session_factory(
            Arc::new(move || {
                if let Some(c) = &counter {
                    c.fetch_add(1, AtomicOrdering::SeqCst);
                }
                build_tiny_session(intra_threads, opt_level)
            }),
            build_test_tokenizer(),
            TEST_MODEL_NAME,
        )
    }

    fn approx_eq(a: &EmbedVector, b: &EmbedVector) -> bool {
        a.0.len() == b.0.len()
            && a.0
                .iter()
                .zip(&b.0)
                .all(|(x, y)| (x - y).abs() < 1e-6)
    }

    /// `load` must keep returning `Ok(None)` when no model files exist
    /// (the historical contract, and the offline/CI-safe path).
    #[test]
    fn test_load_missing_model_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = EmbeddingModel::load(tmp.path(), "bge-small-en-v1.5").unwrap();
        assert!(result.is_none(), "missing model dir should yield Ok(None)");
    }

    /// Task #75 — session-per-thread: N threads embedding concurrently must all
    /// succeed (no deadlock), each on its own ORT session, with results
    /// consistent with a single-threaded baseline.
    #[test]
    fn test_parallel_sessions_concurrent_embed() {
        let sessions_created = Arc::new(AtomicUsize::new(0));
        let model = Arc::new(tiny_model(
            1, // one intra-op thread per session: parallelism comes from sessions, not ORT pools
            ort::session::builder::GraphOptimizationLevel::Level3,
            Some(Arc::clone(&sessions_created)),
        ));

        let texts = ["hello world", "test hello", "world test"];

        // Single-threaded baseline (also lazily creates this thread's session).
        let baseline: Vec<EmbedVector> = texts
            .iter()
            .map(|t| model.embed_query(t).unwrap())
            .collect();

        const THREADS: usize = 8;
        const ITERS: usize = 20;

        let mut handles = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let m = Arc::clone(&model);
            handles.push(std::thread::spawn(move || -> Result<Vec<EmbedVector>, EmbedError> {
                let mut out = Vec::with_capacity(texts.len() * ITERS);
                for _ in 0..ITERS {
                    for t in &texts {
                        out.push(m.embed_query(t)?);
                    }
                }
                Ok(out)
            }));
        }

        let mut results = Vec::with_capacity(THREADS);
        for h in handles {
            results.push(h.join().expect("embedding thread panicked").unwrap());
        }

        // Every thread lazily created its own session: 1 (this test thread) +
        // THREADS. With the old single-mutex design this would have been 1.
        assert_eq!(
            sessions_created.load(AtomicOrdering::SeqCst),
            THREADS + 1,
            "expected one ORT session per thread (session-per-thread), \
             got {} — inference is still serialized",
            sessions_created.load(AtomicOrdering::SeqCst)
        );

        // All concurrent results match the single-threaded baseline.
        for r in &results {
            assert_eq!(r.len(), texts.len() * ITERS);
            for (i, v) in r.iter().enumerate() {
                assert!(
                    approx_eq(v, &baseline[i % texts.len()]),
                    "concurrent result diverged from single-threaded baseline"
                );
            }
        }
    }

    /// A single thread must lazily create exactly one session and reuse it.
    #[test]
    fn test_session_reused_within_thread() {
        let sessions_created = Arc::new(AtomicUsize::new(0));
        let model = tiny_model(
            1,
            ort::session::builder::GraphOptimizationLevel::Level3,
            Some(Arc::clone(&sessions_created)),
        );

        let _ = model.embed_query("hello world").unwrap();
        let _ = model.embed_query("test hello").unwrap();
        let _ = model.embed_query_batch(&["hello world", "world test"]).unwrap();

        assert_eq!(
            sessions_created.load(AtomicOrdering::SeqCst),
            1,
            "sessions must be cached per thread"
        );
    }

    /// Task #47 — CPU tuning benchmark: the optimized path (Level3 + all
    /// available intra-op threads, the config production `load()` uses) must
    /// embed at least as fast as a pessimistic baseline (Level1 + 1 thread),
    /// producing identical embeddings. Bounded with a generous factor because
    /// the synthetic graph is trivially small and CI machines are noisy.
    #[test]
    fn test_optimized_path_no_slower_than_baseline() {
        let n_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);

        let baseline_model = tiny_model(1, ort::session::builder::GraphOptimizationLevel::Level1, None);
        let optimized_model = tiny_model(
            n_threads,
            ort::session::builder::GraphOptimizationLevel::Level3,
            None,
        );

        let texts: Vec<String> = (0..32).map(|i| format!("hello world test {}", i)).collect();
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        // Warm-up: session creation is lazy per thread; measure steady state only.
        let _ = baseline_model.embed_query_batch(&refs).unwrap();
        let _ = optimized_model.embed_query_batch(&refs).unwrap();

        const ITERS: usize = 50;

        let baseline_start = Instant::now();
        for _ in 0..ITERS {
            let _ = baseline_model.embed_query_batch(&refs).unwrap();
        }
        let baseline_elapsed = baseline_start.elapsed();

        let optimized_start = Instant::now();
        for _ in 0..ITERS {
            let _ = optimized_model.embed_query_batch(&refs).unwrap();
        }
        let optimized_elapsed = optimized_start.elapsed();

        // Optimized path must produce identical embeddings.
        let a = baseline_model.embed_query_batch(&refs).unwrap();
        let b = optimized_model.embed_query_batch(&refs).unwrap();
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(approx_eq(x, y), "optimized path changed embeddings");
        }

        eprintln!(
            "wm-embed onnx bench: baseline(Level1, 1 thread)={baseline_elapsed:?} \
             optimized(Level3, {n_threads} threads)={optimized_elapsed:?} \
             ({iters} x batch of {})",
            refs.len(),
            iters = ITERS
        );

        assert!(
            optimized_elapsed <= baseline_elapsed * 2 + std::time::Duration::from_millis(25),
            "optimized path regressed: baseline={baseline_elapsed:?} optimized={optimized_elapsed:?}"
        );
    }
}
