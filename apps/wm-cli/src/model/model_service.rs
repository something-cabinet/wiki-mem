use std::path::PathBuf;
use std::sync::Arc;

use crate::model::ModelAction;
use crate::shared::traits::Command;
use wm_constants::WM_DIR;
use wm_core::engine::MainEngine;

const MODELS_SUBDIR: &str = "models";
const MODEL_LOADED_LABEL: &str = "(loaded)";
const MODEL_UNLOADED_LABEL: &str = "(not loaded)";
const MODEL_ACTIVE_LABEL: &str = "(active)";

#[cfg(feature = "onnx")]
const DOWNLOAD_SPINNER_TICK_MS: u64 = 100;

#[cfg(feature = "onnx")]
const DOWNLOAD_WORKER_PANIC_MESSAGE: &str = "download worker thread panicked";

impl Command for ModelAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_model_action(self)
    }
}

fn run_model_action(action: ModelAction) -> Result<(), anyhow::Error> {
    match action {
        ModelAction::Download { name, .. } => download_model(&name),
        ModelAction::List { .. } => list_models(),
        ModelAction::Status { .. } => show_model_status(),
        ModelAction::Remove { name, .. } => remove_model(&name),
    }
}

fn download_model(name: &str) -> Result<(), anyhow::Error> {
    #[cfg(feature = "onnx")]
    {
        run_onnx_download(name);
    }
    #[cfg(not(feature = "onnx"))]
    {
        let _ = name;
        eprintln!("Model download requires the 'onnx' feature. Rebuild with --features onnx.");
    }
    Ok(())
}

#[cfg(feature = "onnx")]
fn run_onnx_download(name: &str) {
    let spinner = build_download_spinner(name);
    let models_dir = models_dir();
    let result = download_model_isolated(name.to_string(), models_dir);
    spinner.finish_and_clear();
    match result {
        Ok(dir) => println!("Model downloaded to {}", dir.display()),
        Err(error) => eprintln!("Download failed: {}", error),
    }
}

#[cfg(feature = "onnx")]
fn download_model_isolated(name: String, models_dir: PathBuf) -> Result<PathBuf, anyhow::Error> {
    let handle = std::thread::spawn(move || wm_core::embed::download_model(&name, &models_dir));
    match handle.join() {
        Ok(result) => result.map_err(anyhow::Error::from),
        Err(_) => Err(anyhow::anyhow!(DOWNLOAD_WORKER_PANIC_MESSAGE)),
    }
}

#[cfg(feature = "onnx")]
fn build_download_spinner(name: &str) -> indicatif::ProgressBar {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_style(download_spinner_style());
    spinner.enable_steady_tick(std::time::Duration::from_millis(DOWNLOAD_SPINNER_TICK_MS));
    spinner.set_message(format!("Downloading {}...", name));
    spinner
}

#[cfg(feature = "onnx")]
fn download_spinner_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
        .tick_strings(&[
            "▹▹▹▹▹",
            "▸▹▹▹▹",
            "▹▸▹▹▹",
            "▹▹▸▹▹",
            "▹▹▹▸▹",
            "▹▹▹▹▸",
            "▪▪▪▪▪",
        ])
}

fn models_root() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
}

fn models_dir_for_root(root: &std::path::Path) -> PathBuf {
    root.join(WM_DIR).join(MODELS_SUBDIR)
}

fn models_dir() -> PathBuf {
    models_dir_for_root(&models_root())
}

fn list_models() -> Result<(), anyhow::Error> {
    let engine = Arc::new(MainEngine::new());
    let model_name = engine.state.embedder.model_name().to_string();
    print_active_model(&engine, &model_name);
    println!();
    println!("Cached models:");
    print_cached_models(&model_name);
    println!();
    println!("Available for download:");
    print_available_models();
    Ok(())
}

fn print_active_model(engine: &MainEngine, model_name: &str) {
    let loaded_label = if engine.state.embedder.is_loaded() {
        MODEL_LOADED_LABEL
    } else {
        MODEL_UNLOADED_LABEL
    };
    println!("Active model: {} {}", model_name, loaded_label);
    println!(
        "Sections indexed: {}",
        engine.state.vector_store.snapshot().len()
    );
}

fn print_cached_models(model_name: &str) {
    let models_dir = models_dir();
    if !models_dir.exists() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(&models_dir) else {
        return;
    };
    for entry in entries.flatten() {
        print_cached_model_entry(&entry, model_name);
    }
}

fn print_cached_model_entry(entry: &std::fs::DirEntry, model_name: &str) {
    if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
        return;
    }
    let name = entry.file_name().to_string_lossy().to_string();
    let active_label = if name == model_name {
        MODEL_ACTIVE_LABEL
    } else {
        ""
    };
    println!("  - {} {}", name, active_label);
}

fn print_available_models() {
    println!("  - bge-small-en-v1.5 (384-dim, 134 MB, recommended)");
    println!("  - bge-base-en-v1.5 (768-dim, 438 MB)");
    println!("  - all-MiniLM-L6-v2 (384-dim, 90 MB)");
}

fn show_model_status() -> Result<(), anyhow::Error> {
    let engine = Arc::new(MainEngine::new());
    println!("Model:            {}", engine.state.embedder.model_name());
    println!("Loaded:           {}", engine.state.embedder.is_loaded());
    println!("Dimensions:       {}", engine.state.embedder.output_dim());
    println!(
        "Sections indexed: {}",
        engine.state.vector_store.snapshot().len()
    );
    Ok(())
}

fn remove_model(name: &str) -> Result<(), anyhow::Error> {
    let model_dir = models_dir().join(name);
    if !model_dir.exists() {
        println!("Model not found: {}", name);
        return Ok(());
    }
    std::fs::remove_dir_all(&model_dir)?;
    println!("Removed model: {}", name);
    Ok(())
}

#[cfg(all(test, feature = "onnx"))]
mod tests {
    use super::*;

    const TEST_MODEL_NAME: &str = "bge-small-en-v1.5";
    const WEIGHTS_FILE: &str = "model.onnx";
    const TOKENIZER_FILE: &str = "tokenizer.json";
    const STUB_BYTES: &[u8] = b"stub";

    fn fixture_models_dir() -> PathBuf {
        let models_dir =
            std::env::temp_dir().join(format!("wm-cli-download-{}", std::process::id()));
        let model_dir = models_dir.join(TEST_MODEL_NAME);
        std::fs::create_dir_all(&model_dir).expect("create fixture model dir");
        std::fs::write(model_dir.join(WEIGHTS_FILE), STUB_BYTES).expect("write weights stub");
        std::fs::write(model_dir.join(TOKENIZER_FILE), STUB_BYTES).expect("write tokenizer stub");
        models_dir
    }

    #[test]
    fn download_inside_async_runtime_does_not_panic() {
        let models_dir = fixture_models_dir();
        let runtime = tokio::runtime::Runtime::new().expect("build tokio runtime");
        runtime.block_on(async {
            let result = download_model_isolated(TEST_MODEL_NAME.to_string(), models_dir.clone());
            assert!(result.is_ok(), "download failed: {:?}", result.err());
        });
        std::fs::remove_dir_all(&models_dir).ok();
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    const HOME_ROOT: &str = "/tmp/wm-cli-home";
    const MODELS_DIR_TAIL: &str = ".wm/models";

    #[test]
    fn models_dir_resolves_under_hidden_wm_dir() {
        let root = std::path::Path::new(HOME_ROOT);
        let resolved = models_dir_for_root(root);
        assert_eq!(resolved, root.join(WM_DIR).join(MODELS_SUBDIR));
        assert!(
            resolved.ends_with(MODELS_DIR_TAIL),
            "models dir must live under $HOME/.wm/models, got {}",
            resolved.display()
        );
        assert_ne!(
            resolved,
            root.join(MODELS_SUBDIR),
            "models dir must not resolve to $HOME/models"
        );
    }
}
