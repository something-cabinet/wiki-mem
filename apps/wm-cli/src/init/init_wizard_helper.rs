use std::path::Path;

use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};
use wm_constants::{CONFIG_FILE, STATE_DIR, WM_DIR};
use wm_core::config::{self, GitTracking};

const SEMANTIC_PROMPT: &str =
    "Enable semantic search (ONNX embeddings)? This requires downloading a ~134MB model";
const MODEL_PROMPT: &str = "Select embedding model";
const MODEL_NAMES: [&str; 3] = [
    "bge-small-en-v1.5 — 384 dim, ~134MB, recommended",
    "all-MiniLM-L6-v2 — 384 dim, ~90MB, faster",
    "bge-base-en-v1.5 — 768 dim, ~438MB, highest accuracy",
];
const MODEL_BGE_SMALL: &str = "bge-small-en-v1.5";
const MODEL_ALL_MINILM: &str = "all-MiniLM-L6-v2";
const MODEL_BGE_BASE: &str = "bge-base-en-v1.5";
const GIT_PROMPT: &str = "Git tracking mode for .wm/ directory";
const GIT_OPTIONS: [&str; 3] = [
    "git-tracked — track everything (config, wiki pages, memory)",
    "git-ignored — track config + wiki pages; ignore memory, generated files",
    "none — no .gitignore changes (manage manually)",
];
const MEMORY_DIR: &str = "memory";
const VERSIONS_DIR: &str = "versions";
const IGNORED_GIT_DIRS: [&str; 3] = [STATE_DIR, MEMORY_DIR, VERSIONS_DIR];
const PLATFORM_PROMPT: &str =
    "Generate platform agent instruction files? (Space to toggle, Enter to confirm)";
const PLATFORM_ITEMS: [&str; 7] = [
    "CLAUDE.md — Claude Code",
    "OPENCODE.md + AGENTS.md — OpenCode",
    "AGENTS.md — Kiro",
    "GEMINI.md — Gemini",
    ".github/copilot-instructions.md — GitHub Copilot",
    "AGENTS.md — Generic agents",
    "REASONIX.md — Reasonix",
];
const PLATFORM_NAMES: [&str; 7] = [
    "claude", "opencode", "kiro", "gemini", "copilot", "agents", "reasonix",
];

pub fn run_wizard(root: &Path) {
    let theme = ColorfulTheme::default();
    run_semantic_wizard(root, &theme);
    run_git_wizard(root, &theme);
}

pub fn resolve_platforms(platform: Option<String>, no_wizard: bool) -> Vec<String> {
    if let Some(platform) = platform {
        return vec![platform.to_lowercase()];
    }
    if no_wizard || !is_terminal::is_terminal(std::io::stdin()) {
        return Vec::new();
    }
    prompt_platforms()
}

fn run_semantic_wizard(root: &Path, theme: &ColorfulTheme) {
    if !prompt_semantic_enabled(theme) {
        println!("  Semantic search disabled (keyword-only mode)");
        return;
    }
    let model_name = prompt_model_name(theme);
    persist_embedding_model(root, model_name);
    println!("  Semantic search enabled (model: {})", model_name);
}

fn prompt_semantic_enabled(theme: &ColorfulTheme) -> bool {
    Confirm::with_theme(theme)
        .with_prompt(SEMANTIC_PROMPT)
        .default(false)
        .interact()
        .unwrap_or(false)
}

fn prompt_model_name(theme: &ColorfulTheme) -> &'static str {
    let choice = Select::with_theme(theme)
        .with_prompt(MODEL_PROMPT)
        .items(&MODEL_NAMES)
        .default(0)
        .interact()
        .unwrap_or(0);
    model_name_for(choice)
}

fn model_name_for(choice: usize) -> &'static str {
    match choice {
        1 => MODEL_ALL_MINILM,
        2 => MODEL_BGE_BASE,
        _ => MODEL_BGE_SMALL,
    }
}

fn persist_embedding_model(root: &Path, model_name: &str) {
    let config_path = root.join(WM_DIR).join(CONFIG_FILE);
    let Some(mut config) = read_config_json(&config_path) else {
        return;
    };
    if let Some(object) = config.get_mut("embedding").and_then(|embed| embed.as_object_mut()) {
        object.insert("model_name".into(), serde_json::json!(model_name));
    }
    write_config_json(&config_path, &config);
}

fn run_git_wizard(root: &Path, theme: &ColorfulTheme) {
    let git_mode = prompt_git_mode(theme);
    let git_tracking = git_tracking_for(git_mode);
    apply_git_mode(root, git_mode, &git_tracking);
    persist_git_tracking(root, &git_tracking);
}

fn prompt_git_mode(theme: &ColorfulTheme) -> usize {
    let selection = Select::with_theme(theme)
        .with_prompt(GIT_PROMPT)
        .items(&GIT_OPTIONS)
        .default(0)
        .interact()
        .unwrap_or(0);
    selection + 1
}

fn git_tracking_for(git_mode: usize) -> GitTracking {
    match git_mode {
        2 => GitTracking { memory: Some(true), versions: Some(true), state: Some(true) },
        1 => GitTracking { memory: Some(false), versions: Some(false), state: Some(false) },
        _ => GitTracking { memory: None, versions: None, state: None },
    }
}

fn apply_git_mode(root: &Path, git_mode: usize, git_tracking: &GitTracking) {
    if let Err(error) = config::apply_git_tracking(root, git_tracking) {
        eprintln!("  Warning: failed to update .gitignore: {}", error);
        return;
    }
    match git_mode {
        2 => untrack_ignored_dirs(root),
        1 => println!("  .gitignore: .wm/ fully tracked"),
        _ => println!("  .gitignore: unchanged (manage manually)"),
    }
}

fn untrack_ignored_dirs(root: &Path) {
    println!("  .gitignore: .wm/state/, .wm/memory/, .wm/versions/ ignored");
    for dir in &IGNORED_GIT_DIRS {
        untrack_dir(root, dir);
    }
}

fn untrack_dir(root: &Path, dir: &str) {
    let path = format!("{}/{}", WM_DIR, dir);
    let Ok(tracked) = git_ls_files(root, &path) else {
        return;
    };
    if tracked.stdout.is_empty() {
        return;
    }
    git_rm_cached(root, &path);
}

fn git_ls_files(root: &Path, path: &str) -> Result<std::process::Output, std::io::Error> {
    std::process::Command::new("git")
        .args(["ls-files", "--cached"])
        .arg(path)
        .current_dir(root)
        .output()
}

fn git_rm_cached(root: &Path, path: &str) {
    std::process::Command::new("git")
        .args(["rm", "--cached", "-r"])
        .arg(path)
        .current_dir(root)
        .output()
        .ok();
}

fn persist_git_tracking(root: &Path, git_tracking: &GitTracking) {
    let config_path = root.join(WM_DIR).join(CONFIG_FILE);
    let Some(mut config) = read_config_json(&config_path) else {
        return;
    };
    config["git_tracking"] = serde_json::json!(git_tracking);
    write_config_json(&config_path, &config);
}

fn prompt_platforms() -> Vec<String> {
    let theme = ColorfulTheme::default();
    let selections = MultiSelect::with_theme(&theme)
        .with_prompt(PLATFORM_PROMPT)
        .items(&PLATFORM_ITEMS)
        .interact()
        .unwrap_or_default();
    selections
        .iter()
        .map(|&index| PLATFORM_NAMES[index].to_string())
        .collect()
}

fn read_config_json(path: &Path) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<serde_json::Value>(&content).ok()
}

fn write_config_json(path: &Path, config: &serde_json::Value) {
    if let Ok(updated) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(path, &updated);
    }
}
