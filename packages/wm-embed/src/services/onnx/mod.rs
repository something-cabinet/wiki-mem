use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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

pub struct EmbeddingModel {
    session: Mutex<ort::session::Session>,
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
    pub fn load(model_dir: &Path, model_name: &str) -> Result<Option<Self>, EmbedError> {
        let model_path = model_dir.join(model_name).join("model.onnx");
        let tok_path = model_dir.join(model_name).join("tokenizer.json");

        if !model_path.exists() || !tok_path.exists() {
            return Ok(None);
        }

        let _ = ort::init().with_name("wm-onnx").commit();

        let session = ort::session::Session::builder()
            .map_err(|e| EmbedError::Inference(format!("session builder: {}", e)))?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| EmbedError::Inference(format!("optimization: {}", e)))?
            .with_intra_threads(4)
            .map_err(|e| EmbedError::Inference(format!("threads: {}", e)))?
            .commit_from_file(&model_path)
            .map_err(|e| EmbedError::Inference(format!("session load: {}", e)))?;

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

        let dim = 384;
        let cfg = lookup_model_config(model_name);

        Ok(Some(Self {
            session: Mutex::new(session),
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

        let mut session_guard = self
            .session
            .lock()
            .map_err(|_| EmbedError::Inference("session lock poisoned".into()))?;
        let outputs = session_guard
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
            .ok_or_else(|| EmbedError::Inference("non-contiguous output tensor".into()))?;

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
                mean_pooling(flat, &mask_for_pooling, batch_size, seq_len, output_dim)
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
        // TODO: Set real SHA-256 hash here or via WM_MODEL_SHA env var
        sha256: "",
    },
    ModelEntry {
        name: "all-MiniLM-L6-v2",
        dim: 384,
        url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx",
        // TODO: Set real SHA-256 hash here or via WM_MODEL_SHA env var
        sha256: "",
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
        let expected_env = std::env::var("WM_MODEL_SHA").ok();
        let expected: &str = expected_env
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(entry.sha256);

        if expected.is_empty() {
            // Model integrity verification not yet implemented
            println!("  ⚠ Model integrity verification not yet implemented — set WM_MODEL_SHA={} to verify", hash_hex);
        } else if hash_hex != expected {
            let _ = std::fs::remove_file(&model_path);
            return Err(EmbedError::Download(format!(
                "SHA-256 mismatch: got {}, expected {}. The download may be corrupted or the expected hash is outdated.",
                hash_hex, expected
            )));
        } else {
            println!("  ✓ SHA-256 hash matches expected value");
        }
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
