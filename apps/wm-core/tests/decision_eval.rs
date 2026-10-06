#![cfg(feature = "decision")]

#[path = "support/decision_eval.rs"]
mod eval;

use std::path::PathBuf;

use eval::{EvalConfig, Phrasing, ScoreDecode, Serialization};
use wm_core::decision::gliner_backend::GlinerBackend;
use wm_engine::DecisionBackend;

const ENV_MODEL_DIR: &str = "WM_EVAL_MODEL_DIR";
const ENV_MODEL_VARIANT: &str = "WM_EVAL_MODEL_VARIANT";
const DEFAULT_VARIANT: &str = "small";
const VARIANT_SMALL: &str = "gliner2.5-small-v1";
const VARIANT_BASE: &str = "gliner2.5-base-v1";
const VARIANT_MULTI: &str = "gliner2.5-multi-v1";
const VARIANT_DECIDE: &str = "gliner2.5-decide-v1";
const MATRIX_VARIANTS: [&str; 3] = ["small", "base", "decide"];
const SMOKE_LIMIT: usize = 2;
const COMBINED_RESULTS: &str = "wm_decision_eval_matrix.json";

fn variant_dir(name: &str) -> PathBuf {
    let lower = name.to_lowercase();
    let dir_name = match lower.as_str() {
        "small" => VARIANT_SMALL,
        "base" => VARIANT_BASE,
        "multi" => VARIANT_MULTI,
        "decide" => VARIANT_DECIDE,
        other => other,
    };
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(home)
        .join(".wm")
        .join("models")
        .join(dir_name)
}

fn resolve_model() -> (String, PathBuf) {
    if let Ok(explicit) = std::env::var(ENV_MODEL_DIR) {
        return ("explicit".to_owned(), PathBuf::from(explicit));
    }
    let variant = std::env::var(ENV_MODEL_VARIANT).unwrap_or_else(|_| DEFAULT_VARIANT.to_owned());
    let dir = variant_dir(&variant);
    (variant, dir)
}

fn limited_fixture(limit: Option<usize>) -> Vec<eval::EvalRow> {
    let fixture = eval::load_fixture();
    match limit {
        Some(limit) => fixture.into_iter().take(limit).collect(),
        None => fixture,
    }
}

fn write_results(variant: &str, config: &EvalConfig, report: &eval::EvalReport) -> PathBuf {
    let json = serde_json::to_string_pretty(report).expect("report must serialize");
    let name = format!(
        "wm_decision_eval_{variant}_{}_{}_{}.json",
        config.serialization.as_str(),
        config.phrasing.as_str(),
        config.score_decode.as_str()
    );
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, &json).expect("results must write");
    path
}

fn write_combined(paths: &[PathBuf]) -> PathBuf {
    let mut combined: Vec<serde_json::Value> = Vec::new();
    for path in paths {
        let raw = std::fs::read_to_string(path).expect("per-config results must exist");
        combined.push(serde_json::from_str(&raw).expect("per-config results must parse"));
    }
    let path = std::env::temp_dir().join(COMBINED_RESULTS);
    let json = serde_json::to_string_pretty(&serde_json::Value::Array(combined))
        .expect("combined results must serialize");
    std::fs::write(&path, &json).expect("combined results must write");
    path
}

#[test]
#[ignore = "requires a local gliner model and the decision feature"]
fn real_model_eval() {
    let (variant, dir) = resolve_model();
    let backend = GlinerBackend::load(&dir).expect("gliner model must load");
    let fixture = limited_fixture(eval::env_row_limit());
    let config = EvalConfig::from_env();
    let report = eval::run_config(|text, specs| backend.classify(text, specs), &fixture, &config)
        .expect("eval run");
    println!("{}", eval::render_config_line(&variant, &report));
    println!("{}", eval::render_report(&report));
    let path = write_results(&variant, &config, &report);
    println!("results_json={}", path.display());
}

#[test]
#[ignore = "runs the serialization x phrasing matrix over small/base/decide; set WM_EVAL_LIMIT"]
fn real_model_matrix() {
    let fixture = limited_fixture(eval::env_row_limit());
    let mut written: Vec<PathBuf> = Vec::new();
    for variant in MATRIX_VARIANTS {
        let dir = variant_dir(variant);
        if !dir.is_dir() {
            println!("variant {variant} unavailable at {}", dir.display());
            continue;
        }
        let backend = GlinerBackend::load(&dir).expect("gliner model must load");
        for serialization in Serialization::all() {
            for phrasing in Phrasing::all() {
                let config = EvalConfig::new(
                    serialization,
                    phrasing,
                    ScoreDecode::Argmax,
                    EvalConfig::default_trim_words(),
                );
                let report = eval::run_config(
                    |text, specs| backend.classify(text, specs),
                    &fixture,
                    &config,
                )
                .expect("eval run");
                println!("{}", eval::render_config_line(variant, &report));
                written.push(write_results(variant, &config, &report));
            }
        }
    }
    let combined = write_combined(&written);
    println!("combined_json={}", combined.display());
}

#[test]
#[ignore = "loads each available variant and runs WM_EVAL_LIMIT rows as a preflight"]
fn real_model_variant_smoke() {
    let limit = eval::env_row_limit().unwrap_or(SMOKE_LIMIT);
    let fixture = limited_fixture(Some(limit));
    for variant in MATRIX_VARIANTS {
        let dir = variant_dir(variant);
        if !dir.is_dir() {
            println!("variant {variant} unavailable at {}", dir.display());
            continue;
        }
        let backend = GlinerBackend::load(&dir).expect("gliner model must load");
        let config = EvalConfig::default();
        let report = eval::run_config(
            |text, specs| backend.classify(text, specs),
            &fixture,
            &config,
        )
        .expect("smoke run");
        println!("{}", eval::render_config_line(variant, &report));
    }
}
