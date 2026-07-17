use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};

use super::{EmbedError, EmbedVector, Embedder};

pub struct OnnxEmbedder {
    session: Mutex<ort::session::Session>,
    tokenizer: tokenizers::Tokenizer,
    model_name: String,
    dim: usize,
    max_batch_size: usize,
    loaded: bool,
}

impl OnnxEmbedder {
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

        let tokenizer = tokenizers::Tokenizer::from_file(&tok_path)
            .map_err(|e| EmbedError::Tokenization(format!("tokenizer load: {}", e)))?;

        let dim = 384;

        Ok(Some(Self {
            session: Mutex::new(session),
            tokenizer,
            model_name: model_name.to_string(),
            dim,
            max_batch_size: 64,
            loaded: true,
        }))
    }
}

impl Embedder for OnnxEmbedder {
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

        let encoding = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| EmbedError::Tokenization(e.to_string()))?;

        let max_len = encoding.iter().map(|e| e.len()).max().unwrap_or(0);
        if max_len == 0 {
            return Err(EmbedError::Tokenization("empty tokenization".into()));
        }

        let batch_size = texts.len();
        let mut input_ids = vec![0i64; batch_size * max_len];
        let mut attention_mask = vec![0i64; batch_size * max_len];

        for (i, enc) in encoding.iter().enumerate() {
            let ids = enc.get_ids();
            let mask = enc.get_attention_mask();
            for j in 0..max_len {
                let idx = i * max_len + j;
                input_ids[idx] = if j < ids.len() { ids[j] as i64 } else { 0 };
                attention_mask[idx] = if j < mask.len() { mask[j] as i64 } else { 0 };
            }
        }

        let shape = vec![batch_size as i64, max_len as i64];
        let input_tensor = ort::value::Tensor::from_array(
            (shape.clone(), input_ids)
        )
        .map_err(|e| EmbedError::Inference(e.to_string()))?;
        let mask_tensor = ort::value::Tensor::from_array(
            (shape, attention_mask)
        )
        .map_err(|e| EmbedError::Inference(e.to_string()))?;
        let input_values = ort::inputs![input_tensor, mask_tensor];

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

        let flat = output_view.as_slice()
            .ok_or_else(|| EmbedError::Inference("non-contiguous output tensor".into()))?;
        let mut vectors = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            let cls_start = i * seq_len * output_dim;
            let cls_vec: Vec<f32> = flat[cls_start..cls_start + output_dim].to_vec();
            vectors.push(EmbedVector(cls_vec).normalized());
        }

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
        sha256: "0000000000000000000000000000000000000000000000000000000000000000",
    },
    ModelEntry {
        name: "all-MiniLM-L6-v2",
        dim: 384,
        url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx",
        sha256: "0000000000000000000000000000000000000000000000000000000000000000",
    },
];

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
        let downloaded = bytes.len() as u64;
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

        println!("  {:.1} MB downloaded", downloaded as f64 / 1_000_000.0);
        println!("  Verifying SHA-256...");

        let hash_hex = hex::encode(hasher.finalize());
        let expected = entry.sha256;
        if expected != "0000000000000000000000000000000000000000000000000000000000000000" {
            if hash_hex != expected {
                let _ = std::fs::remove_file(&model_path);
                return Err(EmbedError::Download(format!(
                    "SHA-256 mismatch: got {}, expected {}", hash_hex, expected
                )));
            }
            println!("  SHA-256: {} ✅", hash_hex);
        } else {
            println!("  SHA-256: {} (verification skipped — update sha256 in MODEL_REGISTRY)", hash_hex);
        }

        if entry.sha256 != "0000000000000000000000000000000000000000000000000000000000000000" {
            if hash_hex == entry.sha256 {
                println!("  ✓ Hash matches expected value");
            } else {
                let _ = std::fs::remove_file(&model_path);
                return Err(EmbedError::Download(format!(
                    "SHA-256 mismatch: expected {} but got {}. Download may be corrupted.",
                    entry.sha256, hash_hex
                )));
            }
        } else {
            println!("  ⚠ Placeholder hash in registry — verification skipped. Replace with real SHA-256 hash from HuggingFace model card.");
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
        println!("  {:.1} KB downloaded", bytes.len() as f64 / 1000.0);
    }

    let vectors_path = std::env::current_dir()
        .unwrap_or_default()
        .join(".wm")
        .join("state")
        .join("vectors.bin");
    if vectors_path.exists() {
        let header_model_name = std::fs::read(&vectors_path).ok().and_then(|data| {
            if data.len() < 24 {
                return None;
            }
            let name_len = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
            if 24 + name_len > data.len() {
                return None;
            }
            Some(String::from_utf8_lossy(&data[24..24 + name_len]).to_string())
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
        serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".to_string()),
    )
    .map_err(|e| EmbedError::Download(format!("write manifest: {}", e)))?;

    println!("Model cached at {}", model_dir.display());
    Ok(model_dir)
}
