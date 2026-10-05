#[cfg(feature = "decision")]
use std::collections::BTreeMap;
#[cfg(feature = "decision")]
use std::path::PathBuf;

use crate::engine::PageType;
use crate::mcp::prelude::*;
use crate::page::helpers::{build_frontmatter, FrontmatterValue};
use serde_json::json;

use crate::parser;
use crate::status::PageStatus;

#[cfg(feature = "decision")]
use wm_engine::{
    canonical_questions, is_record_bearing, parse_record, validate_record, AnswerValue,
    DecisionRecord, DecisionRuntime, ModelEntry, RECORD_SCHEMA_VERSION,
};

use crate::decision::manifest_store::{
    cached_model_dir, is_cached, load_manifest, manifest_path, DEFAULT_DECISION_MODEL,
};
#[cfg(feature = "decision")]
use crate::decision::manifest_store::models_cache_dir;

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmDecisionAction {
    Create {
        #[schemars(description = "Decision page ID")]
        id: String,
        #[schemars(description = "Decision title")]
        title: String,
        #[schemars(description = "Context (background/why)")]
        context: String,
        #[schemars(description = "Options considered")]
        options: Option<Vec<String>>,
        #[schemars(description = "Rationale for the chosen option")]
        rationale: String,
        #[schemars(description = "Outcome or result")]
        outcome: Option<String>,
        #[schemars(description = "Record state prose; overrides context/rationale when non-empty")]
        content: Option<String>,
        #[schemars(description = "Status: draft/accepted/superseded/rejected/archived")]
        status: Option<PageStatus>,
    },
    Get {
        #[schemars(description = "Decision page ID")]
        id: String,
    },
    Answer {
        #[schemars(description = "Record-bearing page ID (alternative to state+type)")]
        page: Option<String>,
        #[schemars(description = "Record state prose (used with `type`)")]
        state: Option<String>,
        #[serde(rename = "type")]
        #[schemars(
            description = "Page type for ad-hoc state (decision/pattern/concept/howto/reference)"
        )]
        page_type: Option<String>,
    },
    Status {},
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed_async(
        "wm_decision",
        "Architectural decision records and typed-decision runtime (create, get, answer, status)",
        move |input: WmDecisionAction| {
            let engine = engine.clone();
            async move {
                match input {
                    WmDecisionAction::Create {
                        id,
                        title,
                        context,
                        options,
                        rationale,
                        outcome,
                        content,
                        status,
                    } => create_action(&engine, id, title, context, options, rationale, outcome, content, status),
                    WmDecisionAction::Get { id } => get_action(&engine, id),
                    WmDecisionAction::Answer {
                        page,
                        state,
                        page_type,
                    } => answer_action(&engine, page, state, page_type).await,
                    WmDecisionAction::Status {} => status_action(&engine),
                }
            }
        },
    );
}

fn create_action(
    engine: &Arc<EngineState>,
    id: String,
    title: String,
    context: String,
    options: Option<Vec<String>>,
    rationale: String,
    outcome: Option<String>,
    content: Option<String>,
    status: Option<PageStatus>,
) -> Result<serde_json::Value, ToolError> {
    let page_status = status.unwrap_or(PageStatus::Draft);
    if !PageType::Decision.allowed_statuses().contains(&page_status) {
        return Err(ToolError::invalid_params(format!(
            "Invalid status '{}' for decision page. Allowed: {}",
            page_status,
            PageType::Decision
                .allowed_statuses()
                .iter()
                .map(|s| s.as_str())
                .fold(String::new(), |mut acc, s| {
                    if !acc.is_empty() {
                        acc.push_str(", ");
                    }
                    acc.push_str(s);
                    acc
                },)
        )));
    }
    let status = page_status.as_str().to_string();

    let provided = content.unwrap_or_default();
    let state = create_state(
        &provided,
        &context,
        &rationale,
        options.as_deref(),
        outcome.as_deref(),
    );
    let body = crate::page::build_record_body(&PageType::Decision, &state)
        .map_err(|error| ToolError::internal(format!("record body render failed: {error}")))?;

    let mut decision_fields: Vec<(&'static str, FrontmatterValue)> = vec![
        ("context", FrontmatterValue::Scalar(context)),
        ("rationale", FrontmatterValue::Scalar(rationale)),
    ];
    if let Some(opts) = options {
        if !opts.is_empty() {
            decision_fields.push(("options", FrontmatterValue::List(opts)));
        }
    }
    if let Some(outcome) = outcome {
        decision_fields.push(("outcome", FrontmatterValue::Scalar(outcome)));
    }
    let frontmatter = build_frontmatter(&[
        ("title", FrontmatterValue::Scalar(title.clone())),
        (
            "type",
            FrontmatterValue::Scalar(PageType::Decision.as_str().to_owned()),
        ),
        ("status", FrontmatterValue::Scalar(status.clone())),
        ("decision", FrontmatterValue::Nested(decision_fields)),
    ]);

    let _ = crate::page::create_page(engine, &id, &frontmatter, &body)?;
    Ok(json!({
        "id": id,
        "title": title,
        "status": status,
    }))
}

fn create_state(
    content: &str,
    context: &str,
    rationale: &str,
    options: Option<&[String]>,
    outcome: Option<&str>,
) -> String {
    if !content.trim().is_empty() {
        return content.to_owned();
    }
    let mut state = format!("## Context\n\n{context}\n\n## Rationale\n\n{rationale}\n");
    if let Some(options) = options {
        if !options.is_empty() {
            state.push_str("\n## Options\n\n");
            for option in options {
                state.push_str(&format!("- {option}\n"));
            }
        }
    }
    if let Some(outcome) = outcome {
        state.push_str(&format!("\n## Outcome\n\n{outcome}\n"));
    }
    state
}

fn get_action(engine: &EngineState, id: String) -> Result<serde_json::Value, ToolError> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(&id)
        .ok_or_else(|| ToolError::not_found("decision", &id))?;
    let meta = &snapshot.0[*node_idx];

    if meta.page_type != PageType::Decision {
        return Err(ToolError::not_found("decision", &id));
    }

    let content = std::fs::read_to_string(&meta.path)
        .map_err(|error| ToolError::io_error("read", meta.path.to_string_lossy(), error))?;
    let (_frontmatter, body) = parser::extract_frontmatter(&content);

    Ok(json!({
        "id": meta.id,
        "title": meta.title,
        "status": meta.status.as_str(),
        "context": meta.decision_data.as_ref().map(|d| &d.context),
        "options": meta.decision_data.as_ref().map(|d| &d.options),
        "rationale": meta.decision_data.as_ref().map(|d| &d.rationale),
        "outcome": meta.decision_data.as_ref().map(|d| &d.outcome),
        "content": body,
    }))
}

fn status_action(engine: &EngineState) -> Result<serde_json::Value, ToolError> {
    let project_root = engine
        .project_root
        .read()
        .map(|root| root.clone())
        .unwrap_or_default();
    let manifest = load_manifest(&project_root);
    Ok(json!({
        "model": DEFAULT_DECISION_MODEL,
        "manifest": manifest_path(&project_root).display().to_string(),
        "manifest_ok": manifest.is_ok(),
        "manifest_error": manifest.as_ref().err().map(|error| error.to_string()),
        "cached": is_cached(DEFAULT_DECISION_MODEL),
        "model_dir": cached_model_dir(DEFAULT_DECISION_MODEL).display().to_string(),
        "feature_enabled": cfg!(feature = "decision"),
    }))
}

#[cfg(feature = "decision")]
async fn answer_action(
    engine: &EngineState,
    page: Option<String>,
    state: Option<String>,
    page_type: Option<String>,
) -> Result<serde_json::Value, ToolError> {
    let (page_id, record) = resolve_record(engine, page, state, page_type)?;
    let project_root = engine
        .project_root
        .read()
        .map(|root| root.clone())
        .unwrap_or_default();
    let manifest = load_manifest(&project_root).map_err(to_tool_error)?;
    let entry = manifest
        .entry(DEFAULT_DECISION_MODEL)
        .ok_or_else(|| ToolError::not_found("model", DEFAULT_DECISION_MODEL))?
        .clone();
    let cache_dir = models_cache_dir();
    tokio::task::spawn_blocking(move || run_inference(cache_dir, entry, page_id, record))
        .await
        .map_err(|error| ToolError::internal(format!("decision inference task failed: {error}")))?
}

#[cfg(not(feature = "decision"))]
async fn answer_action(
    _engine: &EngineState,
    _page: Option<String>,
    _state: Option<String>,
    _page_type: Option<String>,
) -> Result<serde_json::Value, ToolError> {
    Err(ToolError::internal(
        "wm_decision.answer requires the 'decision' feature. Rebuild with --features decision.",
    ))
}

#[cfg(feature = "decision")]
fn run_inference(
    cache_dir: PathBuf,
    entry: ModelEntry,
    page_id: Option<String>,
    record: DecisionRecord,
) -> Result<serde_json::Value, ToolError> {
    let cached = cache_dir.join(&entry.name);
    let model_dir = match cached.is_dir() {
        true => cached,
        false => crate::decision::model_download::ensure_model(&entry, &cache_dir)
            .map_err(to_tool_error)?,
    };
    let key = format!("{}@{}", model_dir.display(), entry.revision);
    let backend = crate::decision::backend_cache::backend_cache()
        .get_or_load(&key, || {
            crate::decision::gliner_backend::GlinerBackend::load(&model_dir)
        })
        .map_err(to_tool_error)?;
    let runtime = DecisionRuntime::new(
        crate::decision::shared_backend_model::SharedBackend::new(backend),
        entry.name.clone(),
    );
    let result = runtime.answer(page_id, &record).map_err(to_tool_error)?;
    Ok(decision_result_json(&result))
}

#[cfg(feature = "decision")]
fn resolve_record(
    engine: &EngineState,
    page: Option<String>,
    state: Option<String>,
    page_type: Option<String>,
) -> Result<(Option<String>, DecisionRecord), ToolError> {
    if let Some(page_id) = page {
        let snapshot = engine.graph.load();
        let node_idx = snapshot
            .1
            .get(&page_id)
            .ok_or_else(|| ToolError::not_found("page", &page_id))?;
        let meta = &snapshot.0[*node_idx];
        if !is_record_bearing(&meta.page_type) {
            return Err(ToolError::invalid_params(format!(
                "page '{page_id}' is not a record-bearing page"
            )));
        }
        let content = std::fs::read_to_string(&meta.path)
            .map_err(|error| ToolError::io_error("read", meta.path.to_string_lossy(), error))?;
        let (_frontmatter, body) = parser::extract_frontmatter(&content);
        let record = parse_record(&meta.page_type, body).map_err(to_tool_error)?;
        return Ok((Some(page_id), record));
    }

    let state =
        state.ok_or_else(|| ToolError::invalid_params("provide 'page', or 'state' plus 'type'"))?;
    let type_name =
        page_type.ok_or_else(|| ToolError::invalid_params("missing 'type' for ad-hoc state"))?;
    let parsed_type = PageType::from_type_name(&type_name)
        .ok_or_else(|| ToolError::invalid_params(format!("unknown page type '{type_name}'")))?;
    if !is_record_bearing(&parsed_type) {
        return Err(ToolError::invalid_params(format!(
            "page type '{type_name}' is not record-bearing"
        )));
    }
    let record = DecisionRecord {
        schema_version: RECORD_SCHEMA_VERSION,
        state,
        questions: canonical_questions(&parsed_type),
        answers: BTreeMap::new(),
    };
    validate_record(&parsed_type, &record).map_err(to_tool_error)?;
    Ok((None, record))
}

#[cfg(feature = "decision")]
fn to_tool_error(error: wm_engine::DecisionError) -> ToolError {
    let detail = error.to_string();
    if matches!(error, wm_engine::DecisionError::Backend { .. }) {
        return ToolError::internal(detail);
    }
    ToolError::invalid_params(detail)
}

#[cfg(feature = "decision")]
fn decision_result_json(result: &wm_engine::DecisionResult) -> serde_json::Value {
    let answers: serde_json::Map<String, serde_json::Value> = result
        .answers
        .iter()
        .map(|(id, answer)| {
            (
                id.clone(),
                json!({
                    "value": answer_value_json(&answer.value),
                    "probability": answer.probability,
                    "distribution": answer.distribution,
                }),
            )
        })
        .collect();
    json!({
        "page": result.page,
        "model": result.model,
        "answers": answers,
        "chunks": result.chunks,
        "truncated": result.truncated,
    })
}

#[cfg(feature = "decision")]
fn answer_value_json(value: &AnswerValue) -> serde_json::Value {
    match value {
        AnswerValue::Bool(flag) => json!(flag),
        AnswerValue::Label(label) => json!(label),
        AnswerValue::Labels(labels) => json!(labels),
    }
}
